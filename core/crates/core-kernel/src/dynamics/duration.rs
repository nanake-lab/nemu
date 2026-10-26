//! Versioned log-duration recurrence. Open runs are right-censored, not observations.
use super::{DynamicsState, Fault, Result, numeric as n};

pub(super) fn seconds(ns: u64) -> Result<f64> {
    // Exact integer-to-binary64 conversion, then one rounded division. No epsilon.
    if ns >= (1_u64 << 53) {
        return Err(Fault::TimePrecision);
    }
    n::div(ns as f64, 1_000_000_000.0)
}

pub(super) fn log(x: f64) -> Result<f64> {
    if x <= 0.0 {
        return Err(Fault::OutOfRange);
    }
    n::finite(libm::log(x))
}

pub(super) fn exp(x: f64) -> Result<f64> {
    let y = n::finite(libm::exp(x))?;
    if y == 0.0 {
        Err(Fault::Underflow)
    } else {
        Ok(y)
    }
}

pub(super) fn observe(
    old: &DynamicsState,
    next: &mut DynamicsState,
    i: usize,
    residual: f64,
    running_ns: u64,
    rho: f64,
) -> Result<()> {
    next.duration_log_sum[i] = n::mul(rho, old.duration_log_sum[i])?;
    next.duration_mass[i] = n::mul(rho, old.duration_mass[i])?;
    let sign = n::sign(residual);
    let run = &old.runs[i];
    if run.sign != 0 && sign != run.sign {
        let ns = running_ns
            .checked_sub(run.start_ns)
            .ok_or(Fault::EventOrder)?;
        if ns == 0 {
            next.zero_duration_count[i] = n::increment(old.zero_duration_count[i])?;
        } else {
            let observation = log(seconds(ns)?)?;
            next.duration_log_sum[i] = n::add(next.duration_log_sum[i], observation)?;
            next.duration_mass[i] = n::add(next.duration_mass[i], 1.0)?;
        }
    }
    if sign != run.sign {
        next.runs[i].sign = sign;
        next.runs[i].start_ns = if sign == 0 { 0 } else { running_ns };
    }
    let prior = n::mul(old.law.duration_weight, log(old.law.tau_prior_s[i])?)?;
    let numerator = n::add(prior, next.duration_log_sum[i])?;
    let denominator = n::add(old.law.duration_weight, next.duration_mass[i])?;
    next.duration_log_mean[i] = n::div(numerator, denominator)?;
    next.tau[i] = exp(next.duration_log_mean[i])?;
    Ok(())
}
