//! CompileAuthorizedEffects for v2.0 (ADR 0008 §2; v2.0 §5.2, §6, §15).

use std::collections::BTreeSet;

use core_types::semantic::{UNIT_NORM_TOLERANCE, tendency_order};

use super::types::{
    CompileError, EffectiveEffects, ProcessTime, StoredR, StoredTendency, StoredTheta,
    TendencyEffect, Validity,
};

/// Strength s_k = max(0, 2π − 1) (v2.0 §5.2). Negative zero is normalized to
/// +0 so equal results have equal bytes.
pub fn strength(pi: f64) -> f64 {
    let s = 2.0 * pi - 1.0;
    if s > 0.0 { s } else { 0.0 }
}

/// Time projection Φ_R(Δ; R). R does not decay with time in v2.0 (LP5), so
/// the projection is the identity. It exists so the composition law of
/// ADR 0008 §2, Compile(Φ_R(Δ; R)) = Compile(R), is stated and tested.
pub fn project_r(r: &StoredR, _delta_ns: u64) -> StoredR {
    r.clone()
}

/// Compile the effective effects from the stored R and θ at one processing
/// time (ADR 0008 §2). Pure: it only borrows its inputs, reads no clock and
/// does not depend on `process_time`, since nothing in v2.0 expires. The
/// validity snapshot is empty in v2.0.
///
/// Every stored tendency is checked, including candidates, because a stored
/// value outside its domain is a fault of the snapshot whether or not it has
/// an effect now. Only tendencies with s_k > 0 (π > ½, R⁺) enter E_eff (I06).
pub fn compile_authorized_effects(
    stored_r: &StoredR,
    stored_theta: &StoredTheta,
    _process_time: ProcessTime,
    _validity: &Validity,
) -> Result<EffectiveEffects, CompileError> {
    check_theta(stored_theta)?;
    let mut seen = BTreeSet::new();
    for t in &stored_r.tendencies {
        check_tendency(t)?;
        if !seen.insert(t.tendency_id.as_str()) {
            return Err(CompileError::DuplicateTendency {
                tendency_id: t.tendency_id.clone(),
            });
        }
    }
    let mut effects: Vec<TendencyEffect> = stored_r
        .tendencies
        .iter()
        .filter_map(|t| {
            let s = strength(t.pi);
            (s > 0.0).then(|| TendencyEffect {
                tendency_id: t.tendency_id.clone(),
                concept: t.concept.clone(),
                s,
                r: t.r,
                d: t.d,
                direction_validated: t.direction_validated,
            })
        })
        .collect();
    effects.sort_by(|a, b| tendency_order(&a.tendency_id, &b.tendency_id));
    Ok(EffectiveEffects {
        effects,
        theta: *stored_theta,
    })
}

fn check_tendency(t: &StoredTendency) -> Result<(), CompileError> {
    if t.tendency_id.is_empty() || t.concept.is_empty() {
        return Err(CompileError::EmptyIdentifier);
    }
    let id = || Some(t.tendency_id.clone());
    if !t.pi.is_finite() {
        return Err(CompileError::NonFinite {
            tendency_id: id(),
            field: "pi",
        });
    }
    if !t.r.is_finite() {
        return Err(CompileError::NonFinite {
            tendency_id: id(),
            field: "r",
        });
    }
    if t.d.iter().any(|x| !x.is_finite()) {
        return Err(CompileError::NonFinite {
            tendency_id: id(),
            field: "d",
        });
    }
    if !(t.pi > -1.0 && t.pi < 1.0) {
        return Err(CompileError::PiOutOfRange {
            tendency_id: t.tendency_id.clone(),
        });
    }
    if t.r < 0.0 {
        return Err(CompileError::NegativeMagnitude {
            tendency_id: t.tendency_id.clone(),
        });
    }
    // Squares summed in the fixed channel order, as core-types does (V2-32).
    let norm = t.d.iter().map(|x| x * x).sum::<f64>().sqrt();
    if (norm - 1.0).abs() > UNIT_NORM_TOLERANCE {
        return Err(CompileError::DirectionNotUnit {
            tendency_id: t.tendency_id.clone(),
        });
    }
    Ok(())
}

fn check_theta(theta: &StoredTheta) -> Result<(), CompileError> {
    let all = |v: &[f64; 4]| v.iter().all(|x| x.is_finite());
    if !all(&theta.b) {
        return Err(CompileError::NonFinite {
            tendency_id: None,
            field: "b",
        });
    }
    if !all(&theta.g) {
        return Err(CompileError::NonFinite {
            tendency_id: None,
            field: "g",
        });
    }
    if !all(&theta.lambda) {
        return Err(CompileError::NonFinite {
            tendency_id: None,
            field: "lambda",
        });
    }
    if theta.b.iter().any(|b| !(-1.0..=1.0).contains(b)) {
        return Err(CompileError::ThetaOutOfRange { field: "b" });
    }
    // g ∈ [g_min, g_max] with g_min > 0 and λ ≥ λ_min > 0 (v2.0 §8). The law
    // parameters are not an input here, so only positivity is checked; the
    // finer bounds belong to the law's own checks (V2-32).
    if theta.g.iter().any(|g| *g <= 0.0) {
        return Err(CompileError::ThetaOutOfRange { field: "g" });
    }
    if theta.lambda.iter().any(|l| *l <= 0.0) {
        return Err(CompileError::ThetaOutOfRange { field: "lambda" });
    }
    Ok(())
}
