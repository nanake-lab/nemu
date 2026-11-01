use core_store::LineageKind;
use core_types::generated::*;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};
use zeroize::Zeroize;

/// No Debug/Serialize. Credentials are consumed only by the actor and wiped on
/// every normal, rejected, cancelled or channel-failure exit.
pub(crate) struct Authentication(pub TerminalAuthenticate);
impl Drop for Authentication {
    fn drop(&mut self) {
        self.0.token.0.zeroize();
    }
}

pub(crate) enum Input {
    Authenticate {
        connection: u64,
        request: Authentication,
        reply: oneshot::Sender<TerminalAuthResult>,
        output: mpsc::Sender<TerminalResponse>,
    },
    Request {
        connection: u64,
        request: Box<TerminalRequest>,
    },
    Fault {
        connection: u64,
    },
    Closed {
        connection: u64,
    },
}
pub(crate) struct Connection<L: LineageKind> {
    pub session: TerminalSession,
    pub auth: crate::AuthenticatedSession,
    pub output: mpsc::Sender<TerminalResponse>,
    pub uploads: crate::uploads::Uploads<L>,
    pub created: Instant,
}
pub(crate) struct Pending {
    pub connection: u64,
    pub request: TerminalRequest,
    /// Bounded correlation state, captured from the actual published head after
    /// a serial core step. Lost with the connection; never a persistent journal.
    pub published_commit: Option<String>,
}
