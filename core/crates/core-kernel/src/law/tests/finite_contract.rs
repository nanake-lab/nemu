//! Review 6379618075: nonfinite intermediates must fail before saturation.
use super::cases::{event, experience, on};
use super::engine::candidate;
use super::fixture::{law, parameters, tendency, vector};
use super::state_bits::{group_bits_eq, snapshot_bits_eq, tendency_bits_eq};
use super::world::{Fixture, World, theta};
use crate::law::{GroupContent, GroupItem, Law, LawError};

#[test]
fn helper_nonfinite_support_ratio_is_not_clipped() {
    for p in [0.75, -0.75] {
        let deviations = [[p, 0., 0., 0.]];
        let before = deviations.map(|row| row.map(f64::to_bits));
        assert!((2. * p / f64::from_bits(1)).is_infinite());
        let result = crate::law::experience(
            &deviations,
            &[1., 0., 0., 0.],
            &[0.2; 4],
            f64::from_bits(1),
            1.,
            8.,
        );
        assert!(matches!(result, Err(LawError::NonFinite(_))), "{result:?}");
        assert_eq!(deviations.map(|row| row.map(f64::to_bits)), before);
    }
}

#[test]
fn helper_nonfinite_double_projection_is_not_clipped() {
    let result = crate::law::experience(
        &[[1., 0., 0., 0.]],
        &[f64::MAX, 0., 0., 0.],
        &[1e-160; 4],
        f64::MAX,
        1.,
        8.,
    );
    assert!(
        matches!(result, Err(LawError::NonFinite("2 p_c"))),
        "{result:?}"
    );
}

fn full_g_fault(r_min: f64, r_max: f64, expected: &'static str) {
    for with_prefix in [false, true] {
        let mut p = parameters(3., 1., 0.1, 10.);
        p.r_min = r_min;
        p.r_max = r_max;
        let mut f = Fixture::new(10);
        f.law = Law::new(&p).unwrap(); // All parameters remain legal, no new floor.
        let mut w = World::new(&f);
        let before = w.clone();
        let mut k = candidate("review-k", "review-concept");
        k.direction = vector([1., 0., 0., 0.]);
        let mut groups = Vec::new();
        if with_prefix {
            // Teaching can commit even at both extreme legal r_min values.
            // The later experience fault must discard this staged allocation.
            let prefix = GroupItem {
                group_id: "prefix".into(),
                position: 0,
                source_key: "prefix-src".into(),
                members: vec!["prefix-e".into()],
                verifiable: true,
                content: GroupContent::Teaching {
                    speaker: super::world::CARETAKER.into(),
                    tendency_id: "review-k".into(),
                    supports: true,
                },
            };
            let mut positive_control = event(vec![k.clone()], vec![prefix.clone()]);
            positive_control.theta = theta([0.; 4], [0.2; 4]);
            let mut control_world = World::new(&f);
            let control = control_world.apply(&f, &positive_control).unwrap();
            assert!(!snapshot_bits_eq(&control.r, &before.r));
            assert_eq!(control.groups.len(), 1);
            groups.push(prefix);
        }
        groups.push(experience(
            "review-g",
            1,
            &["review-e"],
            &[[0.75, 0., 0., 0.]],
            vec![on("review-k", 1.)],
        ));
        let mut e = event(vec![k], groups);
        e.theta = theta([0.; 4], [0.2; 4]);
        let result = w.apply(&f, &e);
        assert!(
            matches!(result, Err(LawError::NonFinite(label)) if label == expected),
            "{result:?}"
        );
        assert!(snapshot_bits_eq(&w.r, &before.r));
        assert_eq!(w.groups.len(), before.groups.len());
        for (id, g) in &w.groups {
            assert!(group_bits_eq(g, &before.groups[id]));
        }
        assert_eq!(w.items, before.items);
    }
}

#[test]
fn full_g_nonfinite_support_ratio_is_atomic() {
    // Exact review witness: 2 * .75 / 2^-1074 overflows before clipping.
    full_g_fault(f64::from_bits(1), 1., "support ratio");
}

