use super::*;
use crate::fast::space::learned_rate;

fn compiled(d: [f64; 4], lambda: [f64; 4]) -> EffectiveEffects {
    compile_authorized_effects(
        &StoredR {
            tendencies: vec![StoredTendency {
                tendency_id: "t".into(),
                concept: "c".into(),
                pi: 0.75,
                r: 1.0,
                d,
                direction_validated: true,
            }],
        },
        &StoredTheta {
            b: [0.0; 4],
            g: [1.0; 4],
            lambda,
        },
        ProcessTime(0),
        &Validity,
    )
    .unwrap()
}

#[test]
fn f02_extreme_finite_rates_through_authorized_compiler() {
    for rate in [1e308, f64::MAX, f64::MIN_POSITIVE, f64::from_bits(1)] {
        let e = compiled([0.5; 4], [rate; 4]);
        let x = FastState::new(
            [0.25; 4],
            vec![LearnedValue {
                tendency_id: "t".into(),
                value: 0.25,
            }],
        )
        .unwrap();
        assert_eq!(
            learned_rate(&e.effects[0].d, &e.theta.lambda)
                .unwrap()
                .to_bits(),
            rate.to_bits()
        );
        for delta in [0, 1] {
            let y = project_time(&x, &e, delta).unwrap();
            validate(&y).unwrap();
            if delta == 0 {
                assert_eq!(
                    serialize(&x, "r", 0).unwrap(),
                    serialize(&y, "r", 0).unwrap()
                );
            }
        }
    }
    let e = compiled([0.5; 4], [1e308; 4]);
    let x = FastState::new(
        [0.25; 4],
        vec![LearnedValue {
            tendency_id: "t".into(),
            value: 0.25,
        }],
    )
    .unwrap();
    assert_eq!(
        project_time(&x, &e, u64::MAX),
        Err(FastError::NonFinite("lambda_delta"))
    );
}

#[test]
fn learned_rate_matches_independent_reference_and_range() {
    // (0.6*1 + 0.8*8)/(0.6+0.8) = 5, regardless of direction signs.
    for d in [[0.6, 0.8, 0.0, 0.0], [-0.6, 0.8, -0.0, 0.0]] {
        let e = compiled(d, [1.0, 8.0, 20.0, 100.0]);
        assert!((learned_rate(&e.effects[0].d, &e.theta.lambda).unwrap() - 5.0).abs() <= 2e-15);
    }
    let tiny = f64::from_bits(1);
    let e = compiled([1.0, tiny, 0.0, 0.0], [1.0, 1e308, 1.0, 1.0]);
    assert_eq!(
        learned_rate(&e.effects[0].d, &e.theta.lambda).unwrap(),
        1.0 + tiny * 1e308
    );
    let mut rng = Rng(79);
    for _ in 0..10_000 {
        let d = unit_dir([rng.sym(), rng.sym(), rng.sym(), rng.sym() + 0.01]);
        let lambda = std::array::from_fn(|_| 0.01 + 100.0 * rng.unit());
        let e = compiled(d, lambda);
        // The direct quotient is safe on this bounded reference domain.
        let reference = (0..4).map(|i| d[i].abs() * lambda[i]).sum::<f64>()
            / d.iter().map(|v| v.abs()).sum::<f64>();
        let actual = learned_rate(&e.effects[0].d, &e.theta.lambda).unwrap();
        let min = lambda.iter().copied().fold(f64::INFINITY, f64::min);
        let max = lambda.iter().copied().fold(0.0, f64::max);
        assert!((min..=max).contains(&actual));
        assert!((actual - reference).abs() <= 4e-14 * reference);
    }
}

#[test]
fn f02_projection_refuses_invalid_derived_rate_and_baseline() {
    let mut e = compiled([0.5; 4], [1.0; 4]);
    let x = FastState::new(
        [0.0; 4],
        vec![LearnedValue {
            tendency_id: "t".into(),
            value: 0.25,
        }],
    )
    .unwrap();
    for direction in [[0.0; 4], [f64::NAN; 4], [f64::INFINITY; 4]] {
        e.effects[0].d = direction;
        assert!(project_time(&x, &e, 0).is_err());
    }
    e.effects[0].d = [0.5; 4];
    for baseline in [f64::NAN, f64::INFINITY, 1.01] {
        e.theta.b[0] = baseline;
        assert!(project_time(&x, &e, 0).is_err());
    }
}
