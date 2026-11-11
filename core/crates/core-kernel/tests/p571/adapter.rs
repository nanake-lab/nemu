//! Public C and F only. No expected-value or recurrence implementation here.
use core_kernel::{dynamics as c, effects, fast};
use core_types::{
    WireMessage,
    contract::Limits,
    generated::*,
    scalar::{DecimalU64, HexBytes},
};
use serde_json::{Value, json};
use std::time::Instant;

fn limits() -> Limits {
    Limits {
        max_bytes: 1 << 20,
        max_items: 4096,
        max_depth: 32,
    }
}
fn nature(a: [f64; 4]) -> NatureVector {
    NatureVector {
        schema_version: 1,
        connection: a[0],
        empathy: a[1],
        curiosity: a[2],
        achievement: a[3],
    }
}
fn reference(r: &Value, kind: &str) -> ObjectRef {
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(core_types::scalar::unhex(r["sha256"].as_str().unwrap()).unwrap()),
        byte_length: DecimalU64(r["length"].as_u64().unwrap()),
        object_type: kind.into(),
        resource_version: "P5-71-v2".into(),
    }
}
fn number(r: &Value, key: &str) -> f64 {
    r[key].as_f64().unwrap()
}
fn c_law(r: &Value, b: f64, p: f64) -> c::DynamicsLaw {
    c::DynamicsLaw {
        version: r["version"].as_str().unwrap().into(),
        baseline_prior: [b; 4],
        baseline_weight: p,
        gain_prior: [number(r, "gain_prior"); 4],
        sigma_prior: [number(r, "sigma_prior"); 4],
        tau_prior_s: [number(r, "tau_prior_s"); 4],
        variance_weight: number(r, "variance_weight"),
        duration_weight: number(r, "duration_weight"),
        change_prior: number(r, "change_prior"),
        change_prior_weight: number(r, "change_prior_weight"),
        change_min: number(r, "change_min"),
        gain_min: number(r, "gain_min"),
        gain_max: number(r, "gain_max"),
        recovery_min: number(r, "recovery_min"),
        recovery_max: number(r, "recovery_max"),
        detector_shift_sigma: number(r, "detector_shift_sigma"),
        detector_threshold: number(r, "detector_threshold"),
    }
}
fn c_probe(q: &Value) -> Value {
    let r = &q["resource"];
    let case = &q["case"];
    let law = c_law(r, number(case, "b0"), number(case, "p0"));
    let fuses = c::DynamicsFuses {
        jump_notice: number(r, "jump_notice"),
        drift_run: r["drift_run"].as_u64().unwrap(),
        drift_margin: number(r, "drift_margin"),
    };
    let new = case["version"] == "A";
    let genesis = if new {
        c::DynamicsState::genesis_common_discount(law.clone())
    } else {
        c::DynamicsState::genesis(law.clone())
    }
    .unwrap();
    let events: Vec<_> = case["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let t = e["time_ns"].as_u64().unwrap();
            c::ReactionEvent {
                id: e["id"].as_str().unwrap().into(),
                ordinal: e["ordinal"].as_u64().unwrap(),
                time_ns: t,
                running_time: c::RunningTime::from_history(0, t, &[]).unwrap(),
                source: c::ReactionSource::OtherPersonReply,
                evaluation_unmodulated: true,
                reaction: nature([number(e, "a"); 4]),
            }
        })
        .collect();
    let binding = reference(&q["resource_ref"], "echo.law-version");
    let snapshot = |s: &c::DynamicsState| -> (Value, Vec<u8>) {
        if new {
            let v = c::export_snapshot_v3(s, "probe", &binding, &fuses, limits()).unwrap();
            (
                serde_json::to_value(&v).unwrap(),
                v.canonical_bytes(limits()).unwrap(),
            )
        } else {
            let v = c::export_snapshot(s, "probe", &binding, &fuses, limits()).unwrap();
            (
                serde_json::to_value(&v).unwrap(),
                v.canonical_bytes(limits()).unwrap(),
            )
        }
    };
    let mut state = genesis.clone();
    let mut rows = vec![];
    for end in 0..=events.len() {
        let start = Instant::now();
        if end > 0 {
            state = c::stage(&state, Some(&events[end - 1]), &law, &fuses)
                .unwrap()
                .into_state();
        }
        let incremental_ns = start.elapsed().as_nanos() as u64;
        let start = Instant::now();
        let mut replay = genesis.clone();
        for e in &events[..end] {
            replay = c::stage(&replay, Some(e), &law, &fuses)
                .unwrap()
                .into_state();
        }
        let replay_ns = start.elapsed().as_nanos() as u64;
        let start = Instant::now();
        let (value, bytes) = snapshot(&state);
        let (_, other) = snapshot(&replay);
        assert_eq!(bytes, other, "I23 C prefix {end}");
        rows.push(json!({"prefix":end,"incremental_ns":incremental_ns,"replay_ns":replay_ns,
            "snapshot_ns":start.elapsed().as_nanos() as u64,"snapshot":value,"canonical":core_types::scalar::hex(&bytes),"i23":true}));
    }
    json!({"rows":rows})
}

