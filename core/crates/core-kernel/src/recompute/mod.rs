//! Version-independent, pure H -> growth staging (P5-26).
//!
//! These are in-process projections, NOT a second H/L wire format. A trusted
//! caller supplies immutable committed history, accepted interpretations and
//! resolved resources. Nothing here opens storage, calls a model, reads a clock,
//! sends notifications or publishes a snapshot. See the integration request.
//!
//! A second call to `replay` is NOT an independent numerical implementation.
//! Independent verification is explicitly unavailable until a complete second
//! evidence/G/C implementation is supplied. Recorded check outcomes, including
//! unavailability, are replayed without rerunning that external service.

mod engine;
mod inspection;
mod types;

pub use engine::{Progress, replay, start, step};
pub use inspection::{IndependentResult, ProbeReadout, ProbeSet, compare_probes, verify_candidate};
pub use types::*;

#[cfg(test)]
mod tests;