#[test]
fn helper_and_full_g_strength_overflow_are_not_capped() {
    let result = crate::law::experience(
        &[[0.75, 0., 0., 0.]],
        &[1., 0., 0., 0.],
        &[0.2; 4],
        f64::MAX,
        1.,
        8.,
    );
    assert!(
        matches!(result, Err(LawError::NonFinite("strength ratio"))),
        "{result:?}"
    );
    full_g_fault(f64::MAX, f64::MAX, "strength ratio");
}

#[test]
fn helper_member_projection_and_cap_must_be_finite_before_max_min() {
    // A finite mean must not hide nonfinite individual dot products.
    let result = crate::law::experience(
        &[[2., 0., 0., 0.], [-2., 0., 0., 0.]],
        &[f64::MAX, 0., 0., 0.],
        &[1e-160; 4],
        0.05,
        1.,
        8.,
    );
    assert!(
        matches!(result, Err(LawError::NonFinite("member projection"))),
        "{result:?}"
    );
    for cap in [f64::INFINITY, f64::NAN] {
        let result = crate::law::experience(
            &[[0.75, 0., 0., 0.]],
            &[1., 0., 0., 0.],
            &[0.2; 4],
            0.05,
            1.,
            cap,
        );
        assert!(matches!(result, Err(LawError::NonFinite(_))), "{result:?}");
    }
}

#[test]
fn magnitude_ratio_overflow_faults_and_fold_is_atomic() {
    let l = law(3., 1., 0.1, 10.);
    for sum in [2., -2.] {
        let result = crate::law::replay::typical_magnitude(&l, sum, f64::from_bits(1));
        assert!(
            matches!(result, Err(LawError::NonFinite("magnitude ratio"))),
            "{result:?}"
        );
        let mut t = tendency(&l, "k", None);
        crate::law::record(&mut t, &l, 1., 1.).unwrap();
        t.magnitude_sum = sum;
        t.magnitude_weight = f64::from_bits(1);
        t.typical_magnitude = if sum > 0. { 1. } else { 0. };
        let before = t.clone();
        let result = crate::law::replay::fold_group(&mut t, &l, 1., 0., 0., 0., None, true);
        assert!(
            matches!(result, Err(LawError::NonFinite("magnitude ratio"))),
            "{result:?}"
        );
        assert!(tendency_bits_eq(&t, &before));
    }
}

#[test]
fn finite_saturation_and_legal_subnormal_zero_weight_stay_legal() {
    for (p, support) in [(0.75_f64, 1.0_f64), (-0.75, -1.)] {
        let got = crate::law::experience(
            &[[p, 0., 0., 0.]],
            &[1., 0., 0., 0.],
            &[0.2; 4],
            0.05,
            1.,
            8.,
        )
        .unwrap();
        assert_eq!(got.support.to_bits(), support.to_bits());
        assert_eq!(got.intensity.to_bits(), 3.75f64.to_bits());
    }
    let got = crate::law::experience(
        &[[0.75, 0., 0., 0.]],
        &[1., 0., 0., 0.],
        &[0.2; 4],
        1e300,
        1.,
        8.,
    )
    .unwrap();
    assert_eq!(got.intensity, 8.); // Finite ratios still use the prescribed cap.
    let got = crate::law::experience(
        &[[0.; 4]],
        &[1., 0., 0., 0.],
        &[0.2; 4],
        f64::from_bits(1),
        1.,
        8.,
    )
    .unwrap();
    assert_eq!(got.support, -1.); // The smallest legal r_min is not rejected.
    let l = law(3., 1., 0.1, 10.);
    assert_eq!(
        crate::law::replay::typical_magnitude(&l, 2., 1.).unwrap(),
        1.
    );
    assert_eq!(
        crate::law::replay::typical_magnitude(&l, -2., 1.).unwrap(),
        0.
    );
    assert_eq!(
        crate::law::replay::typical_magnitude(&l, f64::from_bits(1), 0.).unwrap(),
        0.
    );
}
