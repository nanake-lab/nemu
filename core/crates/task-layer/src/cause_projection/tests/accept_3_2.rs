//! Acceptance tests for in-scope rows of ADR 0042 §3.2 (Card §6.4).

use super::*;
use crate::cause_projection::{
    BaseFailure, Budget, DItem, Delivered, Eligibility, InvalidInput, Outcome, PackFate,
    RelationKind, SentNote, Unavailable, assemble,
};

#[test]
fn two_packs_share_source_and_correction() {
    let records = vec![
        rec("s", 10, "shared_src"),
        rec("a", 20, "pack1_extra"),
        rec("b", 30, "pack2_extra"),
        rec("c", 40, "corr_of_s"),
    ];
    let slots = vec![
        slot("s1", vec![0.9], 10, &["s", "a"]),
        slot("s2", vec![0.5], 20, &["s", "b"]),
    ];
    let input = input_with_unrelated_base(
        records,
        vec![rel("c", "s", RelationKind::Correction)],
        slots,
    );
    let (plan, _, binding, note) = expect_sent(&input);
    assert_eq!(note, SentNote::AllFit);
    assert_eq!(binding.accepted, 2);
    assert_eq!(
        binding.packs[0].fate,
        PackFate::Accepted {
            new_records: vec![rid("s"), rid("a"), rid("c")],
        }
    );
    assert_eq!(
        binding.packs[1].fate,
        PackFate::Accepted {
            new_records: vec![rid("b")],
        }
    );
    assert_eq!(exp_ids(&plan), vec![rid("s"), rid("a"), rid("b"), rid("c")]);
}

#[test]
fn pack_partly_in_dialogue_keeps_s_in_d_and_puts_a_c_in_e() {
    let records = vec![
        rec("s", 10, "already_in_dialogue"),
        rec("a", 20, "extra_in_pack"),
        rec("c", 30, "corr_of_s"),
        rec("x2", 101, "latest"),
    ];
    let base_no_slot = custom_input(
        b"P:part",
        vec![DItem {
            id: rid("s"),
            delivered: Delivered::Full,
        }],
        "x2",
        records,
        vec![rel("c", "s", RelationKind::Correction)],
        Vec::new(),
    );
    let (base_plan, _, _, _) = expect_sent(&base_no_slot);
    assert_eq!(exp_ids(&base_plan), vec![rid("c")]);

    let mut with_slot = base_no_slot;
    with_slot.slots = vec![slot("s1", vec![0.7], 15, &["s", "a"])];
    let (plan, _, binding, _) = expect_sent(&with_slot);
    assert_eq!(binding.accepted, 1);
    assert_eq!(
        binding.packs[0].fate,
        PackFate::Accepted {
            new_records: vec![rid("a")],
        }
    );
    assert_eq!(exp_ids(&plan), vec![rid("a"), rid("c")]);
    assert_eq!(
        plan.dialogue.into_iter().map(|d| d.id).collect::<Vec<_>>(),
        vec![rid("s")]
    );
}

#[test]
fn largest_pack_overflows_stops_without_trying_shorter_pack() {
    let long_body = "L".repeat(80);
    let records = vec![rec("r_long", 10, &long_body), rec("r_short", 20, "hi")];
    let mut input = input_with_unrelated_base(
        records,
        Vec::new(),
        vec![
            slot("s1", vec![0.9], 10, &["r_long"]),
            slot("s2", vec![0.4], 20, &["r_short"]),
        ],
    );
    let len_with_short = measure_plan(
        &input.protocol,
        vec![("x1", Delivered::Full, &[])],
        vec![("r_short", &[])],
        ("x2", &[]),
        &input.records,
    );
    let len_with_long = measure_plan(
        &input.protocol,
        vec![("x1", Delivered::Full, &[])],
        vec![("r_long", &[])],
        ("x2", &[]),
        &input.records,
    );
    assert!(len_with_long > len_with_short);

    let reserve = 10;
    input.budget = Budget {
        capacity: (len_with_long - 1) + reserve,
        output_reserve: reserve,
    };
    assert!(len_with_short + reserve <= input.budget.capacity);

    let (plan, _, binding, note) = expect_sent(&input);
    assert_eq!(note, SentNote::OptionalOverBudget);
    assert_eq!(binding.accepted, 0);
    assert_eq!(binding.packs[0].fate, PackFate::FirstOverflow);
    assert_eq!(binding.packs[1].fate, PackFate::NotTried);
    assert!(plan.experience.is_empty());
}