fn f_probe(q: &Value) -> Value {
    use fast::contribution as f;
    let r = &q["resource"];
    let case = &q["case"];
    let binding = reference(&q["resource_ref"], "echo.law-version");
    let e = effects::EffectiveEffects {
        effects: vec![],
        theta: effects::StoredTheta {
            b: [number(r, "b"); 4],
            g: [number(r, "g"); 4],
            lambda: [number(r, "lambda"); 4],
        },
    };
    let evaluation = effects::Evaluation {
        drives: vec![],
        f_sum: [0.0; 4],
    };
    let law = f::Law {
        version: &binding,
        g_max: number(r, "g_max"),
        r_max: number(r, "r_max"),
        w_min: number(r, "w_min"),
    };
    let old_law = fast::DriveLaw {
        g_max: law.g_max,
        r_max: law.r_max,
        w_min: law.w_min,
        kappa: number(r, "kappa"),
    };
    let genesis = f::State::from_snapshot(
        FastStateSnapshotV3 {
            schema_version: 1,
            revision: "probe".into(),
            representation_version: f::REPRESENTATION_VERSION.into(),
            anchor_time_ns: DecimalU64(0),
            nature: nature([0.0; 4]),
            learned: vec![],
            slots: vec![],
            law_version: binding.clone(),
        },
        &binding,
        limits(),
    )
    .unwrap();
    let events = case["events"].as_array().unwrap();
    // Inputs contain the frozen historical target and first position; repeated
    // visits resolve through HistoryBasis rather than masquerading as updates.
    let advance = |state: f::State, old: fast::FastState, anchor: u64, end: usize| {
        let ev = &events[end];
        let at = ev["time_ns"].as_u64().unwrap();
        let mut a = [0.0; 4];
        a[0] = number(ev, "target");
        let target = f::compute_target(&e, &evaluation, Some(a), 1.0, &law).unwrap();
        let mut input_ref = reference(&q["input_ref"], "p571.synthetic-history");
        input_ref.resource_version = "P5-71-v2-H".into();
        let effect = f::Effect::new(
            FastEffectRecord {
                schema_version: 1,
                basis: FastBasisIdentity {
                    schema_version: 1,
                    kind: FastBasisKind::FAST_BASIS_RECORD,
                    record_id: Some(ev["root"].as_str().unwrap().into()),
                },
                role: FastEffectRole::FAST_EFFECT_NEW_BASIS,
                target: LawStateVector {
                    schema_version: 1,
                    nature: NatureReal {
                        schema_version: 1,
                        connection: target.drive().nature()[0],
                        empathy: target.drive().nature()[1],
                        curiosity: target.drive().nature()[2],
                        achievement: target.drive().nature()[3],
                    },
                    learned: vec![],
                },
                theta_before: input_ref.clone(),
                effects_before: input_ref.clone(),
                interpretation_refs: vec![input_ref.clone()],
                governance_refs: vec![input_ref],
                law_version: binding.clone(),
            },
            &target,
            limits(),
        )
        .unwrap();
        let projected = state.project_time(&e, at).unwrap();
        let state = if end < 3 {
            projected.apply(&effect, limits()).unwrap()
        } else {
            projected
                .revisit(
                    &[f::HistoryBasis {
                        first_position: ev["first_position"].as_u64().unwrap(),
                        target: &effect,
                        reinterpreted: None,
                        active: true,
                        revisit_governance: &effect.record().governance_refs,
                    }],
                    limits(),
                )
                .unwrap()
                .0
        };
        let old = fast::project_time(&old, &e, at - anchor).unwrap();
        let class = if end < 3 {
            fast::EventClass::NewExogenous
        } else {
            fast::EventClass::InternalRecord {
                n: (end - 2) as u64,
            }
        };
        let drive = fast::compute_drive(
            &e,
            &evaluation,
            &fast::EventInput {
                appraisal: Some(a),
                empathy_weight: 1.0,
                class,
            },
            &old_law,
        )
        .unwrap();
        (state, fast::apply_drive(&old, &drive).unwrap(), at)
    };
    let mut state = genesis.clone();
    let mut old = genesis.visible();
    let mut anchor = 0;
    let mut rows = vec![];
    for end in 0..=events.len() {
        let start = Instant::now();
        if end > 0 {
            (state, old, anchor) = advance(state, old, anchor, end - 1);
        }
        let incremental_ns = start.elapsed().as_nanos() as u64;
        let start = Instant::now();
        let mut replay = (genesis.clone(), genesis.visible(), 0);
        for i in 0..end {
            replay = advance(replay.0, replay.1, replay.2, i);
        }
        let replay_ns = start.elapsed().as_nanos() as u64;
        let bytes = state.canonical_bytes(limits()).unwrap();
        assert_eq!(
            bytes,
            replay.0.canonical_bytes(limits()).unwrap(),
            "I23 A1 prefix {end}"
        );
        assert_eq!(
            old.nature.map(f64::to_bits),
            replay.1.nature.map(f64::to_bits),
            "I23 old prefix {end}"
        );
        rows.push(json!({"prefix":end,"incremental_ns":incremental_ns,"replay_ns":replay_ns,"snapshot":state.snapshot(),
            "canonical":core_types::scalar::hex(&bytes),"old_x_bits":old.nature.map(f64::to_bits),"i23":true}));
    }
    json!({"rows":rows,"timing_note":"both F reference adapters executed; no per-version time division"})
}

pub fn dispatch(q: &Value) -> Value {
    match q["op"].as_str().unwrap() {
        "libm" => {
            let x = if let Some(hex) = q["x_bits"].as_str() {
                f64::from_bits(u64::from_str_radix(hex, 16).unwrap())
            } else {
                number(q, "x")
            };
            let y = match q["function"].as_str().unwrap() {
                "sqrt" => libm::sqrt(x),
                "log" => libm::log(x),
                "exp" => libm::exp(x),
                "expm1" => libm::expm1(x),
                _ => panic!("unknown primitive"),
            };
            json!({"value":y,"value_bits":format!("{:016x}",y.to_bits())})
        }
        "C" => c_probe(q),
        "F" => f_probe(q),
        _ => panic!("unknown protocol operation"),
    }
}
