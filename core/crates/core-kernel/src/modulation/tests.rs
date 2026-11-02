mod boundaries;
mod readouts;

use std::sync::OnceLock;

use core_types::WireMessage;
use core_types::contract::Limits;
use core_types::generated::{Lineage, ObjectRef};
use core_types::scalar::{DecimalU64, HexBytes};

use super::*;
use crate::effects::{ProcessTime, StoredR, StoredTendency, StoredTheta, Validity};
use crate::fast::{FastState, LearnedValue};

fn object(name: &str) -> ObjectRef {
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(vec![1; 32]),
        object_type: name.to_owned(),
        resource_version: "structural-fixture-1".into(),
        byte_length: DecimalU64(1),
    }
}

fn resources() -> CarrierResources {
    CarrierResources {
        carrier: object("fixture-carrier"),
        model: object("fixture-model"),
        tokenizer: object("fixture-tokenizer"),
        direction_set: object("fixture-directions"),
    }
}

fn binding(max_beta: f64) -> DirectionBinding {
    DirectionBinding {
        resources: resources(),
        validation: object("fixture-validation"),
        max_beta,
    }
}

fn direction(beta: f64) -> DirectionInput {
    DirectionInput {
        direction: object("fixture-unit-direction"),
        beta,
        binding: binding(2.0),
    }
}

fn nature() -> [DirectionInput; 4] {
    [0.25, 0.5, 0.125, 1.0].map(direction)
}

fn learned(id: &str) -> LearnedDirectionInput {
    LearnedDirectionInput {
        tendency_id: id.into(),
        concept_id: format!("concept-{id}"),
        direction: direction(0.125),
    }
}

fn config() -> CarrierConfig {
    static CONFIG: OnceLock<CarrierConfig> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            CarrierConfig::new(
                resources(),
                nature(),
                vec![learned("aa"), learned("b"), learned("a")],
                binding(4.0),
            )
            .unwrap()
        })
        .clone()
}

fn body() -> BodyReadoutConfig {
    BodyReadoutConfig::new(
        "body-fixture-1".into(),
        [
            [0.25, 0.0, 0.0, 0.0],
            [0.0, 0.5, 0.0, 0.0],
            [0.0, 0.0, 0.25, 0.25],
        ],
    )
    .unwrap()
}

fn theta() -> StoredTheta {
    StoredTheta {
        b: [0.25, -0.5, 0.75, 0.0],
        g: [1.0; 4],
        lambda: [1.0; 4],
    }
}

fn tendency(id: &str) -> StoredTendency {
    StoredTendency {
        tendency_id: id.into(),
        concept: format!("concept-{id}"),
        pi: 0.75,
        r: 0.5,
        d: [1.0, 0.0, 0.0, 0.0],
        direction_validated: true,
    }
}

fn state(nature: [f64; 4], learned: &[(&str, f64)]) -> FastState {
    FastState::new(
        nature,
        learned
            .iter()
            .map(|(id, value)| LearnedValue {
                tendency_id: (*id).into(),
                value: *value,
            })
            .collect(),
    )
    .unwrap()
}

fn compile(x: &FastState, r: &StoredR) -> Result<NumericCompilation, JFault> {
    compile_numeric(
        JStateView {
            snapshot_id: "synthetic-snapshot",
            anchor_time: ProcessTime(7),
            theta: &theta(),
            x,
            r,
            process_time: ProcessTime(7),
            validity: &Validity,
        },
        &config(),
        &body(),
    )
}

fn nature_values(result: &NumericCompilation) -> [f64; 4] {
    let n = &result.nature_coefficients;
    [n.connection, n.empathy, n.curiosity, n.achievement]
}

fn bits(result: &NumericCompilation) -> Vec<u64> {
    let mut values = nature_values(result).to_vec();
    values.extend(result.learned_coefficients.iter().map(|c| c.coefficient));
    values.extend([
        result.total_bound,
        result.body.facing,
        result.body.tension,
        result.body.energy,
    ]);
    for included in &result.receipt.included {
        values.extend([included.beta, included.coefficient]);
    }
    values.into_iter().map(f64::to_bits).collect()
}

