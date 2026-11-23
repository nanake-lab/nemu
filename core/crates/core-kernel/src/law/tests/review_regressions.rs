//! PR #49: late repetitions, first-creation aliases and removal provenance.
//! These assertions reproduce the review failures before their fixes.

use crate::law::{GroupContent, LawError};

use super::cases::{event, experience, find, on};
use super::engine::{candidate, check_world};
use super::world::{Fixture, World, theta};

#[test]
fn ac2_03_late_repetition_preserves_allocations_after_r_or_theta_changes() {
    for changed_theta in [false, true] {
        let f = Fixture::new(10);
        let mut w = World::new(&f);
        let g1 = experience(
            "g1",
            1,
            &["e1"],
            &[[0.0, 0.2, 0.0, 0.0]],
            vec![on("a", 1.0)],
        );
        let g2 = experience(
            "g2",
            2,
            &["e2"],
            &[[0.0, 0.6, 0.0, 0.0]],
            vec![on("a", 1.0)],
        );
        w.apply(&f, &event(vec![candidate("a", "ca")], vec![g1, g2]))
            .unwrap();
        let before = find(&w, "a").clone();
        let allocation = w.groups["g1"].allocations.clone();
        for repeats in [2, 3, 9] {
            let members: Vec<String> = (0..repeats).map(|i| format!("repeat-{i}")).collect();
            let mut ids: Vec<&str> = members.iter().map(String::as_str).collect();
            ids[0] = "e1";
            let late = experience(
                "g1",
                1,
                &ids,
                &vec![[0.0, 0.2, 0.0, 0.0]; repeats],
                vec![on("a", 1.0)],
            );
            let mut e = event(vec![], vec![late]);
            if changed_theta {
                e.theta = theta([-0.3, 0.1, 0.1, 0.1], [0.4, 0.15, 0.3, 0.2]);
            }
            let out = w.apply(&f, &e).unwrap();
            assert_eq!(w.groups["g1"].members.len(), repeats);
            assert_eq!(w.groups["g1"].allocations, allocation);
            assert_eq!(find(&w, "a"), &before);
            assert!(out.changes.is_empty());
            check_world(&f, &w);
        }
    }
}

#[test]
fn ac2_03_late_repetition_of_a_mixed_group_preserves_both_sides() {
    let f = Fixture::new(10);
    let mut w = World::new(&f);
    let mut opposite = candidate("b", "cb");
    opposite.direction.empathy = -1.0;
    let apps = [[0.0, 0.2, 0.0, 0.0], [0.0, 0.6, 0.0, 0.0]];
    let matches = vec![on("a", 1.0), on("b", 0.5)];
    w.apply(
        &f,
        &event(
            vec![candidate("a", "ca"), opposite],
            vec![experience("g1", 1, &["e1", "e2"], &apps, matches.clone())],
        ),
    )
    .unwrap();
    let before = w.r.tendencies.clone();
    let old = w.groups["g1"].allocations.clone();
    // Whole-group repetition in a different member order, after theta changed.
    let late = experience(
        "g1",
        1,
        &["e4", "e1", "e3", "e2"],
        &[apps[1], apps[0], apps[0], apps[1]],
        matches,
    );
    let mut e = event(vec![], vec![late]);
    e.theta = theta([0.3, 0.1, 0.0, 0.1], [0.2, 0.1, 0.25, 0.15]);
    let out = w.apply(&f, &e).unwrap();
    assert_eq!(w.groups["g1"].allocations, old);
    assert_eq!(w.r.tendencies, before);
    assert!(out.changes.is_empty());
    check_world(&f, &w);
}

#[test]
fn aliases_cannot_win_first_creation_by_record_id_or_match_order() {
    let f = Fixture::new(10);
    let original = candidate("rec-b-00", "canon");
    let group = experience(
        "g1",
        1,
        &["e1"],
        &[[0.0, 0.6, 0.0, 0.0]],
        vec![on("rec-b-00", 1.0)],
    );
    let mut plain = World::new(&f);
    plain
        .apply(&f, &event(vec![original.clone()], vec![group.clone()]))
        .unwrap();
    for count in [1, 2, 20] {
        for reverse in [false, true] {
            let mut e = event(vec![original.clone()], vec![group.clone()]);
            for i in 0..count {
                let id = format!("rec-a-{i:02}");
                let alias = format!("synonym-{i}");
                e.candidates.push(candidate(&id, &alias));
                e.aliases.insert(alias, "canon".into());
                if let GroupContent::Experience { matches, .. } = &mut e.groups[0].content {
                    matches.push(on(&id, 0.1 + 0.04 * f64::from(i)));
                }
            }
            if reverse {
                e.candidates.reverse();
                if let GroupContent::Experience { matches, .. } = &mut e.groups[0].content {
                    matches.reverse();
                }
            }
            let mut aliased = World::new(&f);
            aliased.apply(&f, &e).unwrap();
            assert_eq!(aliased.r, plain.r);
            assert_eq!(aliased.groups, plain.groups);
            check_world(&f, &aliased);
        }
    }
}

