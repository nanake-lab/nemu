//! Determinism, permutation invariance, and input immutability tests (Card §6.5; ADR 0042 §3.2).

use super::*;
use crate::cause_projection::{AssemblyInput, Outcome, RelationKind, assemble};

fn shared_fixture() -> AssemblyInput {
    let records = vec![
        rec("s", 10, "shared_src"),
        rec("a", 20, "pack1_extra"),
        rec("b", 30, "pack2_extra"),
        rec("c", 40, "corr_of_s"),
    ];
    let relations = vec![rel("c", "s", RelationKind::Correction)];
    let slots = vec![
        slot("s1", vec![0.9], 10, &["s", "a"]),
        slot("s2", vec![0.5], 20, &["s", "b"]),
    ];
    input_with_unrelated_base(records, relations, slots)
}

#[test]
fn assemble_twice_yields_identical_outcome_and_bytes() {
    let input = shared_fixture();
    let out1 = assemble(&input, &TestCodec).expect("first call");
    let out2 = assemble(&input, &TestCodec).expect("second call");
    assert_eq!(out1, out2);

    let (Outcome::Sent { bytes: b1, .. }, Outcome::Sent { bytes: b2, .. }) = (out1, out2) else {
        panic!("expected Sent");
    };
    assert_eq!(b1, b2);
}

#[test]
fn reversing_all_input_collections_preserves_bytes_and_binding() {
    let orig = shared_fixture();
    let mut rev = orig.clone();
    rev.records.reverse();
    rev.relations.reverse();
    rev.slots.reverse();
    for s in &mut rev.slots {
        s.members.reverse();
        for m in &mut s.members {
            m.sources.reverse();
        }
    }

    let Outcome::Sent {
        bytes: b_orig,
        binding: bind_orig,
        ..
    } = assemble(&orig, &TestCodec).expect("orig")
    else {
        panic!("expected Sent");
    };
    let Outcome::Sent {
        bytes: b_rev,
        binding: bind_rev,
        ..
    } = assemble(&rev, &TestCodec).expect("rev")
    else {
        panic!("expected Sent");
    };
    assert_eq!(b_orig, b_rev);
    assert_eq!(bind_orig, bind_rev);
}

#[test]
fn assemble_does_not_mutate_input() {
    let input = shared_fixture();
    let snapshot = input.clone();
    let _ = assemble(&input, &TestCodec).expect("valid");
    assert_eq!(input, snapshot);
}
