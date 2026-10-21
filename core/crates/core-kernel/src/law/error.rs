//! Explicit faults of the law engine. Every numeric or structural problem is
//! reported, never clamped away (v2.0 §2.2; AGENTS.md §5 numerics).

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum LawError {
    /// A law parameter is outside what G can compute with (v2.0 §16).
    InvalidParameter(&'static str),
    /// An input value is outside its domain (a(e), μ, b, σ̂, d_k, …).
    InvalidInput(String),
    /// A stored tendency or group violates its own invariants.
    InconsistentState(String),
    /// A computed value is NaN or ±Inf.
    NonFinite(&'static str),
    /// A count would overflow.
    Overflow(&'static str),
    /// R is over capacity and holds no candidate (π ≤ ½) to remove (v2.0 §12; V2-43).
    CapacityExhausted { capacity: u32, personality: usize },
    /// The step needs R state the IDL does not hold yet (V2-48,
    /// `reports/P5-23/BLOCKED.md`). Never a silent approximation.
    ContractGap(&'static str),
    /// A single group's weight exceeds w_max (v2.0 §12; D8). Cannot happen
    /// with checked parameters; reported if it ever does.
    WeightAboveCap {
        group_id: String,
        tendency_id: String,
    },
}

impl fmt::Display for LawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameter(m) => write!(f, "invalid law parameter: {m}"),
            Self::InvalidInput(m) => write!(f, "invalid law input: {m}"),
            Self::InconsistentState(m) => write!(f, "inconsistent law state: {m}"),
            Self::NonFinite(m) => write!(f, "nonfinite value: {m}"),
            Self::Overflow(m) => write!(f, "count overflow: {m}"),
            Self::CapacityExhausted {
                capacity,
                personality,
            } => write!(
                f,
                "R over capacity {capacity} with {personality} personality tendencies and no candidate to remove"
            ),
            Self::ContractGap(m) => write!(f, "law state missing from the IDL (V2-48): {m}"),
            Self::WeightAboveCap {
                group_id,
                tendency_id,
            } => write!(f, "group {group_id} weighs above w_max for {tendency_id}"),
        }
    }
}

impl std::error::Error for LawError {}

pub type LawResult<T> = Result<T, LawError>;
