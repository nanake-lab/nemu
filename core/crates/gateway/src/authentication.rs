use crate::{
    Error, Result,
    connection::{Authentication, Connection},
    gateway::Gateway,
    hooks::CoreHandler,
};
use core_store::LineageKind;
use core_types::{WireMessage, generated::*, scalar::hex, terminal_ingress::check_auth_pair};
use tokio::sync::{mpsc, oneshot};

impl<L: LineageKind, H: CoreHandler<L>> Gateway<L, H> {
    pub(crate) fn authenticate(
        &mut self,
        id: u64,
        request: Authentication,
        reply: oneshot::Sender<TerminalAuthResult>,
        output: mpsc::Sender<TerminalResponse>,
    ) -> Result<()> {
        let r = &request.0;
        let mut nonce = [0; 32];
        getrandom::fill(&mut nonce).map_err(|_| Error::Rejected("session entropy unavailable"))?;
        let session_id = hex(&nonce);
        let installation = self
            .installations
            .iter()
            .find(|i| i.id == r.installation_id);
        // Always compare a token, including unknown installation IDs. No secret
        // enters the session, audit, errors, response, tracing or object store.
        let candidate = installation.unwrap_or(&self.installations[0]);
        let auth = candidate
            .token
            .authenticate(&r.token.0, &candidate.id, &session_id);
        let reason = if self.audit_blocked {
            Some(IngressReason::AUDIT_FULL)
        } else if r.canonical_bytes(self.config.frames.encoding).is_err() {
            Some(IngressReason::INVALID_FRAME)
        } else if installation.is_none() || auth.is_err() {
            Some(IngressReason::AUTH_FAILED)
        } else if !r.protocol_versions.contains(&1) {
            Some(IngressReason::VERSION_UNSUPPORTED)
        } else if self.connections.len() >= self.config.max_connections
            || self.connections.contains_key(&id)
        {
            Some(IngressReason::CAPABILITY_DENIED)
        } else {
            None
        };
        if let Some(reason) = reason {
            let reason = self.audit(reason, None)?;
            let result = TerminalAuthResult {
                schema_version: 1,
                request_id: r.request_id.clone(),
                status: AuthStatus::AUTH_REJECTED,
                session: None,
                reason: Some(reason),
            };
            if check_auth_pair(r, &result, self.config.frames.encoding).is_ok() {
                let _ = reply.send(result);
            }
            return Ok(());
        }
        let session = TerminalSession {
            schema_version: 1,
            session_id,
            installation_id: candidate.id.clone(),
            protocol_version: 1,
            capabilities: candidate.capabilities.clone(),
            budget: self.config.budget.clone(),
        };
        let result = TerminalAuthResult {
            schema_version: 1,
            request_id: r.request_id.clone(),
            status: AuthStatus::AUTHENTICATED,
            session: Some(session.clone()),
            reason: None,
        };
        if check_auth_pair(r, &result, self.config.frames.encoding).is_err()
            || result.protobuf_bytes(self.config.frames.encoding).is_err()
        {
            self.audit(IngressReason::INVALID_FRAME, None)?;
            return Ok(());
        }
        self.connections.insert(
            id,
            Connection {
                session,
                auth: auth?,
                output,
                uploads: Default::default(),
                created: std::time::Instant::now(),
            },
        );
        if reply.send(result).is_err() {
            self.close(id);
        }
        Ok(())
    }
}
