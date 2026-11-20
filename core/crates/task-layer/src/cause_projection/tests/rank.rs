//! Ranking and residual validation tests (ADR 0042 §1.2, U1; Card §6.1).

use super::*;
use crate::cause_projection::{InvalidInput, Outcome, SentNote, assemble};

#[test]
fn rank_abs_max_beats_coordinate_sum() {
    let records = vec![rec("r1", 1, "neg_heavy"), rec("r2", 2, "sum_heavy")];
    let slots = vec![
        slot("s_sum", vec![0.5, 0.5], 10, &["r2"]),
        slot("s_abs", vec![-0.9, 0.1], 20, &["r1"]),
    ];
    let input = input_with_unrelated_base(records, Vec::new(), slots);
    let Outcome::Sent { binding, note, .. } = assemble(&input, &TestCodec).expect("valid") else {
        panic!("expected Sent");
    };
    assert_eq!(note, SentNote::AllFit);
    let order: Vec<_> = binding.packs.into_iter().map(|p| p.basis).collect();
    assert_eq!(order, vec![bid("s_abs"), bid("s_sum")]);
}

#[test]
fn rank_ties_break_by_first_f_pos_then_basis_byte_order() {
    let records = vec![
        rec("r1", 1, "a"),
        rec("r2", 2, "b"),
        rec("r3", 3, "c"),
        rec("r4", 4, "d"),
    ];
    let slots = vec![
        slot("b_late", vec![0.7], 30, &["r1"]),
        slot("b_tie_z", vec![-0.7, 0.2], 10, &["r2"]),
        slot("b_tie_a", vec![0.7, -0.7], 10, &["r3"]),
        slot("b_mid", vec![0.7], 20, &["r4"]),
    ];
    let input = input_with_unrelated_base(records, Vec::new(), slots);
    let Outcome::Sent { binding, .. } = assemble(&input, &TestCodec).expect("valid") else {
        panic!("expected Sent");
    };
    let order: Vec<_> = binding.packs.into_iter().map(|p| p.basis).collect();
    assert_eq!(
        order,
        vec![bid("b_tie_a"), bid("b_tie_z"), bid("b_mid"), bid("b_late")]
    );
}

#[test]
fn rank_drops_zero_negative_zero_and_empty_h() {
    let records = vec![
        rec("r1", 1, "keep"),
        rec("r2", 2, "zero"),
        rec("r3", 3, "neg_zero"),
        rec("r4", 4, "empty"),
    ];
    let slots = vec![
        slot("s_zero", vec![0.0, 0.0], 1, &["r2"]),
        slot("s_neg_zero", vec![-0.0], 2, &["r3"]),
        slot("s_empty", Vec::new(), 3, &["r4"]),
        slot("s_keep", vec![-0.25], 4, &["r1"]),
    ];
    let input = input_with_unrelated_base(records, Vec::new(), slots);
    let Outcome::Sent { plan, binding, .. } = assemble(&input, &TestCodec).expect("valid") else {
        panic!("expected Sent");
    };
    assert_eq!(binding.packs.len(), 1);
    assert_eq!(binding.packs[0].basis, bid("s_keep"));
    assert_eq!(binding.accepted, 1);
    assert_eq!(
        plan.experience
            .iter()
            .map(|e| e.id.clone())
            .collect::<Vec<_>>(),
        vec![rid("r1")]
    );
}

#[test]
fn rank_rejects_nan_and_infinite_h() {
    let base_rec = vec![rec("r1", 1, "body")];

    for bad_coord in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let input = input_with_unrelated_base(
            base_rec.clone(),
            Vec::new(),
            vec![slot("s_bad", vec![0.1, bad_coord], 5, &["r1"])],
        );
        let err = assemble(&input, &TestCodec).expect_err("non-finite h must fail validation");
        assert_eq!(
            err,
            InvalidInput::NonFiniteH {
                basis: bid("s_bad"),
                index: 1,
            }
        );
    }
}
