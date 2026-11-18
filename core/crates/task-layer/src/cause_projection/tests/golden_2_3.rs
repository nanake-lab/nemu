//! Golden tests for the five ADR 0042 §2.3 examples and their variants (Card §6.3).

use super::*;
use crate::cause_projection::{
    BaseFailure, Budget, DItem, Delivered, Outcome, PackFate, RelationKind, SentNote, Slot,
    assemble,
};

#[test]
fn ex1_body_source_and_continuation_separated() {
    let records = vec![
        rec("r0", 10, "origin"),
        rec("r1", 20, "quoted"),
        rec("r9", 25, "reply_target"),
        rec("r2", 30, "cites_r1_has_url_replies_r9"),
    ];
    let relations = vec![
        rel("r2", "r1", RelationKind::BodySource),
        rel("r1", "r0", RelationKind::BodySource),
        rel("r2", "r9", RelationKind::Continuation),
    ];
    let input =
        input_with_unrelated_base(records, relations, vec![slot("s1", vec![0.8], 35, &["r2"])]);
    let (plan, _, binding, _) = expect_sent(&input);
    assert_eq!(
        binding.packs[0].records,
        vec![rid("r0"), rid("r1"), rid("r2")]
    );
    assert_eq!(exp_ids(&plan), vec![rid("r0"), rid("r1"), rid("r2")]);
    assert_eq!(plan.experience[2].continuation_refs, vec![rid("r9")]);
}

#[test]
fn ex1_variant_a_body_source_cycle() {
    let records = vec![
        rec("r0", 10, "origin"),
        rec("r1", 20, "quoted"),
        rec("r9", 25, "reply_target"),
        rec("r2", 30, "cites_r1"),
    ];
    let relations = vec![
        rel("r2", "r1", RelationKind::BodySource),
        rel("r1", "r0", RelationKind::BodySource),
        rel("r0", "r1", RelationKind::BodySource),
        rel("r2", "r9", RelationKind::Continuation),
    ];
    let input =
        input_with_unrelated_base(records, relations, vec![slot("s1", vec![0.8], 35, &["r2"])]);
    let (plan, _, binding, _) = expect_sent(&input);
    assert_eq!(
        binding.packs[0].records,
        vec![rid("r0"), rid("r1"), rid("r2")]
    );
    assert_eq!(exp_ids(&plan), vec![rid("r0"), rid("r1"), rid("r2")]);
    assert_eq!(plan.experience[2].continuation_refs, vec![rid("r9")]);
}

#[test]
fn ex1_variant_b_continuation_target_absent_from_records() {
    let records = vec![
        rec("r0", 10, "origin"),
        rec("r1", 20, "quoted"),
        rec("r2", 30, "cites_r1"),
    ];
    let relations = vec![
        rel("r2", "r1", RelationKind::BodySource),
        rel("r1", "r0", RelationKind::BodySource),
        rel("r2", "r9", RelationKind::Continuation),
    ];
    let input =
        input_with_unrelated_base(records, relations, vec![slot("s1", vec![0.8], 35, &["r2"])]);
    let (plan, _, binding, _) = expect_sent(&input);
    assert_eq!(
        binding.packs[0].records,
        vec![rid("r0"), rid("r1"), rid("r2")]
    );
    assert_eq!(exp_ids(&plan), vec![rid("r0"), rid("r1"), rid("r2")]);
    assert_eq!(plan.experience[2].continuation_refs, vec![rid("r9")]);
}

#[test]
fn ex2_target_updates_retained_without_local_only_revisions() {
    let records = vec![
        rec("t0", 10, "start"),
        rec("t1", 20, "mid"),
        rec("t2", 30, "end"),
        rec("i", 40, "pure_recall"),
        rec("j", 50, "interpreter_revision"),
    ];
    let relations = vec![
        rel("i", "t0", RelationKind::LocalOnly),
        rel("j", "t2", RelationKind::LocalOnly),
    ];
    let input = input_with_unrelated_base(
        records,
        relations,
        vec![slot("s1", vec![0.6], 10, &["t0", "t1", "t2"])],
    );
    let (plan, _, _, _) = expect_sent(&input);
    assert_eq!(exp_ids(&plan), vec![rid("t0"), rid("t1"), rid("t2")]);
}

#[test]
fn ex3_alias_merge_deduplicates_shared_quote_without_merge_record() {
    let records = vec![
        rec("q", 1, "shared_quote"),
        rec("a1", 2, "c1_init"),
        rec("b1", 3, "c2_init"),
        rec("a2", 4, "c1_target"),
        rec("M", 5, "merge_record"),
    ];
    let relations = vec![
        rel("a1", "q", RelationKind::BodySource),
        rel("b1", "q", RelationKind::BodySource),
        rel("M", "a1", RelationKind::LocalOnly),
        rel("M", "b1", RelationKind::LocalOnly),
    ];
    let merged_slot = Slot {
        basis: bid("c1_merged"),
        h: vec![0.9],
        first_f_pos: HPos(2),
        members: vec![member("c1", &["a1", "a2"]), member("c2", &["b1"])],
        merge_undecided: false,
    };
    let input = input_with_unrelated_base(records, relations, vec![merged_slot]);
    let (plan, _, binding, _) = expect_sent(&input);
    assert_eq!(binding.packs.len(), 1);
    assert_eq!(binding.accepted, 1);
    assert_eq!(
        exp_ids(&plan),
        vec![rid("q"), rid("a1"), rid("b1"), rid("a2")]
    );
}