#[test]
fn capacity_removal_lists_groups_even_for_zero_support_or_empty_sides() {
    for zero_support in [false, true] {
        // The removed tendency is the group's only allocation.
        let f = Fixture::new(10);
        let mut w = World::new(&f);
        let mut e = event(
            vec![candidate("a", "ca")],
            vec![experience(
                "g1",
                1,
                &["e1"],
                &[[0.0, -0.2, 0.0, 0.0]],
                vec![on("a", 1.0)],
            )],
        );
        // Exact zero support: p = r_min/2 without subtractive cancellation.
        if zero_support {
            e.theta = theta([0.0; 4], [0.25; 4]);
            if let GroupContent::Experience { appraisals, .. } = &mut e.groups[0].content {
                appraisals[0].empathy = f.law.r_min / 2.0;
            }
        }
        w.apply(&f, &e).unwrap();
        if zero_support {
            assert_eq!(w.groups["g1"].allocations[0].support, 0.0);
        }
        let smaller = Fixture::new(2);
        let out = w.apply(&smaller, &event(vec![], vec![])).unwrap();
        let removed = out.changes.iter().find(|c| c.tendency_id == "a").unwrap();
        assert_eq!(removed.pi_after, None);
        assert_eq!(removed.evidence_groups, ["g1"]);
        assert!(w.groups["g1"].allocations.is_empty());
        check_world(&smaller, &w);
    }
}

#[test]
fn late_group_requires_the_full_immutable_member_set() {
    let f = Fixture::new(10);
    let mut w = World::new(&f);
    let first = experience(
        "g1",
        1,
        &["e1"],
        &[[0.0, 0.2, 0.0, 0.0]],
        vec![on("a", 1.0)],
    );
    w.apply(&f, &event(vec![candidate("a", "ca")], vec![first]))
        .unwrap();
    let before = w.r.clone();
    // Passing only the new member cannot establish whether the group repeats.
    let incomplete = experience(
        "g1",
        1,
        &["e2"],
        &[[0.0, 0.2, 0.0, 0.0]],
        vec![on("a", 1.0)],
    );
    assert!(matches!(
        w.apply(&f, &event(vec![], vec![incomplete])),
        Err(LawError::InconsistentState(_))
    ));
    assert_eq!(w.r, before);
}

#[test]
fn competing_aliases_need_a_representative_but_a_sole_alias_can_create_it() {
    let f = Fixture::new(10);
    let mut w = World::new(&f);
    let before = w.r.clone();
    for count in [1, 2, 20] {
        let mut e = event(vec![], vec![]);
        let mut matches = Vec::new();
        for i in 0..count {
            let id = format!("alias-{i:02}");
            let concept = format!("synonym-{i}");
            e.candidates.push(candidate(&id, &concept));
            e.aliases.insert(concept, "canon".into());
            matches.push(on(&id, 0.1 + 0.04 * f64::from(i)));
        }
        e.groups.push(experience(
            "g1",
            1,
            &["e1"],
            &[[0.0, 0.6, 0.0, 0.0]],
            matches,
        ));
        if count == 1 {
            let mut sole = World::new(&f);
            sole.apply(&f, &e).unwrap();
            assert_eq!(find(&sole, "alias-00").concept.concept_id, "canon");
            assert_eq!(sole.groups["g1"].allocations[0].mu, 0.1);
            check_world(&f, &sole);
        } else {
            assert!(matches!(w.apply(&f, &e), Err(LawError::InvalidInput(_))));
            assert_eq!(w.r, before);
            assert!(w.groups.is_empty());
        }
        // The same aliases are valid once the canonical candidate is supplied.
        e.candidates.push(candidate("original", "canon"));
        if let GroupContent::Experience { matches, .. } = &mut e.groups[0].content {
            matches.push(on("original", 1.0));
        }
        let mut valid = World::new(&f);
        valid.apply(&f, &e).unwrap();
        assert_eq!(find(&valid, "original").pi, 0.4);
        check_world(&f, &valid);
    }
}
