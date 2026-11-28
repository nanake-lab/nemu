use crate::{Error, Result, dispatch::response, gateway::Gateway, hooks::CoreHandler};
use core_store::{LineageKind, Reader};
use core_types::{WireMessage, generated::*};

pub(crate) struct CheckedProgress<L: LineageKind> {
    reader: Reader<L>,
    publication: Option<(u64, String)>,
    settlement: Option<(EventEnvelope, ReceiptV2)>,
    events: Vec<(String, EventEnvelope)>,
}

pub(crate) type ProgressTask<L> = tokio::task::JoinHandle<Result<CheckedProgress<L>>>;

impl<L: LineageKind, H: CoreHandler<L>> Gateway<L, H> {
    pub(crate) fn check_publication(
        &mut self,
        full: bool,
        settlement: Option<(EventEnvelope, ReceiptV2)>,
    ) -> Result<ProgressTask<L>> {
        let job: Box<dyn FnOnce() -> core_store::Result<Reader<L>> + Send> = if full {
            Box::new(self.store.full_publication_check()?)
        } else {
            Box::new(
                self.store
                    .receipt_check(self.receipt_reader.take(), || {})?,
            )
        };
        #[cfg(test)]
        let gate = self.full_gate.take();
        Ok(tokio::task::spawn_blocking(move || {
            #[cfg(test)]
            if let Some((entered, resume)) = gate {
                let _ = entered.send(());
                let _ = resume.recv();
            }
            Ok(CheckedProgress {
                reader: job()?,
                publication: None,
                settlement,
                events: vec![],
            })
        }))
    }

    pub(crate) fn progress(&mut self) -> Result<Option<ProgressTask<L>>> {
        // Gamma operates on durable D before the core can learn or sample time.
        if let Some(event) = self.store.next_raw()?
            && let Some(settled) = self.store.settle_duplicate()?
        {
            let reason = match settled.stage {
                ReceiptStage::FILTERED => None,
                ReceiptStage::REJECTED => Some(self.audit(
                    IngressReason::SOURCE_KEY_CONFLICT,
                    Some(IngressKind::INGRESS_EVENT),
                )?),
                _ => return Err(Error::Rejected("unexpected Gamma outcome")),
            };
            let receipt = ReceiptV2 {
                schema_version: 1,
                stable_source_key: event.stable_source_key.clone(),
                stage: settled.stage,
                event_id: Some(event.id.clone()),
                reason_code: reason,
                raw_log_position: None,
                ingest_time_ns: None,
            };

            // Do not hand the next queued duplicate to the core merely
            // because one duplicate was settled in this tick. Gamma must
            // precede EVERY new core transition, including a retry backlog.
            return Ok(Some(self.check_publication(false, Some((event, receipt)))?));
        }
        let before = self.store.head()?;
        let outcomes = self.handler.process_next(&mut self.store)?;
        let after = self.store.head()?;
        if outcomes.len() > self.config.frames.encoding.max_items {
            return Err(Error::Rejected("core outcome budget exceeded"));
        }
        let changed = after.0 != before.0;
        if !changed && outcomes.is_empty() {
            return Ok(None);
        }
        let limits = self.config.frames.encoding;
        #[cfg(test)]
        let check_gate = self.check_gate.take();
        let check = self
            .store
            .receipt_check(self.receipt_reader.take(), move || {
                #[cfg(test)]
                if let Some((entered, resume)) = check_gate {
                    let _ = entered.send(());
                    let _ = resume.recv();
                }
            })?;
        Ok(Some(tokio::task::spawn_blocking(move || {
            let reader = check()?;
            let publication = if changed {
                let published = reader.current()?.ok_or(Error::Contract)?;
                if after.1 != before.1.checked_add(1).ok_or(Error::Contract)?
                    || published.processed_position.0 != after.1
                    || published.binding.raw_log_position.0 != before.1
                {
                    return Err(Error::Rejected("core must process one durable head"));
                }
                Some((before.1, after.0.ok_or(Error::Contract)?))
            } else {
                None
            };
            let mut events = Vec::with_capacity(outcomes.len());
            for outcome in outcomes {
                outcome
                    .canonical_bytes(limits)
                    .map_err(|_| Error::Contract)?;
                if !outcome.committed {
                    return Err(Error::Rejected("uncommitted core outcome"));
                }
                let (transaction, event) = reader.read_published_transaction(&outcome.snapshot)?;
                if outcome.event_id != event.id
                    || transaction.binding.event_id != event.id
                    || transaction.binding.raw_log_position != event.raw_log_position
                    || transaction.binding.branch_id != outcome.branch_id
                    || outcome.commit_id != core_types::scalar::hex(&outcome.snapshot.digest.0)
                {
                    return Err(Error::Rejected("core receipt publication mismatch"));
                }
                events.push((outcome.commit_id, event));
            }
            Ok(CheckedProgress {
                reader,
                publication,
                settlement: None,
                events,
            })
        })))
    }

