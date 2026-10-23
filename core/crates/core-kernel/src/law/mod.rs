//! G: the law of tendencies, a pure evidence count (v2.0 §5; P5-23).
//!
//! (R, b, σ̂, evidence groups in H order; 𝔏_v) → R′ and staged L entries.
//! G reads evidence and the statistics derived from it (b, σ̂ and the competing
//! tendencies' π); it never reads x, m, the gains g or the recovery rates λ
//! (LP8, I10). No clock, randomness, storage, network or model access.
//!
//! ADR 0019 supersedes the old D2-D5 recurrence and ordering. Historical q
//! and explicit V use the in-process pending state until P5-34 is merged.
//! No pending state implements a wire protocol or opens authoritative storage.

mod audit;
mod candidates;
mod change_rate;
mod engine;
mod error;
mod fuse;
mod group;
mod input;
mod num;
mod params;
pub mod pending;
mod plan;
mod recurrence;
mod relation;
mod replay;
mod shares;
mod support;
mod validation;

#[cfg(test)]
mod tests;

pub use change_rate::{ChangeRateEstimator, EvidenceCounts, FlipFrequency};
pub use engine::{CapacityAudit, Output, g};
pub use error::{LawError, LawResult};
pub use fuse::{check_weight_cap, eviction_choice, exhausted};
pub use input::{
    Candidate, GroupContent, GroupItem, LawContext, Match, MatchTarget, RState, Step, ThetaView,
};
pub use num::{Nature4, exact_mean};
pub use params::Law;
pub use recurrence::{
    MAX_GROUPS, Origin, check_stored, consistent_and_contrary, evidence_amount, initial,
    is_personality, pi, prior, record, rho, strength,
};
pub use relation::{
    CLOSENESS_DIRECTION, Caretaker, RELATION_FORM, RelationConcepts, TRUST_DIRECTION,
    closeness_weight, initial_r, relation_tendency, relation_tendency_id, trust,
};
pub use shares::{ShareInput, shares};
pub use support::{
    GroupSupport, deviation, experience, fluctuation, mean_deviation, teaching, weight,
};
