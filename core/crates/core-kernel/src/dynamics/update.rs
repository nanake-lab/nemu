use super::{
    DynamicsFuses, DynamicsState, Fault, ReactionEvent, Result, ReviewSignal, StagedDynamics,
    numeric as n, types,
};
use core_types::generated::ThetaChange;

/// The one recurrence used by C and the constant-rho reference tests.
pub(super) fn statistics(
    old: &DynamicsState,
    a: [f64; 4],
    running_ns: u64,
    h: f64,
) -> Result<DynamicsState> {
    let p = &old.law;
    let rho = n::sub(1.0, h)?;
    if !(0.0..1.0).contains(&rho) || rho == 0.0 {
        return Err(Fault::PrecisionExhausted);
    }
    let mut s = old.clone();
    s.h = h;
    s.weight = n::add(n::mul(rho, old.weight)?, 1.0)?;
    let old_b = core_types::semantic::nature(&old.values.baseline);
    let mut b = [0.0; 4];
    let mut g = [0.0; 4];
    let mut lambda = [0.0; 4];
    for i in 0..4 {
        let residual = n::sub(a[i], old_b[i])?;
        s.square_sum[i] = n::add(n::mul(rho, old.square_sum[i])?, n::mul(residual, residual)?)?;
        s.reaction_sum[i] = n::add(n::mul(rho, old.reaction_sum[i])?, a[i])?;
        b[i] = n::div(
            n::add(
                n::mul(p.baseline_weight, p.baseline_prior[i])?,
                s.reaction_sum[i],
            )?,
            n::add(p.baseline_weight, s.weight)?,
        )?;
        if !(-1.0..=1.0).contains(&b[i]) {
            return Err(Fault::OutOfRange);
        }
        let prior_variance = n::mul(p.sigma_prior[i], p.sigma_prior[i])?;
        let variance = n::div(
            n::add(n::mul(p.variance_weight, prior_variance)?, s.square_sum[i])?,
            n::add(p.variance_weight, s.weight)?,
        )?;
        s.sigma[i] = n::finite(libm::sqrt(variance))?;
        s.variance[i] = variance;
        g[i] = n::div(n::mul(p.gain_prior[i], p.sigma_prior[i])?, s.sigma[i])?
            .clamp(p.gain_min, p.gain_max);
        super::duration::observe(old, &mut s, i, residual, running_ns, rho)?;
        lambda[i] = n::div(1.0, s.tau[i])?.clamp(p.recovery_min, p.recovery_max);
    }
    s.values = types::theta(b, g, lambda);
    Ok(s)
}

pub(super) fn finish(
    old: &DynamicsState,
    mut next: DynamicsState,
    event: &ReactionEvent,
    f: &DynamicsFuses,
) -> Result<StagedDynamics> {
    let before = types::components(&old.values);
    let after = types::components(&next.values);
    let bounds = super::bounds::derive(old, event, next.h)?;
    bounds.check(&before, &after)?;
    let mut signals = Vec::new();
    for i in 0..12 {
        let delta = n::sub(after[i], before[i])?;
        if delta.abs() > f.jump_notice && !signals.contains(&ReviewSignal::Jump) {
            signals.push(ReviewSignal::Jump);
        }
        let sign = n::sign(delta);
        next.drift_count[i] = if sign == 0 {
            0
        } else if sign == old.drift_sign[i] {
            n::increment(old.drift_count[i])?
        } else {
            1
        };
        next.drift_sign[i] = sign;
        let (lower, upper) = if i < 4 {
            (-1.0, 1.0)
        } else if i < 8 {
            (next.law.gain_min, next.law.gain_max)
        } else {
            (next.law.recovery_min, next.law.recovery_max)
        };
        let distance = if sign > 0 {
            n::sub(upper, after[i])?
        } else {
            n::sub(after[i], lower)?
        };
        if sign != 0 && next.drift_count[i] >= f.drift_run && distance <= f.drift_margin {
            signals.push(ReviewSignal::SlowDrift {
                component: i,
                direction: sign,
            });
        }
    }
    let change = ThetaChange {
        schema_version: 1,
        before: old.values.clone(),
        after: next.values.clone(),
        events: vec![event.id.clone()],
    };
    Ok(StagedDynamics {
        state: next,
        change: Some(change),
        signals,
        bounds: Some(bounds),
    })
}