#[test]
fn base_fills_exact_capacity_overflows_pack_with_new_record() {
    let mut input = input_with_unrelated_base(
        vec![rec("a", 10, "new_cause")],
        Vec::new(),
        vec![slot("s1", vec![0.7], 5, &["a"])],
    );
    let base_len = unrelated_base_len(&input);
    let reserve = 7;
    input.budget = Budget {
        capacity: base_len + reserve,
        output_reserve: reserve,
    };
    let (plan, _, binding, note) = expect_sent(&input);
    assert_eq!(note, SentNote::OptionalOverBudget);
    assert_eq!(binding.accepted, 0);
    assert_eq!(binding.packs[0].fate, PackFate::FirstOverflow);
    assert!(plan.experience.is_empty());
}

#[test]
fn base_fills_exact_capacity_minus_one_fails_base() {
    let mut input = input_with_unrelated_base(
        vec![rec("a", 10, "new_cause")],
        Vec::new(),
        vec![slot("s1", vec![0.7], 5, &["a"])],
    );
    let base_len = unrelated_base_len(&input);
    let reserve = 7;
    input.budget = Budget {
        capacity: base_len + reserve - 1,
        output_reserve: reserve,
    };
    assert_eq!(
        assemble(&input, &TestCodec).expect("valid"),
        Outcome::BaseRequestFailed(BaseFailure::OverCapacity {
            measured: base_len,
            output_reserve: reserve,
            capacity: base_len + reserve - 1,
        })
    );
}

#[test]
fn base_fills_exact_capacity_accepts_zero_new_record_pack() {
    let mut input = input_with_unrelated_base(
        Vec::new(),
        Vec::new(),
        vec![slot("s1", vec![0.7], 5, &["x1"])],
    );
    let base_len = unrelated_base_len(&input);
    let reserve = 7;
    input.budget = Budget {
        capacity: base_len + reserve,
        output_reserve: reserve,
    };
    let (_, _, binding, note) = expect_sent(&input);
    assert_eq!(note, SentNote::AllFit);
    assert_eq!(binding.accepted, 1);
    assert_eq!(
        binding.packs[0].fate,
        PackFate::Accepted {
            new_records: Vec::new(),
        }
    );
}

#[test]
fn unavailable_when_frontier_incomplete() {
    let mut input = input_with_unrelated_base(Vec::new(), Vec::new(), Vec::new());
    input.frontier_complete = false;
    assert_eq!(
        assemble(&input, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::FrontierIncomplete)
    );
}

#[test]
fn unavailable_when_decomposition_unknown() {
    let mut input = input_with_unrelated_base(Vec::new(), Vec::new(), Vec::new());
    input.decomposition_unknown = true;
    assert_eq!(
        assemble(&input, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::DecompositionUnknown)
    );
}

#[test]
fn unavailable_when_candidate_merge_undecided() {
    let mut s = slot("s_undecided", vec![0.6], 10, &["r1"]);
    s.merge_undecided = true;
    let input = input_with_unrelated_base(vec![rec("r1", 1, "src")], Vec::new(), vec![s]);
    assert_eq!(
        assemble(&input, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::MergeUndecided(bid("s_undecided")))
    );
}

#[test]
fn unavailable_when_pack_hits_unknown_relation() {
    let input = input_with_unrelated_base(
        vec![rec("r1", 1, "src"), rec("r2", 2, "other")],
        vec![rel("r1", "r2", RelationKind::Unknown)],
        vec![slot("s1", vec![0.6], 10, &["r1"])],
    );
    assert_eq!(
        assemble(&input, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::UnknownRelation {
            from: rid("r1"),
            to: rid("r2"),
        })
    );
}

#[test]
fn unavailable_when_body_source_target_missing() {
    let input = input_with_unrelated_base(
        vec![rec("r1", 1, "src")],
        vec![rel("r1", "missing_target", RelationKind::BodySource)],
        vec![slot("s1", vec![0.6], 10, &["r1"])],
    );
    assert_eq!(
        assemble(&input, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::MissingRecord(rid("missing_target")))
    );
}

#[test]
fn unavailable_when_pack_record_eligibility_undetermined() {
    let input = input_with_unrelated_base(
        vec![rec_with("r1", 1, "pending", Eligibility::Undetermined)],
        Vec::new(),
        vec![slot("s1", vec![0.6], 10, &["r1"])],
    );
    assert_eq!(
        assemble(&input, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::EligibilityUndetermined(rid("r1")))
    );
}

