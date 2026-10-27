use crate::{Error, Result, dispatch::response, gateway::Gateway, hooks::CoreHandler};
use core_store::LineageKind;
use core_types::{WireMessage, generated::*};

impl<L: LineageKind, H: CoreHandler<L>> Gateway<L, H> {
    pub(crate) fn progress(&mut self) -> Result<()> {
        // Gamma operates on durable D before the core can learn or sample time.
        if let Some(event) = self.store.next_raw()? {
            if let Some(settled) = self.store.settle_duplicate()? {
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
                self.deliver(event.raw_log_position.0, &event, receipt)?;
                // Do not hand the next queued duplicate to the core merely
                // because one duplicate was settled in this tick. Gamma must
                // precede EVERY new core transition, including a retry backlog.
                return Ok(());
            }
        }
        let before = self.store.head()?;
        let outcomes = self.handler.process_next(&mut self.store)?;
        let after = self.store.head()?;
        if after.0 != before.0 {
            // Observe the sole writer's published head immediately after EACH
            // core step, even when its notification is delayed. This avoids a
            // history-sized query/cache and does not accept mere object visibility.
            let published = self.store.reader()?.current()?.ok_or(Error::Contract)?;
            if after.1 != before.1.checked_add(1).ok_or(Error::Contract)?
                || published.processed_position.0 != after.1
                || published.binding.raw_log_position.0 != before.1
            {
                return Err(Error::Rejected("core must process one durable head"));
            }
            if let Some(p) = self.pending.get_mut(&before.1) {
                p.published_commit = after.0;
            }
        }
        if outcomes.len() > self.config.frames.encoding.max_items {
            return Err(Error::Rejected("core outcome budget exceeded"));
        }
        for outcome in outcomes {
            outcome
                .canonical_bytes(self.config.frames.encoding)
                .map_err(|_| Error::Contract)?;
            if !outcome.committed {
                return Err(Error::Rejected("uncommitted core outcome"));
            }
            // Read checks the exact ObjectRef and visibility in THIS lineage.
            let reader = self.store.reader()?;
            let transaction: CoreTransactionV2 = reader.read(&outcome.snapshot)?;
            let event: EventEnvelope = reader.read(&transaction.staged_h)?;
            let Some(p) = self.pending.get(&event.raw_log_position.0) else {
                continue; // no surviving connection is awaiting this notification
            };
            // An uploaded CoreTransaction object can become visible through a
            // closure without being an accepted commit. Require the head digest
            // observed after its actual Store transition, not a handler assertion.
            if p.published_commit.as_ref() != Some(&outcome.commit_id) {
                return Err(Error::Rejected("snapshot is not an observed commit"));
            }
            if outcome.event_id != event.id
                || transaction.binding.event_id != event.id
                || transaction.binding.raw_log_position != event.raw_log_position
                || transaction.binding.branch_id != outcome.branch_id
                || outcome.commit_id != core_types::scalar::hex(&outcome.snapshot.digest.0)
            {
                return Err(Error::Rejected("core receipt publication mismatch"));
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
        if c.output.try_send(reply).is_err() || close {
            self.close(id);
        }
        Ok(())
    }
}
