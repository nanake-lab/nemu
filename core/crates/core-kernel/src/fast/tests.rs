use super::*;
use crate::effects::{
    EffectiveEffects, Evaluation, ProcessTime, SituationMatch, StoredR, StoredTendency,
    StoredTheta, Validity, compile_authorized_effects, evaluate_effects,
};

const FIXTURE: &str = include_str!("../../../../../profiles/structural_fixture/fast/fixture.json");

struct Fixture {
    theta: StoredTheta,
    law: DriveLaw,
}

fn arr(v: &serde_json::Value) -> [f64; 4] {
    let a = v.as_array().unwrap();
    [0, 1, 2, 3].map(|i| a[i].as_f64().unwrap())
}

fn fixture() -> Fixture {
    let v: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    Fixture {
        theta: StoredTheta {
            b: arr(&v["b"]),
            g: arr(&v["g"]),
            lambda: arr(&v["lambda_per_s"]),
        },
        law: DriveLaw {
            g_max: v["g_max"].as_f64().unwrap(),
            r_max: v["r_max"].as_f64().unwrap(),
            w_min: v["w_min"].as_f64().unwrap(),
            kappa: v["kappa"].as_f64().unwrap(),
        },
    }
}

/// xorshift64*: deterministic test randomness without a dependency.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn sym(&mut self) -> f64 {
        2.0 * self.unit() - 1.0
    }
}

fn unit_dir(v: [f64; 4]) -> [f64; 4] {
    let n = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    v.map(|x| x / n)
}

fn effects_with(theta: StoredTheta, n: usize, rng: &mut Rng) -> EffectiveEffects {
    let tendencies = (0..n)
        .map(|k| StoredTendency {
            tendency_id: format!("t{k}"),
            concept: format!("c{k}"),
            pi: 0.55 + 0.4 * rng.unit(),
            r: rng.unit(),
            d: unit_dir([rng.sym(), rng.sym(), rng.sym(), rng.sym() + 0.01]),
            direction_validated: true,
        })
        .collect();
    compile_authorized_effects(&StoredR { tendencies }, &theta, ProcessTime(0), &Validity).unwrap()
}

fn random_x(e: &EffectiveEffects, rng: &mut Rng) -> FastState {
    FastState {
        nature: [rng.sym(), rng.sym(), rng.sym(), rng.sym()],
        learned: e
            .effects
            .iter()
            .map(|t| LearnedValue {
                tendency_id: t.tendency_id.clone(),
                value: rng.sym(),
            })
            .collect(),
    }
}

fn random_event(e: &EffectiveEffects, rng: &mut Rng) -> (Evaluation, EventInput) {
    let mu = SituationMatch {
        mu: e
            .effects
            .iter()
            .map(|t| (t.tendency_id.clone(), rng.unit()))
            .collect(),
    };
    let class = match rng.next() % 4 {
        0 => EventClass::NewExogenous,
        1 => EventClass::OtherExternal,
        2 => EventClass::InternalRecord { n: rng.next() % 6 },
        _ => EventClass::ActionResult {
            n_issued: rng.next() % 6,
        },
    };
    let input = EventInput {
        appraisal: (!rng.next().is_multiple_of(5))
            .then(|| [rng.sym(), rng.sym(), rng.sym(), rng.sym()]),
        empathy_weight: 0.1 + 0.9 * rng.unit(),
        class,
    };
    (evaluate_effects(e, &mu).unwrap(), input)
}

fn lambda_min(e: &EffectiveEffects) -> f64 {
    let mut m = e.theta.lambda.iter().copied().fold(f64::INFINITY, f64::min);
    for t in &e.effects {
        let num: f64 = (0..4).map(|i| t.d[i].abs() * e.theta.lambda[i]).sum();
        let den: f64 = t.d.iter().map(|d| d.abs()).sum();
        m = m.min(num / den);
    }
    m
}

