use std::fmt;

use core_types::generated::{BodyReadoutV2, LearnedCoefficient, NatureReal, ObjectRef};

use crate::effects::{CompileError, ProcessTime, StoredR, StoredTheta, Validity};
use crate::fast::{FastError, FastState};

/// All fields belong to one committed snapshot. The outer runtime, not J,
/// supplies and verifies that snapshot. Projection is always from its anchor.
#[derive(Clone, Copy, Debug)]
pub struct JStateView<'a> {
    pub snapshot_id: &'a str,
    pub anchor_time: ProcessTime,
    pub theta: &'a StoredTheta,
    pub x: &'a FastState,
    pub r: &'a StoredR,
    pub process_time: ProcessTime,
    pub validity: &'a Validity,
}

/// Versions from Ω. ObjectRefs are the existing generated IDL type.
#[derive(Clone, Debug, PartialEq)]
pub struct CarrierResources {
    pub carrier: ObjectRef,
    pub model: ObjectRef,
    pub tokenizer: ObjectRef,
    pub direction_set: ObjectRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilationMode {
    Connected,
    RehearsalWithoutFast,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExclusionReason {
    /// The effect view says P5-31 validation has not passed.
    Unvalidated,
    /// R⁺ says validated, but this carrier's resolved set has no direction.
    MissingDirection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExcludedDirection {
    pub tendency_id: String,
    pub reason: ExclusionReason,
}

/// Local numeric trace only. It is NOT an authority object or a carrier
/// receipt and has no Serialize/WireMessage implementation or invented digest.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericReceipt {
    pub snapshot_id: String,
    pub anchor_time: ProcessTime,
    pub projected_at: ProcessTime,
    pub mode: CompilationMode,
    pub resources: CarrierResources,
    pub combination_validation: ObjectRef,
    /// Fixed nature order followed by canonical learned tendency order.
    pub included: Vec<IncludedDirection>,
    pub excluded: Vec<ExcludedDirection>,
    pub body_readout_version: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IncludedDirection {
    /// None is a nature channel, identified by its position in `included`.
    pub tendency_id: Option<String>,
    pub direction: ObjectRef,
    pub validation: ObjectRef,
    pub beta: f64,
    pub coefficient: f64,
}

/// Unpublished result of the numeric stage, ready for a future typed receipt
/// publisher. Coefficients use generated types, never a parallel wire schema.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericCompilation {
    pub nature_coefficients: NatureReal,
    pub learned_coefficients: Vec<LearnedCoefficient>,
    /// Conservative sum of absolute β for the included directions, not the
    /// magnitude of their signed sum. Applies for every x in the box.
    pub total_bound: f64,
    pub body: BodyReadoutV2,
    pub receipt: NumericReceipt,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JFault {
    Effects(CompileError),
    Fast(FastError),
    InvalidConfig(&'static str),
    Codec(String),
    TimeBeforeAnchor,
    EmptySnapshotId,
    ResourceMismatch,
    DuplicateDirection(String),
    ConceptMismatch(String),
    BetaOutsideValidation,
    TotalBudgetExceeded,
    NonFinite(&'static str),
    RehearsalOnly,
}

impl fmt::Display for JFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for JFault {}
