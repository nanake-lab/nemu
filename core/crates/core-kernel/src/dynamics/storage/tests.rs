//! Real C states; field-wise bit oracle independent of the storage mapping.
use super::super::{Downtime, ReactionEvent, ReactionSource, StagedDynamics, StepBounds, stage};
use super::*;
use core_types::{generated::ThetaValues, scalar::HexBytes};
use serde_json::{Value, json};

fn law() -> DynamicsLaw {
    DynamicsLaw {
        version: "synthetic-storage-law".into(),
        baseline_prior: [0.1, -0.2, 0.0, 0.3],
        gain_prior: [1.0, 0.8, 1.2, 0.9],
        sigma_prior: [0.3, 0.4, 0.2, 0.5],
        tau_prior_s: [10.0, 0.3, 2.0, 7.0],
        baseline_weight: 100.0,
        variance_weight: 10.0,
        duration_weight: 10.0,
        change_prior: 0.05,
        change_prior_weight: 10.0,
        change_min: 0.01,
        gain_min: 0.2,
        gain_max: 3.0,
        recovery_min: 0.001,
        recovery_max: 5.0,
        detector_shift_sigma: 1.0,
        detector_threshold: 2.0,
    }
}
fn fuses() -> DynamicsFuses {
    DynamicsFuses {
        jump_notice: 0.001,
        drift_run: 3,
        drift_margin: 0.0,
    }
}
fn reference() -> ObjectRef {
    // Synthetic trusted-context fixture, never an actual store/resource claim.
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(vec![42; 32]),
        object_type: "echo.law-version".into(),
        resource_version: law().version,
        byte_length: U(123),
    }
}
fn events() -> Vec<ReactionEvent> {
    (0..64)
        .map(|i| {
            let running = (i / 2) * 100_000_000;
            let stopped = if i >= 24 { 5_000_000_000 } else { 0 };
            let origin = 100;
            let time = origin + running + stopped;
            let stops = if stopped > 0 {
                vec![Downtime {
                    start_ns: origin + 1_100_000_000,
                    end_ns: origin + 6_100_000_000,
                }]
            } else {
                vec![]
            };
            ReactionEvent {
                id: format!("synthetic-{i}"),
                ordinal: 1000 + 3 * i,
                time_ns: time,
                running_time: RunningTime::from_history(origin, time, &stops).unwrap(),
                source: ReactionSource::OtherPersonReply,
                evaluation_unmodulated: true,
                reaction: nature([
                    if i % 2 == 0 { 0.8 } else { -0.7 },
                    if i % 11 < 5 { 0.65 } else { -0.9 },
                    if i < 32 { 0.9 } else { -0.8 },
                    if i % 7 < 3 { -0.2 } else { 0.6 },
                ]),
            }
        })
        .collect()
}
fn trajectory() -> Vec<DynamicsState> {
    let mut states = vec![DynamicsState::genesis(law()).unwrap()];
    for event in events() {
        states.push(
            stage(states.last().unwrap(), Some(&event), &law(), &fuses())
                .unwrap()
                .into_state(),
        );
    }
    states
}
fn floats(a: &[f64], b: &[f64]) {
    assert_eq!(
        a.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        b.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
    );
}
fn theta(a: &ThetaValues, b: &ThetaValues) {
    assert_eq!(
        (
            a.schema_version,
            a.baseline.schema_version,
            a.gain.schema_version,
            a.recovery_per_s.schema_version
        ),
        (
            b.schema_version,
            b.baseline.schema_version,
            b.gain.schema_version,
            b.recovery_per_s.schema_version
        )
    );
    floats(
        &super::super::types::components(a),
        &super::super::types::components(b),
    );
}
fn law_bits(p: &DynamicsLaw) -> Vec<u64> {
    // Exhaustive pattern: adding an in-memory field forces this oracle to change.
    let DynamicsLaw {
        version: _,
        baseline_prior,
        gain_prior,
        sigma_prior,
        tau_prior_s,
        baseline_weight,
        variance_weight,
        duration_weight,
        change_prior,
        change_prior_weight,
        change_min,
        gain_min,
        gain_max,
        recovery_min,
        recovery_max,
        detector_shift_sigma,
        detector_threshold,
    } = p;
    baseline_prior
        .iter()
        .chain(gain_prior)
        .chain(sigma_prior)
        .chain(tau_prior_s)
        .copied()
        .chain([
            *baseline_weight,
            *variance_weight,
            *duration_weight,
            *change_prior,
            *change_prior_weight,
            *change_min,
            *gain_min,
            *gain_max,
            *recovery_min,
            *recovery_max,
            *detector_shift_sigma,
            *detector_threshold,
        ])
        .map(f64::to_bits)
        .collect()
}
fn identical(a: &DynamicsState, b: &DynamicsState) {
    let DynamicsState {
        law,
        values,
        weight,
        reaction_sum,
        square_sum,
        sigma,
        variance,
        duration_log_sum,
        duration_mass,
        duration_log_mean,
        zero_duration_count,
        tau,
        runs,
        detector,
        event_count,
        h,
        last_event,
        last_running_time,
        drift_sign,
        drift_count,
    } = a;
    assert_eq!(law.version, b.law.version);
    assert_eq!(law_bits(law), law_bits(&b.law));
    theta(values, &b.values);
    floats(&[*weight, *h], &[b.weight, b.h]);
    for (x, y) in [
        (reaction_sum, &b.reaction_sum),
        (square_sum, &b.square_sum),
        (sigma, &b.sigma),
        (variance, &b.variance),
        (duration_log_sum, &b.duration_log_sum),
        (duration_mass, &b.duration_mass),
        (duration_log_mean, &b.duration_log_mean),
        (tau, &b.tau),
    ] {
        floats(x, y);
    }
    assert_eq!(zero_duration_count, &b.zero_duration_count);
    assert_eq!(event_count, &b.event_count);
    assert_eq!(last_event, &b.last_event);
    assert_eq!(drift_sign, &b.drift_sign);
    assert_eq!(drift_count, &b.drift_count);
    for (r, s) in runs.iter().zip(&b.runs) {
        let Run { sign, start_ns } = r;
        assert_eq!((*sign, *start_ns), (s.sign, s.start_ns));
    }
    let Detector {
        segment_count,
        changes,
        mean,
        positive,
        negative,
    } = detector;
    assert_eq!(
        (*segment_count, *changes),
        (b.detector.segment_count, b.detector.changes)
    );
    floats(mean, &b.detector.mean);
    floats(positive, &b.detector.positive);
    floats(negative, &b.detector.negative);
    match (last_running_time, &b.last_running_time) {
        (None, None) => (),
        (
            Some(RunningTime {
                origin_ns,
                event_ns,
                stopped_ns,
                running_ns,
            }),
            Some(t),
        ) => assert_eq!(
            (*origin_ns, *event_ns, *stopped_ns, *running_ns),
            (t.origin_ns, t.event_ns, t.stopped_ns, t.running_ns)
        ),
        _ => panic!("last running time presence"),
    }
}
fn same_stage(a: &StagedDynamics, b: &StagedDynamics) {
    identical(a.state(), b.state());
    assert_eq!(a.signals, b.signals);
    match (&a.change, &b.change) {
        (None, None) => (),
        (Some(x), Some(y)) => {
            assert_eq!(x, y);
            theta(&x.before, &y.before);
            theta(&x.after, &y.after);
        }
        _ => panic!("change presence"),
    }
    match (&a.bounds, &b.bounds) {
        (None, None) => (),
        (
            Some(StepBounds {
                range,
                single_event,
                range_only,
            }),
            Some(y),
        ) => {
            floats(range, &y.range);
            floats(single_event, &y.single_event);
            assert_eq!(range_only, &y.range_only);
        }
        _ => panic!("bounds presence"),
    }
}
fn continue_from(original: &DynamicsState, restored: DynamicsState, prefix: usize) {
    identical(original, &restored);
    let mut a = original.clone();
    let mut b = restored;
    for event in &events()[prefix..] {
        let x = stage(&a, Some(event), &law(), &fuses()).unwrap();
        let y = stage(&b, Some(event), &law(), &fuses()).unwrap();
        same_stage(&x, &y);
        a = x.into_state();
        b = y.into_state();
    }
    same_stage(
        &stage(&a, None, &law(), &fuses()).unwrap(),
        &stage(&b, None, &law(), &fuses()).unwrap(),
    );
    for source in [
        ReactionSource::SelfAction,
        ReactionSource::TriggeredProgramResult,
        ReactionSource::SelfNarrative,
        ReactionSource::DeceptionInducedReply,
    ] {
        let mut e = events()[0].clone();
        e.source = source;
        let x = stage(&a, Some(&e), &law(), &fuses()).unwrap();
        let y = stage(&b, Some(&e), &law(), &fuses()).unwrap();
        same_stage(&x, &y);
        identical(&a, x.state());
    }
    // Bad eligible continuation is rejected identically without changing either input.
    assert_eq!(
        stage(&a, Some(&events()[0]), &law(), &fuses()),
        stage(&b, Some(&events()[0]), &law(), &fuses())
    );
    identical(&a, &b);
}

