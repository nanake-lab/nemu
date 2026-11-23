//! Support S_{c,k}, strength and weight of one evidence group for one tendency
//! (v2.0 §5.2; D3).
//!
//! A tendency predicts that in matching situations the nature reaction's
//! deviation from baseline points along d_k with magnitude about r̃_k. The
//! group supports k when the reaction reaches the prediction, is neutral at
//! half of it, and is contrary when it is absent or reversed; so a missing
//! reaction is evidence too.

use super::error::{LawError, LawResult};
use super::num::{Nature4, clip, dot, exact_mean, finite};

/// The values of one (group, tendency) pair before shares are applied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupSupport {
    /// S_{c,k} ∈ [−1, 1].
    pub support: f64,
    /// Strength in units of ordinary fluctuation, ≤ w_max.
    pub intensity: f64,
    /// p_c = ⟨Δ_c, d_k⟩; absent for teaching groups.
    pub projection: Option<f64>,
}

/// Δ_c = mean_{e∈c}(a(e) − b), channel by channel, each mean exact (D9).
pub fn mean_deviation(deviations: &[Nature4]) -> LawResult<Nature4> {
    let mut out = [0.0; 4];
    for (i, slot) in out.iter_mut().enumerate() {
        let column: Vec<f64> = deviations.iter().map(|d| d[i]).collect();
        *slot = exact_mean(&column)?;
    }
    Ok(out)
}

/// a(e) − b per channel, in binary64.
pub fn deviation(a: &Nature4, b: &Nature4) -> LawResult<Nature4> {
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = finite(a[i] - b[i], "a(e) - b")?;
    }
    Ok(out)
}

/// σ̂_k = √(Σ_i (d_{k,i} σ̂_i)²), channels in order (v2.0 §5.2: σ̂_k² = Σ d²σ̂²).
pub fn fluctuation(d: &Nature4, sigma: &Nature4) -> LawResult<f64> {
    let sq = |i: usize| (d[i] * sigma[i]) * (d[i] * sigma[i]);
    let s = finite((((sq(0) + sq(1)) + sq(2)) + sq(3)).sqrt(), "sigma_k")?;
    if s <= 0.0 {
        return Err(LawError::InvalidInput("sigma_k must be positive".into()));
    }
    Ok(s)
}

/// Support and strength of an experience group for tendency k.
///
/// `deviations` are a(e) − b of the members in member order; `r_tilde` is
/// max(r_k, r_min) before the group; `mu` is μ_{c,k} ∈ (0, 1].
pub fn experience(
    deviations: &[Nature4],
    d: &Nature4,
    sigma: &Nature4,
    r_tilde: f64,
    mu: f64,
    w_max: f64,
) -> LawResult<GroupSupport> {
    if !(r_tilde.is_finite() && r_tilde > 0.0) {
        return Err(LawError::InvalidInput("r_tilde must be positive".into()));
    }
    if !(mu.is_finite() && mu > 0.0 && mu <= 1.0) {
        return Err(LawError::InvalidInput(format!("mu {mu} outside (0, 1]")));
    }
    let mean = mean_deviation(deviations)?;
    let p = finite(dot(&mean, d), "p_c")?;
    // Saturation is defined only for finite arithmetic. In particular, a
    // positive finite r_tilde may still make this division overflow.
    let twice_p = finite(2.0 * p, "2 p_c")?;
    let ratio = finite(twice_p / r_tilde, "support ratio")?;
    let unbounded_support = finite(ratio - 1.0, "support before clip")?;
    let support = finite(mu * clip(unbounded_support, -1.0, 1.0), "S")?;
    let mut peak = 0.0f64;
    for dev in deviations {
        let projection = finite(dot(dev, d), "member projection")?;
        peak = peak.max(projection.abs());
    }
    let sigma_k = fluctuation(d, sigma)?;
    let raw_strength = finite(r_tilde.max(peak) / sigma_k, "strength ratio")?;
    let intensity = finite(finite(w_max, "w_max")?.min(raw_strength), "strength")?;
    Ok(GroupSupport {
        support,
        intensity,
        projection: Some(p),
    })
}

/// A teaching group (EV7; V2-45): S = +1 for, −1 against; strength w_teach.
pub fn teaching(supports: bool, w_teach: f64) -> GroupSupport {
    GroupSupport {
        support: if supports { 1.0 } else { -1.0 },
        intensity: w_teach,
        projection: None,
    }
}

/// W = strength × credibility × verifiability × share, multiplied in this order.
pub fn weight(intensity: f64, credibility: f64, verifiability: f64, share: f64) -> LawResult<f64> {
    use super::num::nonnegative_product as product;
    let first = product(intensity, credibility, "UnderflowEvidenceWeight")?;
    let second = product(first, verifiability, "UnderflowEvidenceWeight")?;
    product(second, share, "UnderflowEvidenceWeight")
}