#[test]
fn ineligible_in_pack_excludes_pack_and_tries_next() {
    let records = vec![
        rec_inelig("r_bad", 10, "revoked", "revoked"),
        rec("r_ok", 20, "valid_cause"),
    ];
    let slots = vec![
        slot("s1", vec![0.9], 10, &["r_bad"]),
        slot("s2", vec![0.4], 20, &["r_ok"]),
    ];
    let input = input_with_unrelated_base(records, Vec::new(), slots);
    let (plan, _, binding, note) = expect_sent(&input);
    assert_eq!(note, SentNote::AllFit);
    assert_eq!(binding.accepted, 1);
    assert_eq!(
        binding.packs[0].fate,
        PackFate::Excluded {
            record: Some(rid("r_bad")),
            reason: "revoked".into(),
        }
    );
    assert_eq!(
        binding.packs[1].fate,
        PackFate::Accepted {
            new_records: vec![rid("r_ok")],
        }
    );
    assert_eq!(exp_ids(&plan), vec![rid("r_ok")]);
}

#[test]
fn ineligible_in_base_fails_base_request() {
    let mut input = input_with_unrelated_base(Vec::new(), Vec::new(), Vec::new());
    input.records[0].eligibility = Eligibility::Ineligible {
        reason: "prohibited_d".into(),
    };
    assert_eq!(
        assemble(&input, &TestCodec).expect("valid"),
        Outcome::BaseRequestFailed(BaseFailure::Ineligible {
            record: rid("x1"),
            reason: "prohibited_d".into(),
        })
    );
}

#[test]
fn all_candidates_excluded_returns_no_eligible_experience() {
    let records = vec![
        rec_inelig("r_bad1", 10, "bad1", "no_root"),
        rec_inelig("r_bad2", 20, "bad2", "revoked"),
    ];
    let slots = vec![
        slot("s1", vec![0.9], 10, &["r_bad1"]),
        slot("s2", vec![0.5], 20, &["r_bad2"]),
    ];
    let input = input_with_unrelated_base(records, Vec::new(), slots);
    let (plan, _, binding, note) = expect_sent(&input);
    assert_eq!(note, SentNote::NoEligibleExperience);
    assert_eq!(binding.accepted, 0);
    assert_eq!(binding.packs.len(), 2);
    assert!(plan.experience.is_empty());
}

#[test]
fn dialogue_containing_latest_deduplicated_into_o() {
    let mut input = input_with_unrelated_base(Vec::new(), Vec::new(), Vec::new());
    let p03 = Delivered::Partial { start: 0, end: 3 };
    input.dialogue = vec![
        DItem {
            id: rid("x2"),
            delivered: Delivered::Full,
        },
        DItem {
            id: rid("x1"),
            delivered: p03,
        },
        DItem {
            id: rid("x1"),
            delivered: Delivered::Full,
        },
    ];
    let (plan, _, _, _) = expect_sent(&input);
    assert_eq!(plan.dialogue.len(), 1);
    assert_eq!(plan.dialogue[0].id, rid("x1"));
    assert_eq!(plan.dialogue[0].delivered, p03);
    assert_eq!(plan.latest, rid("x2"));
}

#[test]
fn all_sent_plans_have_strict_false() {
    let empty_slots = input_with_unrelated_base(Vec::new(), Vec::new(), Vec::new());
    assert!(!expect_sent(&empty_slots).0.strict);

    let fit = input_with_unrelated_base(
        vec![rec("a", 10, "cause")],
        Vec::new(),
        vec![slot("s1", vec![0.5], 10, &["a"])],
    );
    assert!(!expect_sent(&fit).0.strict);

    let mut overflow = fit;
    overflow.budget.capacity = unrelated_base_len(&overflow);
    assert!(!expect_sent(&overflow).0.strict);
}

#[test]
fn validation_rejects_bad_partial_range_and_duplicate_slot_basis() {
    let mut bad_range = input_with_unrelated_base(Vec::new(), Vec::new(), Vec::new());
    bad_range.dialogue = vec![DItem {
        id: rid("x1"),
        delivered: Delivered::Partial { start: 4, end: 2 },
    }];
    assert_eq!(
        assemble(&bad_range, &TestCodec),
        Err(InvalidInput::InvalidPartialRange {
            id: rid("x1"),
            start: 4,
            end: 2,
            body_len: 6,
        })
    );

    let dup_basis = input_with_unrelated_base(
        vec![rec("r1", 1, "a")],
        Vec::new(),
        vec![
            slot("same", vec![0.5], 1, &["r1"]),
            slot("same", vec![0.8], 2, &["r1"]),
        ],
    );
    assert_eq!(
        assemble(&dup_basis, &TestCodec),
        Err(InvalidInput::DuplicateSlotBasis(bid("same")))
    );
}
