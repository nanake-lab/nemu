use crate::{kernel, scenario::*, verifier::*};
use core_kernel::fast::{EventClass, EventInput};

fn result(dividend: u64) -> (Request, Receipt) {
    let request = Request {
        action_id: "a1".into(),
        question: Question {
            dividend,
            divisor: 7,
        },
        issued_round: 120,
        issued_ns: NS,
        deadline_ns: 3 * NS,
    };
    let receipt = Receipt {
        action_id: "a1".into(),
        question_digest: request.question.digest(),
        quotient: dividend / 7,
        remainder: dividend % 7,
        verifier_version: VERSION.into(),
    };
    (request, receipt)
}

#[test]
fn one_line_ablation_matches_production_when_weight_is_one() {
    for class in [
        EventClass::InternalRecord { n: 0 },
        EventClass::ActionResult { n_issued: 0 },
    ] {
        for appraisal in [
            Some([0.0, 0.0, 0.0, 0.2]),
            Some([0.0, 0.0, 0.0, -0.2]),
            None,
        ] {
            let input = EventInput {
                appraisal,
                empathy_weight: 1.0,
                class,
            };
            let a = kernel::pulse(
                &kernel::initial(),
                &kernel::effects(),
                NS,
                input,
                0.2,
                false,
            );
            let b = kernel::pulse(&kernel::initial(), &kernel::effects(), NS, input, 0.2, true);
            assert_eq!(a, b);
        }
    }
}

#[test]
fn matched_state_late_result_changes_only_class_multiplier() {
    let mut input = EventInput {
        appraisal: Some([0.0, 0.0, 0.0, 0.2]),
        empathy_weight: 1.0,
        class: EventClass::ActionResult { n_issued: 0 },
    };
    let early = kernel::pulse(
        &kernel::initial(),
        &kernel::effects(),
        NS,
        input,
        0.2,
        false,
    );
    input.class = EventClass::ActionResult { n_issued: 120 };
    let late = kernel::pulse(
        &kernel::initial(),
        &kernel::effects(),
        NS,
        input,
        0.2,
        false,
    );
    let ablated = kernel::pulse(&kernel::initial(), &kernel::effects(), NS, input, 0.2, true);
    assert_eq!(early, ablated);
    assert_eq!(late.2, early.2); // identical projected x, not a saturation confound
    for (a, b) in early.1.iter().zip(&late.1) {
        assert_eq!(*b, a * 2.0_f64.powi(-120));
    }
    println!(
        "early_drive={:?} late_drive={:?} ratio={:.17e}",
        early.1,
        late.1,
        2.0_f64.powi(-120)
    );
}

#[test]
fn one_late_verified_result_from_rest_refutes_uniform_return() {
    let mut gate = Verifier::new(&[]);
    let (mut request, receipt) = result(1400);
    request.issued_ns = 121 * NS;
    request.deadline_ns = 123 * NS;
    let now = request.issued_ns + NS / 5;
    assert!(now as f64 / NS as f64 > envelope()["t_ret_s"].as_f64().unwrap());
    let verdict = gate.accept(&request, &receipt, now);
    assert!(verdict.release);
    let mut rest = kernel::initial();
    rest.nature = [0.0; 4];
    let input = EventInput {
        appraisal: Some([0.0, 0.0, 0.0, 0.2]),
        empathy_weight: 1.0,
        class: EventClass::ActionResult { n_issued: 120 },
    };
    let a = kernel::pulse(&rest, &kernel::effects(), NS / 5, input, 0.0, false);
    let c = kernel::pulse(&rest, &kernel::effects(), NS / 5, input, 0.0, true);
    assert!(a.0.nature[3] <= EPSILON);
    assert_eq!(c.0.nature[3], 0.2);
    assert!(c.0.nature[3] > EPSILON);
    println!(
        "minimal counterexample: t=121.2s > T_ret, x_before=rest, A={}, C={}, epsilon={}",
        a.0.nature[3], c.0.nature[3], EPSILON
    );
}

#[test]
fn aliases_new_action_ids_and_changed_proofs_do_not_release_again() {
    let mut gate = Verifier::new(&[]);
    let (mut request, mut receipt) = result(1400);
    assert!(gate.accept(&request, &receipt, 2 * NS).release);
    assert_eq!(
        gate.accept(&request, &receipt, 2 * NS).reason,
        "duplicate_completion"
    );
    request.action_id = "new-action-same-question".into();
    receipt.action_id = request.action_id.clone();
    let repeated = gate.accept(&request, &receipt, 2 * NS);
    assert!(repeated.pulse_allowed);
    assert!(!repeated.release);
    assert_eq!(repeated.reason, "known_question");
    assert_eq!(gate.known.len(), 1);
}

