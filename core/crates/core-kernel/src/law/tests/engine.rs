//! G end to end on random worlds (pending structures until P5-32): I25 and
//! the replay identity of ADR 0016 after every step, AC2-02, AC2-03 repeats,
//! AC2-05, AC2-21 with re-sharing, input-order independence. Hand-built
//! scenarios are in `cases.rs`.

use std::collections::BTreeMap;

use core_types::semantic;

use crate::law::engine::Output;
use crate::law::pending::{PendingGroup, PendingTendency};
use crate::law::replay::{Stored, check_pending, replay};
use crate::law::{Candidate, GroupContent, GroupItem, LawError, MatchTarget, Step, weight};

use super::fixture::{concept, text, vector};
use super::world::{Fixture, Generator, World, theta};

pub(super) fn step<'a>(
    candidates: &'a [Candidate],
    groups: &'a [GroupItem],
    aliases: &'a BTreeMap<String, String>,
) -> Step<'a> {
    Step {
        revision: "r1",
        candidates,
        groups,
        aliases,
    }
}

pub(super) fn candidate(record_id: &str, concept_id: &str) -> Candidate {
    Candidate {
        record_id: record_id.to_owned(),
        concept: concept(concept_id),
        predicate: vec![text("when this happens")],
        direction: vector([0.0, 1.0, 0.0, 0.0]),
        relation: None,
    }
}

pub(super) fn stored_of<'a>(groups: &'a [PendingGroup], id: &str) -> Vec<Stored<'a>> {
    groups
        .iter()
        .flat_map(|g| {
            g.allocations
                .iter()
                .filter(move |a| a.base.tendency_id == id)
                .map(move |a| Stored {
                    group_id: &g.base.group_id,
                    allocation: a,
                    credibility: g.base.credibility,
                    verifiability: g.base.verifiability,
                })
        })
        .collect()
}

/// Everything that must hold of R after every step.
pub(super) fn check_world(f: &Fixture, w: &World) {
    let l = &f.law;
    let groups = w.group_list();
    assert!(w.r.base.tendencies.is_empty());
    assert!(w.r.tendencies.len() <= f.capacity as usize);
    for t in &w.r.tendencies {
        semantic::check_tendency(&t.base).unwrap();
        check_pending(t, l, stored_of(&groups, t.id())).unwrap();
        // ADR 0016 §4: replaying the stored allocations reproduces the tendency bit for bit.
        let again = replay(t, stored_of(&groups, t.id()), l, f.ctx().caretaker).unwrap();
        assert_eq!(&again, t, "replay identity for {}", t.id());
    }
    for g in &groups {
        assert!(g.base.allocations.is_empty());
        for side in [1.0f64, -1.0] {
            let on: Vec<f64> = g
                .allocations
                .iter()
                .filter(|a| a.base.support != 0.0 && a.base.support.signum() == side)
                .map(|a| a.base.share)
                .collect();
            if !on.is_empty() {
                let total: f64 = on.iter().sum();
                assert!(
                    (total - 1.0).abs() <= 1e-12,
                    "I25: {} side {side}: {total}",
                    g.base.group_id
                );
            }
        }
        for a in &g.allocations {
            assert!(a.base.support != 0.0 || a.base.share == 0.0);
            let w = weight(
                a.base.intensity,
                g.base.credibility,
                g.base.verifiability,
                a.base.share,
            )
            .unwrap();
            assert!(w <= l.w_max, "AC2-21 single-group cap");
            assert!(a.mu > 0.0 && a.mu <= 1.0);
        }
    }
}

/// L: exactly the tendencies whose value changed, with their π before and after.
pub(super) fn check_changes(before: &World, after: &World, out: &Output) {
    let b: BTreeMap<&str, &PendingTendency> =
        before.r.tendencies.iter().map(|t| (t.id(), t)).collect();
    let a: BTreeMap<&str, &PendingTendency> =
        after.r.tendencies.iter().map(|t| (t.id(), t)).collect();
    let mut expected: Vec<&str> = b
        .keys()
        .chain(a.keys())
        .copied()
        .filter(|id| b.get(id) != a.get(id))
        .collect();
    expected.sort_by(|x, y| semantic::tendency_order(x, y));
    expected.dedup();
    let got: Vec<&str> = out.changes.iter().map(|c| c.tendency_id.as_str()).collect();
    assert_eq!(got, expected);
    for c in &out.changes {
        assert_eq!(
            c.pi_before,
            b.get(c.tendency_id.as_str()).map(|t| t.base.pi)
        );
        assert_eq!(c.pi_after, a.get(c.tendency_id.as_str()).map(|t| t.base.pi));
        semantic::check_tendency_change(c).unwrap();
    }
}

