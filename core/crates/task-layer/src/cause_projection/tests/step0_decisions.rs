//! Acceptance tests for every Step 0 decision Q1–Q9 finalized on PR #221 (Card §6.4).

use super::*;
use crate::cause_projection::{
    DItem, Delivered, Eligibility, InvalidInput, Outcome, PackFate, RelationKind, SentNote, Slot,
    Unavailable, assemble,
};

#[test]
fn q1_partial_d_item_not_in_e_unless_full_independently_needed() {
    let records = vec![
        rec("a1", 10, "spoken_half|unspoken_tail"),
        rec("c", 15, "corr_of_a1"),
        rec("o2", 20, "next_input"),
    ];
    let unref = custom_input(
        b"P",
        vec![
            DItem {
                id: rid("a1"),
                delivered: Delivered::Partial { start: 0, end: 11 },
            },
            DItem {
                id: rid("a1"),
                delivered: Delivered::Full,
            },
        ],
        "o2",
        records,
        Vec::new(),
        Vec::new(),
    );
    let (plan, _, binding, _) = expect_sent(&unref);
    assert_eq!(plan.dialogue.len(), 1);
    assert_eq!(
        plan.dialogue[0].delivered,
        Delivered::Partial { start: 0, end: 11 }
    );
    assert!(plan.experience.is_empty());
    assert_eq!(binding.base_records, vec![rid("a1"), rid("o2")]);

    let mut with_corr = unref.clone();
    with_corr.relations = vec![rel("c", "a1", RelationKind::Correction)];
    assert_eq!(exp_ids(&expect_sent(&with_corr).0), vec![rid("c")]);

    let mut cited_by_o = unref.clone();
    cited_by_o.relations = vec![rel("o2", "a1", RelationKind::BodySource)];
    assert_eq!(exp_ids(&expect_sent(&cited_by_o).0), vec![rid("a1")]);

    let mut in_slot = unref;
    in_slot.slots = vec![slot("s1", vec![0.8], 25, &["a1"])];
    let (p_slot, _, b_slot, _) = expect_sent(&in_slot);
    assert_eq!(
        b_slot.packs[0].fate,
        PackFate::Accepted {
            new_records: vec![rid("a1")],
        }
    );
    assert_eq!(exp_ids(&p_slot), vec![rid("a1")]);
}

#[test]
fn q2_empty_member_sources_excludes_pack_and_empty_slot_members_invalid() {
    let single_empty = input_with_unrelated_base(
        Vec::new(),
        Vec::new(),
        vec![slot("internal_branch", vec![0.8], 10, &[])],
    );
    let (plan, _, binding, note) = expect_sent(&single_empty);
    assert_eq!(note, SentNote::NoEligibleExperience);
    assert_eq!(binding.accepted, 0);
    assert!(plan.experience.is_empty());
    assert_eq!(
        binding.packs[0].fate,
        PackFate::Excluded {
            record: None,
            reason: "no_external_source".into(),
        }
    );

    let merged_half_empty = input_with_unrelated_base(
        vec![rec("r1", 10, "ext")],
        Vec::new(),
        vec![Slot {
            basis: bid("merged"),
            h: vec![0.8],
            first_f_pos: HPos(10),
            members: vec![member("b1", &["r1"]), member("b2", &[])],
            merge_undecided: false,
        }],
    );
    let (p_m, _, b_m, n_m) = expect_sent(&merged_half_empty);
    assert_eq!(n_m, SentNote::NoEligibleExperience);
    assert_eq!(b_m.accepted, 0);
    assert!(p_m.experience.is_empty());
    assert_eq!(b_m.packs[0].records, vec![rid("r1")]);
    assert_eq!(
        b_m.packs[0].fate,
        PackFate::Excluded {
            record: None,
            reason: "no_external_source".into(),
        }
    );

    let no_members = input_with_unrelated_base(
        Vec::new(),
        Vec::new(),
        vec![Slot {
            basis: bid("empty_members"),
            h: vec![0.5],
            first_f_pos: HPos(10),
            members: Vec::new(),
            merge_undecided: false,
        }],
    );
    assert_eq!(
        assemble(&no_members, &TestCodec),
        Err(InvalidInput::EmptySlotMembers(bid("empty_members")))
    );
}

