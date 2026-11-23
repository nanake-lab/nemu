//! R1 / V2-97(3): retain the complete last-good state until an explicit repair.
//! These tests assert that state directly; replay parity alone misses this bug.
use super::*;

fn rejected_history(outcome: CheckOutcome) -> Vec<Commit<Event>> {
    // Reuse P5-26's synthetic old-law fixture, never a live history.
    let mut h: Vec<_> = (1..=5)
        .map(|i| {
            commit(
                i,
                Command::Event(experience(i, [0.7, -0.2, 0.3, -0.1], i == 1)),
            )
        })
        .collect();
    h[4].check = Some(CheckRecord {
        alarm_id: "fault-5".into(),
        binding: Binding {
            lineage: "experiment".into(),
            prefix: h[4].prefix.clone(),
            scope: scope(),
        },
        triggers: vec![Trigger::ComputationFault],
        outcome,
    });
    h
}

fn holds_until_repair(outcome: CheckOutcome, pause_resume: bool) {
    let kernel = Legacy::new();
    let mut h = rejected_history(outcome);
    let before = replay(&kernel, "experiment", scope(), &h[..4]).unwrap();
    let mut current = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert!(kernel.same_bits(current.state(), before.state()));
    assert!(!current.paused());
    let commands = if pause_resume {
        vec![
            Command::Pause,
            Command::Event(experience(7, [0.6, 0.0, 0.0, 0.0], false)),
            Command::Resume,
        ]
    } else {
        (6..=8)
            .map(|i| Command::Event(experience(i, [0.6, 0.0, 0.0, 0.0], false)))
            .collect()
    };
    for (offset, command) in commands.into_iter().enumerate() {
        h.push(commit(6 + offset as u64, command));
        current = step(&kernel, &current, &h).unwrap();
        let full = replay(&kernel, "experiment", scope(), &h).unwrap();
        println!(
            "R1 position={}: before_theta_events={}, incremental_theta_events={}, replay_theta_events={}, dirty={}, paused={}",
            h.last().unwrap().position,
            before.state().theta.event_count(),
            current.state().theta.event_count(),
            full.state().theta.event_count(),
            current.needs_recompute(),
            current.paused()
        );
        assert!(
            kernel.same_bits(current.state(), before.state()),
            "R1: incremental reapplied rejected history without repair"
        );
        assert!(
            kernel.same_bits(full.state(), before.state()),
            "R1: replay reapplied rejected history without repair"
        );
        assert!(current.needs_recompute());
    }
    assert!(!current.paused()); // A fault hold is not a manual growth pause.
}

#[test]
fn r1_inconsistent_then_ordinary_events_keep_last_good_state() {
    holds_until_repair(CheckOutcome::Inconsistent, false);
}

#[test]
fn r1_math_fault_then_ordinary_events_keep_last_good_state() {
    holds_until_repair(
        CheckOutcome::ComputationFault("impossible-candidate".into()),
        false,
    );
}

#[test]
fn r1_inconsistent_then_pause_resume_keeps_last_good_state() {
    holds_until_repair(CheckOutcome::Inconsistent, true);
}

#[test]
fn r1_math_fault_then_pause_resume_keeps_last_good_state() {
    holds_until_repair(
        CheckOutcome::ComputationFault("impossible-candidate".into()),
        true,
    );
}

fn repair(scope: Scope, fault: &CheckRecord) -> Command<Event> {
    Command::Rebuild {
        scope,
        reason: RebuildReason::Repair {
            alarm_id: fault.alarm_id.clone(),
            binding: fault.binding.clone(),
        },
    }
}

#[test]
fn r1_repair_requires_current_fault_binding_and_resumed_growth() {
    // V2-101 prevents a repair for another law, lineage, prefix or alarm from
    // releasing this fault; AC2-18 still honors an explicit manual pause.
    let kernel = Legacy::new();
    let h = rejected_history(CheckOutcome::Inconsistent);
    let rejected = replay(&kernel, "experiment", scope(), &h).unwrap();
    let fault = rejected.unresolved_fault().unwrap();
    for field in 0..6 {
        let mut wrong = fault.clone();
        match field {
            0 => wrong.alarm_id.push('!'),
            1 => wrong.binding.lineage.push('!'),
            2 => wrong.binding.prefix.push('!'),
            3 => wrong.binding.scope.law.push('!'),
            4 => wrong.binding.scope.interpretation.push('!'),
            _ => wrong.binding.scope.sources.push('!'),
        }
        let mut next = h.clone();
        next.push(commit(6, repair(scope(), &wrong)));
        assert!(matches!(
            step(&kernel, &rejected, &next),
            Err(Fault::Binding(_))
        ));
    }
    let mut next = h.clone();
    next.push(commit(6, Command::Pause));
    let paused = step(&kernel, &rejected, &next).unwrap();
    next.push(commit(7, repair(scope(), fault)));
    assert!(matches!(
        step(&kernel, &paused, &next),
        Err(Fault::Binding(_))
    ));
    assert!(kernel.same_bits(rejected.state(), paused.state()));
    // A successful repair catches up all retained history and releases growth.
    next.pop();
    next.push(commit(7, Command::Resume));
    let resumed = step(&kernel, &paused, &next).unwrap();
    assert!(kernel.same_bits(resumed.state(), rejected.state()));
    next.push(commit(8, repair(scope(), fault)));
    let repaired = step(&kernel, &resumed, &next).unwrap();
    assert_eq!(repaired.state().theta.event_count(), 5);
    assert!(repaired.unresolved_fault().is_none());
    assert!(!repaired.needs_recompute());
    next.push(commit(
        9,
        Command::Event(experience(9, [0.6, 0.0, 0.0, 0.0], false)),
    ));
    let after = step(&kernel, &repaired, &next).unwrap();
    assert_eq!(after.state().theta.event_count(), 6);
    let full = replay(&kernel, "experiment", scope(), &next).unwrap();
    assert!(kernel.same_bits(after.state(), full.state()));
}

