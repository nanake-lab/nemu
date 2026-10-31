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
