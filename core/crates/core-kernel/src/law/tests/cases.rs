//! G on hand-built evidence groups: the first group and r_k, late members,
//! EV5 back-collection, teaching. Candidates, aliases, relations, the capacity
//! fuse, input validation and I10 are in `boundaries.rs`.

use std::collections::BTreeMap;

use crate::law::pending::PendingTendency;
use crate::law::{Candidate, GroupContent, GroupItem, Match, MatchTarget};

use super::engine::{candidate, check_world};
use super::fixture::vector;
use super::world::{A_ATT, CARETAKER, Event, Fixture, World, default_theta};

pub(super) fn experience(
    id: &str,
    position: u64,
    members: &[&str],
    appraisals: &[[f64; 4]],
    matches: Vec<Match>,
) -> GroupItem {
    GroupItem {
        group_id: id.to_owned(),
        position,
        source_key: format!("src:{id}"),
        members: members.iter().map(|m| (*m).to_owned()).collect(),
        verifiable: true,
        content: GroupContent::Experience {
            appraisals: appraisals.iter().map(|a| vector(*a)).collect(),
            matches,
        },
    }
}

pub(super) fn on(id: &str, mu: f64) -> Match {
    Match {
        target: MatchTarget::Tendency(id.to_owned()),
        mu,
    }
}

pub(super) fn event(candidates: Vec<Candidate>, groups: Vec<GroupItem>) -> Event {
    Event {
        revision: "r".to_owned(),
        candidates,
        groups,
        aliases: BTreeMap::new(),
        theta: default_theta(),
    }
}

pub(super) fn find<'a>(w: &'a World, id: &str) -> &'a PendingTendency {
    w.r.tendencies.iter().find(|t| t.id() == id).unwrap()
}

#[test]
fn first_group_records_ordinal_zero_and_the_typical_magnitude() {
    let f = Fixture::new(10);
    let mut w = World::new(&f);
    // Empathy channel: b = −0.2, so a(e) = 0.2 deviates by 0.4 along d = e₂.
    let g1 = experience(
        "g1",
        1,
        &["e1"],
        &[[0.0, 0.2, 0.0, 0.0]],
        vec![on("rec-1", 0.5)],
    );
    w.apply(&f, &event(vec![candidate("rec-1", "c-kind")], vec![g1]))
        .unwrap();
    let t = find(&w, "rec-1");
    let a = &w.groups["g1"].allocations[0];
    assert_eq!(
        (a.ordinal, a.mu, a.projection),
        (0, 0.5, Some(0.4 - 0.0f64.max(0.0)))
    );
    assert_eq!((t.magnitude_sum, t.magnitude_weight), (0.5 * 0.4, 0.5));
    assert_eq!(t.base.typical_magnitude, 0.4);
    // r̃ = max(0, r_min) = 0.05 before the group: 2·0.4/0.05 − 1 clips to 1, S = μ.
    assert_eq!(a.base.support, 0.5);
    assert!(t.base.pi > 0.0);
    check_world(&f, &w);
}

#[test]
fn late_member_updates_the_group_without_a_second_count() {
    let f = Fixture::new(10);
    let mut w = World::new(&f);
    let g1 = experience(
        "g1",
        1,
        &["e1"],
        &[[0.0, 0.2, 0.0, 0.0]],
        vec![on("rec-1", 1.0)],
    );
    let g2 = experience(
        "g2",
        2,
        &["e2"],
        &[[0.0, 0.6, 0.0, 0.0]],
        vec![on("rec-1", 1.0)],
    );
    w.apply(
        &f,
        &event(vec![candidate("rec-1", "c-kind")], vec![g1.clone(), g2]),
    )
    .unwrap();
    let before = find(&w, "rec-1").clone();
    assert_eq!(before.base.group_count.0, 2);
    // A later member joins g1: its reaction is the opposite of the prediction.
    let late = experience(
        "g1",
        1,
        &["e1", "e3"],
        &[[0.0, 0.2, 0.0, 0.0], [0.0, -1.0, 0.0, 0.0]],
        vec![on("rec-1", 1.0)],
    );
    let out = w.apply(&f, &event(vec![], vec![late])).unwrap();
    let after = find(&w, "rec-1");
    assert_eq!(after.base.group_count.0, 2, "not a new group");
    assert_eq!(
        w.groups["g1"].allocations[0].ordinal, 0,
        "ordinal unchanged"
    );
    assert!(after.base.pi < before.base.pi, "the group now contradicts");
    assert_eq!(out.changes[0].evidence_groups, ["g1"]);
    check_world(&f, &w);
}

