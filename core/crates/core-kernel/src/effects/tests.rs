use super::*;

fn theta() -> StoredTheta {
    StoredTheta {
        b: [0.1, -0.2, 0.0, 0.3],
        g: [1.0, 0.8, 1.2, 0.5],
        lambda: [0.01, 0.02, 0.005, 0.001],
    }
}

fn unit(v: [f64; 4]) -> [f64; 4] {
    let n = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    [v[0] / n, v[1] / n, v[2] / n, v[3] / n]
}

fn tendency(id: &str, pi: f64, r: f64, d: [f64; 4]) -> StoredTendency {
    StoredTendency {
        tendency_id: id.to_owned(),
        concept: format!("concept/{id}"),
        pi,
        r,
        d,
        direction_validated: true,
    }
}

fn compile(r: &StoredR) -> Result<EffectiveEffects, CompileError> {
    compile_authorized_effects(r, &theta(), ProcessTime(0), &Validity)
}

#[test]
fn i06_candidates_have_no_effect_on_x_or_m() {
    let r = StoredR {
        tendencies: vec![
            tendency("cand-neg", -0.4, 0.5, [1.0, 0.0, 0.0, 0.0]),
            tendency("cand-zero", 0.0, 0.5, [0.0, 1.0, 0.0, 0.0]),
            tendency("cand-half", 0.5, 0.5, [0.0, 0.0, 1.0, 0.0]),
            tendency("trait", 0.75, 0.4, [0.0, 0.0, 0.0, 1.0]),
        ],
    };
    let eff = compile(&r).unwrap();
    assert_eq!(eff.effects.len(), 1);
    assert_eq!(eff.effects[0].tendency_id, "trait");
    assert_eq!(eff.effects[0].s, 0.5);
    // Candidates that the matcher scored are ignored; only R⁺ drives.
    let mu = SituationMatch {
        mu: vec![
            ("cand-neg".into(), 1.0),
            ("cand-zero".into(), 1.0),
            ("cand-half".into(), 1.0),
            ("trait".into(), 0.5),
        ],
    };
    let ev = evaluate_effects(&eff, &mu).unwrap();
    assert_eq!(ev.drives.len(), 1);
    assert_eq!(ev.drives[0].drive, 0.25);
    assert_eq!(ev.drives[0].f, [0.0, 0.0, 0.0, 0.1]);
    assert_eq!(ev.f_sum, [0.0, 0.0, 0.0, 0.1]);
}

#[test]
fn strength_matches_v2_formula_and_has_no_negative_zero() {
    assert_eq!(strength(0.75), 0.5);
    assert_eq!(strength(0.5).to_bits(), 0.0_f64.to_bits());
    assert_eq!(strength(-0.9).to_bits(), 0.0_f64.to_bits());
    assert_eq!(strength(0.999), 2.0 * 0.999 - 1.0);
}

#[test]
fn evaluate_uses_fixed_product_order() {
    let d = unit([1.0, -2.0, 0.5, 3.0]);
    let r = StoredR {
        tendencies: vec![tendency("k", 0.9, 0.37, d)],
    };
    let eff = compile(&r).unwrap();
    let mu = SituationMatch {
        mu: vec![("k".into(), 0.6)],
    };
    let ev = evaluate_effects(&eff, &mu).unwrap();
    let s = 2.0 * 0.9 - 1.0;
    let drive = s * 0.6;
    let scale = drive * 0.37;
    for (f, di) in ev.drives[0].f.iter().zip(d) {
        assert_eq!(f.to_bits(), (scale * di).to_bits());
    }
    assert_eq!(ev.drives[0].drive.to_bits(), drive.to_bits());
}

#[test]
fn unlisted_match_is_zero_and_negative_zero_is_normalized() {
    let r = StoredR {
        tendencies: vec![
            tendency("a", 0.8, 0.5, [0.0, -1.0, 0.0, 0.0]),
            tendency("b", 0.8, 0.5, [-1.0, 0.0, 0.0, 0.0]),
        ],
    };
    let eff = compile(&r).unwrap();
    let mu = SituationMatch {
        mu: vec![("b".into(), -0.0)],
    };
    let ev = evaluate_effects(&eff, &mu).unwrap();
    for drive in &ev.drives {
        assert_eq!(drive.mu.to_bits(), 0.0_f64.to_bits());
        assert_eq!(drive.drive.to_bits(), 0.0_f64.to_bits());
        for x in drive.f {
            assert_eq!(x.to_bits(), 0.0_f64.to_bits());
        }
    }
    for x in ev.f_sum {
        assert_eq!(x.to_bits(), 0.0_f64.to_bits());
    }
}

