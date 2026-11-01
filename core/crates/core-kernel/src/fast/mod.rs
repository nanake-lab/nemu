//! The fast state x and its operator F (Echo Core v2.0 §6; ADR 0008 §1;
//! ADR 0026 §6.3). Pure functions only; wiring into `process_head` is P5-19.
//!
//! - [`FastState`]: X = [−1, 1]^{4 + |R⁺|}, the four nature channels then one
//!   component per tendency of R⁺ in canonical tendency order.
//! - [`project_time`]: Φ_x(Δ; x) = x_rest + e^{−ΛΔ}(x − x_rest), written as
//!   x + (−expm1(−λΔ))(x_rest − x) so that Δ = 0 is the identity bit for bit,
//!   then clipped to X (audit D-2).
//! - [`compute_drive`]: D(e) from θ, the effects of R⁺ (P5-27) and the event;
//!   its signature has no x̄, so the event map is non-expansive (ADR 0008 §1).
//!   Self-caused events are weighted by κ^n (ADR 0026 §6.3).
//! - [`apply_drive`]: F_in, x̃ = Proj_X(x̄ + D(e)).
//! - [`f_out`]: adds new learned components at 0 and drops departed ones.
//! - [`reset_to_init`], [`washout_sufficient`]: recovery after contamination
//!   (O §5.3); the runtime decides when to use them (P5-19).

mod drive;
mod recovery;
mod space;
mod types;

#[cfg(test)]
mod tests;

pub use drive::{compute_drive, empathy_weight, kappa_power, next_internal_count};
pub use recovery::{reset_to_init, washout_sufficient};
pub use space::{
    REPRESENTATION_VERSION, apply_drive, deserialize, distance, f_out, project_time, serialize,
    validate,
};
pub use types::*;
