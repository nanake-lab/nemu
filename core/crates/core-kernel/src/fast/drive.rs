//! D(e), the drive of one event (v2.0 §6; ADR 0026 §6.3).
//!
//! Nothing in this file reads x̄: the drive depends only on θ, the effects of
//! R⁺, the event's appraisal and match, and its class. Together with the
//! coordinate-wise clip in `apply_drive` this makes every event map
//! non-expansive in the ∞-norm (ADR 0008 §1).

use super::types::{Drive, DriveLaw, EventClass, EventInput, FastError, LearnedValue};
use crate::effects::{EffectiveEffects, Evaluation};
use core_types::semantic::tendency_order;
use std::cmp::Ordering;

fn plus_zero(v: f64) -> f64 {
    if v == 0.0 { 0.0 } else { v }
}

/// κ^n by binary exponentiation in a fixed order. Cross-target compatibility
/// still requires golden-vector/replay evidence (S §6). Values that underflow
/// binary64 become 0: the weight of a very
/// late internal record is then exactly zero, which keeps D bounded and the
/// event map non-expansive.
pub fn kappa_power(kappa: f64, n: u64) -> f64 {
    let mut result = 1.0_f64;
    let mut base = kappa;
    let mut e = n;
    while e > 0 {
        if e & 1 == 1 {
            result *= base;
        }
        e >>= 1;
        if e > 0 {
            base *= base;
        }
    }
    plus_zero(result)
}

/// The internal count after an event of this class (ADR 0026 §6.3): a new
/// exogenous event resets it to 0, an internal record committed at count n
/// leaves n + 1, everything else keeps it.
pub fn next_internal_count(class: EventClass, n: u64) -> u64 {
    match class {
        EventClass::NewExogenous => 0,
        EventClass::InternalRecord { n: at } => at.saturating_add(1),
        EventClass::OtherExternal | EventClass::ActionResult { .. } => n,
    }
}

/// w for the empathy channel (v2.0 §6 reference): (1 + π)/2 of the closeness
/// tendency toward the person involved, clipped to [w_min, 1]. With several
/// people involved the closest one counts (the largest weight); with nobody
/// involved w = 1. Reading registered with this card (V2-33 left the choice
/// to P5-10).
pub fn empathy_weight(closeness_pis: &[f64], w_min: f64) -> Result<f64, FastError> {
    if !(w_min.is_finite() && w_min > 0.0 && w_min <= 1.0) {
        return Err(FastError::OutOfRange("w_min"));
    }
    let mut best: Option<f64> = None;
    for pi in closeness_pis {
        if !pi.is_finite() {
            return Err(FastError::NonFinite("closeness"));
        }
        if !(*pi > -1.0 && *pi < 1.0) {
            return Err(FastError::OutOfRange("closeness"));
        }
        let w = ((1.0 + pi) / 2.0).clamp(w_min, 1.0);
        best = Some(best.map_or(w, |b: f64| b.max(w)));
    }
    Ok(best.unwrap_or(1.0))
}

fn class_weight(class: EventClass, kappa: f64) -> f64 {
    match class {
        EventClass::NewExogenous | EventClass::OtherExternal => 1.0,
        EventClass::InternalRecord { n } => kappa_power(kappa, n),
        EventClass::ActionResult { n_issued } => kappa_power(kappa, n_issued),
    }
}

fn check_law(law: &DriveLaw) -> Result<(), FastError> {
    let ok = law.g_max.is_finite()
        && law.g_max > 0.0
        && law.r_max.is_finite()
        && law.r_max >= 0.0
        && law.w_min.is_finite()
        && law.w_min > 0.0
        && law.w_min <= 1.0
        && law.kappa.is_finite()
        && law.kappa > 0.0
        && law.kappa < 1.0;
    if ok {
        Ok(())
    } else {
        Err(FastError::OutOfRange("law"))
    }
}

