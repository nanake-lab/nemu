use super::{
    DynamicsFuses, DynamicsState, Fault, ReactionEvent, Result, ReviewSignal, StagedDynamics,
    numeric as n, types,
};
use core_types::generated::ThetaChange;

/// The one recurrence used by C and the constant-rho reference tests.
pub(super) fn statistics(
    old: &DynamicsState,
    a: [f64; 4],
    time_ns: u64,
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
        g[i] = n::div(n::mul(p.gain_prior[i], p.sigma_prior[i])?, s.sigma[i])?
            .clamp(p.gain_min, p.gain_max);
        s.duration_sum[i] = n::mul(rho, old.duration_sum[i])?;
        s.duration_mass[i] = n::mul(rho, old.duration_mass[i])?;
        let sign = n::sign(residual);
        let run = &old.runs[i];
        if run.sign != 0 && sign != run.sign {
            let ns = time_ns.checked_sub(run.start_ns).ok_or(Fault::EventOrder)?;
            if ns >= (1_u64 << 53) {
                return Err(Fault::TimePrecision);
            }
            let seconds = n::div(ns as f64, 1_000_000_000.0)?;
            s.duration_sum[i] = n::add(s.duration_sum[i], seconds)?;
            s.duration_mass[i] = n::add(s.duration_mass[i], 1.0)?;
        }
        if sign != run.sign {
            s.runs[i].sign = sign;
            s.runs[i].start_ns = if sign == 0 { 0 } else { time_ns };
        }
        s.tau[i] = n::div(
            n::add(
                n::mul(p.duration_weight, p.tau_prior_s[i])?,
                s.duration_sum[i],
            )?,
            n::add(p.duration_weight, s.duration_mass[i])?,
        )?;
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
    let mut signals = Vec::new();
    for i in 0..12 {
        let delta = n::sub(after[i], before[i])?;
        if delta.abs() >= f.max_step && !signals.contains(&ReviewSignal::StepLimit) {
            signals.push(ReviewSignal::StepLimit);
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
    if !signals.is_empty() {
        return Ok(StagedDynamics {
            proposed_values: Some(next.values),
            state: old.clone(),
            change: None,
            signals,
            held: true,
        });
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
        held: false,
        proposed_values: None,
    })
}
