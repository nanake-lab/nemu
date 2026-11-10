//! ADR 0035 §9 P5-69: only synthetic fixed-H inputs, no runtime/real-time claim.
use super::*;
use core_types::{
    WireMessage,
    contract::Limits,
    generated::{ObjectRef, ThetaSnapshotV3},
    scalar::{DecimalU64, HexBytes},
};

const LIMITS: Limits = Limits {
    max_bytes: 1_048_576,
    max_items: 4096,
    max_depth: 32,
};

fn initial(b0: f64, p0: f64) -> DynamicsState {
    let mut p = law();
    p.version = "synthetic-ADR0035-D07-A-v1".into();
    p.baseline_prior = [b0; 4];
    p.baseline_weight = p0;
    DynamicsState::genesis_common_discount(p).unwrap()
}
fn reference(s: &DynamicsState) -> ObjectRef {
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(vec![69; 32]),
        object_type: "echo.law-version".into(),
        resource_version: s.law.version.clone(),
        byte_length: DecimalU64(69),
    }
}
fn frozen(s: &DynamicsState, a: f64, rho: f64, i: u64) -> DynamicsState {
    update::statistics(s, [a; 4], i * 1_000_000_000, 1.0 - rho).unwrap()
}
fn b(s: &DynamicsState) -> f64 {
    s.values.baseline.connection
}

#[test]
fn d07_1074_step_rational_oracle_and_gradual_underflow_golden() {
    // §9(1,2,10): every prefix has exact-rational N/T error endpoints, plus
    // independent nearest/even operation bits. Includes the reachable q/2 tie.
    let data: serde_json::Value = serde_json::from_str(include_str!("d07-vectors.json")).unwrap();
    let mut s = initial(1.0, 1.0);
    for (i, row) in data["underflow"].as_array().unwrap().iter().enumerate() {
        let v: Vec<f64> = row
            .as_array()
            .unwrap()
            .iter()
            .map(|x| f64::from_bits(x.as_str().unwrap().parse().unwrap()))
            .collect();
        let old_p = s.prior_mass.unwrap();
        s = frozen(&s, v[0], 0.5, i as u64 + 1);
        for (got, expected) in [s.prior_mass.unwrap(), s.weight, s.reaction_sum[0], b(&s)]
            .into_iter()
            .zip(&v[1..5])
        {
            assert_eq!(got.to_bits(), expected.to_bits(), "prefix {}", i + 1);
        }
        assert!(b(&s) >= v[5] && b(&s) <= v[6]);
        assert!(s.prior_mass.unwrap() <= old_p);
        if old_p.is_normal() {
            assert!(s.prior_mass.unwrap() < old_p);
        }
    }
    assert_eq!(s.prior_mass.unwrap().to_bits(), 1);
    assert_eq!(b(&s).to_bits(), 0);
    let s = frozen(&s, 0.5, 0.5, 1075);
    assert_eq!(s.prior_mass.unwrap().to_bits(), 0);
    assert!(b(&s) > 0.0); // zero P must not stop learning
}

#[test]
fn d07_subnormal_stagnation_iff_ties_and_normal_boundary() {
    // §3.5(1), §9(2,10): exact dyadic integer oracle for all k=1..64;
    // equality keeps even k, but decreases odd k. No epsilon floor.
    for k in 1_u64..=64 {
        for (num, den) in [(1_u64, 2_u64), (3, 4), (7, 8)] {
            let mut s = initial(1.0, 1.0);
            s.prior_mass = Some(f64::from_bits(k));
            let next = frozen(&s, -0.5, num as f64 / den as f64, 1);
            let gap_twice = 2 * (den - num) * k;
            let stuck = gap_twice < den || (gap_twice == den && k % 2 == 0);
            assert_eq!(next.prior_mass.unwrap().to_bits() == k, stuck);
            assert!(next.prior_mass.unwrap() <= s.prior_mass.unwrap());
            if stuck {
                assert!(2 * (den - num) * k <= den);
            }
            assert!(b(&next) < 0.0); // b still advances even if P stalls
        }
    }
    let mut s = initial(1.0, 1.0);
    s.prior_mass = Some(f64::MIN_POSITIVE);
    let next = frozen(&s, -0.5, 1.0 - 2.0_f64.powi(-53), 1);
    assert_eq!(next.prior_mass, s.prior_mass);
    // §3.2 also permits underflow in U' and P'*b0, not only in P and b's division.
    let mut tiny = initial(0.5, 1.0);
    tiny.prior_mass = Some(f64::from_bits(2));
    tiny.reaction_sum = [f64::from_bits(1); 4];
    let next = frozen(&tiny, -0.5, 0.5, 1);
    assert_eq!(next.reaction_sum[0], -0.5);
    assert_eq!(b(&next), -0.5);
    // Remaining paths retain their fault rule (§3.2); do not blanket allow underflow.
    assert_eq!(numeric::mul(f64::from_bits(1), 0.5), Err(Fault::Underflow));
    let s = initial(0.0, 1.0);
    assert_eq!(
        update::statistics(&s, [2.0_f64.powi(-538); 4], 0, 0.5),
        Err(Fault::Underflow)
    );
}

