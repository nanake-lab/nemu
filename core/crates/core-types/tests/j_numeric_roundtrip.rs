//! P5-42: real P5-12 numeric outputs mapped into IDL. Synthetic memory only.
use core_kernel::{
    effects::{ProcessTime, StoredR, StoredTendency, StoredTheta, Validity},
    fast::{FastState, LearnedValue},
    modulation::*,
};
use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::*,
    semantic::check_j_message,
};
use serde_json::{Value, json};

const LIMITS: Limits = Limits {
    max_bytes: 1 << 20,
    max_items: 4096,
    max_depth: 32,
};

#[test]
fn p5_12_coefficients_bound_body_and_excluded_list_roundtrip_bitwise() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../contracts/fixtures/j-resources.json")).unwrap();
    let set: JDirectionSet = serde_json::from_value(fixture["direction_set"].clone()).unwrap();
    let template: JEffectReceipt = serde_json::from_value(fixture["receipt"].clone()).unwrap();
    let body: BodyReadoutParameters = serde_json::from_value(fixture["body"].clone()).unwrap();
    let validation: JDirectionValidation =
        serde_json::from_value(fixture["validation"].clone()).unwrap();
    let resources = CarrierResources {
        carrier: set.hook.carrier.clone(),
        model: set.hook.model.clone(),
        tokenizer: set.hook.tokenizer.clone(),
        direction_set: template.direction_set.clone(),
    };
    let binding = DirectionBinding {
        resources: resources.clone(),
        validation: template.validation.clone(),
        max_beta: validation.combination_max_beta,
    };
    let input = |d: &JDirection| DirectionInput {
        direction: d.vector.clone(),
        beta: d.beta,
        binding: DirectionBinding {
            max_beta: validation
                .directions
                .iter()
                .find(|v| v.vector == d.vector)
                .unwrap()
                .max_beta,
            ..binding.clone()
        },
    };
    let config = CarrierConfig::new(
        resources,
        std::array::from_fn(|i| input(&set.directions[i])),
        vec![LearnedDirectionInput {
            tendency_id: "learned".into(),
            concept_id: "concept".into(),
            direction: input(&set.directions[4]),
        }],
        binding,
    )
    .unwrap();
    let row = |r: &NatureReal| [r.connection, r.empathy, r.curiosity, r.achievement];
    let body_config = BodyReadoutConfig::new(
        body.version.clone(),
        [row(&body.facing), row(&body.tension), row(&body.energy)],
    )
    .unwrap();
    let theta = StoredTheta {
        b: [0.0; 4],
        g: [1.0; 4],
        lambda: [1.0; 4],
    };
    let ids = ["learned", "unverified", "missing-data"];
    let r = StoredR {
        tendencies: ids
            .iter()
            .map(|id| StoredTendency {
                tendency_id: (*id).into(),
                concept: if *id == "learned" {
                    "concept".into()
                } else {
                    format!("concept-{id}")
                },
                pi: 0.75,
                r: 0.5,
                d: [1.0, 0.0, 0.0, 0.0],
                direction_validated: *id != "unverified",
            })
            .collect(),
    };
    for nature in [[0.5, -0.5, 0.25, -1.0], [0.0; 4], [1.0; 4]] {
        let x = FastState::new(
            nature,
            ids.iter()
                .map(|id| LearnedValue {
                    tendency_id: (*id).into(),
                    value: 0.5,
                })
                .collect(),
        )
        .unwrap();
        let numeric = compile_numeric(
            JStateView {
                snapshot_id: "synthetic",
                anchor_time: ProcessTime(7),
                theta: &theta,
                x: &x,
                r: &r,
                process_time: ProcessTime(7),
                validity: &Validity,
            },
            &config,
            &body_config,
        )
        .unwrap();
        let mut receipt = template.clone();
        receipt.snapshot_id = numeric.receipt.snapshot_id.clone();
        receipt.anchor_time_ns.0 = numeric.receipt.anchor_time.0;
        receipt.projected_at_ns.0 = numeric.receipt.projected_at.0;
        receipt.total_bound = numeric.total_bound;
        receipt.body = numeric.body.clone();
        receipt.directions = numeric
            .receipt
            .included
            .iter()
            .enumerate()
            .map(|(i, d)| JDirectionUse {
                schema_version: 1,
                kind: match i {
                    0 => JDirectionKind::J_CONNECTION,
                    1 => JDirectionKind::J_EMPATHY,
                    2 => JDirectionKind::J_CURIOSITY,
                    3 => JDirectionKind::J_ACHIEVEMENT,
                    _ => JDirectionKind::J_LEARNED,
                },
                tendency_id: d.tendency_id.clone(),
                status: JDirectionStatus::J_INCLUDED,
                direction: Some(d.direction.clone()),
                validation: Some(d.validation.clone()),
                beta: Some(d.beta),
                coefficient: Some(d.coefficient),
            })
            .collect();
        receipt
            .directions
            .extend(numeric.receipt.excluded.iter().map(|d| JDirectionUse {
                schema_version: 1,
                kind: JDirectionKind::J_LEARNED,
                tendency_id: Some(d.tendency_id.clone()),
                status: match d.reason {
                    ExclusionReason::Unvalidated => JDirectionStatus::J_UNVALIDATED,
                    ExclusionReason::MissingDirection => JDirectionStatus::J_MISSING,
                },
                direction: None,
                validation: None,
                beta: None,
                coefficient: None,
            }));
        receipt.directions[4..].sort_by(|a, b| {
            core_types::semantic::tendency_order(
                a.tendency_id.as_deref().unwrap(),
                b.tendency_id.as_deref().unwrap(),
            )
        });
        check_j_message(
            "JEffectReceipt",
            &serde_json::to_value(&receipt).unwrap(),
            LIMITS,
        )
        .unwrap();
        let bytes = receipt.canonical_bytes(LIMITS).unwrap();
        let decoded = JEffectReceipt::from_canonical(&bytes, LIMITS).unwrap();
        assert_eq!(decoded.total_bound.to_bits(), numeric.total_bound.to_bits());
        for (a, b) in [
            decoded.body.facing,
            decoded.body.tension,
            decoded.body.energy,
        ]
        .iter()
        .zip([
            numeric.body.facing,
            numeric.body.tension,
            numeric.body.energy,
        ]) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
        let coefficients: Vec<_> = decoded
            .directions
            .iter()
            .filter_map(|d| d.coefficient)
            .map(f64::to_bits)
            .collect();
        let mut expected = row(&numeric.nature_coefficients).to_vec();
        expected.extend(numeric.learned_coefficients.iter().map(|d| d.coefficient));
        assert_eq!(
            coefficients,
            expected.into_iter().map(f64::to_bits).collect::<Vec<_>>()
        );
        let excluded: Vec<_> = decoded
            .directions
            .iter()
            .filter(|d| d.status != JDirectionStatus::J_INCLUDED)
            .map(|d| (d.tendency_id.as_deref().unwrap(), d.status))
            .collect();
        assert_eq!(
            excluded,
            vec![
                ("unverified", JDirectionStatus::J_UNVALIDATED),
                ("missing-data", JDirectionStatus::J_MISSING)
            ]
        );
        let schema = Schema::builtin().unwrap();
        let value = serde_json::to_value(&decoded).unwrap();
        let digest = schema.digest("JEffectReceipt", &value, LIMITS).unwrap();
        // Existing envelope references the receipt, never vice versa.
        let mut envelope = json!({"schema_version":1,"snapshot_id":decoded.snapshot_id,
            "projected_at_ns":decoded.projected_at_ns,"x_digest":decoded.x_digest,"r_digest":decoded.r_digest,
            "theta_digest":decoded.theta_digest,"carrier":decoded.carrier,"model":decoded.model,"tokenizer":decoded.tokenizer,
            "total_bound":numeric.total_bound,"effect_receipt_digest":digest,"direction_set":decoded.direction_set,
            "nature_coefficients":numeric.nature_coefficients,"learned_coefficients":numeric.learned_coefficients});
        let wire = schema
            .encode_canonical("ModulationEnvelopeV2", &envelope, LIMITS)
            .unwrap();
        envelope = schema
            .decode_canonical("ModulationEnvelopeV2", &wire, LIMITS, 0)
            .unwrap();
        assert_eq!(envelope["effect_receipt_digest"], digest);
    }
}
