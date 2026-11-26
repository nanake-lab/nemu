use crate::{Error, Result, gateway::Gateway, hooks::CoreHandler};
use core_store::LineageKind;
use core_types::{WireMessage, generated::*};

/// In-process dispatch only. On the pipe each body is exactly the generated
/// protobuf message, with the same bounded length prefix as a paired reply.
pub enum TerminalOutput {
    Chunk(OutputChunk),
    End(OutputEnd),
    Approval(ApprovalRequestV2),
}

impl<L: LineageKind, H: CoreHandler<L>> Gateway<L, H> {
    pub(crate) fn send_state(&mut self, id: u64) -> Result<()> {
        let c = self.connections.get_mut(&id).ok_or(Error::Disconnected)?;
        let Some(state) = &self.state else {
            return Ok(());
        };
        if c.last_state.as_ref() == Some(state) {
            return Ok(());
        }
        let bytes = state
            .protobuf_bytes(self.config.frames.encoding)
            .map_err(|_| Error::Contract)?;
        c.last_state = Some(state.clone());
        self.send_bytes(id, bytes);
        Ok(())
    }
    pub(crate) fn send_output(&mut self, session: &str, message: TerminalOutput) -> Result<()> {
        let Some((&id, c)) = self
            .connections
            .iter()
            .find(|(_, c)| c.session.session_id == session)
        else {
            return Ok(());
        };
        let limits = self.config.frames.encoding;
        let bytes = match message {
            TerminalOutput::Chunk(chunk) => chunk.protobuf_bytes(limits),
            TerminalOutput::End(end) => end.protobuf_bytes(limits),
            TerminalOutput::Approval(approval)
                if c.session
                    .capabilities
                    .contains(&IngressKind::INGRESS_APPROVAL) =>
            {
                approval.protobuf_bytes(limits)
            }
            TerminalOutput::Approval(_) => return Ok(()),
        }
        .map_err(|_| Error::Contract)?;
        self.send_bytes(id, bytes);
        Ok(())
    }
    fn send_bytes(&mut self, id: u64, bytes: Vec<u8>) {
        let c = &self.connections[&id];
        if c.created.elapsed() >= self.config.session_lifetime
            || bytes.len() > c.session.budget.max_frame_bytes as usize
            || c.output.try_send(bytes).is_err()
        {
            self.close(id);
        }
    }
}