#[test]
fn j06_positive_two_layers_match_independent_exact_coefficients() {
    let x = state(
        [0.5, -0.25, 0.75, -1.0],
        &[("a", 0.5), ("b", -0.25), ("aa", 0.125)],
    );
    let r = StoredR {
        tendencies: vec![tendency("aa"), tendency("a"), tendency("b")],
    };
    let result = compile(&x, &r).unwrap();
    assert_eq!(nature_values(&result), [0.125, -0.125, 0.09375, -1.0]);
    assert_eq!(
        result
            .learned_coefficients
            .iter()
            .map(|c| (c.tendency_id.as_str(), c.coefficient))
            .collect::<Vec<_>>(),
        vec![("a", 0.0625), ("b", -0.03125), ("aa", 0.015625)]
    );
    assert_eq!(result.total_bound, 2.25);
    assert!(result.receipt.excluded.is_empty());
    assert_eq!(result.receipt.included.len(), 7);
    assert_eq!(
        (result.body.facing, result.body.tension, result.body.energy),
        (0.625, -0.125, 0.4375)
    );
}

#[test]
fn zero_and_rest_are_distinct_and_do_not_subtract_baseline() {
    let r = StoredR {
        tendencies: vec![tendency("a")],
    };
    let zero = compile(&state([0.0; 4], &[("a", 0.0)]), &r).unwrap();
    assert_eq!(nature_values(&zero).map(f64::to_bits), [0; 4]);
    assert_eq!(zero.learned_coefficients[0].coefficient.to_bits(), 0);
    assert_eq!(
        (zero.body.facing, zero.body.tension, zero.body.energy),
        (0.5, 0.0, 0.5)
    );
    let rest = compile(&state(theta().b, &[("a", 0.0)]), &r).unwrap();
    assert_eq!(nature_values(&rest), [0.0625, -0.25, 0.09375, 0.0]);
    assert_eq!(rest.learned_coefficients[0].coefficient.to_bits(), 0);
}

#[test]
fn j06_unvalidated_and_missing_directions_are_excluded_and_reported() {
    let mut unvalidated = tendency("a");
    unvalidated.direction_validated = false;
    let r = StoredR {
        tendencies: vec![unvalidated, tendency("z"), tendency("b")],
    };
    let x = state([0.0; 4], &[("a", 1.0), ("b", 1.0), ("z", 1.0)]);
    let result = compile(&x, &r).unwrap();
    assert_eq!(result.learned_coefficients.len(), 1);
    assert_eq!(result.learned_coefficients[0].tendency_id, "b");
    assert_eq!(
        result.receipt.excluded,
        vec![
            ExcludedDirection {
                tendency_id: "a".into(),
                reason: ExclusionReason::Unvalidated
            },
            ExcludedDirection {
                tendency_id: "z".into(),
                reason: ExclusionReason::MissingDirection
            }
        ]
    );
    assert_eq!(result.total_bound, 2.0);
}

#[test]
fn j06_candidates_cannot_affect_m_even_with_available_direction() {
    let mut candidate = tendency("a");
    candidate.pi = 0.5;
    let x = state([0.5; 4], &[]);
    let empty = compile(&x, &StoredR::default()).unwrap();
    let actual = compile(
        &x,
        &StoredR {
            tendencies: vec![candidate],
        },
    )
    .unwrap();
    assert_eq!(empty, actual);
    assert_eq!(bits(&empty), bits(&actual));
}

