//! P5-24: pure assembly of a supplied, governed H prefix.
//!
//! This module does not authenticate H, manufacture semantic properties, call
//! models, or publish state. The owning storage/semantic layers must supply the
//! recorded projections described in `types`. Missing or ambiguous inputs fail
//! closed; they are never inferred from text, IDs, appraisals or current state.
//! Production registration and upstream gaps: tasks/requests/P5-24-inputs.md.

mod assemble;
mod candidates;
mod eligibility;
mod groups;
mod records;
mod types;

pub use assemble::assemble;
pub use types::*;

#[cfg(test)]
mod tests;
