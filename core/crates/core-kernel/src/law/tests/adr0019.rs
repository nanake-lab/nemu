//! Production G regressions beyond the scalar reference fixture.
use super::cases::{event, experience, find, on};
use super::engine::{candidate, check_world};
use super::world::{Fixture, World, theta};
use crate::law::{GroupContent, LawError, record};

#[test]
fn several_backfills_share_this_append_position_and_use_group_id_order() {
    let f = Fixture::new(20);
    let mut w = World::new(&f);
    let mut early = experience("z", 1, &["e1"], &[[0., 0.6, 0., 0.]], vec![on("a", 1.)]);
    let mut later = experience("b", 2, &["e2"], &[[0., 0.2, 0., 0.]], vec![on("a", 1.)]);
    w.apply(
        &f,
        &event(
            vec![candidate("a", "ca")],
            vec![early.clone(), later.clone()],
        ),
    )
    .unwrap();
    for g in [&mut early, &mut later] {
        if let GroupContent::Experience { matches, .. } = &mut g.content {
            matches.push(on("k", 1.));
        }
    }
    let mut other = w.clone();
    let e = event(vec![candidate("k", "ck")], vec![early, later]);
    let out = w.apply(&f, &e).unwrap();
    let mut reversed = e;
    reversed.groups.reverse();
    other.apply(&f, &reversed).unwrap();
    assert_eq!(w.r, other.r);
    assert_eq!(w.groups, other.groups);
    assert_eq!(out.counterfactual["k"], 0.);
    let ordinal = |gid: &str| {
        w.groups[gid]
            .allocations
            .iter()
            .find(|a| a.tendency_id == "k")
            .unwrap()
            .ordinal
            .0
    };
    assert_eq!((ordinal("b"), ordinal("z")), (0, 1));
    check_world(&f, &w);
}

#[test]
fn change_rate_is_versioned_evidence_only_and_precedes_the_group() {
    use crate::law::{ChangeRateEstimator, EvidenceCounts, FlipFrequency};
    let f = Fixture::new(20);
    let ctx = f.ctx();
    assert_eq!(ctx.change_rate_version(), (&f.law_ref, "flip-frequency-v1"));
    let estimator = FlipFrequency {
        h0: 0.3,
        n_h: 10.,
        minimum: None,
    };
    assert_eq!(
        estimator
            .before_group(EvidenceCounts {
                groups: 0,
                flips: 0
            })
            .unwrap(),
        0.3
    );
    assert_eq!(
        estimator
            .before_group(EvidenceCounts {
                groups: 3,
                flips: 1
            })
            .unwrap(),
        4. / 13.
    );
    assert!(
        estimator
            .before_group(EvidenceCounts {
                groups: 1 << 53,
                flips: 0
            })
            .is_err()
    );
    for h in [0., 0.02, 0.5, 0.999] {
        let mut state = super::fixture::tendency(&f.law, "k", None);
        for _ in 0..3 {
            record(&mut state, &f.law, 3., 1.).unwrap();
        }
        let before = state.pi;
        crate::law::recurrence::apply(&mut state, &f.law, 1. - h, 0.01, 1.).unwrap();
        assert!(state.pi > before);
    }
}

#[test]
fn backfill_cf_omits_first_allocations_and_preserves_all_old_q_bits() {
    let f = Fixture::new(20);
    let mut w = World::new(&f);
    let old = experience("old", 1, &["e1"], &[[0., 0.6, 0., 0.]], vec![on("a", 1.)]);
    let later = experience("later", 2, &["e2"], &[[0., 0.4, 0., 0.]], vec![on("a", 1.)]);
    w.apply(
        &f,
        &event(vec![candidate("a", "ca")], vec![old.clone(), later]),
    )
    .unwrap();
    let before = w.clone();
    let mut back = old;
    if let GroupContent::Experience { matches, .. } = &mut back.content {
        matches.push(on("z", 1.));
    }
    let new = experience(
        "new",
        3,
        &["e3"],
        &[[0., 0.6, 0., 0.]],
        vec![on("a", 1.), on("z", 1.)],
    );
    let out = w
        .apply(&f, &event(vec![candidate("z", "cz")], vec![new, back]))
        .unwrap();
    assert_eq!(out.counterfactual["z"].to_bits(), 0.0f64.to_bits());
    assert!(find(&w, "z").pi > out.counterfactual["z"]);
    assert!(out.counterfactual["a"] <= find(&before, "a").pi);
    assert!(out.after_additions["a"] > out.counterfactual["a"]);
    for (gid, g) in &before.groups {
        for a in &g.allocations {
            let new = w.groups[gid]
                .allocations
                .iter()
                .find(|n| n.tendency_id == a.tendency_id)
                .unwrap();
            assert_eq!(a.actual_retention.to_bits(), new.actual_retention.to_bits());
        }
    }
    check_world(&f, &w);
}