#[test]
fn false_binding_wrong_certificate_unknown_version_and_overflow_are_not_novel() {
    let (request, good) = result(1400);
    for mode in 0..5 {
        let mut bad = good.clone();
        match mode {
            0 => bad.question_digest = "forged".into(),
            1 => bad.quotient += 1,
            2 => bad.verifier_version = "model-says-new".into(),
            3 => {
                bad.quotient = u64::MAX;
                bad.remainder = u64::MAX;
            }
            _ => bad.action_id = "wrong-action".into(),
        }
        let mut gate = Verifier::new(&[]);
        let verdict = gate.accept(&request, &bad, 2 * NS);
        assert!(!verdict.release);
        assert!(gate.known.is_empty());
    }
}

#[test]
fn negative_result_is_verified_without_selecting_on_outcome() {
    let mut gate = Verifier::new(&[]);
    let (request, receipt) = result(1401);
    assert_eq!(receipt.remainder, 1);
    assert!(gate.accept(&request, &receipt, 2 * NS).release);
}

#[test]
fn commit_deadline_not_return_or_retry_time_controls_pulse() {
    let (request, receipt) = result(1400);
    assert!(
        Verifier::new(&[])
            .accept(&request, &receipt, request.deadline_ns)
            .release
    );
    let late = Verifier::new(&[]).accept(&request, &receipt, request.deadline_ns + 1);
    assert!(!late.release);
    assert!(!late.pulse_allowed);
    assert_eq!(late.reason, "expired_at_commit");
    // Request still carries n_issued=120; a later reset of global n cannot alter it.
    assert_eq!(request.issued_round, 120);
}

#[test]
fn serialized_receipts_rebuild_identical_gate_state_and_verdicts() {
    let fixtures = fixtures();
    let fixture = &fixtures[2];
    let restored: Fixture = serde_json::from_str(&serde_json::to_string(fixture).unwrap()).unwrap();
    assert_eq!(run(fixture, "C"), run(&restored, "C"));
    let mut one = Verifier::new(&fixture.known_at_start);
    let mut two = Verifier::new(&fixture.known_at_start);
    for event in &fixture.events {
        if let Some((request, receipt)) = &event.result {
            assert_eq!(
                one.accept(request, receipt, event.time_ns),
                two.accept(request, receipt, event.time_ns)
            );
            two = serde_json::from_str(&serde_json::to_string(&two).unwrap()).unwrap();
        }
    }
    assert_eq!(one, two);
}

#[test]
fn verifier_does_not_read_world_labels_or_text() {
    let mut fixture = fixtures().remove(2);
    let before = run(&fixture, "C");
    for event in &mut fixture.events {
        event.text = "I claim I discovered something entirely new".into();
        event.oracle_new_fact = !event.oracle_new_fact;
        event.oracle_divisible = !event.oracle_divisible;
    }
    let after = run(&fixture, "C");
    for (a, b) in before.0.iter().zip(&after.0) {
        assert_eq!(a["verdict"], b["verdict"]);
        assert_eq!(a["drive"], b["drive"]);
        assert_eq!(a["after"], b["after"]);
    }
}

#[test]
fn two_scenarios_preserve_box_but_reject_claim_of_unchanged_return_guarantee() {
    for fixture in fixtures() {
        let (_, a) = run(&fixture, "A");
        let (_, b) = run(&fixture, "B");
        let (ct, c) = run(&fixture, "C");
        for s in [&a, &b, &c] {
            assert!(s["max_abs_state"].as_f64().unwrap() <= 1.0);
        }
        assert_eq!(a["old_uniform_return_holds_in_observed_window"], true);
        assert_eq!(b["old_uniform_return_holds_in_observed_window"], false);
        if fixture.scenario.starts_with("s1") {
            assert_eq!(a["final_state"], c["final_state"]);
            assert_eq!(c["old_uniform_return_holds_in_observed_window"], true);
            assert!(ct.iter().all(|t| t["release"] == false));
        } else {
            assert_eq!(c["old_uniform_return_holds_in_observed_window"], false);
            assert_eq!(c["first_counterexample"]["drive"][3], -0.2);
            assert!(c["first_counterexample"]["after"][3].as_f64().unwrap() < 0.0);
        }
    }
}
