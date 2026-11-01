//! V2-96: real recurrence identities plus outward binary64 error enclosures.
use super::{
    DynamicsState, Fault, ReactionEvent, Result, duration,
    interval::{I, p},
    numeric as n,
};

/// Derived for this event, never supplied by a profile or a caller.
#[derive(Clone, Debug, PartialEq)]
pub struct StepBounds {
    /// b[0..4], g[4..8], lambda[8..12]; outward-rounded range widths.
    pub range: [f64; 12],
    /// Single-event identities, including local rounding and old quotient defects.
    pub single_event: [f64; 12],
    /// A proof intermediate could not tighten the always-valid range bound.
    /// This is not a numerical fault in the production recurrence.
    pub range_only: [bool; 12],
}
impl StepBounds {
    pub fn limits(&self) -> [f64; 12] {
        std::array::from_fn(|i| self.range[i].min(self.single_event[i]))
    }
    pub(super) fn check(&self, before: &[f64; 12], after: &[f64; 12]) -> Result<()> {
        for (i, limit) in self.limits().iter().enumerate() {
            let change = n::sub(n::finite(after[i])?, n::finite(before[i])?)?.abs();
            if change > *limit {
                return Err(Fault::ComputationBound { component: i });
            }
        }
        Ok(())
    }
}

struct Moment {
    image: I,
    delta: f64,
}

struct PreviousMoment {
    prior_weight: f64,
    prior: I,
    sum: f64,
    mass: f64,
    current: f64,
}

/// n(1-rho)(prior-current) must remain even when observation == current.
/// defect accounts for current being a rounded quotient of stored sufficient stats.
fn moment(previous: PreviousMoment, observation: I, added: f64, rho: f64) -> Result<Moment> {
    let PreviousMoment {
        prior_weight: n,
        prior,
        sum,
        mass,
        current,
    } = previous;
    let (n, sum, mass, current, added, rho) = (p(n), p(sum), p(mass), p(current), p(added), p(rho));
    let defect = (n * prior + sum) - current * (n + mass);
    let denominator = n + (rho * mass + added);
    let numerator =
        (n * (p(1.0) - rho) * (prior - current) + added * (observation - current)) + rho * defect;
    let identity = numerator / denominator;
    let image = (n * prior + (rho * sum + added * observation)) / denominator;
    // Both expressions enclose the exact real quotient; this difference also
    // encloses its forward roundoff through the specified recurrence operations.
    let error = (image - (current + identity)).abs();
    let delta = (p(identity.abs()) + p(error)).hi.next_up();
    if !image.valid() || !delta.is_finite() {
        return Err(Fault::PrecisionExhausted);
    }
    Ok(Moment { image, delta })
}

fn checked(x: f64) -> Result<f64> {
    if !x.is_finite() || x < 0.0 {
        Err(Fault::PrecisionExhausted)
    } else {
        Ok(x)
    }
}

impl StepBounds {
    fn set_single_event(&mut self, i: usize, bound: Result<f64>) {
        match bound {
            Ok(value) => self.single_event[i] = value,
            Err(_) => {
                // A failed enclosure is not evidence of an impossible candidate.
                // V2-96's range theorem remains valid, including for subnormals.
                self.single_event[i] = self.range[i];
                self.range_only[i] = true;
            }
        }
    }
}

pub(super) fn derive(old: &DynamicsState, event: &ReactionEvent, h: f64) -> Result<StepBounds> {
    let law = &old.law;
    let rho = n::sub(1.0, h)?;
    let a = core_types::semantic::nature(&event.reaction);
    let b = core_types::semantic::nature(&old.values.baseline);
    let g = core_types::semantic::scale(&old.values.gain);
    let lambda = core_types::semantic::scale(&old.values.recovery_per_s);
    let mut result = StepBounds {
        range: [2.0; 12],
        single_event: [0.0; 12],
        range_only: [false; 12],
    };
    for i in 0..4 {
        let baseline = moment(
            PreviousMoment {
                prior_weight: law.baseline_weight,
                prior: p(law.baseline_prior[i]),
                sum: old.reaction_sum[i],
                mass: old.weight,
                current: b[i],
            },
            p(a[i]),
            1.0,
            rho,
        );
        result.set_single_event(i, baseline.map(|value| value.delta));

        result.range[4 + i] = checked((p(law.gain_max) - p(law.gain_min)).hi)?;
        let gain_bound = (|| {
            let residual = p(a[i]) - p(b[i]);
            let variance = moment(
                PreviousMoment {
                    prior_weight: law.variance_weight,
                    prior: p(law.sigma_prior[i]) * p(law.sigma_prior[i]),
                    sum: old.square_sum[i],
                    mass: old.weight,
                    current: old.variance[i],
                },
                residual * residual,
                1.0,
                rho,
            )?;
            let root_old = p(old.variance[i]).sqrt();
            let root_new = variance.image.sqrt();
            let k = p(law.gain_prior[i]) * p(law.sigma_prior[i]);
            let gain_image = (k / root_new).clip(law.gain_min, law.gain_max);
            let gain_before = (k / root_old).clip(law.gain_min, law.gain_max);
            // clip is 1-Lipschitz. This is the inverse-square-root identity of V2-96.
            let derivative = k * p(variance.delta) / (root_new * root_old * (root_new + root_old));
            let gain_error = (gain_before - p(g[i])).abs();
            checked(
                (derivative + p(gain_error) + p(gain_image.width()))
                    .hi
                    .next_up(),
            )
        })();
        result.set_single_event(4 + i, gain_bound);

        result.range[8 + i] = checked((p(law.recovery_max) - p(law.recovery_min)).hi)?;
        let recovery_bound = (|| {
            let sign = n::sign(n::sub(a[i], b[i])?);
            let run = &old.runs[i];
            let mut added = 0.0;
            let mut observation = 0.0;
            if run.sign != 0 && sign != run.sign {
                let ns = event
                    .running_time
                    .running_ns()
                    .checked_sub(run.start_ns)
                    .ok_or(Fault::EventOrder)?;
                if ns > 0 {
                    added = 1.0;
                    observation = duration::log(duration::seconds(ns)?)?;
                }
            }
            // libm log's returned binary64 value is a versioned numerical input here.
            let mean = moment(
                PreviousMoment {
                    prior_weight: law.duration_weight,
                    prior: p(duration::log(law.tau_prior_s[i])?),
                    sum: old.duration_log_sum[i],
                    mass: old.duration_mass[i],
                    current: old.duration_log_mean[i],
                },
                p(observation),
                added,
                rho,
            )?;
            let lambda_image = (p(1.0) / mean.image.exp()).clip(law.recovery_min, law.recovery_max);
            let lambda_before = (p(1.0) / p(old.duration_log_mean[i]).exp())
                .clip(law.recovery_min, law.recovery_max);
            // Mean value theorem: |exp(-z')-exp(-z)| <= exp(-min(z,z')) |z'-z|.
            let slope = (p(0.0) - p(mean.image.lo.min(old.duration_log_mean[i]))).exp();
            let lambda_error = (lambda_before - p(lambda[i])).abs();
            checked(
                (slope * p(mean.delta) + p(lambda_error) + p(lambda_image.width()))
                    .hi
                    .next_up(),
            )
        })();
        result.set_single_event(8 + i, recovery_bound);
    }
    Ok(result)
}