#[test]
fn ac2_15_contraction_holds_on_intervals_with_events() {
    let f = fixture();
    let mut rng = Rng(0x5eed_0010);
    for trial in 0..200 {
        let theta = StoredTheta {
            b: [
                rng.sym() * 0.5,
                rng.sym() * 0.5,
                rng.sym() * 0.5,
                rng.sym() * 0.5,
            ],
            g: [0.2 + 0.8 * rng.unit(), 0.2 + 0.8 * rng.unit(), 1.0, 0.5],
            lambda: [
                0.2 + rng.unit(),
                0.2 + rng.unit(),
                0.2 + rng.unit(),
                0.2 + rng.unit(),
            ],
        };
        let e = effects_with(theta, (trial % 4) as usize, &mut rng);
        let lmin = lambda_min(&e);
        let mut x = random_x(&e, &mut rng);
        let mut y = random_x(&e, &mut rng);
        let d0 = distance(&x, &y).unwrap();
        let mut t = 0.0_f64;
        for _ in 0..30 {
            let dt_ns = rng.next() % 3_000_000_000;
            t += dt_ns as f64 / 1e9;
            x = project_time(&x, &e, dt_ns).unwrap();
            y = project_time(&y, &e, dt_ns).unwrap();
            let (ev, input) = random_event(&e, &mut rng);
            let drive = compute_drive(&e, &ev, &input, &f.law).unwrap();
            x = apply_drive(&x, &drive).unwrap();
            y = apply_drive(&y, &drive).unwrap();
            let bound = (-lmin * t).exp() * d0;
            let d = distance(&x, &y).unwrap();
            assert!(
                d <= bound * (1.0 + 1e-9) + 1e-15,
                "trial {trial}: {d} > {bound}"
            );
        }
    }
}

#[test]
fn ac2_15_without_events_x_relaxes_to_rest() {
    let mut rng = Rng(7);
    let theta = StoredTheta {
        b: [0.3, -0.4, 0.0, 0.9],
        g: [1.0; 4],
        lambda: [0.5, 1.0, 2.0, 0.25],
    };
    let e = effects_with(theta, 3, &mut rng);
    let rest = FastState {
        nature: theta.b,
        learned: e
            .effects
            .iter()
            .map(|t| LearnedValue {
                tendency_id: t.tendency_id.clone(),
                value: 0.0,
            })
            .collect(),
    };
    let mut x = random_x(&e, &mut rng);
    let mut prev = distance(&x, &rest).unwrap();
    for _ in 0..200 {
        x = project_time(&x, &e, 500_000_000).unwrap();
        let d = distance(&x, &rest).unwrap();
        assert!(d <= prev);
        prev = d;
    }
    assert!(prev < 1e-9, "{prev}");
}

#[test]
fn zero_delta_is_identity_bit_for_bit() {
    let theta = StoredTheta {
        b: [0.5, -0.5, 1.0, -1.0],
        g: [1.0; 4],
        lambda: [1.0, 3.0, 0.1, 9.0],
    };
    let mut rng = Rng(11);
    let e = effects_with(theta, 2, &mut rng);
    let samples = [1e-17, -1e-300, 0.999_999_999_999, -1.0, 1.0, 0.0, 5e-324];
    for &v in &samples {
        let mut x = random_x(&e, &mut rng);
        x.nature = [v; 4];
        for l in &mut x.learned {
            l.value = v;
        }
        let y = project_time(&x, &e, 0).unwrap();
        for i in 0..4 {
            assert_eq!(y.nature[i].to_bits(), x.nature[i].to_bits());
        }
        for (a, b) in y.learned.iter().zip(&x.learned) {
            assert_eq!(a.value.to_bits(), b.value.to_bits());
        }
    }
}

#[test]
fn f01_piecewise_projection_and_no_accumulated_decay() {
    let mut rng = Rng(13);
    let theta = StoredTheta {
        b: [0.1, 0.2, -0.3, 0.0],
        g: [1.0; 4],
        lambda: [0.3, 0.7, 1.1, 2.0],
    };
    let e = effects_with(theta, 2, &mut rng);
    for _ in 0..500 {
        let x = random_x(&e, &mut rng);
        let s = rng.next() % 5_000_000_000;
        let t = rng.next() % 5_000_000_000;
        let once = project_time(&x, &e, s + t).unwrap();
        let twice = project_time(&project_time(&x, &e, s).unwrap(), &e, t).unwrap();
        assert!(distance(&once, &twice).unwrap() <= 1e-12);
        // Reading again from the same anchor does not decay further.
        assert_eq!(project_time(&x, &e, s + t).unwrap(), once);
    }
}

#[test]
fn f02_projection_and_drive_stay_in_x() {
    let f = fixture();
    let mut rng = Rng(17);
    for _ in 0..20_000 {
        let theta = StoredTheta {
            b: [rng.sym(), rng.sym(), rng.sym(), rng.sym()],
            g: [1.0; 4],
            lambda: [
                rng.unit() + 1e-9,
                rng.unit() + 1e-9,
                1e3 * rng.unit() + 1e-9,
                1.0,
            ],
        };
        let e = effects_with(theta, 2, &mut rng);
        let x = random_x(&e, &mut rng);
        let y = project_time(&x, &e, rng.next() % 100_000_000_000).unwrap();
        validate(&y).unwrap();
        let (ev, input) = random_event(&e, &mut rng);
        let z = apply_drive(&y, &compute_drive(&e, &ev, &input, &f.law).unwrap()).unwrap();
        validate(&z).unwrap();
    }
}

