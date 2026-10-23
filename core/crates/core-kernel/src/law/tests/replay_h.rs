//! I23: persisted incremental G versus all accepted events in H order.
//! D5 additionally reconstructs each tendency using only stored allocations.

use crate::law::pending::{EvidenceGroup, Snapshot, Tendency};

use core_types::contract::Limits;

use crate::law::replay::replay;
use crate::law::{GroupContent, LawError};

use super::cases::{event, experience, on};
use super::engine::{candidate, check_world, stored_of};
use super::world::{Event, Fixture, Generator, World, theta};

const LIMITS: Limits = Limits {
    max_bytes: 16_777_216,
    max_items: 100_000,
    max_depth: 64,
};

fn bits(t: &Tendency) -> [u64; 8] {
    [
        t.evidence_a,
        t.evidence_b,
        t.evidence_v,
        t.h_hat,
        t.pi,
        t.typical_magnitude,
        t.magnitude_sum,
        t.magnitude_weight,
    ]
    .map(f64::to_bits)
}

fn reload(w: &mut World) {
    w.r = Snapshot::from_canonical(&w.r.canonical_bytes(LIMITS).unwrap(), LIMITS).unwrap();
    for t in &mut w.r.tendencies {
        *t = Tendency::from_canonical(&t.canonical_bytes(LIMITS).unwrap(), LIMITS).unwrap();
    }
    for g in w.groups.values_mut() {
        *g = EvidenceGroup::from_canonical(&g.canonical_bytes(LIMITS).unwrap(), LIMITS).unwrap();
    }
}

fn equal(a: &World, b: &World) {
    assert_eq!(
        a.r.canonical_bytes(LIMITS).unwrap(),
        b.r.canonical_bytes(LIMITS).unwrap()
    );
    for (ta, tb) in a.r.tendencies.iter().zip(&b.r.tendencies) {
        assert_eq!(bits(ta), bits(tb));
    }
    assert_eq!(a.groups.len(), b.groups.len());
    for (id, group) in &a.groups {
        assert_eq!(
            group.canonical_bytes(LIMITS).unwrap(),
            b.groups[id].canonical_bytes(LIMITS).unwrap()
        );
    }
}

fn check_prefix(f: &Fixture, incremental: &World, history: &[Event]) {
    check_world(f, incremental);
    let groups = incremental.group_list();
    for t in &incremental.r.tendencies {
        let rebuilt = replay(
            t,
            stored_of(&groups, &t.tendency_id),
            &f.law,
            f.ctx().caretaker,
        )
        .unwrap();
        assert_eq!(bits(&rebuilt), bits(t));
        assert_eq!(
            rebuilt.canonical_bytes(LIMITS).unwrap(),
            t.canonical_bytes(LIMITS).unwrap()
        );
    }
    let mut full = World::new(f);
    for e in history {
        full.apply(f, e).unwrap();
    }
    equal(incremental, &full);
}

fn scenario(f: &Fixture, events: &[Event]) {
    let mut incremental = World::new(f);
    for (i, e) in events.iter().enumerate() {
        reload(&mut incremental);
        incremental.apply(f, e).unwrap();
        check_prefix(f, &incremental, &events[..=i]);
    }
}

#[test]
fn i23_d5_back_collection_late_member_and_capacity_are_bitwise_reproducible() {
    let f = Fixture::new(10);
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
    let first = event(vec![candidate("a", "ca")], vec![g1.clone(), g2.clone()]);
    // Back-collection: old allocations keep S/intensity/p_c while new ones
    // use the new theta; the changed share is persisted and replayed.
    let mut back = g1.clone();
    if let GroupContent::Experience { matches, .. } = &mut back.content {
        matches.push(on("b", 0.7));
    }
    let mut e = event(vec![candidate("b", "cb")], vec![back]);
    e.theta = theta([-0.3, 0.1, 0.1, 0.1], [0.4, 0.15, 0.3, 0.2]);
    scenario(&f, &[first.clone(), e]);
    // New information in a late member changes the old allocation in place.
    let late = experience(
        "g1",
        1,
        &["e1", "e3"],
        &[[0.0, 0.2, 0.0, 0.0], [0.0, -1.0, 0.0, 0.0]],
        vec![on("a", 1.0)],
    );
    let mut e = event(vec![], vec![late]);
    e.theta = theta([0.3, -0.3, 0.1, 0.1], [0.4, 0.35, 0.3, 0.2]);
    scenario(&f, &[first.clone(), e]);
    // Pure repetition is now a member-list change only, including after reload.
    let repeated = experience(
        "g1",
        1,
        &["e1", "e3"],
        &[[0.0, 0.2, 0.0, 0.0]; 2],
        vec![on("a", 1.0)],
    );
    let mut e = event(vec![], vec![repeated]);
    e.theta = theta([0.1, 0.1, 0.1, 0.1], [0.4, 0.15, 0.3, 0.2]);
    scenario(&f, &[first, e]);
    // Removing b changes a's old share, not its original support or theta.
    let cap = Fixture::new(4);
    let mut shared = g1;
    if let GroupContent::Experience { matches, .. } = &mut shared.content {
        matches.push(on("b", 1.0));
    }
    let before = event(
        vec![candidate("a", "ca"), candidate("b", "cb")],
        vec![shared, g2],
    );
    let after = event(
        vec![candidate("d", "cd")],
        vec![
            experience(
                "g3",
                3,
                &["e3"],
                &[[0.0, 0.6, 0.0, 0.0]],
                vec![on("d", 1.0)],
            ),
            experience(
                "g4",
                4,
                &["e4"],
                &[[0.0, 0.6, 0.0, 0.0]],
                vec![on("d", 1.0)],
            ),
        ],
    );
    scenario(&cap, &[before, after]);
}

#[test]
fn i23_random_h_prefixes_survive_canonical_reload_and_full_recomputation() {
    let mut accepted = 0;
    let mut capacity_faults = 0;
    for seed in 0..5 {
        let f = Fixture::new(10);
        let mut incremental = World::new(&f);
        let mut generator = Generator::new(9900 + seed);
        let mut history = Vec::new();
        for _ in 0..25 {
            let e = generator.event(&incremental);
            reload(&mut incremental);
            match incremental.apply(&f, &e) {
                Ok(_) => {
                    history.push(e);
                    accepted += 1;
                    check_prefix(&f, &incremental, &history);
                }
                Err(LawError::CapacityExhausted { .. }) => capacity_faults += 1,
                Err(err) => panic!("{err}"),
            }
        }
    }
    assert_eq!((accepted, capacity_faults), (125, 0));
    println!(
        "I23: {accepted} H prefixes; persisted incremental, full H and D5 test envelope bytes (legacy fields plus raw V/q) and eight scalar bit patterns equal"
    );
}
