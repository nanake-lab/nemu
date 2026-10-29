//! P5-12: pure numeric J and the body readout (v2.0 §9).
//!
//! This module deliberately has NO production envelope/persistence entry point.
//! The IDL defines ModulationEnvelopeV2, but not the referenced direction,
//! validation, readout resources or effect receipt. These in-process views do
//! not define a second wire format or attest that a model passed validation.
//! A trusted resource resolver and the missing contracts must precede runtime
//! use; see reports/P5-12/BLOCKED.md and request P5-12-resources-and-receipt.
//!
//! Input permissions: stored θ, x, R, explicit anchor/time and empty validity;
//! immutable resource views. No text, H, z, L, model, storage or clock access.

mod compile;
mod config;
mod numeric;
mod readout;
mod types;

pub use compile::{compile_numeric, rehearsal_without_fast};
pub use config::{CarrierConfig, DirectionBinding, DirectionInput, LearnedDirectionInput};
pub use readout::BodyReadoutConfig;
pub use types::*;

#[cfg(test)]
mod tests;