    pub(crate) fn finish_progress(&mut self, mut checked: CheckedProgress<L>) -> Result<()> {
        self.store.accept_publication(&mut checked.reader)?;
        self.state = match checked.reader.current_snapshot()? {
            Some(head) => self.handler.committed_run_state(&checked.reader, &head)?,
            None => None,
        };
        self.receipt_reader = Some(checked.reader);
        for id in self.connections.keys().copied().collect::<Vec<_>>() {
            self.send_state(id)?;
        }
        if let Some((event, receipt)) = checked.settlement {
            return self.deliver(event.raw_log_position.0, &event, receipt);
        }
        if let Some((position, commit)) = checked.publication
            && let Some(p) = self.pending.get_mut(&position)
        {
            p.published_commit = Some(commit);
        }
        for (commit, event) in checked.events {
            let Some(p) = self.pending.get(&event.raw_log_position.0) else {
                continue; // no surviving connection is awaiting this notification
            };
            // An uploaded CoreTransaction object can become visible through a
            // closure without being an accepted commit. Require the head digest
            // observed after its actual Store transition, not a handler assertion.
            if p.published_commit.as_ref() != Some(&commit) {
                return Err(Error::Rejected("snapshot is not an observed commit"));
            }
            let receipt = ReceiptV2 {
                schema_version: 1,
                stable_source_key: event.stable_source_key.clone(),
                stage: ReceiptStage::CORE_COMMITTED,
                event_id: Some(event.id.clone()),
                reason_code: None,
                raw_log_position: None,
                ingest_time_ns: None,
            };
            self.deliver(event.raw_log_position.0, &event, receipt)?;
        }
        Ok(())
    }
    fn deliver(&mut self, position: u64, event: &EventEnvelope, receipt: ReceiptV2) -> Result<()> {
        let Some(pending) = self.pending.remove(&position) else {
            return Ok(());
        };
        let id = pending.connection;
        let Some(c) = self.connections.get_mut(&id) else {
            return Ok(());
        };
        let input = pending.request.event.as_ref().ok_or(Error::Contract)?;
        if input.id != event.id || input.stable_source_key != event.stable_source_key {
            return Err(Error::Rejected("receipt request mismatch"));
        }
        let close = receipt.reason_code == Some(IngressReason::AUDIT_FULL);
        let mut reply = response(&pending.request, IngressResponseKind::EVENT_RECEIPT);
        reply.receipt = Some(receipt);
        core_types::terminal_ingress::check_response(
            &pending.request,
            &reply,
            &c.session,
            self.config.frames.encoding,
        )
        .map_err(|_| Error::Contract)?;
        c.uploads.release(input);
        if c.output
            .try_send(
                reply
                    .protobuf_bytes(self.config.frames.encoding)
                    .map_err(|_| Error::Contract)?,
            )
            .is_err()
            || close
        {
            self.close(id);
        }
        Ok(())
    }
}
