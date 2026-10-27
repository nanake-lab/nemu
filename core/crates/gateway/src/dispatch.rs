use crate::{
    Error, Result,
    connection::{Connection, Pending},
    gateway::Gateway,
    hooks::{CoreHandler, Decision},
};
use core_store::LineageKind;
use core_types::{
    WireMessage,
    generated::*,
    scalar::{DecimalI64, DecimalU64},
    terminal_ingress::{check_request, check_response, rejected_receipt},
};

pub(crate) fn response(request: &TerminalRequest, kind: IngressResponseKind) -> TerminalResponse {
    TerminalResponse {
        schema_version: 1,
        request_id: request.request_id.clone(),
        session_id: request.session_id.clone(),
        protocol_version: request.protocol_version,
        reply_to: request.kind,
        kind,
        receipt: None,
        upload: None,
        query: None,
        reason: None,
    }
}
impl<L: LineageKind, H: CoreHandler<L>> Gateway<L, H> {
    pub(crate) fn dispatch(&mut self, id: u64, request: TerminalRequest) -> Result<()> {
        let Some(mut c) = self.connections.remove(&id) else {
            self.audit(IngressReason::AUTH_FAILED, Some(request.kind))?;
            return Ok(());
        };
        // A session ID is never a bearer credential transferable to another pipe.
        if c.session.session_id != request.session_id
            || c.session.protocol_version != request.protocol_version
            || c.created.elapsed() >= self.config.session_lifetime
        {
            self.audit(IngressReason::SESSION_EXPIRED, Some(request.kind))?;
            self.close(id);
            return Ok(()); // no falsely paired reply for an unbound request
        }
        // Admission checks expiry too, so a busy input queue cannot defer it
        // indefinitely by keeping the periodic cleanup branch unselected.
        c.uploads.expire(self.config.upload_lifetime);
        let decision = self.admit(id, &mut c, &request)?;
        let mut close = false;
        let reply = match decision {
            Ok(reply) => reply,
            Err(reason) => {
                let reason = self.audit(reason, Some(request.kind))?;
                close = reason == IngressReason::AUDIT_FULL;
                let mut reply = response(&request, IngressResponseKind::INGRESS_REJECTED);
                reply.reason = Some(reason);
                reply
            }
        };
        close |= reply
            .receipt
            .as_ref()
            .is_some_and(|r| r.reason_code == Some(IngressReason::AUDIT_FULL));
        if check_response(&request, &reply, &c.session, self.config.frames.encoding).is_err() {
            self.audit(IngressReason::INVALID_FRAME, Some(request.kind))?;
            self.close(id);
            return Ok(());
        }
        if c.output.try_send(reply).is_err() {
            self.close(id);
            return Ok(());
        }
        if close {
            self.close(id);
        } else {
            self.connections.insert(id, c);
        }
        Ok(())
    }
    fn admit(
        &mut self,
        id: u64,
        c: &mut Connection<L>,
        r: &TerminalRequest,
    ) -> Result<Decision<TerminalResponse>> {
        let limits = self.config.frames.encoding;
        if self.audit_blocked {
            return Ok(Err(IngressReason::AUDIT_FULL));
        }
        if !c.session.capabilities.contains(&r.kind) {
            return Ok(Err(IngressReason::CAPABILITY_DENIED));
        }
        if r.protobuf_bytes(limits).map_or(true, |b| {
            b.len() > c.session.budget.max_frame_bytes as usize
        }) {
            return Ok(Err(IngressReason::INVALID_FRAME));
        }
        if let Some(e) = &r.event
            && e.payload_digest != e.payload_ref.digest
        {
            return Ok(Err(IngressReason::OBJECT_MISMATCH));
        }
        if let Some(u) = &r.upload
            && (u.data.0.len() > c.session.budget.max_chunk_bytes as usize
                || u.object.byte_length.0 > u64::from(c.session.budget.max_object_bytes))
        {
            return Ok(Err(IngressReason::UPLOAD_BUDGET_FULL));
        }
        if check_request(r, Some(&c.session), limits).is_err() {
            return Ok(Err(IngressReason::OBJECT_MISMATCH));
        }
        let reply = match r.kind {
            IngressKind::INGRESS_UPLOAD => {
                let receipt = match c.uploads.chunk(
                    &self.store,
                    r.upload.as_ref().ok_or(Error::Contract)?,
                    &c.session.budget,
                    limits,
                )? {
                    Ok(v) => v,
                    Err(reason) => return Ok(Err(reason)),
                };
                let mut reply = response(r, IngressResponseKind::UPLOAD_RECEIPT);
                reply.upload = Some(receipt);
                reply
            }
            IngressKind::INGRESS_EVENT => return self.event(id, c, r),
            IngressKind::INGRESS_QUERY => {
                let q = r.query.as_ref().ok_or(Error::Contract)?;
                let result = match self.handler.query(&mut self.store, &c.session, q)? {
                    Ok(v) => v,
                    Err(reason) => return Ok(Err(reason)),
                };
                if crate::terminal::validate_query_result(q, &result, limits).is_err() {
                    return Ok(Err(IngressReason::INVALID_FRAME));
                }
                let mut reply = response(r, IngressResponseKind::QUERY_REPLY);
                reply.query = Some(result);
                reply
            }
            IngressKind::INGRESS_APPROVAL => {
                if let Err(reason) = self.handler.approval(
                    &mut self.store,
                    &c.session,
                    r.approval.as_ref().ok_or(Error::Contract)?,
                )? {
                    return Ok(Err(reason));
                }
                response(r, IngressResponseKind::CONTROL_ACK)
            }
            IngressKind::INGRESS_CANCEL => {
                if let Err(reason) = self.handler.cancel(
                    &mut self.store,
                    &c.session,
                    r.cancel.as_ref().ok_or(Error::Contract)?,
                )? {
                    return Ok(Err(reason));
                }
                response(r, IngressResponseKind::CONTROL_ACK)
            }
            IngressKind::INGRESS_GROWTH => {
                if let Err(reason) = self.handler.growth(
                    &mut self.store,
                    &c.session,
                    r.growth.as_ref().ok_or(Error::Contract)?,
                )? {
                    return Ok(Err(reason));
                }
                response(r, IngressResponseKind::CONTROL_ACK)
            }
        };
        Ok(Ok(reply))
    }
    fn event(
        &mut self,
        id: u64,
        c: &mut Connection<L>,
        r: &TerminalRequest,
    ) -> Result<Decision<TerminalResponse>> {
        let e = r.event.as_ref().ok_or(Error::Contract)?;
        let limits = self.config.frames.encoding;
        let payload = match c.uploads.payload(&self.store, &e.payload_ref, limits) {
            Ok(v) => v,
            Err(Error::Contract) => return Ok(Err(IngressReason::OBJECT_MISMATCH)),
            Err(v) => return Err(v),
        };
        let source: SourceMetadata = match c.uploads.read(&self.store, &e.provenance, limits) {
            Ok(v) => v,
            Err(Error::Contract) => return Ok(Err(IngressReason::OBJECT_MISMATCH)),
            Err(v) => return Err(v),
        };
        let grants = match self
            .handler
            .authorize_event(&self.store, &c.session, e, &source)
        {
            Ok(v) => v,
            Err(reason) => return Ok(Err(reason)),
        };
        if matches!(&payload, crate::TerminalPayload::Desktop(p) if p.title.is_some() && !grants.desktop_titles)
            || matches!(&payload, crate::TerminalPayload::Codex(_) if !grants.codex_sessions)
        {
            return Ok(Err(IngressReason::CAPABILITY_DENIED));
        }
        // Generated projection ONLY for existing semantic validation. Neither
        // sentinel position/time is persisted; W02 alone allocates both fields.
        let projection = EventEnvelope {
            schema_version: 1,
            id: e.id.clone(),
            stable_source_key: e.stable_source_key.clone(),
            kind: e.kind,
            event_time_ns: e.event_time_ns,
            ingest_time_ns: DecimalI64(0),
            raw_log_position: DecimalU64(0),
            parents: e.parents.clone(),
            payload_ref: e.payload_ref.clone(),
            provenance: e.provenance.clone(),
            device: c.session.installation_id.clone(),
            session: c.session.session_id.clone(),
            payload_digest: e.payload_digest.clone(),
            action_id: e.action_id.clone(),
            trial_id: e.trial_id.clone(),
            correction_of: e.correction_of.clone(),
            derivation: e.derivation,
        };
        if crate::validate_event(&c.auth, grants, projection, payload, source, limits).is_err() {
            return Ok(Err(IngressReason::INVALID_FRAME));
        }
        let metadata: usize = self
            .pending
            .values()
            .map(|p| {
                p.request
                    .protobuf_bytes(limits)
                    .map_or(limits.max_bytes, |b| b.len())
            })
            .sum();
        let capacity = if self.pending.len() >= self.config.max_pending_requests {
            Some(IngressReason::PENDING_RECORDS_FULL)
        } else if metadata
            .saturating_add(r.protobuf_bytes(limits).map_err(|_| Error::Contract)?.len())
            > limits.max_bytes
        {
            Some(IngressReason::PENDING_BYTES_FULL)
        } else {
            None
        };
        let receipt = if let Some(reason) = capacity {
            let reason = self.audit(reason, Some(r.kind))?;
            rejected_receipt(e, reason)
        } else {
            match self.store.ingest_terminal(e, &c.session) {
                Ok(receipt) => {
                    let position = receipt.raw_log_position.ok_or(Error::Contract)?.0;
                    self.pending.insert(
                        position,
                        Pending {
                            connection: id,
                            request: r.clone(),
                            published_commit: None,
                        },
                    );
                    receipt
                }
                Err(error) => match error.safe_ingress_reason() {
                    Some(reason) => {
                        let reason = self.audit(reason, Some(r.kind))?;
                        rejected_receipt(e, reason)
                    }
                    None => return Err(error.into()), // unknown outcome is a disconnect, not safe rejection
                },
            }
        };
        let mut reply = response(r, IngressResponseKind::EVENT_RECEIPT);
        reply.receipt = Some(receipt);
        Ok(Ok(reply))
    }
}