#[test]
fn f02_invalid_inputs_are_refused_not_repaired() {
    let f = fixture();
    let mut rng = Rng(19);
    let e = effects_with(f.theta, 1, &mut rng);
    let ev = evaluate_effects(
        &e,
        &SituationMatch {
            mu: vec![("t0".into(), 1.0)],
        },
    )
    .unwrap();
    let ok = EventInput {
        appraisal: Some([1.0, -1.0, 0.5, 0.0]),
        empathy_weight: 1.0,
        class: EventClass::NewExogenous,
    };
    assert!(compute_drive(&e, &ev, &ok, &f.law).is_ok());
    let nan = EventInput {
        appraisal: Some([f64::NAN, 0.0, 0.0, 0.0]),
        ..ok
    };
    assert_eq!(
        compute_drive(&e, &ev, &nan, &f.law),
        Err(FastError::NonFinite("appraisal"))
    );
    let wide = EventInput {
        appraisal: Some([1.5, 0.0, 0.0, 0.0]),
        ..ok
    };
    assert!(matches!(
        compute_drive(&e, &ev, &wide, &f.law),
        Err(FastError::OutOfRange(_))
    ));
    let low_w = EventInput {
        empathy_weight: 0.01,
        ..ok
    };
    assert!(compute_drive(&e, &ev, &low_w, &f.law).is_err());
    // An oversized gain (g > g_max) makes the conservative bound fail: refused, not scaled.
    let mut big = e.clone();
    big.theta.g = [5.0; 4];
    assert!(matches!(
        compute_drive(&big, &ev, &ok, &f.law),
        Err(FastError::DriveOverBound { .. })
    ));
    // Dimension mismatch between the drive and x, or between x and E_eff.
    let x = random_x(&e, &mut rng);
    let other = effects_with(f.theta, 3, &mut rng);
    assert_eq!(project_time(&x, &other, 1), Err(FastError::Misaligned));
    let drive = compute_drive(&e, &ev, &ok, &f.law).unwrap();
    let x_other = random_x(&other, &mut rng);
    assert_eq!(apply_drive(&x_other, &drive), Err(FastError::Misaligned));
    // Points outside X and non-finite coordinates are rejected.
    let mut bad = x.clone();
    bad.nature[0] = 1.0 + f64::EPSILON;
    assert!(validate(&bad).is_err());
    bad.nature[0] = f64::NAN;
    assert!(validate(&bad).is_err());
    // A huge but legal drive saturates at the boundary.
    let edge = FastState {
        nature: [1.0, -1.0, 0.0, 0.0],
        ..x
    };
    let z = apply_drive(&edge, &drive).unwrap();
    validate(&z).unwrap();
}

#[test]
fn f03_two_histories_converge_after_driving_stops() {
    let f = fixture();
    let mut rng = Rng(23);
    let e = effects_with(f.theta, 2, &mut rng);
    let mut x = random_x(&e, &mut rng);
    let mut y = random_x(&e, &mut rng);
    for _ in 0..10 {
        let (ev, ix) = random_event(&e, &mut rng);
        let (ev2, iy) = random_event(&e, &mut rng);
        x = apply_drive(
            &project_time(&x, &e, 100_000_000).unwrap(),
            &compute_drive(&e, &ev, &ix, &f.law).unwrap(),
        )
        .unwrap();
        y = apply_drive(
            &project_time(&y, &e, 100_000_000).unwrap(),
            &compute_drive(&e, &ev2, &iy, &f.law).unwrap(),
        )
        .unwrap();
    }
    let eps: f64 = 1e-6;
    let lmin = lambda_min(&e);
    let t_x = (2.0_f64 / eps).ln() / lmin;
    let t_ns = (t_x * 1e9).ceil() as u64;
    let d = distance(
        &project_time(&x, &e, t_ns).unwrap(),
        &project_time(&y, &e, t_ns).unwrap(),
    )
    .unwrap();
    assert!(d <= eps, "{d}");
}

#[test]
fn f03_lambda_zero_is_rejected() {
    let f = fixture();
    let zero = StoredTheta {
        lambda: [1.0, 0.0, 1.0, 1.0],
        ..f.theta
    };
    assert!(
        compile_authorized_effects(&StoredR::default(), &zero, ProcessTime(0), &Validity).is_err()
    );
    let mut rng = Rng(29);
    let mut e = effects_with(f.theta, 0, &mut rng);
    e.theta.lambda[1] = 0.0;
    let x = random_x(&e, &mut rng);
    assert!(matches!(
        project_time(&x, &e, 1),
        Err(FastError::OutOfRange("lambda"))
    ));
}

