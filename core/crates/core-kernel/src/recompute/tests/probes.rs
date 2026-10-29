use super::*;
use crate::{effects::*, fast::FastState, modulation::*};
use core_types::generated::*;

fn compile(vector: [f64; 4], gain: f64) -> Result<NumericCompilation> {
    let resources = CarrierResources {
        carrier: golden(),
        model: golden(),
        tokenizer: golden(),
        direction_set: golden(),
    };
    let binding = DirectionBinding {
        resources: resources.clone(),
        validation: golden(),
        max_beta: 4.0,
    };
    let direction = DirectionInput {
        direction: golden(),
        beta: 0.5,
        binding: binding.clone(),
    };
    let config = CarrierConfig::new(
        resources,
        std::array::from_fn(|_| direction.clone()),
        vec![],
        binding,
    )
    .unwrap();
    let body = BodyReadoutConfig::new(
        "fixture-body".into(),
        [
            [0.25, 0.0, 0.0, 0.0],
            [0.0, 0.5, 0.0, 0.0],
            [0.0, 0.0, 0.25, 0.25],
        ],
    )
    .unwrap();
    let x = FastState::new(vector.map(|v| v * gain), vec![]).unwrap();
    let theta = StoredTheta {
        b: [0.0; 4],
        g: [1.0; 4],
        lambda: [0.1; 4],
    };
    compile_numeric(
        JStateView {
            snapshot_id: "synthetic",
            anchor_time: ProcessTime(0),
            process_time: ProcessTime(0),
            theta: &theta,
            x: &x,
            r: &StoredR::default(),
            validity: &Validity,
        },
        &config,
        &body,
    )
    .map_err(|e| Fault::Computation(format!("{e:?}")))
}
fn probes() -> ProbeSet {
    ProbeSet::new("fixture-probes".into(), scope().law, [0.5; 4], 0.125, 0.125).unwrap()
}

#[test]
fn ac2_21_exact_eight_neutral_probes_real_j_coefficients_and_body() {
    let set = probes();
    for (i, vector) in set.vectors().iter().enumerate() {
        assert_eq!(vector.iter().filter(|v| **v != 0.0).count(), 1);
        assert_eq!(vector[i / 2], if i % 2 == 0 { 0.5 } else { -0.5 });
    }
    let before = set.evaluate(|v| compile(v, 0.0)).unwrap();
    let equal = set.evaluate(|v| compile(v, 0.5)).unwrap();
    assert!(!compare_probes(&set, &scope(), &before, &equal).unwrap()); // equal limit
    let beyond = set.evaluate(|v| compile(v, 1.0)).unwrap();
    assert!(compare_probes(&set, &scope(), &before, &beyond).unwrap());
    assert!(!compare_probes(&set, &scope(), &beyond, &beyond).unwrap());
}

#[test]
fn probes_reject_nan_overflow_stale_resources_and_reordered_rows() {
    let set = probes();
    let before = set.evaluate(|v| compile(v, 1.0)).unwrap();
    let mut after = before.clone();
    after[0].compilation.body.energy = f64::NAN;
    assert!(matches!(
        compare_probes(&set, &scope(), &before, &after),
        Err(Fault::Computation(_))
    ));
    let mut after = before.clone();
    after.swap(0, 1);
    assert!(matches!(
        compare_probes(&set, &scope(), &before, &after),
        Err(Fault::Binding(_))
    ));
    let mut after = before.clone();
    after[1]
        .compilation
        .receipt
        .resources
        .carrier
        .resource_version
        .push('!');
    assert!(compare_probes(&set, &scope(), &before, &after).is_err());
    let mut wrong = scope();
    wrong.law.push('!');
    assert!(compare_probes(&set, &wrong, &before, &before).is_err());
    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(ProbeSet::new("x".into(), "v".into(), [0.5; 4], invalid, 0.1).is_err());
    }
}

#[test]
fn learned_coefficients_are_matched_by_identity_missing_is_zero() {
    let set = probes();
    let before = set.evaluate(|v| compile(v, 1.0)).unwrap();
    let mut after = before.clone();
    after[3]
        .compilation
        .learned_coefficients
        .push(LearnedCoefficient {
            schema_version: 1,
            tendency_id: "new".into(),
            coefficient: 0.5,
        });
    assert!(compare_probes(&set, &scope(), &before, &after).unwrap());
    after[3]
        .compilation
        .learned_coefficients
        .push(LearnedCoefficient {
            schema_version: 1,
            tendency_id: "new".into(),
            coefficient: 0.25,
        });
    assert!(matches!(
        compare_probes(&set, &scope(), &before, &after),
        Err(Fault::InvalidInput(_))
    ));
}
