//! Native V3 q/V round trips followed by production G, with no V2 projection.
use super::capacity_v3::roundtrip;
use super::cases::{event, experience, find, on};
use super::engine::candidate;
use super::state_bits::{group_bits_eq, snapshot_bits_eq, tendency_bits_eq};
use super::world::{Fixture, World};

#[test]
fn v3_preserves_v_that_cannot_be_recovered_from_b_and_continues_g_bitwise() {
    let f = Fixture::new(8);
    let mut live = World::new(&f);
    let tiny = experience(
        "tiny",
        1,
        &["e1"],
        &[[0., 0.2, 0., 0.]],
        vec![on("k", 1e-200)],
    );
    live.apply(&f, &event(vec![candidate("k", "ck")], vec![tiny]))
        .unwrap();
    let t = find(&live, "k");
    assert!(t.evidence_absolute > 0.);
    assert_eq!(
        t.evidence_b - (f.law.n0 + f.law.kappa1),
        0.,
        "B-c loses the stored V"
    );
    let q = live.groups["tiny"].allocations[0].actual_retention;
    assert!(q < 1. && q > 0.);
    let next = event(
        vec![],
        vec![experience(
            "next",
            2,
            &["e2"],
            &[[0., 0.6, 0., 0.]],
            vec![on("k", 1.)],
        )],
    );
    let before = live.clone();
    live.apply(&f, &next).unwrap();
    for protobuf in [false, true] {
        let mut restored = before.clone();
        restored.r = roundtrip(&restored.r, protobuf);
        for t in &mut restored.r.tendencies {
            let decoded = roundtrip(t, protobuf);
            assert!(tendency_bits_eq(t, &decoded));
            *t = decoded;
        }
        for g in restored.groups.values_mut() {
            let decoded = roundtrip(g, protobuf);
            assert!(group_bits_eq(g, &decoded));
            *g = decoded;
            for a in &mut g.allocations {
                *a = roundtrip(a, protobuf);
            }
        }
        assert_eq!(
            restored.groups["tiny"].allocations[0]
                .actual_retention
                .to_bits(),
            q.to_bits()
        );
        restored.apply(&f, &next).unwrap();
        assert!(snapshot_bits_eq(&live.r, &restored.r));
        for (id, g) in &live.groups {
            assert!(group_bits_eq(g, &restored.groups[id]));
        }
    }
}

#[test]
fn v3_corrupt_q_or_v_is_rejected_before_g_changes_anything() {
    let f = Fixture::new(8);
    let mut w = World::new(&f);
    w.apply(
        &f,
        &event(
            vec![candidate("k", "ck")],
            vec![experience(
                "g",
                1,
                &["e"],
                &[[0., 0.2, 0., 0.]],
                vec![on("k", 1.)],
            )],
        ),
    )
    .unwrap();
    let empty = event(vec![], vec![]);
    for value in [f64::NAN, f64::INFINITY, -1.] {
        let mut bad = w.clone();
        bad.r
            .tendencies
            .iter_mut()
            .find(|t| t.tendency_id == "k")
            .unwrap()
            .evidence_absolute = value;
        assert!(
            f.run(&bad.r, &bad.group_list(), &empty.theta, &empty.step())
                .is_err()
        );
    }
    for value in [f64::NAN, f64::INFINITY, -1., 0., 1.1] {
        let mut bad = w.clone();
        bad.groups.get_mut("g").unwrap().allocations[0].actual_retention = value;
        assert!(
            f.run(&bad.r, &bad.group_list(), &empty.theta, &empty.step())
                .is_err()
        );
    }
}
