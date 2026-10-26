//! Recovery after a hard fault left x contaminated (O §5.3). One cannot
//! subtract a past source from x, so recovery either resets x to the fixed
//! x_init or waits for a washout that provably meets the error bound. Both
//! break causal continuity, and the runtime records that (P5-19).

use super::space::validate;
use super::types::{ContinuityBreak, FastError, FastState, LearnedValue};
use crate::effects::EffectiveEffects;

/// Reset x to the fixed x_init: nature channels from genesis, every learned
/// component of the current R⁺ at 0. Returns the break to be recorded.
pub fn reset_to_init(
    x_init_nature: [f64; 4],
    effects: &EffectiveEffects,
) -> Result<(FastState, ContinuityBreak), FastError> {
    let x = FastState {
        nature: x_init_nature,
        learned: effects
            .effects
            .iter()
            .map(|e| LearnedValue {
                tendency_id: e.tendency_id.clone(),
                value: 0.0,
            })
            .collect(),
    };
    validate(&x)?;
    Ok((x, ContinuityBreak::Reset))
}

/// True when a washout of `elapsed_ns` under rates of at least `lambda_min`
/// has provably brought any contamination within `epsilon`: every coordinate
/// difference is at most 2 in X and shrinks at least by e^{−λ_min t} without
/// events, so the condition is 2·e^{−λ_min t} ≤ ε. Only valid when no event
/// drove x in the meantime and θ is safe; the caller checks that.
pub fn washout_sufficient(
    elapsed_ns: u64,
    lambda_min: f64,
    epsilon: f64,
) -> Result<bool, FastError> {
    if !(lambda_min.is_finite() && lambda_min > 0.0) {
        return Err(FastError::OutOfRange("lambda_min"));
    }
    if !(epsilon.is_finite() && epsilon > 0.0) {
        return Err(FastError::OutOfRange("epsilon"));
    }
    let t = elapsed_ns as f64 / 1e9;
    Ok(2.0 * libm::exp(-(lambda_min * t)) <= epsilon)
}
