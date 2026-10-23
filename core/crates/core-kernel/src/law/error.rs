//! Explicit faults of the law engine. Every numeric or structural problem is
//! reported, never clamped away (v2.0 §2.2; AGENTS.md §5 numerics).

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum LawError {
    NumericFault(&'static str),
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
    /// Rounding pushed π_k out of (−1, 1): the fixed anchor has become
    /// negligible next to V_k in binary64 (ADR 0019).
    /// Reported at the step that would produce it, never stored (D9).
    PrecisionExhausted(String),
    /// R is over capacity and holds no candidate (π ≤ ½) to remove (v2.0 §12; V2-43).
    CapacityExhausted {
        capacity: u32,
        personality: usize,
    },
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
            Self::NumericFault(m) => write!(f, "numeric fault: {m}"),
            Self::InvalidParameter(m) => write!(f, "invalid law parameter: {m}"),
            Self::InvalidInput(m) => write!(f, "invalid law input: {m}"),
            Self::InconsistentState(m) => write!(f, "inconsistent law state: {m}"),
            Self::NonFinite(m) => write!(f, "nonfinite value: {m}"),
            Self::Overflow(m) => write!(f, "count overflow: {m}"),
            Self::PrecisionExhausted(m) => write!(f, "binary64 precision exhausted: {m}"),
            Self::CapacityExhausted {
                capacity,
                personality,
            } => write!(
                f,
                "R over capacity {capacity} with {personality} personality tendencies and no candidate to remove"
            ),
            Self::WeightAboveCap {
                group_id,
                tendency_id,
            } => write!(f, "group {group_id} weighs above w_max for {tendency_id}"),
        }
    }
}

impl std::error::Error for LawError {}

pub type LawResult<T> = Result<T, LawError>;
