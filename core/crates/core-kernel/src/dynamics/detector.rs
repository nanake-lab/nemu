//! Two-sided Gaussian Page-CUSUM reference (versioned, pending review).
//! For residual r and shift d, log p(r|+d)/p(r|0) = d*r/s² - d²/(2*s²).
//! Sign alternation alone is never counted as a change point.
use super::{DynamicsLaw, Result, numeric as n, types::Detector};

pub(super) fn observe(old: &Detector, a: [f64; 4], law: &DynamicsLaw) -> Result<Detector> {
    let mut next = old.clone();
    next.segment_count = n::increment(old.segment_count)?;
    if old.segment_count == 0 {
        next.mean = a;
        return Ok(next);
    }
    let mut changed = false;
    for (i, &a_i) in a.iter().enumerate() {
        let variance = n::mul(law.sigma_prior[i], law.sigma_prior[i])?;
        let shift = n::mul(law.detector_shift_sigma, law.sigma_prior[i])?;
        let residual = n::sub(a_i, old.mean[i])?;
        let linear = n::div(n::mul(shift, residual)?, variance)?;
        let penalty = n::div(n::mul(shift, shift)?, n::mul(2.0, variance)?)?;
        next.positive[i] = n::add(old.positive[i], n::sub(linear, penalty)?)?.max(0.0);
        next.negative[i] = n::add(old.negative[i], n::sub(-linear, penalty)?)?.max(0.0);
        changed |= next.positive[i] >= law.detector_threshold
            || next.negative[i] >= law.detector_threshold;
        next.mean[i] = n::add(
            old.mean[i],
            n::div(residual, n::count(next.segment_count)?)?,
        )?;
    }
    if changed {
        next.changes = n::increment(old.changes)?;
        next.segment_count = 1;
        next.mean = a;
        next.positive = [0.0; 4];
        next.negative = [0.0; 4];
    }
    Ok(next)
}