#[test]
fn actual_states_all_prefixes_roundtrip_and_continuation() {
    let states = trajectory();
    assert!(
        states
            .last()
            .unwrap()
            .zero_duration_count
            .iter()
            .any(|x| *x > 0)
    );
    assert!(
        states
            .last()
            .unwrap()
            .duration_mass
            .iter()
            .any(|x| *x > 0.0)
    );
    assert!(
        states
            .iter()
            .any(|s| s.duration_log_sum.iter().any(|x| *x < 0.0))
    );
    assert!(states.iter().any(|s| s.detector.changes > 0));
    assert!(states.iter().any(|s| s.drift_count.iter().any(|x| *x > 0)));
    let mut records = vec![];
    for (prefix, state) in states.iter().enumerate() {
        let s = export_snapshot(
            state,
            &format!("synthetic-prefix-{prefix}"),
            &reference(),
            &fuses(),
        )
        .unwrap();
        let cb = s.canonical_bytes(LIMITS).unwrap();
        let pb = s.protobuf_bytes(LIMITS).unwrap();
        for decoded in [
            ThetaSnapshotV2::from_canonical(&cb, LIMITS).unwrap(),
            ThetaSnapshotV2::from_protobuf(&pb, LIMITS).unwrap(),
        ] {
            continue_from(
                state,
                restore_snapshot(&decoded, &reference(), &law(), &fuses()).unwrap(),
                prefix,
            );
        }
        records.push(json!({"prefix":prefix,"value":s,"canonical":core_types::scalar::hex(&cb),"protobuf":core_types::scalar::hex(&pb)}));
    }
    if let Some(path) = std::env::var_os("P536_EXPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&records).unwrap()).unwrap();
        println!("exported {} actual C states", records.len());
    }
    // External acceptance mode is additional to the unconditional real-state test.
    // Missing, duplicated, partial or unknown-language results cannot pass this mode.
    if let Some(path) = std::env::var_os("P536_RESTORE") {
        let rows: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(rows.len(), states.len() * 4);
        let mut seen = std::collections::BTreeSet::new();
        for row in rows {
            let prefix = row["prefix"].as_u64().unwrap() as usize;
            let language = row["language"].as_str().unwrap();
            let encoding = row["encoding"].as_str().unwrap();
            assert!(["python", "typescript"].contains(&language));
            assert!(["canonical", "protobuf"].contains(&encoding));
            assert!(seen.insert((prefix, language.to_owned(), encoding.to_owned())));
            let bytes = core_types::scalar::unhex(row["hex"].as_str().unwrap()).unwrap();
            let decoded = if encoding == "canonical" {
                ThetaSnapshotV2::from_canonical(&bytes, LIMITS)
            } else {
                ThetaSnapshotV2::from_protobuf(&bytes, LIMITS)
            }
            .unwrap();
            continue_from(
                &states[prefix],
                restore_snapshot(&decoded, &reference(), &law(), &fuses()).unwrap(),
                prefix,
            );
        }
        println!(
            "restored and continued {} independently re-encoded Python/TypeScript states",
            seen.len()
        );
    }
}

