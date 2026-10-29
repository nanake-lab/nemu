//! I23 with explicit synthetic H boundary events and candidate proposals.
use super::capacity_v3::{fixture, roundtrip};
use super::cases::{event, experience, on};
use super::engine::{candidate, check_changes, check_world};
use super::pyrandom::PyRandom;
use super::state_bits::{group_bits_eq, snapshot_bits_eq};
use super::world::{Event, World};
use crate::law::{Output, protected_candidates};

fn advance(w: &mut World, cap: u32, k: u32, e: &Event, order: &mut Vec<String>) -> Output {
    // This fixture has canonical, non-relation candidates only. Rebuild its
    // proposal chronology from the input H, never from a serialized sidecar.
    for c in &e.candidates {
        order.retain(|id| id != &c.record_id);
        order.push(c.record_id.clone());
    }
    let f = fixture(cap, k);
    let before = w.clone();
    let mut step = e.step();
    step.proposal_order = order;
    let out = f.run(&w.r, &w.group_list(), &e.theta, &step).unwrap();
    w.r = out.r.clone();
    for g in &out.groups {
        w.groups.insert(g.group_id.clone(), g.clone());
    }
    check_world(&f, w);
    check_changes(&before, w, &out);
    let protected = protected_candidates(&w.r.tendencies, order, k).unwrap();
    if w.r.tendencies.len() > cap as usize {
        assert!(out.capacity_notice.is_some());
        assert!(
            w.r.tendencies
                .iter()
                .all(|t| crate::law::is_personality(t.pi) || protected.contains(&t.tendency_id))
        );
    }
    out
}

#[test]
fn i23_k1_k2_boundary_history_and_reproposal_survive_both_v3_codecs() {
    let mut checked = 0;
    let mut notices = 0;
    let mut evictions = 0;
    for k in [1, 2] {
        for seed in [5201, 5202, 5203] {
            let mut rng = PyRandom::new(seed);
            let mut live = World::new(&fixture(8, k));
            let mut order = Vec::new();
            let mut history = Vec::new();
            let mut all_evidence = Vec::new();
            for i in 0..25 {
                let cap = [8, 4, 2, 6, 3][i / 5];
                let id = format!("k{}", i % 7);
                let other = format!("k{}", (i + 3) % 7);
                let item = experience(
                    &format!("g{i}"),
                    i as u64 + 1,
                    &[&format!("e{i}")],
                    &[[0., rng.uniform(-0.7, 0.7), 0., 0.]],
                    vec![on(&id, 1.), on(&other, 0.5)],
                );
                all_evidence.push(item);
                // EV5 supplies all earlier matching groups on re-proposal.
                // Giving every group here is a conservative complete H view.
                let mut e = event(
                    vec![candidate(&id, &format!("c{id}"))],
                    all_evidence.clone(),
                );
                e.revision = format!("r{i}");
                let out = advance(&mut live, cap, k, &e, &mut order);
                notices += usize::from(out.capacity_notice.is_some());
                evictions += out.capacity_audits.len();
                history.push((cap, e));
                for protobuf in [false, true] {
                    let mut persisted = World::new(&fixture(8, k));
                    let mut reconstructed_order = Vec::new();
                    for (cap, e) in &history {
                        persisted.r = roundtrip(&persisted.r, protobuf);
                        for g in persisted.groups.values_mut() {
                            *g = roundtrip(g, protobuf);
                        }
                        advance(&mut persisted, *cap, k, e, &mut reconstructed_order);
                    }
                    assert!(snapshot_bits_eq(&live.r, &persisted.r));
                    assert_eq!(live.groups.len(), persisted.groups.len());
                    for (id, g) in &live.groups {
                        assert!(group_bits_eq(g, &persisted.groups[id]));
                    }
                    checked += 1;
                }
            }
        }
    }
    assert_eq!(checked, 300);
    assert!(notices > 0 && evictions > 0);
    println!(
        "I23 V3 capacity: {checked} codec/prefix comparisons, {notices} notices, {evictions} evictions; K=1/2, three seeds, no skipped events"
    );
}
