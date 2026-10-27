use std::fmt;
#[derive(Debug)]
pub enum Error {
    Store(core_store::Error),
    Sql(rusqlite::Error),
    Invalid(String),
    Io(std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(e) => write!(f, "{e}"),
            Self::Sql(e) => write!(f, "derived index: {e}"),
            Self::Invalid(e) => write!(f, "fact query: {e}"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for Error {}
impl From<core_store::Error> for Error {
    fn from(e: core_store::Error) -> Self {
        Self::Store(e)
    }
}
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sql(e)
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::Invalid(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Invalid(e.to_string())
    }
}