/// Runs a seeded world for `steps` events, checking every step.
pub(super) fn run_world(
    seed: u32,
    steps: usize,
    capacity: u32,
) -> (Fixture, World, Generator, usize) {
    let f = Fixture::new(capacity);
    let mut w = World::new(&f);
    let mut events = Generator::new(seed);
    let mut capacity_hits = 0;
    for _ in 0..steps {
        let e = events.event(&w);
        let before = w.clone();
        match w.apply(&f, &e) {
            Ok(out) => check_changes(&before, &w, &out),
            Err(LawError::CapacityExhausted { .. }) => {
                capacity_hits += 1;
                w = before;
            }
            Err(err) => panic!("seed {seed}: {err}"),
        }
        check_world(&f, &w);
    }
    (f, w, events, capacity_hits)
}

#[test]
fn ac2_02_i25_and_replay_identity_hold_after_every_step_of_random_worlds() {
    let mut groups = 0;
    let mut allocations = 0;
    for seed in 0..40 {
        let (_, w, _, _) = run_world(2_300 + seed, 120, 10);
        groups += w.groups.len();
        allocations += w
            .groups
            .values()
            .map(|g| g.allocations.len())
            .sum::<usize>();
    }
    println!(
        "G worlds: 40 x 120 events, {groups} groups, {allocations} allocations; I25 and replay identity hold after every step"
    );
    assert!(allocations > 1000, "the worlds exercise G: {allocations}");
}

#[test]
fn ac2_21_capacity_with_resharing_keeps_i25_in_small_worlds() {
    for seed in 0..40 {
        run_world(2_400 + seed, 100, 4);
    }
}

#[test]
fn ac2_03_the_same_group_arriving_again_changes_nothing() {
    let mut checked = 0;
    for seed in 0..20 {
        let (f, w, mut events, _) = run_world(2_500 + seed, 80, 10);
        for item in w.items.values() {
            // Skip groups that would legitimately gain an allocation: a target in R
            // without an allocation here, or a person whose relation tendency left R
            // and would be created again on involvement (D6).
            let g = &w.groups[&item.group_id];
            let has = |id: &str| g.allocations.iter().any(|a| a.base.tendency_id == id);
            let in_r = |id: &str| w.r.tendencies.iter().any(|t| t.id() == id);
            let regains = match &item.content {
                GroupContent::Experience { matches, .. } => {
                    matches.iter().any(|m| match &m.target {
                        MatchTarget::Tendency(t) => in_r(t) && !has(t),
                        MatchTarget::Relation { person_id, kind } => {
                            let id = crate::law::relation_tendency_id(*kind, person_id);
                            !in_r(&id) || !has(&id)
                        }
                    })
                }
                GroupContent::Teaching { tendency_id, .. } => {
                    in_r(tendency_id) && !has(tendency_id)
                }
            };
            if regains {
                continue;
            }
            let b: [f64; 4] = std::array::from_fn(|_| events.rng.uniform(-0.5, 0.5));
            let s: [f64; 4] = std::array::from_fn(|_| events.rng.uniform(0.05, 0.5));
            let out = f
                .run(
                    &w.r,
                    &w.group_list(),
                    &theta(b, s),
                    &step(&[], std::slice::from_ref(item), &BTreeMap::new()),
                )
                .unwrap();
            assert_eq!(out.r, w.r, "{}", item.group_id);
            assert!(out.changes.is_empty() && out.groups.is_empty());
            checked += 1;
        }
    }
    println!("AC2-03: {checked} groups re-delivered under other b and sigma: R unchanged");
    assert!(checked > 500);
}

#[test]
fn step_order_of_inputs_does_not_matter() {
    for seed in 0..20 {
        let (f, w, mut events, _) = run_world(2_600 + seed, 40, 10);
        for _ in 0..20 {
            let e = events.event(&w);
            let mut shuffled = e.clone();
            shuffled.groups.reverse();
            shuffled.candidates.reverse();
            let a = f.run(&w.r, &w.group_list(), &e.theta, &e.step());
            let b = f.run(&w.r, &w.group_list(), &shuffled.theta, &shuffled.step());
            assert_eq!(a, b);
        }
    }
}

#[test]
fn ac2_05_without_new_evidence_r_is_bitwise_unchanged() {
    for seed in 0..30 {
        let (f, w, mut events, _) = run_world(2_700 + seed, 60, 10);
        for _ in 0..10 {
            let b: [f64; 4] = std::array::from_fn(|_| events.rng.uniform(-1.0, 1.0));
            let s: [f64; 4] = std::array::from_fn(|_| events.rng.uniform(0.01, 1.0));
            let out = f
                .run(
                    &w.r,
                    &w.group_list(),
                    &theta(b, s),
                    &step(&[], &[], &BTreeMap::new()),
                )
                .unwrap();
            assert_eq!(out.r, w.r);
            assert!(out.groups.is_empty() && out.changes.is_empty());
        }
    }
}