#[test]
fn effects_are_in_canonical_tendency_order_whatever_the_input_order() {
    // CanonicalV1 set order: length first, then bytes.
    let ids = ["zz", "a", "ab", "b", "aaa"];
    let mut r = StoredR::default();
    for (n, id) in ids.iter().enumerate() {
        r.tendencies.push(tendency(
            id,
            0.6 + 0.05 * n as f64,
            0.1,
            unit([1.0, 1.0, 0.0, 0.0]),
        ));
    }
    let eff = compile(&r).unwrap();
    let order: Vec<&str> = eff.effects.iter().map(|e| e.tendency_id.as_str()).collect();
    assert_eq!(order, ["a", "b", "ab", "zz", "aaa"]);
    r.tendencies.reverse();
    assert_eq!(compile(&r).unwrap(), eff);
}

#[test]
fn i21_inputs_are_only_borrowed_and_repeated_calls_agree() {
    let r = StoredR {
        tendencies: vec![tendency("k", 0.7, 0.2, [0.0, 0.0, 1.0, 0.0])],
    };
    let th = theta();
    let (r0, th0) = (r.clone(), th);
    let first = compile_authorized_effects(&r, &th, ProcessTime(5), &Validity).unwrap();
    let mu = SituationMatch {
        mu: vec![("k".into(), 1.0)],
    };
    let mu0 = mu.clone();
    let e1 = evaluate_effects(&first, &mu).unwrap();
    let e2 = evaluate_effects(&first, &mu).unwrap();
    assert_eq!(e1, e2);
    assert_eq!(r, r0);
    assert_eq!(th, th0);
    assert_eq!(mu, mu0);
    assert_eq!(first.theta, th);
}

#[test]
fn adr0008_composition_law_and_no_time_dependence() {
    let r = StoredR {
        tendencies: (0..20)
            .map(|n| {
                let pi = -0.95 + 0.0975 * n as f64;
                tendency(
                    &format!("k{n}"),
                    pi,
                    0.05 * n as f64,
                    unit([1.0, n as f64, 2.0, -1.0]),
                )
            })
            .collect(),
    };
    let th = theta();
    let at = |t: u64| compile_authorized_effects(&r, &th, ProcessTime(t), &Validity).unwrap();
    let base = at(0);
    for (t, delta) in [
        (0_u64, 0_u64),
        (1, 1),
        (10_u64.pow(9), 3_600 * 10_u64.pow(9)),
        (u64::MAX, u64::MAX),
    ] {
        let projected = project_r(&r, delta);
        let via_projection =
            compile_authorized_effects(&projected, &th, ProcessTime(t), &Validity).unwrap();
        assert_eq!(via_projection, at(t));
        assert_eq!(via_projection, base);
    }
}

#[test]
fn stored_values_outside_their_domain_are_refused() {
    let good = || tendency("k", 0.7, 0.2, [1.0, 0.0, 0.0, 0.0]);
    let cases: Vec<(StoredTendency, &str)> = vec![
        (StoredTendency { pi: 1.0, ..good() }, "PiOutOfRange"),
        (StoredTendency { pi: -1.0, ..good() }, "PiOutOfRange"),
        (
            StoredTendency {
                pi: f64::NAN,
                ..good()
            },
            "NonFinite",
        ),
        (StoredTendency { r: -0.1, ..good() }, "NegativeMagnitude"),
        (
            StoredTendency {
                r: f64::INFINITY,
                ..good()
            },
            "NonFinite",
        ),
        (
            StoredTendency {
                d: [0.9, 0.0, 0.0, 0.0],
                ..good()
            },
            "DirectionNotUnit",
        ),
        (
            StoredTendency {
                d: [1.0, 1e-5, 0.0, 0.0],
                ..good()
            },
            "DirectionNotUnit",
        ),
        (
            StoredTendency {
                d: [f64::NAN, 0.0, 0.0, 0.0],
                ..good()
            },
            "NonFinite",
        ),
        (
            StoredTendency {
                tendency_id: String::new(),
                ..good()
            },
            "EmptyIdentifier",
        ),
        (
            StoredTendency {
                concept: String::new(),
                ..good()
            },
            "EmptyIdentifier",
        ),
    ];
    for (t, want) in cases {
        let err = compile(&StoredR {
            tendencies: vec![t],
        })
        .unwrap_err();
        assert!(format!("{err:?}").starts_with(want), "{err:?} vs {want}");
    }
    // A candidate outside its domain is a fault too, even though it has no effect.
    let bad_candidate = StoredTendency {
        pi: 0.1,
        d: [2.0, 0.0, 0.0, 0.0],
        ..good()
    };
    assert!(matches!(
        compile(&StoredR {
            tendencies: vec![bad_candidate]
        }),
        Err(CompileError::DirectionNotUnit { .. })
    ));
    let dup = StoredR {
        tendencies: vec![good(), good()],
    };
    assert!(matches!(
        compile(&dup),
        Err(CompileError::DuplicateTendency { .. })
    ));
    // A tolerance-sized deviation of a few ulps is accepted (V2-32).
    let near = StoredTendency {
        d: [1.0 + 1e-13, 0.0, 0.0, 0.0],
        ..good()
    };
    assert!(
        compile(&StoredR {
            tendencies: vec![near]
        })
        .is_ok()
    );
}