/// D(e) (v2.0 §6), weighted by the event class (ADR 0026 §6.3):
///
/// - D_i = c · (g_i · w_i · (a_i − b_i) + Σ_{k∈R⁺} f_{k,i}), i = 1…4;
/// - D_k = c · s_k · μ_k, k ∈ R⁺;
///
/// with c = 1 for external events, κ^n for an internal record and κ^{n_issued}
/// for an action result. w_i = 1 except the empathy channel. When the
/// appraisal is missing the g(a − b) term is 0; the effects of R⁺ still apply.
///
/// Before weighting, every channel is checked against the conservative bound
/// η_x = max(1, 2 g_max + r_max · max_i Σ_k |d_{k,i}|), summing absolute
/// values so that cancellation cannot hide an oversized term (O §5.2). A drive
/// over the bound is refused, never scaled down.
pub fn compute_drive(
    effects: &EffectiveEffects,
    evaluation: &Evaluation,
    input: &EventInput,
    law: &DriveLaw,
) -> Result<Drive, FastError> {
    check_law(law)?;
    let aligned = evaluation.drives.len() == effects.effects.len()
        && evaluation
            .drives
            .iter()
            .zip(&effects.effects)
            .all(|(d, e)| d.tendency_id == e.tendency_id);
    if !aligned {
        return Err(FastError::Misaligned);
    }
    if effects.effects.iter().any(|e| e.tendency_id.is_empty())
        || !effects.effects.windows(2).all(|pair| {
            tendency_order(&pair[0].tendency_id, &pair[1].tendency_id) == Ordering::Less
        })
    {
        return Err(FastError::Misaligned);
    }
    let w = input.empathy_weight;
    if !w.is_finite() {
        return Err(FastError::NonFinite("empathy_weight"));
    }
    if !(law.w_min..=1.0).contains(&w) {
        return Err(FastError::OutOfRange("empathy_weight"));
    }
    if let Some(a) = input.appraisal {
        if a.iter().any(|v| !v.is_finite()) {
            return Err(FastError::NonFinite("appraisal"));
        }
        if a.iter().any(|v| !(-1.0..=1.0).contains(v)) {
            return Err(FastError::OutOfRange("appraisal"));
        }
    }
    let theta = &effects.theta;
    let weights = [1.0, w, 1.0, 1.0];
    for i in 0..4 {
        if !theta.b[i].is_finite() || !theta.g[i].is_finite() {
            return Err(FastError::NonFinite("theta_drive"));
        }
        if !(-1.0..=1.0).contains(&theta.b[i]) || theta.g[i] <= 0.0 {
            return Err(FastError::OutOfRange("theta_drive"));
        }
        if theta.g[i] > law.g_max {
            return Err(FastError::DriveOverBound { channel: i });
        }
    }
    for e in &effects.effects {
        if !e.r.is_finite() || e.d.iter().any(|v| !v.is_finite()) {
            return Err(FastError::NonFinite("effect"));
        }
        if !(0.0..=law.r_max).contains(&e.r) {
            return Err(FastError::OutOfRange("r"));
        }
    }

    let mut direction_mass = 0.0_f64;
    for i in 0..4 {
        let mass = effects
            .effects
            .iter()
            .fold(0.0, |acc, e| acc + e.d[i].abs());
        if !mass.is_finite() {
            return Err(FastError::NonFinite("direction_mass"));
        }
        direction_mass = direction_mass.max(mass);
    }
    let eta = 2.0 * law.g_max + law.r_max * direction_mass;
    if !eta.is_finite() {
        return Err(FastError::NonFinite("eta"));
    }
    let eta = eta.max(1.0);

    let c = class_weight(input.class, law.kappa);
    if !(c.is_finite() && (0.0..=1.0).contains(&c)) {
        return Err(FastError::OutOfRange("class_weight"));
    }
    let mut nature = [0.0; 4];
    for i in 0..4 {
        let base = match input.appraisal {
            Some(a) => (theta.g[i] * weights[i]) * (a[i] - theta.b[i]),
            None => 0.0,
        };
        let mut abs_sum = base.abs();
        let mut sum = 0.0;
        // The checked contributions and the applied sum share this exact
        // canonical tendency order. The redundant DTO sum is only verified.
        for d in &evaluation.drives {
            if !d.f[i].is_finite() {
                return Err(FastError::NonFinite("contribution"));
            }
            abs_sum += d.f[i].abs();
            sum = plus_zero(sum + d.f[i]);
        }
        if !abs_sum.is_finite() || abs_sum > eta {
            return Err(FastError::DriveOverBound { channel: i });
        }
        if !evaluation.f_sum[i].is_finite() {
            return Err(FastError::NonFinite("f_sum"));
        }
        if sum.to_bits() != plus_zero(evaluation.f_sum[i]).to_bits() {
            return Err(FastError::InconsistentSum { channel: i });
        }
        let total = base + sum;
        let value = c * total;
        if !total.is_finite() || !value.is_finite() {
            return Err(FastError::NonFinite("drive"));
        }
        if total.abs() > eta || value.abs() > eta {
            return Err(FastError::DriveOverBound { channel: i });
        }
        nature[i] = plus_zero(value);
    }
    let learned = evaluation
        .drives
        .iter()
        .enumerate()
        .map(|(k, d)| {
            let value = c * d.drive;
            if !d.drive.is_finite() || !value.is_finite() {
                return Err(FastError::NonFinite("learned_drive"));
            }
            if !(0.0..=1.0).contains(&d.drive) || !(0.0..=1.0).contains(&value) || value > eta {
                return Err(FastError::DriveOverBound { channel: 4 + k });
            }
            Ok(LearnedValue {
                tendency_id: d.tendency_id.clone(),
                value: plus_zero(value),
            })
        })
        .collect::<Result<Vec<_>, FastError>>()?;
    Ok(Drive { nature, learned })
}