#[test]
fn r1_checks_reviews_and_version_changes_do_not_substitute_for_repair() {
    // A new independent-check observation (even Consistent) is not a recorded
    // repair, nor is a verdict on the law. V2-97(2)/(3) are distinct paths.
    let kernel = Legacy::new();
    for outcome in [
        CheckOutcome::Consistent,
        CheckOutcome::Unavailable(Unavailable::Missing),
        CheckOutcome::Unavailable(Unavailable::Failed),
        CheckOutcome::Unavailable(Unavailable::TimedOut),
    ] {
        let mut h = rejected_history(CheckOutcome::Inconsistent);
        let held = replay(&kernel, "experiment", scope(), &h).unwrap();
        let fault = held.unresolved_fault().unwrap().clone();
        h.push(commit(
            6,
            Command::Event(experience(6, [0.6, 0.0, 0.0, 0.0], false)),
        ));
        h[5].check = Some(CheckRecord {
            alarm_id: "observation-6".into(),
            binding: Binding {
                lineage: "experiment".into(),
                prefix: h[5].prefix.clone(),
                scope: scope(),
            },
            triggers: vec![Trigger::ReadoutJump],
            outcome,
        });
        let mut current = step(&kernel, &held, &h).unwrap();
        assert!(kernel.same_bits(current.state(), held.state()));
        assert!(
            current
                .audits()
                .last()
                .unwrap()
                .notify_developer_and_caretaker
        );
        for verdict in [ReviewVerdict::LawCorrect, ReviewVerdict::LawIncorrect] {
            h.push(commit(
                h.len() as u64 + 1,
                Command::Review {
                    alarm_id: fault.alarm_id.clone(),
                    binding: fault.binding.clone(),
                    verdict,
                },
            ));
            current = step(&kernel, &current, &h).unwrap();
            assert!(kernel.same_bits(current.state(), held.state()));
        }
        let mut new = scope();
        new.law = "old-v2-test".into();
        h.push(commit(
            9,
            Command::Rebuild {
                scope: new.clone(),
                reason: RebuildReason::LawVersion,
            },
        ));
        current = step(&kernel, &current, &h).unwrap();
        assert_eq!(current.target_scope(), &new);
        assert_eq!(current.applied_scope(), &scope());
        assert_eq!(current.unresolved_fault(), Some(&fault));
        assert!(kernel.same_bits(current.state(), held.state()));
        assert_eq!(current.audits().last().unwrap().kind, AuditKind::FaultHeld);
        let mut stale = h.clone();
        stale.push(commit(10, repair(scope(), &fault)));
        assert!(matches!(
            step(&kernel, &current, &stale),
            Err(Fault::Binding(_))
        ));
        h.push(commit(10, repair(new.clone(), &fault)));
        let repaired = step(&kernel, &current, &h).unwrap();
        let mut plain = h[..6].to_vec();
        for entry in &mut plain {
            entry.check = None;
        }
        let expected = replay(&kernel, "experiment", new, &plain).unwrap();
        assert!(kernel.same_bits(repaired.state(), expected.state()));
        assert!(repaired.unresolved_fault().is_none());
    }
}

#[test]
fn r1_failed_repair_retains_fault_and_requires_new_bound_repair() {
    // V2-97(3): a repair attempt is not success. Its own rejection is replayable
    // and invalidates the old fault reference for subsequent repair attempts.
    let kernel = Legacy::new();
    let mut h = rejected_history(CheckOutcome::Inconsistent);
    let held = replay(&kernel, "experiment", scope(), &h).unwrap();
    let original = held.unresolved_fault().unwrap().clone();
    h.push(commit(6, repair(scope(), &original)));
    h[5].check = Some(CheckRecord {
        alarm_id: "fault-6".into(),
        binding: Binding {
            lineage: "experiment".into(),
            prefix: h[5].prefix.clone(),
            scope: scope(),
        },
        triggers: vec![Trigger::ComputationFault],
        outcome: CheckOutcome::ComputationFault("repair-still-broken".into()),
    });
    let failed = step(&kernel, &held, &h).unwrap();
    assert!(kernel.same_bits(failed.state(), held.state()));
    let fault = failed.unresolved_fault().unwrap().clone();
    assert_eq!(fault.alarm_id, "fault-6");
    h.push(commit(
        7,
        Command::Event(experience(7, [0.6, 0.0, 0.0, 0.0], false)),
    ));
    let later = step(&kernel, &failed, &h).unwrap();
    assert!(kernel.same_bits(later.state(), held.state()));
    assert_eq!(
        later.audits().last().unwrap().unresolved_fault,
        Some(fault.clone())
    );
    let mut stale = h.clone();
    stale.push(commit(8, repair(scope(), &original)));
    assert!(matches!(
        step(&kernel, &later, &stale),
        Err(Fault::Binding(_))
    ));
    h.push(commit(8, repair(scope(), &fault)));
    let repaired = step(&kernel, &later, &h).unwrap();
    let full = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert!(kernel.same_bits(repaired.state(), full.state()));
    assert_eq!(repaired.state().theta.event_count(), 6);
    assert!(!repaired.needs_recompute());
    assert!(repaired.unresolved_fault().is_none());
}
