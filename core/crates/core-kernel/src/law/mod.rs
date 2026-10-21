//! G: the law of tendencies, a pure evidence count (v2.0 §5; P5-23).
//!
//! (R, b, σ̂, evidence groups in H order; 𝔏_v) → R′ and staged L entries.
//! G reads evidence and the statistics derived from it (b, σ̂ and the competing
//! tendencies' π); it never reads x, m, the gains g or the recovery rates λ
//! (LP8, I10). No clock, randomness, storage, network or model access.
//!
//! The canonical calculation order and the readings this card fixes are in
//! `docs/traceability/P5-23.md` §1 (D1–D11) and the difference register
//! V2-38 to V2-48. Parts that need the R fields missing from the IDL (V2-48)
//! are not implemented yet (`reports/P5-23/BLOCKED.md`).

mod error;
mod num;
mod params;
mod recurrence;
mod support;

#[cfg(test)]
mod tests;

pub use error::{LawError, LawResult};
pub use num::{Nature4, exact_mean};
pub use params::Law;
pub use recurrence::{
    MAX_GROUPS, Origin, check_stored, consistent_and_contrary, evidence_amount, initial,
    is_personality, pi, prior, record, rho, strength,
};
pub use support::{
    GroupSupport, deviation, experience, fluctuation, mean_deviation, teaching, weight,
};
