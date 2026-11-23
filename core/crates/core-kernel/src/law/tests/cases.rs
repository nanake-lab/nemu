//! G on hand-built evidence groups: the first group and r_k, late members,
//! EV5 back-collection, teaching. Candidates, aliases, relations, the capacity
//! fuse, input validation and I10 are in `boundaries.rs`.

use crate::law::state::Tendency;
use std::collections::BTreeMap;

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

pub(super) fn find<'a>(w: &'a World, id: &str) -> &'a Tendency {
    w.r.tendencies
        .iter()
        .find(|t| t.tendency_id.as_str() == id)
        .unwrap()
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
        (a.ordinal.0, a.mu, a.projection),
        (0, 0.5, Some(0.4 - 0.0f64.max(0.0)))
    );
    assert_eq!((t.magnitude_sum, t.magnitude_weight), (0.5 * 0.4, 0.5));
    assert_eq!(t.typical_magnitude, 0.4);
    // r̃ = max(0, r_min) = 0.05 before the group: 2·0.4/0.05 − 1 clips to 1, S = μ.
    assert_eq!(a.support, 0.5);
    assert!(t.pi > 0.0);
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
    assert_eq!(before.group_count.0, 2);
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
    assert_eq!(after.group_count.0, 2, "not a new group");
    assert_eq!(
        w.groups["g1"].allocations[0].ordinal.0, 0,
        "ordinal unchanged"
    );
    assert!(after.pi < before.pi, "the group now contradicts");
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
    assert_eq!(w.groups["g1"].allocations[0].share, 1.0);
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
    let share = |id: &str| g.allocations.iter().find(|a| a.tendency_id == id).unwrap();
    assert!(share("rec-1").share < 1.0);
    assert_eq!(share("rec-1").share + share("rec-2").share, 1.0);
    assert_eq!(
        share("rec-1").ordinal.0,
        0,
        "the old allocation keeps its ordinal"
    );
    assert_eq!(
        share("rec-2").ordinal.0,
        0,
        "first group of the new tendency"
    );
    let now = find(&w, "rec-1");
    assert!(now.pi < old.pi, "the old tendency gives up part of g1");
    assert_eq!(
        now.group_count, old.group_count,
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
    let pi = |w: &World| find(w, "rec-1").pi;
    assert!(pi(&parent) > pi(&stranger), "teaching weighs with trust");
    assert_eq!(
        parent.groups["t1"].credibility,
        (1.0 + A_ATT / (4.0 + A_ATT)) / 2.0
    );
    assert_eq!(stranger.groups["t1"].credibility, 0.5);
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
    assert_eq!(parent.groups["t1"].members, ["e5", "e9"]);
    // Teaching does not enter r_k.
    assert_eq!(find(&parent, "rec-1").magnitude_weight, 0.0);
    check_world(&f, &parent);
}

/// Review of PR #52: M and U are rounded separately. An experience group with
/// μ = 2^-1074 and p_c = 2, then a teaching group (only ρ = 1 − 75/101), leave
/// M = 2^-1074 and U = 0. The state is valid (r_k = 0), replays bit for bit, and the
/// next step, with or without evidence, proceeds.
#[test]
fn subnormal_edge_u_zero_with_m_nonzero_is_carried_and_recovers() {
    let mut f = Fixture::new(10);
    f.law = super::fixture::law(3.0, 1.0, 0.75, 100.0);
    let mut w = World::new(&f);
    let mut c = candidate("rec-1", "c-edge");
    c.direction = vector([1.0, 0.0, 0.0, 0.0]);
    let mu = f64::from_bits(1);
    let g1 = experience(
        "g1",
        1,
        &["e1"],
        &[[1.0, 0.0, 0.0, 0.0]],
        vec![on("rec-1", mu)],
    );
    // b = −1 on the connection channel: a(e) − b = 2 along d, so p_c = 2.
    let edge = super::world::theta([-1.0, 0.0, 0.0, 0.0], [0.2; 4]);
    let mut first = event(vec![c], vec![g1]);
    first.theta = edge.clone();
    w.apply(&f, &first).unwrap();
    let t = find(&w, "rec-1");
    assert_eq!(
        (t.magnitude_sum.to_bits(), t.magnitude_weight.to_bits()),
        (2, 1)
    );
    assert_eq!(w.groups["g1"].allocations[0].projection, Some(2.0));
    let teach = GroupItem {
        group_id: "t1".to_owned(),
        position: 2,
        source_key: "speaker:p9".to_owned(),
        members: vec!["e2".to_owned()],
        verifiable: true,
        content: GroupContent::Teaching {
            speaker: "p9".to_owned(),
            tendency_id: "rec-1".to_owned(),
            supports: true,
        },
    };
    w.apply(&f, &event(vec![], vec![teach])).unwrap();
    let t = find(&w, "rec-1").clone();
    assert_eq!((t.group_count.0, t.flip_count.0), (2, 0));
    assert_eq!(
        (t.magnitude_sum.to_bits(), t.magnitude_weight.to_bits()),
        (1, 0),
        "the reviewer's bit patterns"
    );
    assert_eq!(t.typical_magnitude, 0.0);
    check_world(&f, &w);
    // The next step without evidence: R unchanged.
    let before = w.r.clone();
    let out = w.apply(&f, &event(vec![], vec![])).unwrap();
    assert!(out.changes.is_empty());
    assert_eq!(w.r, before);
    // The next step with evidence: recorded as usual.
    let g3 = experience(
        "g3",
        3,
        &["e3"],
        &[[0.0, 0.6, 0.0, 0.0]],
        vec![on("rec-1", 1.0)],
    );
    w.apply(&f, &event(vec![], vec![g3])).unwrap();
    assert_eq!(find(&w, "rec-1").group_count.0, 3);
    check_world(&f, &w);
}

/// D9: when n₀ + κ is negligible next to B in binary64, recording a group could give
/// |A| = B and π = ±1, outside the open interval of v2.0 §1. G reports it at that step.
#[test]
fn precision_exhaustion_is_an_explicit_fault_at_the_step() {
    let l = super::fixture::law(1e-300, 0.0, 0.1, 10.0);
    let mut t = super::fixture::tendency(&l, "k", None);
    let before = t.clone();
    let err = crate::law::record(&mut t, &l, 1.0, 1.0).unwrap_err();
    assert!(
        matches!(err, crate::law::LawError::PrecisionExhausted(_)),
        "{err}"
    );
    assert_eq!(t, before, "nothing is stored");
    // A starting π of 1 is refused the same way (A_att overwhelms n₀ + κ).
    let huge = super::fixture::law(1e-300, 0.0, 0.1, 10.0);
    let concept = super::fixture::concept("c");
    let direction = vector([1.0, 0.0, 0.0, 0.0]);
    let predicate = [super::fixture::text("p")];
    let relation = core_types::generated::PersonRelation {
        schema_version: 1,
        person_id: "u".to_owned(),
        kind: core_types::generated::RelationKind::CLOSENESS,
    };
    let err = crate::law::initial(
        &huge,
        &crate::law::Origin {
            tendency_id: "k",
            concept: &concept,
            predicate: &predicate,
            direction: &direction,
            relation: Some(&relation),
            prior: Some(1.0),
            initial_evidence: 0.0,
        },
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::law::LawError::PrecisionExhausted(_)),
        "{err}"
    );
}
