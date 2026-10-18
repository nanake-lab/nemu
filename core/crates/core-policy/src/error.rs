//! Errors for scope handling. All failures are explicit; nothing defaults to
//! "matches" or "disjoint".

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyError {
    InvalidCatalog(String),
    UnknownField(String),
    /// Field exists but is not on the endpoint's whitelist (O 3.3).
    FieldNotAllowed {
        field: String,
        endpoint: u32,
    },
    /// Atom kind does not match the field kind.
    KindMismatch(String),
    /// Value outside the declared domain or precision.
    OutOfDomain(String),
    EmptySet(String),
    EmptyInterval(String),
    EmptyCombination,
    LimitExceeded(&'static str),
    Encoding(String),
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCatalog(s) => write!(f, "invalid catalog: {s}"),
            Self::UnknownField(s) => write!(f, "unknown field: {s}"),
            Self::FieldNotAllowed { field, endpoint } => {
                write!(f, "field {field} not allowed for endpoint {endpoint}")
            }
            Self::KindMismatch(s) => write!(f, "kind mismatch: {s}"),
            Self::OutOfDomain(s) => write!(f, "out of domain: {s}"),
            Self::EmptySet(s) => write!(f, "empty set: {s}"),
            Self::EmptyInterval(s) => write!(f, "empty interval: {s}"),
            Self::EmptyCombination => write!(f, "empty boolean combination"),
            Self::LimitExceeded(s) => write!(f, "limit exceeded: {s}"),
            Self::Encoding(s) => write!(f, "encoding: {s}"),
        }
    }
}

impl std::error::Error for PolicyError {}
