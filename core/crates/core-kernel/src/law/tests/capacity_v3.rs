//! V2-80/105 at G's transaction boundary. Orders are explicit synthetic H views.
use super::cases::{event, experience, find, on};
use super::engine::candidate;
use super::world::{Event, Fixture, World};
use crate::law::{LawError, Output, protected_candidates};
use core_types::{WireMessage, contract::Limits};

pub(super) fn fixture(capacity: u32, k: u32) -> Fixture {
    let mut f = Fixture::new(capacity);
    f.protected_candidates = k;
    f
}

fn apply(f: &Fixture, w: &mut World, e: &Event, order: &[&str]) -> Output {
    let ids: Vec<String> = order.iter().map(|id| (*id).to_owned()).collect();
    let mut step = e.step();
    step.proposal_order = &ids;
    let out = f.run(&w.r, &w.group_list(), &e.theta, &step).unwrap();
    w.r = out.r.clone();
    for g in &out.groups {
        w.groups.insert(g.group_id.clone(), g.clone());
    }
    out
}

#[test]
fn ac2_21_newest_weak_candidate_is_protected_old_evidence_is_reshared_with_same_q() {
    let f = fixture(4, 1);
    let mut w = World::new(&f);
    let old = experience(
        "old",
        1,
        &["e1"],
        &[[0., -0.8, 0., 0.]],
        vec![on("a", 1.), on("b", 0.5)],
    );
    apply(
        &f,
        &mut w,
        &event(vec![candidate("a", "ca"), candidate("b", "cb")], vec![old]),
        &["a", "b"],
    );
    let q = w.groups["old"].allocations[0].actual_retention;
    let before = find(&w, "a").pi;
    let out = apply(
        &f,
        &mut w,
        &event(vec![candidate("c", "cc")], vec![]),
        &["a", "b", "c"],
    );
    assert!(w.ids().contains(&"c".to_owned()));
    assert!(
        !w.ids().contains(&"b".to_owned()),
        "weaker old candidate leaves, newest c stays"
    );
    assert_eq!(find(&w, "c").evidence_absolute, 0.);
    assert_eq!(out.capacity_audits[0].removed, "b");
    assert!(out.capacity_notice.is_none());
    let a = &w.groups["old"].allocations[0];
    assert_eq!(a.tendency_id, "a");
    assert_eq!(a.share, 1.);
    assert_eq!(a.actual_retention.to_bits(), q.to_bits());
    assert!(find(&w, "a").pi <= before);
    assert_eq!(
        out.changes
            .iter()
            .find(|c| c.tendency_id == "b")
            .unwrap()
            .evidence_groups,
        ["old"]
    );
}

#[test]
fn ac2_21_protection_survives_events_without_new_candidates_and_uses_h_order() {
    for k in [1, 2] {
        let wide = fixture(10, k);
        let mut w = World::new(&wide);
        apply(
            &wide,
            &mut w,
            &event(
                vec![
                    candidate("z", "cz"),
                    candidate("a", "ca"),
                    candidate("m", "cm"),
                ],
                vec![],
            ),
            &["z", "a", "m"],
        );
        // Same evidence mass: unprotected ties still use canonical ID, while
        // recency follows H rather than alphabetic/byte ordering.
        let small = fixture(2 + k, k);
        let out = apply(&small, &mut w, &event(vec![], vec![]), &["z", "a", "m"]);
        assert_eq!(w.r.tendencies.len(), (2 + k) as usize);
        assert!(w.ids().contains(&"m".to_owned()));
        assert_eq!(w.ids().contains(&"a".to_owned()), k == 2);
        assert!(!w.ids().contains(&"z".to_owned()));
        assert!(out.capacity_notice.is_none());
    }
}

#[test]
fn ac2_21_exhaustion_defers_candidates_but_existing_tendencies_keep_growing() {
    let f = fixture(1, 1); // recorded boundary lowered below the two genesis personalities
    let mut w = World::new(&f);
    let before = w.r.clone();
    let old_id = "rel:trust:u-star";
    let e = event(
        vec![candidate("later", "clater")],
        vec![experience(
            "g1",
            1,
            &["e1"],
            &[[0., 0., -0.4, -0.4]],
            vec![on(old_id, 1.), on("later", 1.)],
        )],
    );
    let out = apply(&f, &mut w, &e, &["later"]);
    let notice = out.capacity_notice.unwrap();
    assert_eq!(notice.deferred_candidates, ["later"]);
    assert_eq!(notice.retained, 2);
    assert!(!w.ids().contains(&"later".to_owned()));
    assert_eq!(find(&w, old_id).group_count.0, 1);
    assert_ne!(
        find(&w, old_id).pi,
        before
            .tendencies
            .iter()
            .find(|t| t.tendency_id == old_id)
            .unwrap()
            .pi
    );
    assert_eq!(w.groups["g1"].allocations.len(), 1);
    assert!(!out.changes.is_empty());
    assert!(
        out.capacity_audits.is_empty(),
        "personality is never removed"
    );
    // Numeric errors remain atomic even while admission is stopped.
    let frozen = w.r.clone();
    let bad = event(
        vec![],
        vec![experience(
            "g2",
            2,
            &["e2"],
            &[[f64::INFINITY, 0., 0., 0.]],
            vec![on(old_id, 1.)],
        )],
    );
    assert!(
        f.run(&w.r, &w.group_list(), &bad.theta, &bad.step())
            .is_err()
    );
    assert_eq!(w.r, frozen);
}

