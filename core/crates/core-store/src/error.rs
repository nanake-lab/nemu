use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Sql(rusqlite::Error),
    Contract(String),
    Rejected(&'static str),
    /// Known no-D-write capacity rejection; I/O and SQL remain outcome-unknown.
    Backpressure(core_types::generated::IngressReason),
}
pub type Result<T> = std::result::Result<T, Error>;
impl Error {
    /// Only a proven no-D-write capacity refusal can become a retryable receipt.
    /// SQL, I/O and contract errors are deliberately not classified by text.
    pub fn safe_ingress_reason(&self) -> Option<core_types::generated::IngressReason> {
        match self {
            Self::Backpressure(reason) => Some(*reason),
            _ => None,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backpressure(reason) => write!(f, "ingress capacity: {reason:?}"),
            Self::Io(e) => write!(f, "storage I/O: {e}"),
            Self::Sql(e) => write!(f, "storage SQL: {e}"),
            Self::Contract(e) => write!(f, "contract: {e}"),
            Self::Rejected(e) => write!(f, "storage rejected: {e}"),
        }
    }
}
impl std::error::Error for Error {}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sql(e)
    }
}
impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::Contract(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Contract(e.to_string())
    }
}
pub(crate) fn require(ok: bool, reason: &'static str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::Rejected(reason))
    }
}
