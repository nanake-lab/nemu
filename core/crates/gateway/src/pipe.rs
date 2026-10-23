//! Current-user, local-only pipe binding. No production bootstrap is exported:
//! the IDL authentication/upload adapter is a prerequisite, not a guessed RPC.
use crate::{
    Error, Result,
    framing::{FrameConfig, Framed},
};
use core_store::{LineageKind, Store};
use core_types::generated::Lineage;
use tokio::net::windows::named_pipe::NamedPipeServer;

pub struct Listener {
    server: NamedPipeServer,
    config: FrameConfig,
}
pub fn current_pipe_name() -> Result<String> {
    Ok(format!(
        r"\\.\pipe\nemu-core-{}",
        crate::windows::current_sid()?
    ))
}
impl Listener {
    pub fn bind<L: LineageKind>(_store: &Store<L>, config: FrameConfig) -> Result<Self> {
        Self::for_lineage(L::KIND, config, "")
    }
    pub(crate) fn for_lineage(lineage: Lineage, config: FrameConfig, suffix: &str) -> Result<Self> {
        if !matches!(lineage, Lineage::REHEARSAL | Lineage::LIFE) {
            return Err(Error::Rejected("experiment lineage has no named pipe"));
        }
        let name = format!(
            r"\\.\pipe\nemu-core-{}{suffix}",
            crate::windows::current_sid()?
        );
        Ok(Self {
            server: crate::windows::create_pipe(&name, config)?,
            config,
        })
    }
    /// Raw stream for the future generated authentication adapter only. Calling
    /// this does not grant authority to ingest; validate_event requires a token
    /// capability and ingest_validated retains the core-store writer boundary.
    pub async fn accept(&mut self) -> Result<Framed<&mut NamedPipeServer>> {
        tokio::time::timeout(self.config.deadline, self.server.connect())
            .await
            .map_err(|_| Error::Timeout)??;
        Framed::new(&mut self.server, self.config)
    }
    /// End an authenticated or failed session, discarding unread bytes, while
    /// retaining the first-instance handle and its ACL for the next connection.
    pub fn disconnect(&mut self) -> Result<()> {
        Ok(self.server.disconnect()?)
    }
    #[cfg(test)]
    pub(crate) fn raw_handle(&self) -> std::os::windows::io::RawHandle {
        use std::os::windows::io::AsRawHandle;
        self.server.as_raw_handle()
    }
}
