//! EvaluateEffects for v2.0 (v2.0 §6).

use std::collections::BTreeMap;

use super::types::{EffectiveEffects, EvaluateError, Evaluation, SituationMatch, TendencyDrive};

/// Combine E_eff with the situation match μ(e) of one event:
/// f_k(e) = s_k μ_k(e) r_k d_k and D_k(e) = s_k μ_k(e) for every k ∈ R⁺.
///
/// The products are taken in the fixed order ((s·μ)·r)·d_i, and Σ_k f_k is
/// summed in canonical tendency order, so the result is bit-for-bit
/// reproducible. Pure: it borrows its inputs and writes nothing back (I21).
pub fn evaluate_effects(
    effects: &EffectiveEffects,
    situation: &SituationMatch,
) -> Result<Evaluation, EvaluateError> {
    let mut mu_by_id: BTreeMap<&str, f64> = BTreeMap::new();
    for (id, mu) in &situation.mu {
        if !(mu.is_finite() && (0.0..=1.0).contains(mu)) {
            return Err(EvaluateError::MuOutOfRange {
                tendency_id: id.clone(),
            });
        }
        if mu_by_id.insert(id.as_str(), *mu).is_some() {
            return Err(EvaluateError::DuplicateMatch {
                tendency_id: id.clone(),
            });
        }
    }
    let mut drives = Vec::with_capacity(effects.effects.len());
    let mut f_sum = [0.0_f64; 4];
    for e in &effects.effects {
        // A listed −0 is the same match as 0; keep the stored bytes canonical.
        let mu = match mu_by_id.get(e.tendency_id.as_str()) {
            Some(m) if *m != 0.0 => *m,
            _ => 0.0,
        };
        let drive = e.s * mu;
        let scale = drive * e.r;
        let mut f = [0.0_f64; 4];
        for i in 0..4 {
            f[i] = plus_zero(scale * e.d[i]);
            f_sum[i] = plus_zero(f_sum[i] + f[i]);
        }
        drives.push(TendencyDrive {
            tendency_id: e.tendency_id.clone(),
            mu,
            f,
            drive: plus_zero(drive),
        });
    }
    Ok(Evaluation { drives, f_sum })
}

/// Normalize −0 to +0 (ADR 0019 §8: stored results carry no negative zero).
fn plus_zero(x: f64) -> f64 {
    if x == 0.0 { 0.0 } else { x }
}
