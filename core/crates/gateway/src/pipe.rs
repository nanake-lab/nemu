//! Current-user, local-only pipe binding for the generated ingress service.
use crate::{
    Error, Result,
    framing::{FrameConfig, Framed},
};
use core_store::{LineageKind, Store};
use core_types::generated::Lineage;
use tokio::net::windows::named_pipe::NamedPipeServer;

pub struct Listener {
    server: NamedPipeServer,
    name: String,
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
            server: crate::windows::create_pipe(&name, config, true)?,
            name,
            config,
        })
    }
    /// Raw stream for the generated authentication adapter. Calling
    /// this does not grant authority to ingest; validate_event requires a token
    /// capability and ingest_validated retains the core-store writer boundary.
    pub async fn accept(&mut self) -> Result<Framed<&mut NamedPipeServer>> {
        tokio::time::timeout(self.config.deadline, self.server.connect())
            .await
            .map_err(|_| Error::Timeout)??;
        Framed::new(&mut self.server, self.config)
    }
    /// Rotate before handing a connection to its I/O task. Name ownership and
    /// the explicit SID ACL remain continuous across concurrent connections.
    pub async fn accept_owned(&mut self) -> Result<Framed<NamedPipeServer>> {
        tokio::time::timeout(self.config.deadline, self.server.connect())
            .await
            .map_err(|_| Error::Timeout)??;
        let next = crate::windows::create_pipe(&self.name, self.config, false)?;
        let connected = std::mem::replace(&mut self.server, next);
        Framed::new(connected, self.config)
    }
    /// End an authenticated or failed session. Create the next ACL-protected
    /// instance before closing this one, so ownership of the pipe name never
    /// lapses. Fresh handles prevent stale overlapped reads crossing sessions.
    pub fn disconnect(&mut self) -> Result<()> {
        let next = crate::windows::create_pipe(&self.name, self.config, false)?;
        self.server.disconnect()?;
        self.server = next;
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn raw_handle(&self) -> std::os::windows::io::RawHandle {
        use std::os::windows::io::AsRawHandle;
        self.server.as_raw_handle()
    }
}