#[test]
fn f05_f_out_is_identity_when_r_plus_is_unchanged_and_aligns_otherwise() {
    let f = fixture();
    let mut rng = Rng(31);
    let e = effects_with(f.theta, 3, &mut rng);
    let x = random_x(&e, &mut rng);
    assert_eq!(f_out(&x, &e).unwrap(), x);
    // t1 leaves R⁺, t9 enters.
    let mut next = e.clone();
    next.effects.retain(|t| t.tendency_id != "t1");
    let mut entered = e.effects[0].clone();
    entered.tendency_id = "t9".into();
    next.effects.push(entered);
    let y = f_out(&x, &next).unwrap();
    assert_eq!(y.nature, x.nature);
    let ids: Vec<&str> = y.learned.iter().map(|l| l.tendency_id.as_str()).collect();
    assert_eq!(ids, ["t0", "t2", "t9"]);
    assert_eq!(y.learned[0].value.to_bits(), x.learned[0].value.to_bits());
    assert_eq!(y.learned[1].value.to_bits(), x.learned[2].value.to_bits());
    assert_eq!(y.learned[2].value, 0.0);
}

#[test]
fn f05_same_event_gives_the_same_pulse() {
    let f = fixture();
    let mut rng = Rng(37);
    let e = effects_with(f.theta, 2, &mut rng);
    let x = random_x(&e, &mut rng);
    let (ev, input) = random_event(&e, &mut rng);
    let d1 = compute_drive(&e, &ev, &input, &f.law).unwrap();
    let d2 = compute_drive(&e, &ev, &input, &f.law).unwrap();
    assert_eq!(d1, d2);
    assert_eq!(apply_drive(&x, &d1).unwrap(), apply_drive(&x, &d2).unwrap());
}

#[test]
fn drive_signature_has_no_fast_state() {
    // ADR 0008 §1: D(e) must not read x̄. Enforced by the type of compute_drive.
    let _: fn(&EffectiveEffects, &Evaluation, &EventInput, &DriveLaw) -> Result<Drive, FastError> =
        compute_drive;
}

#[test]
fn self_caused_events_are_weighted_by_kappa_powers() {
    let f = fixture();
    let mut rng = Rng(41);
    let e = effects_with(f.theta, 1, &mut rng);
    let ev = evaluate_effects(
        &e,
        &SituationMatch {
            mu: vec![("t0".into(), 1.0)],
        },
    )
    .unwrap();
    let base = EventInput {
        appraisal: Some([0.8, 0.0, -0.4, 0.2]),
        empathy_weight: 1.0,
        class: EventClass::NewExogenous,
    };
    let full = compute_drive(&e, &ev, &base, &f.law).unwrap();
    for n in 0..8 {
        let k = kappa_power(f.law.kappa, n);
        let internal = compute_drive(
            &e,
            &ev,
            &EventInput {
                class: EventClass::InternalRecord { n },
                ..base
            },
            &f.law,
        )
        .unwrap();
        let result = compute_drive(
            &e,
            &ev,
            &EventInput {
                class: EventClass::ActionResult { n_issued: n },
                ..base
            },
            &f.law,
        )
        .unwrap();
        for i in 0..4 {
            assert_eq!(internal.nature[i].to_bits(), (k * full.nature[i]).to_bits());
            assert_eq!(result.nature[i], internal.nature[i]);
        }
        assert_eq!(internal.learned[0].value, k * full.learned[0].value);
    }
    let other = compute_drive(
        &e,
        &ev,
        &EventInput {
            class: EventClass::OtherExternal,
            ..base
        },
        &f.law,
    )
    .unwrap();
    assert_eq!(other, full);
}

#[test]
fn kappa_power_matches_direct_products_and_vanishes_for_large_n() {
    let k = 0.83_f64;
    let mut naive = 1.0_f64;
    // n = 0, 1, 2 involve the same operations as the direct product.
    for n in 0..3 {
        assert_eq!(kappa_power(k, n), naive, "n = {n}");
        naive *= k;
    }
    assert_eq!(kappa_power(0.5, 10), 1.0 / 1024.0);
    assert_eq!(kappa_power(0.5, 2000).to_bits(), 0.0_f64.to_bits());
    assert!(kappa_power(k, 40) < kappa_power(k, 39));
}

