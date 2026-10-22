use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Sql(rusqlite::Error),
    Contract(String),
    Rejected(&'static str),
}
pub type Result<T> = std::result::Result<T, Error>;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