#[test]
fn d07_birth_share_real_oracle_and_binary64_mass_paths() {
    // §9(2): generator checks both REAL bounds exactly with Fraction, separately
    // from this comparison of rounded P/S across four P0 and three rho values.
    // The same mandatory Python already used by kernel's reference suites;
    // --check performs rational assertions and never changes golden data.
    let output = std::process::Command::new("python")
        .arg(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/dynamics/tests/d07_oracle.py"),
        )
        .arg("--check")
        .output()
        .expect("Python rational oracle must run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data: serde_json::Value = serde_json::from_str(include_str!("d07-vectors.json")).unwrap();
    let mut s = initial(0.0, 1.0);
    for row in data["birth_steps"].as_array().unwrap() {
        let a = row.as_array().unwrap();
        let f = |i: usize| f64::from_bits(a[i].as_str().unwrap().parse().unwrap());
        let n: u64 = a[1].as_str().unwrap().parse().unwrap();
        if n == 1 {
            s = initial(0.0, f(0));
        }
        s = frozen(&s, if n.is_multiple_of(2) { 0.5 } else { -0.5 }, f(2), n);
        assert_eq!(s.prior_mass.unwrap().to_bits(), f(3).to_bits());
        assert_eq!(s.weight.to_bits(), f(4).to_bits());
    }
}

#[test]
fn ac2_05_14_qualified_projection_only_and_duplicate_mutant() {
    // §9(3,4,9): missing A is represented by None, never by a fabricated zero.
    // RepeatedExperience/DesktopObservation are recorded caller classifications;
    // this pure boundary does not claim to discover contact identities itself.
    let s = advance(&initial(0.25, 4.0), &event(1, -0.75));
    identical(&s, stage(&s, None, &s.law, &fuses()).unwrap().state());
    for source in [
        ReactionSource::SelfAction,
        ReactionSource::TriggeredProgramResult,
        ReactionSource::SelfNarrative,
        ReactionSource::DesktopObservation,
        ReactionSource::RepeatedExperience,
    ] {
        let mut e = event(2, 1.0);
        e.time_ns = u64::MAX; // excluded inputs do not advance any clock/statistic
        e.source = source;
        identical(&s, stage(&s, Some(&e), &s.law, &fuses()).unwrap().state());
    }
    let duplicate_mutant = advance(&s, &event(2, -0.75));
    assert_ne!(duplicate_mutant.weight.to_bits(), s.weight.to_bits());
    assert_ne!(duplicate_mutant.prior_mass, s.prior_mass);
    assert_eq!(
        stage(&s, Some(&event(1, -0.75)), &s.law, &fuses()),
        Err(Fault::EventOrder)
    );
}

#[test]
fn ac2_08_18_19_v3_prefix_restore_and_replay() {
    // §9(7), I23: canonical bytes compare all floats/integers/markers. Pausing
    // omits C calls; catch-up replays stored inputs. No L transaction is claimed.
    let genesis = initial(0.25, 4.0);
    let events: Vec<_> = (1..=32)
        .map(|i| event(i, if i % 7 < 3 { 0.75 } else { -0.5 }))
        .collect();
    let replay = |es: &[ReactionEvent]| es.iter().fold(genesis.clone(), |s, e| advance(&s, e));
    let mut incremental = genesis.clone();
    for (i, e) in events.iter().enumerate() {
        incremental = advance(&incremental, e);
        identical(&incremental, &replay(&events[..=i]));
        let snapshot = export_snapshot_v3(
            &incremental,
            "synthetic",
            &reference(&genesis),
            &fuses(),
            LIMITS,
        )
        .unwrap();
        for decoded in [
            ThetaSnapshotV3::from_canonical(&snapshot.canonical_bytes(LIMITS).unwrap(), LIMITS)
                .unwrap(),
            ThetaSnapshotV3::from_protobuf(&snapshot.protobuf_bytes(LIMITS).unwrap(), LIMITS)
                .unwrap(),
        ] {
            let restored = restore_snapshot_v3(
                &decoded,
                &reference(&genesis),
                &genesis.law,
                &fuses(),
                LIMITS,
            )
            .unwrap();
            identical(&restored, &incremental);
            let resumed = events[i + 1..].iter().fold(restored, |s, e| advance(&s, e));
            identical(&resumed, &replay(&events));
        }
    }
    let paused = replay(&events[..8]);
    identical(
        &paused,
        stage(&paused, None, &paused.law, &fuses()).unwrap().state(),
    );
    identical(
        &events[8..].iter().fold(paused, |s, e| advance(&s, e)),
        &incremental,
    );
    let corrected: Vec<_> = events
        .iter()
        .filter(|e| e.ordinal % 3 != 0)
        .cloned()
        .collect();
    let mut never_used = genesis.clone();
    for e in &events {
        if e.ordinal % 3 != 0 {
            never_used = advance(&never_used, e);
        }
    }
    identical(&replay(&corrected), &never_used);
}

#[test]
fn d07_v3_binding_legacy_separation_and_full_p0_domain() {
    // §§2,3.6,7: old snapshots cannot silently acquire P; complete V3 binding.
    let s = initial(0.25, 4.0);
    assert_eq!(
        export_snapshot(&s, "x", &reference(&s), &fuses(), LIMITS),
        Err(SnapshotError::BindingMismatch)
    );
    let old = DynamicsState::genesis(s.law.clone()).unwrap();
    assert_eq!(
        export_snapshot_v3(&old, "x", &reference(&s), &fuses(), LIMITS),
        Err(SnapshotError::BindingMismatch)
    );
    let snapshot = export_snapshot_v3(&s, "x", &reference(&s), &fuses(), LIMITS).unwrap();
    let mut wrong = s.law.clone();
    wrong.baseline_weight = 8.0;
    assert_eq!(
        restore_snapshot_v3(&snapshot, &reference(&s), &wrong, &fuses(), LIMITS),
        Err(SnapshotError::BindingMismatch)
    );
    let mut invalid = snapshot.clone();
    invalid.prior_mass = -1.0;
    assert!(restore_snapshot_v3(&invalid, &reference(&s), &s.law, &fuses(), LIMITS).is_err());
    for p0 in [f64::from_bits(1), (1_u64 << 53) as f64, f64::MAX] {
        let state = initial(0.0, p0);
        let snap = export_snapshot_v3(&state, "x", &reference(&state), &fuses(), LIMITS).unwrap();
        identical(
            &state,
            &restore_snapshot_v3(&snap, &reference(&state), &state.law, &fuses(), LIMITS).unwrap(),
        );
    }
}

#[test]
fn e07_fixed_h_frozen_rho_birth_difference_and_finite_perturbation() {
    // §9(5): P0=2,rho=1/2 makes T=2 and alpha=1/2 exactly. For these
    // 32 dyadic inputs every displayed equality is exact, not a tolerance.
    let mut left = initial(0.25, 2.0);
    let mut right = initial(-0.25, 2.0);
    let mut control = initial(0.25, 2.0);
    let mut disturbed = control.clone();
    let mut previous_gap = 0.0;
    for i in 1..=32 {
        let a = if i % 3 == 0 { -0.5 } else { 0.5 };
        left = frozen(&left, a, 0.5, i);
        right = frozen(&right, a, 0.5, i);
        let beta = 2.0_f64.powi(-(i as i32));
        assert_eq!((b(&left) - b(&right)).to_bits(), (beta * 0.5).to_bits());
        assert_eq!(
            left.prior_mass.unwrap() / (left.prior_mass.unwrap() + left.weight),
            beta
        );
        assert!(beta <= 2.0 / (2.0 + i as f64));
        control = frozen(&control, a, 0.5, i);
        disturbed = frozen(&disturbed, if i == 8 { -0.75 } else { a }, 0.5, i);
        let gap = b(&disturbed) - b(&control);
        if i > 8 {
            assert_eq!(gap.to_bits(), (previous_gap * 0.5_f64).to_bits());
        }
        previous_gap = gap;
    }
    assert_ne!(b(&control).to_bits(), 0.25_f64.to_bits()); // reference is life, not birth
    // §9(9): fixed n_b mutant fails exactly this shared-input birth-share identity.
    let mut fixed = DynamicsState::genesis(left.law.clone()).unwrap();
    fixed = frozen(&fixed, 0.5, 0.5, 1);
    assert_ne!(
        b(&fixed).to_bits(),
        b(&frozen(&initial(0.25, 2.0), 0.5, 0.5, 1)).to_bits()
    );
}

#[test]
fn e07_dynamic_rho_sign_symmetry_and_permanent_world_change() {
    // §9(5,9), E07: dynamic detector is exercised separately; no frozen-alpha
    // identity is claimed for different histories. Reflections must have equal
    // qualification and masses (kills support-sign selection); x is absent.
    let mut positive = initial(0.0, 2.0);
    let mut negative = positive.clone();
    let mut rates = Vec::new();
    for i in 1..=128 {
        let a = if i <= 64 { 0.5 } else { -0.75 };
        positive = advance(&positive, &event(i, a));
        negative = advance(&negative, &event(i, -a));
        assert_eq!(positive.weight.to_bits(), negative.weight.to_bits());
        assert_eq!(positive.prior_mass, negative.prior_mass);
        assert_eq!(b(&positive).to_bits(), (-b(&negative)).to_bits());
        rates.push(positive.h.to_bits());
        if i == 64 {
            assert!(b(&positive) > 0.0);
        }
    }
    assert!(rates.windows(2).any(|w| w[0] != w[1]));
    assert!(b(&positive) < 0.0); // follows a permanent change, not a timer returning to b0
    let real = advance(&initial(0.0, 2.0), &event(1, 0.75));
    let x_mutant = advance(&initial(0.0, 2.0), &event(1, -0.75));
    assert_ne!(b(&real), b(&x_mutant)); // replacing recorded a by opposite x is observable
}

#[test]
fn d07_new_defect_bound_strict_rejection_and_old_prior_term_mutant() {
    // §9(6,9): a==b and exact zero defect means no change. A legacy permanent
    // prior predicts nonzero drift and gives a bound that wrongly accepts it.
    let mut s = initial(1.0, 2.0);
    s.prior_mass = Some(1.0);
    s.weight = 1.0;
    s.reaction_sum = [0.0; 4];
    s.values.baseline = nature([0.5; 4]);
    let e = event(1, 0.5);
    let next = frozen(&s, 0.5, 0.5, 1);
    assert_eq!(b(&next), 0.5);
    let bound = bounds::derive(&s, &e, 0.5).unwrap();
    let mut old = s.clone();
    old.prior_mass = None;
    old.law.baseline_weight = 1.0;
    let old_bound = bounds::derive(&old, &e, 0.5).unwrap();
    let mutant = frozen(&old, 0.5, 0.5, 1);
    assert!(bound.single_event[0] < (b(&mutant) - b(&next)).abs());
    assert!(
        bound
            .check(&components(&s.values), &components(&mutant.values))
            .is_err()
    );
    old_bound
        .check(&components(&old.values), &components(&mutant.values))
        .unwrap();
    let limits = bound.limits();
    bound.check(&[0.0; 12], &limits).unwrap();
    for i in 0..12 {
        let mut beyond = limits;
        beyond[i] = beyond[i].next_up();
        assert_eq!(
            bound.check(&[0.0; 12], &beyond),
            Err(Fault::ComputationBound { component: i })
        );
    }
    // A strong but legal input applies and alarms; no clipping/holding candidate.
    let s = initial(0.0, 2.0);
    let before = s.clone();
    let mut alarm = fuses();
    alarm.jump_notice = f64::from_bits(1);
    let good = stage(&s, Some(&event(1, 1.0)), &s.law, &alarm).unwrap();
    assert!(good.signals.contains(&ReviewSignal::Jump));
    assert!(b(good.state()) > 0.0);
    let mut faulty = good.state.clone();
    faulty.values.baseline.connection = 1.0;
    assert!(update::finish(&s, faulty, &event(1, 1.0), &alarm).is_err());
    identical(&s, &before);
    let mut outside = event(1, 1.0);
    outside.reaction.connection = 1.0_f64.next_up();
    assert_eq!(
        stage(&s, Some(&outside), &s.law, &alarm),
        Err(Fault::OutOfRange)
    );
}
