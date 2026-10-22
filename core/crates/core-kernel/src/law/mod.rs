//! G: the law of tendencies, a pure evidence count (v2.0 §5; P5-23).
//!
//! (R, b, σ̂, evidence groups in H order; 𝔏_v) → R′ and staged L entries.
//! G reads evidence and the statistics derived from it (b, σ̂ and the competing
//! tendencies' π); it never reads x, m, the gains g or the recovery rates λ
//! (LP8, I10). No clock, randomness, storage, network or model access.
//!
//! The canonical calculation order and the readings this card fixes are in
//! `docs/traceability/P5-23.md` §1 (D1–D12) and the difference register
//! V2-38 to V2-49. R is the IDL `RSnapshotV2` with its `EvidenceGroupV2`
//! summaries (P5-32, ADR 0016); nothing here defines a stored or cross-process
//! type of its own.

mod engine;
mod error;
mod fuse;
mod group;
mod input;
mod num;
mod params;
mod plan;
mod recurrence;
mod relation;
mod replay;
mod shares;
mod support;

#[cfg(test)]
mod tests;

pub use engine::{Output, g};
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
