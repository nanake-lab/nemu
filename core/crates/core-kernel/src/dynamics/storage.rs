//! Lossless P5-36 bridge. No store, clock, history replay or authority mutation.
use super::{
    DynamicsFuses, DynamicsLaw, DynamicsState, RunningTime,
    types::{Detector, Run, nature, scale},
};
use core_types::{
    WireMessage,
    contract::Limits,
    generated::{
        ObjectRef, ThetaEstimatorParameters, ThetaEstimatorState, ThetaLastEvent, ThetaRunningTime,
        ThetaSnapshotV2,
    },
    scalar::{DecimalI64 as I, DecimalU64 as U},
    semantic::{THETA_NUMERICS_ID, check_theta_snapshot_v2, theta_parameters_digest},
};

/// Decode/restore errors are not a C growth-computation Fault. The caller owns
/// storage recovery and authenticated H/resource binding; this module cannot resume it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    Invalid(String),
    BindingMismatch,
}
impl From<String> for SnapshotError {
    fn from(value: String) -> Self {
        Self::Invalid(value)
    }
}
impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SnapshotError {}

fn parameters(
    law: &DynamicsLaw,
    f: &DynamicsFuses,
) -> Result<ThetaEstimatorParameters, SnapshotError> {
    law.validate()
        .map_err(|e| SnapshotError::Invalid(format!("law: {e:?}")))?;
    f.validate()
        .map_err(|e| SnapshotError::Invalid(format!("calibration: {e:?}")))?;
    Ok(ThetaEstimatorParameters {
        schema_version: 1,
        law_id: law.version.clone(),
        numerics_id: THETA_NUMERICS_ID.into(),
        baseline_prior: nature(law.baseline_prior),
        gain_prior: scale(law.gain_prior),
        sigma_prior: scale(law.sigma_prior),
        tau_prior_s: scale(law.tau_prior_s),
        baseline_weight: law.baseline_weight,
        variance_weight: law.variance_weight,
        duration_weight: law.duration_weight,
        change_prior: law.change_prior,
        change_prior_weight: law.change_prior_weight,
        change_min: law.change_min,
        gain_min: law.gain_min,
        gain_max: law.gain_max,
        recovery_min: law.recovery_min,
        recovery_max: law.recovery_max,
        detector_shift_sigma: law.detector_shift_sigma,
        detector_threshold: law.detector_threshold,
        jump_notice: f.jump_notice,
        drift_run: U(f.drift_run),
        drift_margin: f.drift_margin,
    })
}

/// Export the actual stored bits. `law_version` must be independently authenticated
/// by the caller. A digest and matching label alone cannot authenticate a resource.
pub fn export_snapshot(
    state: &DynamicsState,
    revision: &str,
    law_version: &ObjectRef,
    fuses: &DynamicsFuses,
    limits: Limits,
) -> Result<ThetaSnapshotV2, SnapshotError> {
    let parameters = parameters(&state.law, fuses)?;
    let parameters_digest = theta_parameters_digest(&parameters, limits)?;
    let s = state;
    let snapshot = ThetaSnapshotV2 {
        schema_version: 1,
        revision: revision.into(),
        law_version: law_version.clone(),
        values: s.values.clone(),
        parameters,
        parameters_digest,
        estimator: ThetaEstimatorState {
            schema_version: 1,
            weight: s.weight,
            reaction_sum: s.reaction_sum.to_vec(),
            square_sum: s.square_sum.to_vec(),
            sigma: scale(s.sigma),
            variance: s.variance.to_vec(),
            duration_log_sum: s.duration_log_sum.to_vec(),
            duration_mass: s.duration_mass.to_vec(),
            duration_log_mean: s.duration_log_mean.to_vec(),
            zero_duration_count: s.zero_duration_count.map(U).to_vec(),
            tau_s: scale(s.tau),
            run_sign: s.runs.iter().map(|r| I(i64::from(r.sign))).collect(),
            run_start_ns: s.runs.iter().map(|r| U(r.start_ns)).collect(),
            detector_segment_count: U(s.detector.segment_count),
            detector_changes: U(s.detector.changes),
            detector_mean: s.detector.mean.to_vec(),
            detector_positive: s.detector.positive.to_vec(),
            detector_negative: s.detector.negative.to_vec(),
            event_count: U(s.event_count),
            h: s.h,
            last_event: s
                .last_event
                .as_ref()
                .map(|(ordinal, time, id)| ThetaLastEvent {
                    schema_version: 1,
                    ordinal: U(*ordinal),
                    time_ns: U(*time),
                    event_id: id.clone(),
                }),
            last_running_time: s.last_running_time.as_ref().map(|t| ThetaRunningTime {
                schema_version: 1,
                origin_ns: U(t.origin_ns),
                event_ns: U(t.event_ns),
                stopped_ns: U(t.stopped_ns),
                running_ns: U(t.running_ns),
            }),
            drift_sign: s.drift_sign.map(|x| I(i64::from(x))).to_vec(),
            drift_count: s.drift_count.map(U).to_vec(),
        },
    };
    check_theta_snapshot_v2(&snapshot, limits)?;
    Ok(snapshot)
}