#[test]
fn deterministic_projection_replay_and_input_immutability() {
    let x = state([1.0, -1.0, 0.0, 0.5], &[("a", 0.75)]);
    let r = StoredR {
        tendencies: vec![tendency("a")],
    };
    let theta = theta();
    let config = config();
    let originals = (x.clone(), r.clone(), theta, config.clone());
    let view = JStateView {
        snapshot_id: "replay",
        anchor_time: ProcessTime(5),
        theta: &theta,
        x: &x,
        r: &r,
        process_time: ProcessTime(1_000_000_005),
        validity: &Validity,
    };
    let expected = compile_numeric(view, &config, &body()).unwrap();
    for _ in 0..64 {
        let actual = compile_numeric(view, &config, &body()).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(bits(&actual), bits(&expected));
    }
    let later = compile_numeric(
        JStateView {
            process_time: ProcessTime(2_000_000_005),
            ..view
        },
        &config,
        &body(),
    )
    .unwrap();
    assert_ne!(nature_values(&expected), nature_values(&later));
    let replay = compile_numeric(view, &config, &body()).unwrap();
    assert_eq!(bits(&replay), bits(&expected));
    assert_eq!(expected.receipt.projected_at, view.process_time);
    assert_eq!((x, r, theta, config), originals);
}

#[test]
fn storage_and_configuration_order_do_not_change_numeric_results() {
    let x = state([0.5; 4], &[("a", 0.5), ("b", -0.5), ("aa", 1.0)]);
    let mut r = StoredR {
        tendencies: vec![tendency("a"), tendency("b"), tendency("aa")],
    };
    let expected = compile(&x, &r).unwrap();
    r.tendencies.reverse();
    let actual = compile(&x, &r).unwrap();
    assert_eq!(bits(&actual), bits(&expected));
    assert_eq!(actual.receipt, expected.receipt);
    let forward = CarrierConfig::new(
        resources(),
        nature(),
        vec![learned("a"), learned("b"), learned("aa")],
        binding(4.0),
    )
    .unwrap();
    assert_eq!(forward, config());
}

#[test]
fn fixed_dyadic_samples_obey_absolute_bound_and_nonconstant_j() {
    let r = StoredR {
        tendencies: vec![tendency("a")],
    };
    for step in 0..1024_u32 {
        let values =
            std::array::from_fn(|i| (f64::from((step * (i as u32 + 3)) % 257) - 128.0) / 128.0);
        let x = state(values, &[("a", (f64::from(step % 129) - 64.0) / 64.0)]);
        let result = compile(&x, &r).unwrap();
        let n = nature_values(&result);
        assert_eq!(
            n,
            [values[0] / 4.0, values[1] / 2.0, values[2] / 8.0, values[3]]
        );
        let learned = result.learned_coefficients[0].coefficient;
        assert_eq!(learned, x.learned[0].value / 8.0);
        // Structural unit directions: nature along e1..e4, learned along -e1.
        let injected = [n[0] - learned, n[1], n[2], n[3]];
        let norm = injected.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!(norm <= result.total_bound);
        assert!(n.iter().chain([&learned]).map(|v| v.abs()).sum::<f64>() <= result.total_bound);
    }
}

#[test]
fn generated_numeric_and_body_types_pass_existing_canonical_codec() {
    let result = compile(
        &state([0.5; 4], &[("a", 0.5)]),
        &StoredR {
            tendencies: vec![tendency("a")],
        },
    )
    .unwrap();
    let limits = Limits {
        max_bytes: 1 << 20,
        max_items: 4096,
        max_depth: 32,
    };
    let bytes = result.body.canonical_bytes(limits).unwrap();
    assert_eq!(
        core_types::generated::BodyReadoutV2::from_canonical(&bytes, limits).unwrap(),
        result.body
    );
    let bytes = result.nature_coefficients.canonical_bytes(limits).unwrap();
    assert_eq!(
        core_types::generated::NatureReal::from_canonical(&bytes, limits).unwrap(),
        result.nature_coefficients
    );
    for c in result.learned_coefficients {
        let bytes = c.canonical_bytes(limits).unwrap();
        assert_eq!(
            core_types::generated::LearnedCoefficient::from_canonical(&bytes, limits).unwrap(),
            c
        );
    }
}
