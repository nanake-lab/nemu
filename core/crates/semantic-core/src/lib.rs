#![forbid(unsafe_code)]
//! Deterministic IRL governance. Only the outer transaction publishes a stage.
//! Legacy graph and record preflight remains available for the original DTOs.

pub mod aliases;
pub mod graph;
pub mod irl;
mod irl_graph;
pub mod objects;
pub mod records;
pub mod registry;
pub mod source_rules;

use core_types::{WireMessage, contract::Limits};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Schema,
    Budget,
    Binding,
    Object,
    Duplicate,
    Reference,
    Source,
    Epistemic,
    Concept,
    AliasCycle,
    Numeric,
    MissingContract,
    UnsupportedSource,
}
pub type Result<T> = std::result::Result<T, Fault>;

impl Fault {
    /// Existing formal failure status for the owning transaction to record.
    /// Carries no fabricated z, model values, audit text, or commit capability.
    pub fn irl_status(self) -> core_types::generated::DegradedStatus {
        use core_types::generated::{DegradedStatus, FailureCode, FailureReason, RunStatus};
        let code = match self {
            Self::Budget => FailureCode::RESOURCE_LIMIT,
            Self::Binding => FailureCode::VERSION_MISMATCH,
            Self::Numeric => FailureCode::NUMERIC_INVALID,
            Self::MissingContract => FailureCode::DEPENDENCY_FAULT,
            _ => FailureCode::STRUCTURAL_FAULT,
        };
        DegradedStatus {
            schema_version: 1,
            states: vec![RunStatus::IRL_DEGRADED],
            reasons: vec![
                FailureReason {
                    schema_version: 1,
                    code: FailureCode::IRL_FAILED,
                    detail_ref: None,
                },
                FailureReason {
                    schema_version: 1,
                    code,
                    detail_ref: None,
                },
            ],
        }
    }
}

pub(crate) fn wire<T: WireMessage>(value: &T, limits: Limits) -> Result<Vec<u8>> {
    value.canonical_bytes(limits).map_err(|_| Fault::Schema)
}

#[cfg(test)]
mod adapter_tests;
#[cfg(test)]
mod irl_tests;
#[cfg(test)]
mod record_tests;
#[cfg(test)]
mod tests;
