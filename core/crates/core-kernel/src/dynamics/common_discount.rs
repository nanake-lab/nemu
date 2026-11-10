//! ADR 0035 §§2, 3.2, 3.6: explicit D07-A resource/snapshot boundary.
use super::{
    DynamicsFuses, DynamicsLaw, DynamicsState, SnapshotError, storage,
    types::{nature, scale},
};
use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::{ObjectRef, ThetaEstimatorParametersV2, ThetaSnapshotV3},
    scalar::{DecimalU64, HexBytes, unhex},
    semantic::check_theta_message,
};

/// P5-68 law_state.proto numeric-path registration; no parameter defaults.
pub const COMMON_DISCOUNT_NUMERICS_ID: &str = "ADR0035-D07-gradual-underflow-v1";

pub fn common_discount_parameters(
    law: &DynamicsLaw,
    fuses: &DynamicsFuses,
    limits: Limits,
) -> Result<ThetaEstimatorParametersV2, SnapshotError> {
    law.validate_common_discount()
        .map_err(|e| SnapshotError::Invalid(format!("law: {e:?}")))?;
    fuses
        .validate()
        .map_err(|e| SnapshotError::Invalid(format!("fuses: {e:?}")))?;
    let parameters = ThetaEstimatorParametersV2 {
        schema_version: 1,
        law_id: law.version.clone(),
        numerics_id: COMMON_DISCOUNT_NUMERICS_ID.into(),
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
        jump_notice: fuses.jump_notice,
        drift_run: DecimalU64(fuses.drift_run),
        drift_margin: fuses.drift_margin,
    };
    checked_value(&parameters, limits)?;
    Ok(parameters)
}

// Decode via the owning codec to pass its normalized value to the owning semantic
// boundary. This avoids another parameter/state schema or a production JSON dependency.
fn checked_value<T: WireMessage>(value: &T, limits: Limits) -> Result<(), SnapshotError> {
    let bytes = value.canonical_bytes(limits)?;
    let decoded = Schema::builtin()?.decode_canonical(T::NAME, &bytes, limits, 0)?;
    check_theta_message(T::NAME, &decoded, limits)?;
    Ok(())
}

pub fn export_snapshot_v3(
    state: &DynamicsState,
    revision: &str,
    law_version: &ObjectRef,
    fuses: &DynamicsFuses,
    limits: Limits,
) -> Result<ThetaSnapshotV3, SnapshotError> {
    let prior_mass = state.prior_mass.ok_or(SnapshotError::BindingMismatch)?;
    let parameters = common_discount_parameters(&state.law, fuses, limits)?;
    let schema = Schema::builtin()?;
    let value = schema.decode_canonical(
        ThetaEstimatorParametersV2::NAME,
        &parameters.canonical_bytes(limits)?,
        limits,
        0,
    )?;
    let parameters_digest = HexBytes(unhex(&schema.digest(
        ThetaEstimatorParametersV2::NAME,
        &value,
        limits,
    )?)?);
    let snapshot = ThetaSnapshotV3 {
        schema_version: 1,
        revision: revision.into(),
        law_version: law_version.clone(),
        values: state.values.clone(),
        estimator: storage::estimator(state),
        parameters,
        parameters_digest,
        prior_mass,
    };
    checked_value(&snapshot, limits)?;
    Ok(snapshot)
}

/// Restore only a V3 snapshot bound to an independently resolved resource.
/// V2 cannot enter here; switching versions requires replay from genesis (§3.6).
pub fn restore_snapshot_v3(
    snapshot: &ThetaSnapshotV3,
    expected_law_ref: &ObjectRef,
    law: &DynamicsLaw,
    fuses: &DynamicsFuses,
    limits: Limits,
) -> Result<DynamicsState, SnapshotError> {
    checked_value(snapshot, limits)?;
    let expected = common_discount_parameters(law, fuses, limits)?;
    if &snapshot.law_version != expected_law_ref
        || snapshot.parameters.canonical_bytes(limits)? != expected.canonical_bytes(limits)?
    {
        return Err(SnapshotError::BindingMismatch);
    }
    storage::restore_estimator(
        &snapshot.estimator,
        &snapshot.values,
        law,
        Some(snapshot.prior_mass),
    )
}