#[test]
fn ac2_21_new_personality_does_not_let_rejected_candidate_influence_old_evidence() {
    let f = fixture(3, 1);
    let mut w = World::new(&f);
    apply(
        &f,
        &mut w,
        &event(vec![candidate("old", "cold")], vec![]),
        &["old"],
    );
    let prior = w.clone();
    let g = experience("g1", 1, &["e1"], &[[0., 1., 0., 0.]], vec![on("old", 1.)]);
    let mut incoming = event(vec![candidate("new", "cnew")], vec![g.clone()]);
    incoming.theta = super::world::theta([0.1, -0.2, 0., 0.3], [0.2; 4]);
    let out = apply(&f, &mut w, &incoming, &["old", "new"]);
    assert!(find(&w, "old").pi > 0.5);
    assert!(!w.ids().contains(&"new".to_owned()));
    assert_eq!(out.capacity_notice.unwrap().deferred_candidates, ["new"]);
    let mut control = prior;
    incoming.candidates.clear();
    apply(&f, &mut control, &incoming, &["old"]);
    assert_eq!(w.r, control.r);
    assert_eq!(
        w.groups, control.groups,
        "no tentative shares/q escape the retry"
    );
}

#[test]
fn ac2_21_raise_capacity_then_reproposal_collects_old_evidence_after_v3_reload() {
    let low = fixture(2, 1);
    let high = fixture(3, 1);
    let mut w = World::new(&low);
    let old = experience("old", 1, &["e1"], &[[0., 0.2, 0., 0.]], vec![on("k", 1.)]);
    let deferred = event(vec![candidate("k", "ck")], vec![old.clone()]);
    apply(&low, &mut w, &deferred, &["k"]);
    assert!(w.groups["old"].allocations.is_empty());
    let before = w.clone();
    let raised = apply(&high, &mut w, &event(vec![], vec![]), &["k"]);
    assert_eq!(
        w.r, before.r,
        "raising capacity does not invent a re-proposal"
    );
    assert!(raised.capacity_notice.is_none());
    let mut controls = Vec::new();
    for protobuf in [false, true] {
        let mut restored = before.clone();
        restored.r = roundtrip(&restored.r, protobuf);
        for g in restored.groups.values_mut() {
            *g = roundtrip(g, protobuf);
        }
        let accepted = apply(
            &high,
            &mut restored,
            &event(vec![candidate("k", "ck")], vec![old.clone()]),
            &["k"],
        );
        assert!(accepted.capacity_notice.is_none());
        assert_eq!(find(&restored, "k").group_count.0, 1);
        assert!(find(&restored, "k").evidence_absolute > 0.);
        assert_eq!(restored.groups["old"].allocations[0].ordinal.0, 0);
        controls.push(restored);
    }
    apply(
        &high,
        &mut w,
        &event(vec![candidate("k", "ck")], vec![old]),
        &["k"],
    );
    for restored in controls {
        assert_eq!(restored.r, w.r);
        assert_eq!(restored.groups, w.groups);
    }
}

pub(super) fn roundtrip<T: WireMessage>(value: &T, protobuf: bool) -> T {
    let limits = Limits {
        max_bytes: 16_777_216,
        max_items: 100_000,
        max_depth: 64,
    };
    if protobuf {
        T::from_protobuf(&value.protobuf_bytes(limits).unwrap(), limits).unwrap()
    } else {
        T::from_canonical(&value.canonical_bytes(limits).unwrap(), limits).unwrap()
    }
}

#[test]
fn ac2_21_protection_rejects_incomplete_or_duplicate_history_no_id_guessing() {
    let f = fixture(10, 1);
    let t = super::fixture::tendency(&f.law, "k", None);
    assert!(matches!(
        protected_candidates(std::slice::from_ref(&t), &[], 1),
        Err(LawError::InvalidInput(_))
    ));
    assert!(protected_candidates(std::slice::from_ref(&t), &["k".into(), "k".into()], 1).is_err());
    assert_eq!(
        protected_candidates(&[t], &["k".into()], 1).unwrap().len(),
        1
    );
}

#[test]
fn ac2_21_deferred_synonyms_are_one_hypothesis_and_existing_aliases_still_learn() {
    let f = fixture(2, 1);
    let mut w = World::new(&f);
    let mut e = event(
        vec![candidate("canon", "cc"), candidate("alias", "syn")],
        vec![],
    );
    e.aliases.insert("syn".into(), "cc".into());
    let out = apply(&f, &mut w, &e, &["canon"]);
    assert_eq!(out.capacity_notice.unwrap().deferred_candidates, ["canon"]);
    assert_eq!(w.r, f.r0());

    let mut wide = fixture(3, 1);
    let mut w = World::new(&wide);
    apply(
        &wide,
        &mut w,
        &event(vec![candidate("k", "ck")], vec![]),
        &["k"],
    );
    // Lowering the boundary cannot let an alias bypass a stopped admission.
    wide.capacity = 2;
    let mut e = event(
        vec![candidate("alias", "syn")],
        vec![experience(
            "g",
            1,
            &["e"],
            &[[0., 0.2, 0., 0.]],
            vec![on("alias", 1.)],
        )],
    );
    e.aliases.insert("syn".into(), "ck".into());
    let out = apply(&wide, &mut w, &e, &["k"]);
    assert!(out.capacity_notice.unwrap().deferred_candidates.is_empty());
    assert_eq!(find(&w, "k").group_count.0, 1);
    assert!(!w.ids().contains(&"alias".into()));
}