#[test]
fn rejects_corruption_and_wrong_trusted_context_without_mutation() {
    let original = trajectory().remove(17);
    let snapshot = export_snapshot(&original, "synthetic", &reference(), &fuses()).unwrap();
    let mut wrong = reference();
    wrong.digest.0[0] ^= 1;
    assert_eq!(
        restore_snapshot(&snapshot, &wrong, &law(), &fuses()),
        Err(SnapshotError::BindingMismatch)
    );
    let mut wrong_law = law();
    wrong_law.detector_threshold += 1.0;
    assert_eq!(
        restore_snapshot(&snapshot, &reference(), &wrong_law, &fuses()),
        Err(SnapshotError::BindingMismatch)
    );
    let mut wrong_fuses = fuses();
    wrong_fuses.drift_run += 1;
    assert_eq!(
        restore_snapshot(&snapshot, &reference(), &law(), &wrong_fuses),
        Err(SnapshotError::BindingMismatch)
    );
    let mutations: Vec<fn(&mut ThetaSnapshotV2)> = vec![
        |s| s.parameters_digest.0[0] ^= 1,
        |s| s.parameters.numerics_id = "unknown".into(),
        |s| s.law_version.resource_version = "other".into(),
        |s| s.estimator.reaction_sum.pop().map(drop).unwrap(),
        |s| s.estimator.duration_mass[0] = -1.0,
        |s| s.estimator.variance[0] = f64::NAN,
        |s| s.estimator.detector_changes = U(s.estimator.event_count.0 + 1),
        |s| s.estimator.last_event = None,
        |s| s.estimator.last_running_time.as_mut().unwrap().running_ns.0 += 1,
        |s| s.estimator.run_sign[0] = I(2),
        |s| s.estimator.drift_count[0] = U(1 << 53),
        |s| s.schema_version = 2,
    ];
    for mutate in mutations {
        let mut bad = snapshot.clone();
        mutate(&mut bad);
        assert!(restore_snapshot(&bad, &reference(), &law(), &fuses()).is_err());
    }
    let bytes = snapshot.canonical_bytes(LIMITS).unwrap();
    assert!(ThetaSnapshotV2::from_canonical(&bytes[..bytes.len() - 1], LIMITS).is_err());
    identical(&original, &trajectory()[17]);
    // A lossy bridge which reconstructs variance from sigma would fail the oracle.
    let states = trajectory();
    assert!(states.iter().any(|s|(0..4).any(|i|s.variance[i].to_bits()!=(s.sigma[i]*s.sigma[i]).to_bits())));
}