fn fixed<T: Clone, const N: usize>(v: &[T]) -> Result<[T; N], SnapshotError> {
    v.to_vec()
        .try_into()
        .map_err(|_| SnapshotError::Invalid("fixed array length".into()))
}

/// Construct a new state only after all validation and trusted-context matching.
/// Does not receive the previous state, replay events, or fill missing statistics.
pub fn restore_snapshot(
    snapshot: &ThetaSnapshotV2,
    expected_law_ref: &ObjectRef,
    law: &DynamicsLaw,
    fuses: &DynamicsFuses,
    limits: Limits,
) -> Result<DynamicsState, SnapshotError> {
    check_theta_snapshot_v2(snapshot, limits)?;
    let expected = parameters(law, fuses)?;
    if &snapshot.law_version != expected_law_ref
        || snapshot.parameters.canonical_bytes(limits)? != expected.canonical_bytes(limits)?
    {
        return Err(SnapshotError::BindingMismatch);
    }
    let s = &snapshot.estimator;
    let run_sign: [I; 4] = fixed(&s.run_sign)?;
    let run_start: [U; 4] = fixed(&s.run_start_ns)?;
    let drift: [I; 12] = fixed(&s.drift_sign)?;
    Ok(DynamicsState {
        law: law.clone(),
        values: snapshot.values.clone(),
        weight: s.weight,
        reaction_sum: fixed(&s.reaction_sum)?,
        square_sum: fixed(&s.square_sum)?,
        sigma: core_types::semantic::scale(&s.sigma),
        variance: fixed(&s.variance)?,
        duration_log_sum: fixed(&s.duration_log_sum)?,
        duration_mass: fixed(&s.duration_mass)?,
        duration_log_mean: fixed(&s.duration_log_mean)?,
        zero_duration_count: fixed::<U, 4>(&s.zero_duration_count)?.map(|x| x.0),
        tau: core_types::semantic::scale(&s.tau_s),
        // The semantic boundary above proved each sign is -1, 0 or 1.
        runs: std::array::from_fn(|i| Run {
            sign: run_sign[i].0 as i8,
            start_ns: run_start[i].0,
        }),
        detector: Detector {
            segment_count: s.detector_segment_count.0,
            changes: s.detector_changes.0,
            mean: fixed(&s.detector_mean)?,
            positive: fixed(&s.detector_positive)?,
            negative: fixed(&s.detector_negative)?,
        },
        event_count: s.event_count.0,
        h: s.h,
        last_event: s
            .last_event
            .as_ref()
            .map(|e| (e.ordinal.0, e.time_ns.0, e.event_id.clone())),
        last_running_time: s.last_running_time.as_ref().map(|t| RunningTime {
            origin_ns: t.origin_ns.0,
            event_ns: t.event_ns.0,
            stopped_ns: t.stopped_ns.0,
            running_ns: t.running_ns.0,
        }),
        drift_sign: drift.map(|x| x.0 as i8),
        drift_count: fixed::<U, 12>(&s.drift_count)?.map(|x| x.0),
    })
}

#[cfg(test)]
mod tests;
