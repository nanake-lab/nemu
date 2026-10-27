//! Effects of learned tendencies, simplified to Echo Core v2.0 (P5-27).
//!
//! Two pure phases (ADR 0008 §2):
//!
//! - [`compile_authorized_effects`] reads the stored R and θ once and returns
//!   E_eff: for every tendency in R⁺ (π > ½) its strength s_k = max(0, 2π − 1),
//!   typical magnitude r_k, direction d_k, concept reference and whether its
//!   model direction has been validated, together with θ. R does not decay with
//!   time in v2.0, so the time projection Φ_R is the identity and the result
//!   does not depend on the processing time.
//! - [`evaluate_effects`] combines E_eff with the situation match μ(e) of one
//!   event and returns f_k(e) = s_k μ_k(e) r_k d_k and D_k(e) = s_k μ_k(e) for
//!   the fast state F (P5-10, v2.0 §6).
//!
//! Superseded by v2.0 §15 and removed here: the q envelope, validity
//! contracts, takeover, qualification, the scope DSL and its overlap check.

mod compile;
mod evaluate;
mod types;

#[cfg(test)]
mod tests;

pub use compile::{compile_authorized_effects, project_r, strength};
pub use evaluate::evaluate_effects;
pub use types::*;