#[test]
fn alarm_boundary_and_large_integers_survive_restore() {
    use super::super::ReviewSignal;
    let p = law();
    let f = DynamicsFuses {
        drift_margin: 5.0,
        ..fuses()
    };
    let mut state = DynamicsState::genesis(p.clone()).unwrap();
    let mut saw_drift = false;
    let mut saw_jump = false;
    for mut event in events() {
        // Exact integer path, well above binary64's consecutive-integer limit.
        event.ordinal += 1 << 53;
        let snapshot = export_snapshot(&state, "synthetic-alarm", &reference(), &f).unwrap();
        let bytes = snapshot.canonical_bytes(LIMITS).unwrap();
        let decoded = ThetaSnapshotV2::from_canonical(&bytes, LIMITS).unwrap();
        let restored = restore_snapshot(&decoded, &reference(), &p, &f).unwrap();
        identical(&state, &restored);
        let a = stage(&state, Some(&event), &p, &f).unwrap();
        let b = stage(&restored, Some(&event), &p, &f).unwrap();
        same_stage(&a, &b);
        saw_jump |= a.signals.contains(&ReviewSignal::Jump);
        saw_drift |= a
            .signals
            .iter()
            .any(|s| matches!(s, ReviewSignal::SlowDrift { .. }));
        state = a.into_state();
    }
    assert!(saw_jump && saw_drift);
    // Real genesis accepts a positive subnormal baseline: preserve its bits.
    let mut p = law();
    p.baseline_prior[0] = f64::from_bits(1);
    let state = DynamicsState::genesis(p.clone()).unwrap();
    let snapshot = export_snapshot(&state, "synthetic-subnormal", &reference(), &f).unwrap();
    let decoded =
        ThetaSnapshotV2::from_protobuf(&snapshot.protobuf_bytes(LIMITS).unwrap(), LIMITS).unwrap();
    identical(
        &state,
        &restore_snapshot(&decoded, &reference(), &p, &f).unwrap(),
    );
}
