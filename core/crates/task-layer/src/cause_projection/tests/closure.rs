//! Closure traversal and validation tests (ADR 0042 §1.3; Card §6.2).

use super::*;
use crate::cause_projection::{InvalidInput, Outcome, RelationKind, assemble};

#[test]
fn closure_body_source_cycle_terminates_once_each() {
    let records = vec![rec("r0", 10, "body0"), rec("r1", 20, "body1")];
    let relations = vec![
        rel("r0", "r1", RelationKind::BodySource),
        rel("r1", "r0", RelationKind::BodySource),
    ];
    let slots = vec![slot("s1", vec![0.8], 30, &["r1"])];
    let input = input_with_unrelated_base(records, relations, slots);
    let Outcome::Sent { plan, binding, .. } = assemble(&input, &TestCodec).expect("valid") else {
        panic!("expected Sent");
    };
    assert_eq!(binding.packs[0].records, vec![rid("r0"), rid("r1")]);
    let exp_ids: Vec<_> = plan.experience.into_iter().map(|e| e.id).collect();
    assert_eq!(exp_ids, vec![rid("r0"), rid("r1")]);
}

#[test]
fn closure_chained_correction_of_correction() {
    let records = vec![
        rec("s", 10, "orig"),
        rec("c1", 20, "corr1"),
        rec("c2", 30, "corr2"),
    ];
    let relations = vec![
        rel("c1", "s", RelationKind::Correction),
        rel("c2", "c1", RelationKind::Correction),
    ];
    let slots = vec![slot("s1", vec![0.8], 40, &["s"])];
    let input = input_with_unrelated_base(records, relations, slots);
    let Outcome::Sent { plan, binding, .. } = assemble(&input, &TestCodec).expect("valid") else {
        panic!("expected Sent");
    };
    assert_eq!(
        binding.packs[0].records,
        vec![rid("s"), rid("c1"), rid("c2")]
    );
    let exp_ids: Vec<_> = plan.experience.into_iter().map(|e| e.id).collect();
    assert_eq!(exp_ids, vec![rid("s"), rid("c1"), rid("c2")]);
}

#[test]
fn closure_outgoing_correction_does_not_include_target() {
    let records = vec![rec("s", 10, "source"), rec("y", 20, "other_corrected")];
    let relations = vec![rel("s", "y", RelationKind::Correction)];
    let slots = vec![slot("s1", vec![0.8], 30, &["s"])];
    let input = input_with_unrelated_base(records, relations, slots);
    let Outcome::Sent { plan, binding, .. } = assemble(&input, &TestCodec).expect("valid") else {
        panic!("expected Sent");
    };
    assert_eq!(binding.packs[0].records, vec![rid("s")]);
    let exp_ids: Vec<_> = plan.experience.into_iter().map(|e| e.id).collect();
    assert_eq!(exp_ids, vec![rid("s")]);
}

#[test]
fn closure_local_only_not_expanded_either_direction() {
    let records = vec![
        rec("s", 10, "source"),
        rec("in_local", 20, "local_in"),
        rec("out_local", 30, "local_out"),
    ];
    let relations = vec![
        rel("in_local", "s", RelationKind::LocalOnly),
        rel("s", "out_local", RelationKind::LocalOnly),
    ];
    let slots = vec![slot("s1", vec![0.8], 40, &["s"])];
    let input = input_with_unrelated_base(records, relations, slots);
    let Outcome::Sent { plan, binding, .. } = assemble(&input, &TestCodec).expect("valid") else {
        panic!("expected Sent");
    };
    assert_eq!(binding.packs[0].records, vec![rid("s")]);
    let exp_ids: Vec<_> = plan.experience.into_iter().map(|e| e.id).collect();
    assert_eq!(exp_ids, vec![rid("s")]);
}

#[test]
fn closure_distinct_ids_same_body_both_retained() {
    let records = vec![
        rec("alice_said", 10, "same exact words"),
        rec("bob_said", 20, "same exact words"),
    ];
    let slots = vec![slot("s1", vec![0.8], 30, &["alice_said", "bob_said"])];
    let input = input_with_unrelated_base(records, Vec::new(), slots);
    let Outcome::Sent { plan, binding, .. } = assemble(&input, &TestCodec).expect("valid") else {
        panic!("expected Sent");
    };
    assert_eq!(
        binding.packs[0].records,
        vec![rid("alice_said"), rid("bob_said")]
    );
    let exp_ids: Vec<_> = plan.experience.into_iter().map(|e| e.id).collect();
    assert_eq!(exp_ids, vec![rid("alice_said"), rid("bob_said")]);
}

#[test]
fn closure_duplicate_id_conflicting_body_rejected_and_identical_deduped() {
    let dup_ok = vec![rec("r1", 10, "same"), rec("r1", 10, "same")];
    let ok_input =
        input_with_unrelated_base(dup_ok, Vec::new(), vec![slot("s1", vec![0.5], 20, &["r1"])]);
    let Outcome::Sent { binding, .. } = assemble(&ok_input, &TestCodec).expect("identical dedup")
    else {
        panic!("expected Sent");
    };
    assert_eq!(binding.packs[0].records, vec![rid("r1")]);

    let dup_bad = vec![rec("r1", 10, "body_a"), rec("r1", 10, "body_b")];
    let bad_input = input_with_unrelated_base(
        dup_bad,
        Vec::new(),
        vec![slot("s1", vec![0.5], 20, &["r1"])],
    );
    let err = assemble(&bad_input, &TestCodec).expect_err("conflicting duplicate ID must fail");
    assert_eq!(err, InvalidInput::ConflictingDuplicateRecord(rid("r1")));
}
