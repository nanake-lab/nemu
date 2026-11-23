#![deny(unsafe_code)]
//! W18 authenticated Windows gateway over generated ingress contracts/core-store.
//! Business authority and core transitions are supplied by a trusted CoreHandler.
mod auth;
#[cfg(windows)]
mod authentication;
#[cfg(windows)]
mod connection;
#[cfg(windows)]
mod dispatch;
pub mod framing;
#[cfg(windows)]
pub mod gateway;
pub mod hooks;
mod ingress;
#[cfg(windows)]
mod network;
#[cfg(windows)]
pub mod pipe;
#[cfg(windows)]
mod progress;
pub mod terminal;
#[cfg(windows)]
mod uploads;
mod validation;
#[cfg(windows)]
#[allow(unsafe_code)]
mod windows;

pub use auth::{AuthenticatedSession, TerminalToken};
pub use ingress::ingest_validated;
pub use validation::{Grants, TerminalPayload, ValidatedEvent, validate_event};

pub enum Error {
    Rejected(&'static str),
    Contract,
    Io(std::io::Error),
    Timeout,
    Store(core_store::Error),
    Disconnected,
}
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never forward parser/store diagnostics, payloads, or credentials to logs.
        f.write_str(match self {
            Self::Rejected(reason) => reason,
            Self::Contract => "invalid contract message",
            Self::Io(_) => "gateway I/O failed",
            Self::Timeout => "gateway deadline exceeded",
            Self::Store(_) => "gateway storage operation failed",
            Self::Disconnected => "terminal disconnected",
        })
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<core_store::Error> for Error {
    fn from(error: core_store::Error) -> Self {
        Self::Store(error)
    }
}
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests;