#[test]
fn q3_unknown_relation_triggers_on_incoming_or_outgoing_visited_only() {
    let incoming = input_with_unrelated_base(
        vec![rec("u", 10, "unknown_sender")],
        vec![rel("u", "x1", RelationKind::Unknown)],
        Vec::new(),
    );
    assert_eq!(
        assemble(&incoming, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::UnknownRelation {
            from: rid("u"),
            to: rid("x1"),
        })
    );

    let disconnected = input_with_unrelated_base(
        vec![rec("u", 10, "unrelated_a"), rec("v", 20, "unrelated_b")],
        vec![rel("u", "v", RelationKind::Unknown)],
        Vec::new(),
    );
    assert_eq!(expect_sent(&disconnected).3, SentNote::NoEligibleExperience);
}

#[test]
fn q4_upfront_candidate_unavailable_blocks_after_overflow_and_with_ineligible() {
    let long_body = "X".repeat(80);
    let mut after_overflow = input_with_unrelated_base(
        vec![
            rec("r1", 10, &long_body),
            rec_with("r2", 20, "pending", Eligibility::Undetermined),
        ],
        Vec::new(),
        vec![
            slot("s1", vec![0.9], 10, &["r1"]),
            slot("s2", vec![0.2], 20, &["r2"]),
        ],
    );
    after_overflow.budget.capacity = unrelated_base_len(&after_overflow);
    assert_eq!(
        assemble(&after_overflow, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::EligibilityUndetermined(rid("r2")))
    );

    let mixed_in_pack = input_with_unrelated_base(
        vec![
            rec_with(
                "r_bad",
                10,
                "bad",
                Eligibility::Ineligible {
                    reason: "revoked".into(),
                },
            ),
            rec_with("r_u", 20, "pending", Eligibility::Undetermined),
            rec("r_ok", 30, "ok"),
        ],
        Vec::new(),
        vec![
            slot("s1", vec![0.9], 10, &["r_bad", "r_u"]),
            slot("s2", vec![0.5], 30, &["r_ok"]),
        ],
    );
    assert_eq!(
        assemble(&mixed_in_pack, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::EligibilityUndetermined(rid("r_u")))
    );
}

#[test]
fn q5_continuation_target_ineligible_or_undetermined_kept_in_refs() {
    for elig in [
        Eligibility::Ineligible {
            reason: "revoked".into(),
        },
        Eligibility::Undetermined,
    ] {
        let input = custom_input(
            b"P",
            Vec::new(),
            "o",
            vec![
                rec_with("r9", 10, "forbidden_or_pending", elig),
                rec("o", 20, "reply"),
            ],
            vec![rel("o", "r9", RelationKind::Continuation)],
            Vec::new(),
        );
        assert_eq!(
            expect_sent(&input).0.latest_continuation_refs,
            vec![rid("r9")]
        );
    }
}

#[test]
fn q6_missing_slot_seed_returns_missing_record_and_global_unavailable_precedes_missing_base() {
    let missing_seed = input_with_unrelated_base(
        Vec::new(),
        Vec::new(),
        vec![slot("s1", vec![0.6], 10, &["s_missing"])],
    );
    assert_eq!(
        assemble(&missing_seed, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::MissingRecord(rid("s_missing")))
    );

    let mut incomplete_with_missing_base =
        input_with_unrelated_base(Vec::new(), Vec::new(), Vec::new());
    incomplete_with_missing_base.frontier_complete = false;
    incomplete_with_missing_base.required_facts = vec![rid("f_missing")];
    assert_eq!(
        assemble(&incomplete_with_missing_base, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::FrontierIncomplete)
    );

    let mut decomp_with_missing_latest =
        input_with_unrelated_base(Vec::new(), Vec::new(), Vec::new());
    decomp_with_missing_latest.decomposition_unknown = true;
    decomp_with_missing_latest.latest = rid("o_missing");
    assert_eq!(
        assemble(&decomp_with_missing_latest, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::DecompositionUnknown)
    );
}

