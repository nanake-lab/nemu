//! W04: effect resolution and authorisation (O 3, 12.1; S 12).
//!
//! Two pure phases: `compile_authorized_effects` turns the committed R/θ
//! views into the effective set at one processing time; `evaluate_effects`
//! evaluates that set against one endpoint's typed context. In v1.0 R is
//! empty and θ holds only genesis, but these functions are complete and are
//! exercised with structural fixtures.

mod admission;
mod compile;
mod dag;
mod evaluate;
mod params;
mod types;

#[cfg(test)]
mod tests;

pub use admission::{
    AdmissionRejection, CandidateEffect, RouteFault, admit_candidate, audit_drive_route,
};
pub use compile::compile_authorized_effects;
pub use dag::DagError;
pub use evaluate::{
    ContextError, EvaluateError, Evaluation, ReceiptEntry, TypedContext, evaluate_effects,
};
pub use params::{EffectParameters, PARAMETERS_TAG};
pub use types::*;