#[test]
fn zero_and_tiny_opposite_backfills_do_not_touch_the_other_side() {
    for support_zero in [true, false] {
        let mut f = Fixture::new(20);
        f.law.r_min = 1.;
        let mut w = World::new(&f);
        let old = experience(
            "old",
            1,
            &["e1"],
            &[[0., 0., 0., 0.]],
            vec![on("a", 1.), on("b", 1.)],
        );
        let mut first = event(
            vec![candidate("a", "ca"), candidate("b", "cb")],
            vec![old.clone()],
        );
        first.theta = theta([0.; 4], [1.; 4]);
        w.apply(&f, &first).unwrap();
        let before = w.clone();
        let mut back = old;
        // Existing S=-1 records stay fixed; z is either exactly zero or tiny positive.
        if let GroupContent::Experience { matches, .. } = &mut back.content {
            matches.push(on("z", if support_zero { 1. } else { 1e-200 }));
        }
        // Only the fresh candidate uses the new baseline; old records stay fixed.
        let mut e = event(vec![candidate("z", "cz")], vec![back]);
        e.theta = theta([0., if support_zero { -0.5 } else { -1. }, 0., 0.], [1.; 4]);
        let out = w.apply(&f, &e).unwrap();
        assert_eq!(find(&w, "a"), find(&before, "a"));
        assert_eq!(find(&w, "b"), find(&before, "b"));
        assert_eq!(
            out.counterfactual["a"].to_bits(),
            find(&before, "a").pi.to_bits()
        );
        for id in ["a", "b"] {
            assert_eq!(
                w.groups["old"]
                    .allocations
                    .iter()
                    .find(|a| a.tendency_id == id),
                before.groups["old"]
                    .allocations
                    .iter()
                    .find(|a| a.tendency_id == id)
            );
        }
        check_world(&f, &w);
    }
}

#[test]
fn late_insertion_fault_discards_candidates_revisions_and_earlier_additions() {
    let mut f = Fixture::new(20);
    f.law.w_max = 1.;
    let mut w = World::new(&f);
    let old = experience("old", 1, &["e1"], &[[0., 0.6, 0., 0.]], vec![on("a", 1.)]);
    w.apply(&f, &event(vec![candidate("a", "ca")], vec![old.clone()]))
        .unwrap();
    let before = w.clone();
    let mut back = old;
    if let GroupContent::Experience { matches, .. } = &mut back.content {
        matches.push(on("z", 1.));
    }
    let tiny = experience(
        "tiny",
        2,
        &["e2"],
        &[[0., 0.6, 0., 0.]],
        vec![on("a", f64::from_bits(1)), on("z", f64::from_bits(1))],
    );
    // W times S, after the nonzero dyadic share, is below the subnormal range.
    let result = w.apply(&f, &event(vec![candidate("z", "cz")], vec![back, tiny]));
    assert!(matches!(
        result,
        Err(LawError::NumericFault("UnderflowEvidence"))
    ));
    assert_eq!(w.r, before.r);
    assert_eq!(w.groups, before.groups);
}

#[test]
fn zero_signal_preserves_evidence_and_q_but_can_add_magnitude() {
    let l = super::fixture::law(3., 1., 0.1, 10.);
    let mut t = super::fixture::tendency(&l, "a", None);
    record(&mut t, &l, 2., 1.).unwrap();
    let before = t.clone();
    let q = crate::law::replay::record_group(&mut t, &l, 3., 0., 0.5, Some(0.4)).unwrap();
    assert_eq!(q, 1.);
    assert_eq!(t.evidence_a.to_bits(), before.evidence_a.to_bits());
    assert_eq!(t.evidence_v.to_bits(), before.evidence_v.to_bits());
    assert_eq!(t.pi.to_bits(), before.pi.to_bits());
    assert_eq!(t.magnitude_sum, 0.2);
    assert_eq!(t.magnitude_weight, 0.5);
}

#[test]
fn capacity_has_a_separate_revision_audit_and_frozen_q() {
    let f = Fixture::new(20);
    let mut w = World::new(&f);
    let old = experience(
        "old",
        1,
        &["e1"],
        &[[0., -0.8, 0., 0.]],
        vec![on("a", 1.), on("b", 0.5)],
    );
    w.apply(
        &f,
        &event(vec![candidate("a", "ca"), candidate("b", "cb")], vec![old]),
    )
    .unwrap();
    let before = w.clone();
    let small = Fixture::new(3);
    let out = w.apply(&small, &event(vec![], vec![])).unwrap();
    assert_eq!(out.capacity_audits.len(), 1);
    let audit = &out.capacity_audits[0];
    for t in &before.r.tendencies {
        assert_eq!(audit.before[&t.tendency_id], t.pi);
    }
    let survivor = w.groups["old"].allocations.first().unwrap();
    let original = before.groups["old"]
        .allocations
        .iter()
        .find(|a| a.tendency_id == survivor.tendency_id)
        .unwrap();
    assert_eq!(
        survivor.actual_retention.to_bits(),
        original.actual_retention.to_bits()
    );
    assert!(audit.after[&survivor.tendency_id] <= audit.before[&survivor.tendency_id]);
    check_world(&small, &w);
}