#[test]
fn q7_candidate_unavailable_precedes_base_ineligible_and_base_over_capacity() {
    let mut s = slot("s1", vec![0.5], 10, &["r1"]);
    s.merge_undecided = true;
    let mut base_ineligible =
        input_with_unrelated_base(vec![rec("r1", 1, "src")], Vec::new(), vec![s]);
    base_ineligible.records[1].eligibility = Eligibility::Ineligible {
        reason: "bad_o".into(),
    };
    assert_eq!(
        assemble(&base_ineligible, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::MergeUndecided(bid("s1")))
    );

    let mut base_over_cap = input_with_unrelated_base(
        vec![rec_with("r1", 1, "pending", Eligibility::Undetermined)],
        Vec::new(),
        vec![slot("s1", vec![0.5], 10, &["r1"])],
    );
    base_over_cap.budget.capacity = 1;
    assert_eq!(
        assemble(&base_over_cap, &TestCodec).expect("valid"),
        Outcome::ProjectionUnavailable(Unavailable::EligibilityUndetermined(rid("r1")))
    );
}

#[test]
fn q8_excluded_pack_after_first_overflow_keeps_excluded_fate() {
    let records = vec![
        rec("r1", 10, &"A".repeat(40)),
        rec_with(
            "r_bad",
            20,
            "bad",
            Eligibility::Ineligible {
                reason: "revoked".into(),
            },
        ),
    ];
    let mut input = input_with_unrelated_base(
        records,
        Vec::new(),
        vec![
            slot("s1", vec![0.9], 10, &["r1"]),
            slot("s2", vec![0.4], 20, &["r_bad"]),
        ],
    );
    input.budget.capacity = unrelated_base_len(&input);

    let (_, _, binding, note) = expect_sent(&input);
    assert_eq!(note, SentNote::OptionalOverBudget);
    assert_eq!(binding.packs[0].fate, PackFate::FirstOverflow);
    assert_eq!(
        binding.packs[1].fate,
        PackFate::Excluded {
            record: Some(rid("r_bad")),
            reason: "revoked".into(),
        }
    );
}

#[test]
fn q9_multiple_unavailable_relations_resolved_in_canonical_order() {
    let rels_unknown_and_missing = vec![
        rel("x1", "m2", RelationKind::BodySource),
        rel("x1", "u", RelationKind::Unknown),
        rel("x1", "m1", RelationKind::BodySource),
    ];
    let fwd = input_with_unrelated_base(Vec::new(), rels_unknown_and_missing.clone(), Vec::new());
    let mut rev_rels = rels_unknown_and_missing;
    rev_rels.reverse();
    let rev = input_with_unrelated_base(Vec::new(), rev_rels, Vec::new());
    let expected = Outcome::ProjectionUnavailable(Unavailable::UnknownRelation {
        from: rid("x1"),
        to: rid("u"),
    });
    assert_eq!(assemble(&fwd, &TestCodec).expect("valid"), expected);
    assert_eq!(assemble(&rev, &TestCodec).expect("valid"), expected);

    let rels_multi_missing = vec![
        rel("x1", "m2", RelationKind::BodySource),
        rel("x1", "m1", RelationKind::BodySource),
    ];
    let fwd_m = input_with_unrelated_base(Vec::new(), rels_multi_missing.clone(), Vec::new());
    let mut rev_m_rels = rels_multi_missing;
    rev_m_rels.reverse();
    let rev_m = input_with_unrelated_base(Vec::new(), rev_m_rels, Vec::new());
    let expected_m = Outcome::ProjectionUnavailable(Unavailable::MissingRecord(rid("m1")));
    assert_eq!(assemble(&fwd_m, &TestCodec).expect("valid"), expected_m);
    assert_eq!(assemble(&rev_m, &TestCodec).expect("valid"), expected_m);
}