#[test]
fn theta_outside_its_domain_is_refused() {
    let r = StoredR::default();
    let bad = [
        StoredTheta {
            b: [1.5, 0.0, 0.0, 0.0],
            ..theta()
        },
        StoredTheta {
            g: [0.0, 1.0, 1.0, 1.0],
            ..theta()
        },
        StoredTheta {
            lambda: [1.0, -1.0, 1.0, 1.0],
            ..theta()
        },
        StoredTheta {
            b: [f64::NAN, 0.0, 0.0, 0.0],
            ..theta()
        },
    ];
    for th in bad {
        assert!(compile_authorized_effects(&r, &th, ProcessTime(0), &Validity).is_err());
    }
    let empty = compile_authorized_effects(&r, &theta(), ProcessTime(0), &Validity).unwrap();
    assert!(empty.effects.is_empty());
    let ev = evaluate_effects(&empty, &SituationMatch::default()).unwrap();
    assert_eq!(ev.f_sum, [0.0; 4]);
}

#[test]
fn situation_match_outside_its_domain_is_refused() {
    let r = StoredR {
        tendencies: vec![tendency("k", 0.7, 0.2, [1.0, 0.0, 0.0, 0.0])],
    };
    let eff = compile(&r).unwrap();
    for mu in [1.0 + f64::EPSILON, -0.1, f64::NAN, f64::INFINITY] {
        let m = SituationMatch {
            mu: vec![("k".into(), mu)],
        };
        assert!(matches!(
            evaluate_effects(&eff, &m),
            Err(EvaluateError::MuOutOfRange { .. })
        ));
    }
    let dup = SituationMatch {
        mu: vec![("k".into(), 0.2), ("k".into(), 0.2)],
    };
    assert!(matches!(
        evaluate_effects(&eff, &dup),
        Err(EvaluateError::DuplicateMatch { .. })
    ));
}

#[test]
fn f_sum_is_summed_in_canonical_order() {
    // Values chosen so that the summation order changes the last bit.
    let r = StoredR {
        tendencies: vec![
            tendency("c", 0.9999, 1.0, [1.0, 0.0, 0.0, 0.0]),
            tendency("a", 0.9999, 1e-16, [1.0, 0.0, 0.0, 0.0]),
            tendency("b", 0.9999, 1e-16, [1.0, 0.0, 0.0, 0.0]),
        ],
    };
    let eff = compile(&r).unwrap();
    let mu = SituationMatch {
        mu: vec![("a".into(), 1.0), ("b".into(), 1.0), ("c".into(), 1.0)],
    };
    let ev = evaluate_effects(&eff, &mu).unwrap();
    let f: Vec<f64> = ev.drives.iter().map(|d| d.f[0]).collect();
    let canonical = (0.0 + f[0]) + f[1] + f[2];
    assert_eq!(ev.f_sum[0].to_bits(), canonical.to_bits());
}

#[test]
fn direction_validation_is_passed_through_without_changing_drive() {
    let mut t = tendency("k", 0.8, 0.3, [0.0, 1.0, 0.0, 0.0]);
    let validated = compile(&StoredR {
        tendencies: vec![t.clone()],
    })
    .unwrap();
    t.direction_validated = false;
    let unvalidated = compile(&StoredR {
        tendencies: vec![t],
    })
    .unwrap();
    assert!(validated.effects[0].direction_validated);
    assert!(!unvalidated.effects[0].direction_validated);
    let mu = SituationMatch {
        mu: vec![("k".into(), 0.5)],
    };
    assert_eq!(
        evaluate_effects(&validated, &mu).unwrap(),
        evaluate_effects(&unvalidated, &mu).unwrap()
    );
}