#[test]
fn ex4_correction_over_budget_fails_base_request() {
    let records = vec![
        rec("d0", 10, "old_statement"),
        rec("c", 20, "correction_to_d0"),
        rec("x2", 101, "latest"),
    ];
    let p = b"P:base".to_vec();
    let base_without_c = measure_plan(
        &p,
        vec![("d0", Delivered::Full, &[])],
        vec![],
        ("x2", &[]),
        &records,
    );
    let base_with_c = measure_plan(
        &p,
        vec![("d0", Delivered::Full, &[])],
        vec![("c", &[])],
        ("x2", &[]),
        &records,
    );
    assert!(base_with_c > base_without_c);

    let reserve = 10;
    let capacity = (base_with_c - 1) + reserve;
    assert!(base_without_c + reserve <= capacity);

    let mut input = custom_input(
        &p,
        vec![DItem {
            id: rid("d0"),
            delivered: Delivered::Full,
        }],
        "x2",
        records,
        vec![rel("c", "d0", RelationKind::Correction)],
        Vec::new(),
    );
    input.budget = Budget {
        capacity,
        output_reserve: reserve,
    };

    assert_eq!(
        assemble(&input, &TestCodec).expect("valid"),
        Outcome::BaseRequestFailed(BaseFailure::OverCapacity {
            measured: base_with_c,
            output_reserve: reserve,
            capacity,
        })
    );
}

#[test]
fn ex4_variant_optional_pack_overflow_still_sends_base() {
    let mut input = input_with_unrelated_base(
        vec![
            rec("d0", 10, "old_statement"),
            rec("c", 20, "correction_to_d0"),
        ],
        vec![rel("c", "d0", RelationKind::Correction)],
        vec![slot("s1", vec![0.8], 30, &["d0"])],
    );
    let base_only = unrelated_base_len(&input);
    let with_pack = measure_plan(
        &input.protocol,
        vec![("x1", Delivered::Full, &[])],
        vec![("d0", &[]), ("c", &[])],
        ("x2", &[]),
        &input.records,
    );
    assert!(with_pack > base_only);

    let reserve = 5;
    input.budget = Budget {
        capacity: (with_pack - 1) + reserve,
        output_reserve: reserve,
    };

    let (plan, _, binding, note) = expect_sent(&input);
    assert_eq!(note, SentNote::OptionalOverBudget);
    assert_eq!(binding.accepted, 0);
    assert_eq!(binding.packs[0].fate, PackFate::FirstOverflow);
    assert_eq!(binding.packs[0].records, vec![rid("d0"), rid("c")]);
    assert!(plan.experience.is_empty());
}

fn long_chain_records_and_rels() -> (Vec<Record>, Vec<Relation>) {
    let records = (0_u64..=6)
        .map(|k| rec(&format!("r{k}"), k * 10 + 10, &format!("msg_{k}")))
        .collect();
    let relations = (1_u64..=6)
        .map(|k| {
            rel(
                &format!("r{k}"),
                &format!("r{}", k - 1),
                RelationKind::Continuation,
            )
        })
        .collect();
    (records, relations)
}

#[test]
fn ex5_long_dialogue_continuation_chain_does_not_backfill_e() {
    let (records, relations) = long_chain_records_and_rels();
    let input = custom_input(
        b"P:long",
        vec![
            DItem {
                id: rid("r4"),
                delivered: Delivered::Full,
            },
            DItem {
                id: rid("r5"),
                delivered: Delivered::Full,
            },
        ],
        "r6",
        records,
        relations,
        Vec::new(),
    );

    let (plan, _, binding, note) = expect_sent(&input);
    assert_eq!(note, SentNote::NoEligibleExperience);
    assert_eq!(binding.accepted, 0);
    assert!(plan.experience.is_empty());
    assert_eq!(plan.dialogue[0].continuation_refs, vec![rid("r3")]);
    assert_eq!(plan.dialogue[1].continuation_refs, vec![rid("r4")]);
    assert_eq!(plan.latest_continuation_refs, vec![rid("r5")]);
}

#[test]
fn ex5_variant_a_body_source_enters_e() {
    let (mut records, mut relations) = long_chain_records_and_rels();
    records.push(rec("q", 1, "old_quoted"));
    relations.push(rel("r6", "q", RelationKind::BodySource));

    let input = custom_input(
        b"P:long",
        vec![
            DItem {
                id: rid("r4"),
                delivered: Delivered::Full,
            },
            DItem {
                id: rid("r5"),
                delivered: Delivered::Full,
            },
        ],
        "r6",
        records,
        relations,
        Vec::new(),
    );

    let (plan, _, _, _) = expect_sent(&input);
    assert_eq!(exp_ids(&plan), vec![rid("q")]);
}

#[test]
fn ex5_variant_b_body_source_over_budget_fails_base() {
    let (mut records, mut relations) = long_chain_records_and_rels();
    records.push(rec("q", 1, "old_quoted"));
    relations.push(rel("r6", "q", RelationKind::BodySource));

    let p = b"P:long".to_vec();
    let expected_len = measure_plan(
        &p,
        vec![
            ("r4", Delivered::Full, &["r3"]),
            ("r5", Delivered::Full, &["r4"]),
        ],
        vec![("q", &[])],
        ("r6", &["r5"]),
        &records,
    );
    let reserve = 4;
    let capacity = (expected_len - 1) + reserve;

    let mut input = custom_input(
        &p,
        vec![
            DItem {
                id: rid("r4"),
                delivered: Delivered::Full,
            },
            DItem {
                id: rid("r5"),
                delivered: Delivered::Full,
            },
        ],
        "r6",
        records,
        relations,
        Vec::new(),
    );
    input.budget = Budget {
        capacity,
        output_reserve: reserve,
    };

    assert_eq!(
        assemble(&input, &TestCodec).expect("valid"),
        Outcome::BaseRequestFailed(BaseFailure::OverCapacity {
            measured: expected_len,
            output_reserve: reserve,
            capacity,
        })
    );
}