#[test]
fn ev5_back_collection_reshares_old_groups_and_keeps_i25() {
    let f = Fixture::new(10);
    let mut w = World::new(&f);
    let g1 = experience(
        "g1",
        1,
        &["e1"],
        &[[0.0, 0.6, 0.0, 0.0]],
        vec![on("rec-1", 1.0)],
    );
    let g2 = experience(
        "g2",
        2,
        &["e2"],
        &[[0.0, 0.6, 0.0, 0.0]],
        vec![on("rec-1", 1.0)],
    );
    w.apply(
        &f,
        &event(vec![candidate("rec-1", "c-kind")], vec![g1.clone(), g2]),
    )
    .unwrap();
    let old = find(&w, "rec-1").clone();
    assert_eq!(w.groups["g1"].allocations[0].base.share, 1.0);
    // A new candidate along the same direction explains g1 too (EV5): it shares g1.
    let mut back = g1;
    if let GroupContent::Experience { matches, .. } = &mut back.content {
        matches.push(on("rec-2", 1.0));
    }
    let out = w
        .apply(&f, &event(vec![candidate("rec-2", "c-gentle")], vec![back]))
        .unwrap();
    let g = &w.groups["g1"];
    assert_eq!(g.allocations.len(), 2);
    let share = |id: &str| {
        g.allocations
            .iter()
            .find(|a| a.base.tendency_id == id)
            .unwrap()
    };
    assert!(share("rec-1").base.share < 1.0);
    assert_eq!(share("rec-1").base.share + share("rec-2").base.share, 1.0);
    assert_eq!(
        share("rec-1").ordinal,
        0,
        "the old allocation keeps its ordinal"
    );
    assert_eq!(share("rec-2").ordinal, 0, "first group of the new tendency");
    let now = find(&w, "rec-1");
    assert!(
        now.base.pi < old.base.pi,
        "the old tendency gives up part of g1"
    );
    assert_eq!(
        now.base.group_count, old.base.group_count,
        "re-sharing is not a new group"
    );
    let ids: Vec<&str> = out.changes.iter().map(|c| c.tendency_id.as_str()).collect();
    assert_eq!(ids, ["rec-1", "rec-2"]);
    check_world(&f, &w);
}

#[test]
fn teaching_counts_by_trust_and_repeats_do_not_refresh() {
    let f = Fixture::new(10);
    let mut w = World::new(&f);
    w.apply(&f, &event(vec![candidate("rec-1", "c-direct")], vec![]))
        .unwrap();
    let teach = |id: &str, speaker: &str, members: &[&str]| GroupItem {
        group_id: id.to_owned(),
        position: 5,
        source_key: format!("speaker:{speaker}"),
        members: members.iter().map(|m| (*m).to_owned()).collect(),
        verifiable: true,
        content: GroupContent::Teaching {
            speaker: speaker.to_owned(),
            tendency_id: "rec-1".to_owned(),
            supports: true,
        },
    };
    let mut parent = w.clone();
    parent
        .apply(&f, &event(vec![], vec![teach("t1", CARETAKER, &["e5"])]))
        .unwrap();
    let mut stranger = w.clone();
    stranger
        .apply(&f, &event(vec![], vec![teach("t1", "p9", &["e5"])]))
        .unwrap();
    let pi = |w: &World| find(w, "rec-1").base.pi;
    assert!(pi(&parent) > pi(&stranger), "teaching weighs with trust");
    assert_eq!(
        parent.groups["t1"].base.credibility,
        (1.0 + A_ATT / (4.0 + A_ATT)) / 2.0
    );
    assert_eq!(stranger.groups["t1"].base.credibility, 0.5);
    // Saying the same thing again joins the group and changes nothing (EV7).
    let before = parent.r.clone();
    let out = parent
        .apply(
            &f,
            &event(vec![], vec![teach("t1", CARETAKER, &["e5", "e9"])]),
        )
        .unwrap();
    assert_eq!(parent.r.tendencies, before.tendencies);
    assert!(out.changes.is_empty());
    assert_eq!(parent.groups["t1"].base.members, ["e5", "e9"]);
    // Teaching does not enter r_k.
    assert_eq!(find(&parent, "rec-1").magnitude_weight, 0.0);
    check_world(&f, &parent);
}