#[test]
fn internal_count_follows_the_event_classes() {
    assert_eq!(
        next_internal_count(EventClass::InternalRecord { n: 0 }, 0),
        1
    );
    assert_eq!(
        next_internal_count(EventClass::InternalRecord { n: 4 }, 4),
        5
    );
    assert_eq!(next_internal_count(EventClass::NewExogenous, 9), 0);
    assert_eq!(next_internal_count(EventClass::OtherExternal, 9), 9);
    assert_eq!(
        next_internal_count(EventClass::ActionResult { n_issued: 2 }, 9),
        9
    );
}

#[test]
fn missing_appraisal_keeps_only_the_tendency_effects() {
    let f = fixture();
    let mut rng = Rng(43);
    let e = effects_with(f.theta, 2, &mut rng);
    let ev = evaluate_effects(
        &e,
        &SituationMatch {
            mu: vec![("t0".into(), 0.5), ("t1".into(), 1.0)],
        },
    )
    .unwrap();
    let input = EventInput {
        appraisal: None,
        empathy_weight: 1.0,
        class: EventClass::NewExogenous,
    };
    let d = compute_drive(&e, &ev, &input, &f.law).unwrap();
    for i in 0..4 {
        assert_eq!(d.nature[i], ev.f_sum[i]);
    }
}

#[test]
fn empathy_weight_takes_the_closest_person() {
    assert_eq!(empathy_weight(&[], 0.1).unwrap(), 1.0);
    assert_eq!(empathy_weight(&[0.0], 0.1).unwrap(), 0.5);
    assert_eq!(empathy_weight(&[-0.9, 0.6], 0.1).unwrap(), 0.8);
    assert_eq!(empathy_weight(&[-0.95], 0.1).unwrap(), 0.1);
    assert!(empathy_weight(&[1.0], 0.1).is_err());
    assert!(empathy_weight(&[0.2], 0.0).is_err());
}

#[test]
fn snapshot_round_trip_is_bit_exact() {
    let mut rng = Rng(47);
    let f = fixture();
    let e = effects_with(f.theta, 3, &mut rng);
    for _ in 0..50 {
        let x = random_x(&e, &mut rng);
        let bytes = serialize(&x, "rev-1", 123_456_789).unwrap();
        let (y, rev, anchor) = deserialize(&bytes).unwrap();
        assert_eq!((rev.as_str(), anchor), ("rev-1", 123_456_789));
        assert_eq!(serialize(&y, "rev-1", 123_456_789).unwrap(), bytes);
        for i in 0..4 {
            assert_eq!(y.nature[i].to_bits(), x.nature[i].to_bits());
        }
    }
    let empty = FastState {
        nature: [0.0; 4],
        learned: vec![],
    };
    assert!(
        serialize(&empty, "", 0).is_err(),
        "revision must be non-empty"
    );
}

#[test]
fn snapshot_with_unknown_representation_is_refused() {
    use core_types::WireMessage;
    use core_types::generated::{FastStateSnapshotV2, NatureVector};
    let msg = FastStateSnapshotV2 {
        schema_version: 1,
        revision: "r".into(),
        representation_version: "other/1".into(),
        anchor_time_ns: core_types::scalar::DecimalU64(0),
        nature: NatureVector {
            schema_version: 1,
            connection: 0.0,
            empathy: 0.0,
            curiosity: 0.0,
            achievement: 0.0,
        },
        learned: vec![],
    };
    let limits = core_types::contract::Limits {
        max_bytes: 1 << 20,
        max_items: 4096,
        max_depth: 32,
    };
    let bytes = msg.canonical_bytes(limits).unwrap();
    assert!(matches!(
        deserialize(&bytes),
        Err(FastError::RepresentationVersion(_))
    ));
}

#[test]
fn recovery_reset_and_washout() {
    let mut rng = Rng(53);
    let f = fixture();
    let e = effects_with(f.theta, 2, &mut rng);
    let (x, brk) = reset_to_init([0.1, 0.0, -0.1, 0.0], &e).unwrap();
    assert_eq!(brk, ContinuityBreak::Reset);
    assert!(x.learned.iter().all(|l| l.value == 0.0));
    assert!(reset_to_init([2.0, 0.0, 0.0, 0.0], &e).is_err());
    // 2·e^{−λt} ≤ ε  ⇔  t ≥ ln(2/ε)/λ.
    let eps = 1e-3;
    let t = (2.0_f64 / eps).ln();
    assert!(!washout_sufficient(((t - 0.01) * 1e9) as u64, 1.0, eps).unwrap());
    assert!(washout_sufficient(((t + 0.01) * 1e9) as u64, 1.0, eps).unwrap());
    assert!(washout_sufficient(1, 0.0, eps).is_err());
}
