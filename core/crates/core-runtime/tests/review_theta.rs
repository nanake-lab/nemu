mod common;
use common::{adapters::*, *};
use core_runtime::{Error, binding::Bound, pipeline::Step, stages::*};
use core_store::Experiment;
use core_types::{WireMessage, generated::*, semantic::check_theta_snapshot_v2};

// Start semantically valid before mutating exactly one reviewed boundary.
fn valid_case() -> Case {
    let case = Case::new();
    check_theta_snapshot_v2(&case.base.theta, limits()).unwrap();
    case
}
fn invalidate(theta: &mut ThetaSnapshotV2, field: usize) {
    match field {
        0 => theta.values.recovery_per_s.connection = 0.0,
        1 => theta.estimator.sigma.empathy = 0.0,
        2 => theta.estimator.tau_s.curiosity = 0.0,
        3 => {
            theta.estimator.reaction_sum.pop();
        }
        _ => theta.parameters_digest.0[0] ^= 1,
    }
}
struct InvalidC {
    inner: C,
    field: usize,
}
impl Dynamics for InvalidC {
    fn update(&mut self, input: DynamicsInput<'_>) -> StageResult<Bound<Change<ThetaSnapshotV2>>> {
        let mut response = self.inner.update(input)?;
        invalidate(&mut response.value.value, self.field);
        response.value.ledger[0]
            .theta_change
            .as_mut()
            .unwrap()
            .after = response.value.value.values.clone();
        // Reproduce the review: valid encoding and a matching ledger do not
        // imply positive time constants/divisors (operators §5.3; v2 §§6/8).
        response.value.value.canonical_bytes(limits()).unwrap();
        assert!(check_theta_snapshot_v2(&response.value.value, limits()).is_err());
        Ok(response)
    }
}
/// R2/R3: reject invalid divisors, incomplete state or a wrong parameter digest
/// before any mathematical stage (ADR 0029 explicit semantic boundary).
#[test]
fn review_r2_r3_baseline_rejects_invalid_divisors_estimator_and_digest() {
    for field in 0..5 {
        let mut case = valid_case();
        invalidate(&mut case.base.theta, field);
        let log = Log::default();
        let mut stages = StageSet::<Experiment>::complete(
            parts(&log),
            case.binding.environment.clone(),
            limits(),
        )
        .unwrap();
        let result = stages.process_head(case.input(false));
        assert!(
            result.is_err(),
            "invalid baseline theta accepted: field={field}"
        );
        assert!(
            matches!(result, Err(Error::Contract(message)) if message == check_theta_snapshot_v2(&case.base.theta, limits()).unwrap_err())
        );
        assert!(log.borrow().is_empty());
    }
}
/// R2/R3: full binding and an updated ledger must not mask invalid V2 state.
#[test]
fn review_r2_r3_c_rejects_invalid_divisors_estimator_and_digest_before_fout() {
    for field in 0..5 {
        let case = valid_case();
        let log = Log::default();
        let mut components = parts(&log);
        components.c = Some(Box::new(InvalidC {
            inner: C(log.clone()),
            field,
        }));
        let mut stages = StageSet::<Experiment>::complete(
            components,
            case.binding.environment.clone(),
            limits(),
        )
        .unwrap();
        let result = stages.process_head(case.input(false));
        assert!(
            result.is_err(),
            "invalid C output crossed STAGED: field={field}"
        );
        let mut rejected = case.base.theta.clone();
        invalidate(&mut rejected, field);
        assert!(
            matches!(result, Err(Error::Contract(message)) if message == check_theta_snapshot_v2(&rejected, limits()).unwrap_err())
        );
        assert!(log.borrow().contains(&Step::C));
        assert!(!log.borrow().contains(&Step::FOut));
    }
}
/// Positive control: the same path accepts a legal, nonidentity C update.
#[test]
fn review_r2_valid_c_output_still_reaches_staged() {
    let case = valid_case();
    let mut stages = StageSet::<Experiment>::complete(
        parts(&Log::default()),
        case.binding.environment.clone(),
        limits(),
    )
    .unwrap();
    let staged = stages.process_head(case.input(false)).unwrap();
    check_theta_snapshot_v2(&staged.snapshot().theta, limits()).unwrap();
    assert_eq!(staged.snapshot().theta.values.baseline.connection, 0.3);
}

/// The public maintenance entry is also an input boundary for F's projection.
#[test]
fn review_r2_maintenance_rejects_invalid_theta_before_projection() {
    let mut case = valid_case();
    invalidate(&mut case.base.theta, 0);
    let log = Log::default();
    let stages =
        StageSet::<Experiment>::complete(parts(&log), case.binding.environment.clone(), limits())
            .unwrap();
    assert!(stages.maintenance(&case.base, &time(30)).is_err());
    assert!(log.borrow().is_empty());
}
