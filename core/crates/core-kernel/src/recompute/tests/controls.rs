use super::*;

fn history() -> Vec<Commit<Event>> {
    (1..=5)
        .map(|i| {
            commit(
                i,
                Command::Event(experience(i, [0.7, -0.2, 0.3, -0.1], i == 1)),
            )
        })
        .collect()
}
fn check(commit: &mut Commit<Event>, scope: Scope, outcome: CheckOutcome) {
    commit.check = Some(CheckRecord {
        alarm_id: format!("alarm-{}", commit.position),
        binding: Binding {
            lineage: "experiment".into(),
            prefix: commit.prefix.clone(),
            scope,
        },
        triggers: vec![Trigger::ReadoutJump],
        outcome,
    });
}

#[test]
fn ac2_18_pause_preserves_every_field_and_resume_catches_up() {
    let kernel = Legacy::new();
    // Positions of real experiences remain unchanged; controls occupy gaps.
    let mut h: Vec<_> = (1..=5)
        .map(|i| {
            commit(
                i * 2,
                Command::Event(experience(i * 2, [0.7, -0.2, 0.3, -0.1], i == 1)),
            )
        })
        .collect();
    let expected = replay(&kernel, "experiment", scope(), &h).unwrap();
    let frozen = replay(&kernel, "experiment", scope(), &h[..2]).unwrap();
    h.insert(2, commit(5, Command::Pause));
    let paused = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert!(paused.paused());
    assert!(kernel.same_bits(paused.state(), frozen.state()));
    assert_eq!(paused.state().theta.event_count(), 2);
    h.push(commit(11, Command::Resume));
    let resumed = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert!(!resumed.paused());
    assert!(kernel.same_bits(resumed.state(), expected.state()));
    assert_eq!(resumed.state().theta.event_count(), 5);
    assert!(!kernel.same_bits(expected.state(), frozen.state())); // kills identity C/G
}

#[test]
fn ac2_19_source_correction_rebuilds_q_and_theta_as_if_absent() {
    let kernel = Legacy::new();
    let mut h = history();
    if let Command::Event(Event::Experience(e)) = &mut h[2].command {
        e.excluded_from.insert("s2".into());
    }
    let before = replay(&kernel, "experiment", scope(), &h).unwrap();
    let mut corrected = scope();
    corrected.sources = "s2".into();
    h.push(commit(
        6,
        Command::Rebuild {
            scope: corrected,
            reason: RebuildReason::SourceCorrection,
        },
    ));
    let result = replay(&kernel, "experiment", scope(), &h).unwrap();
    let absent: Vec<_> = history().into_iter().filter(|r| r.position != 3).collect();
    let expected = replay(&kernel, "experiment", scope(), &absent).unwrap();
    assert!(kernel.same_bits(result.state(), expected.state()));
    assert!(!kernel.same_bits(result.state(), before.state()));
    assert_eq!(result.state().theta.event_count(), 4);
    assert_ne!(
        result.state().r.tendencies[0].evidence_absolute.to_bits(),
        before.state().r.tendencies[0].evidence_absolute.to_bits()
    );
}

#[test]
fn ac2_20_new_law_recalculates_entire_h_and_records_diff() {
    let kernel = Legacy::new();
    let mut h = history();
    let old = replay(&kernel, "experiment", scope(), &h).unwrap();
    let mut new = scope();
    new.law = "old-v2-test".into();
    let expected = replay(&kernel, "experiment", new.clone(), &h).unwrap();
    h.push(commit(
        6,
        Command::Rebuild {
            scope: new.clone(),
            reason: RebuildReason::LawVersion,
        },
    ));
    let result = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert!(kernel.same_bits(result.state(), expected.state()));
    assert!(!kernel.same_bits(old.state(), result.state()));
    let audit = result.audits().last().unwrap();
    assert!(audit.changed);
    assert_eq!(audit.after_scope, new);
    assert_eq!(audit.before_scope, scope());
    assert!(matches!(
        audit.kind,
        AuditKind::Rebuild(RebuildReason::LawVersion)
    ));
}

#[test]
fn reinterpretation_uses_explicit_new_records_without_mutating_old() {
    let kernel = Legacy::new();
    let mut h = history();
    for commit in &mut h {
        if let Command::Event(Event::Experience(e)) = &mut commit.command {
            let mut new = e.original.clone();
            new.appraisals[0].record_id.push_str("-new");
            new.appraisals[0].producer_version = "2".into();
            new.appraisals[0].appraisal = vector([-0.6, 0.1, -0.2, 0.3]);
            e.interpretations.insert("i2".into(), new);
        }
    }
    let original = h.clone();
    let mut new = scope();
    new.interpretation = "i2".into();
    let expected = replay(&kernel, "experiment", new.clone(), &h).unwrap();
    h.push(commit(
        6,
        Command::Rebuild {
            scope: new,
            reason: RebuildReason::Reinterpretation,
        },
    ));
    let result = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert!(kernel.same_bits(result.state(), expected.state()));
    assert_eq!(&h[..5], &original);
}

#[test]
fn ac2_20_21_unavailable_or_consistent_checks_alarm_and_apply() {
    let kernel = Legacy::new();
    let plain = history();
    let expected = replay(&kernel, "experiment", scope(), &plain).unwrap();
    for outcome in [
        CheckOutcome::Consistent,
        CheckOutcome::Unavailable(Unavailable::Missing),
        CheckOutcome::Unavailable(Unavailable::Failed),
        CheckOutcome::Unavailable(Unavailable::TimedOut),
    ] {
        let mut h = plain.clone();
        check(&mut h[4], scope(), outcome.clone());
        let out = replay(&kernel, "experiment", scope(), &h).unwrap();
        assert!(kernel.same_bits(out.state(), expected.state()));
        assert!(!out.paused());
        let alarm = out.audits().last().unwrap();
        assert!(alarm.notify_developer_and_caretaker);
        assert_eq!(alarm.check.as_ref().unwrap().outcome, outcome);
        for verdict in [ReviewVerdict::LawCorrect, ReviewVerdict::LawIncorrect] {
            let mut reviewed = h.clone();
            reviewed.push(commit(
                6,
                Command::Review {
                    alarm_id: "alarm-5".into(),
                    binding: alarm.binding.clone(),
                    verdict,
                },
            ));
            let review = replay(&kernel, "experiment", scope(), &reviewed).unwrap();
            assert!(kernel.same_bits(out.state(), review.state()));
            assert!(!review.paused());
        }
    }
}

#[test]
fn v2_97_mismatch_or_math_fault_retains_whole_state_and_repair_replays_h() {
    let kernel = Legacy::new();
    for outcome in [
        CheckOutcome::Inconsistent,
        CheckOutcome::ComputationFault("impossible-candidate".into()),
    ] {
        let mut h = history();
        let expected = replay(&kernel, "experiment", scope(), &h).unwrap();
        let before = replay(&kernel, "experiment", scope(), &h[..4]).unwrap();
        check(&mut h[4], scope(), outcome.clone());
        let rejected = replay(&kernel, "experiment", scope(), &h).unwrap();
        assert!(kernel.same_bits(rejected.state(), before.state()));
        assert!(!rejected.paused());
        assert!(rejected.needs_recompute());
        assert_eq!(
            rejected.audits().last().unwrap().kind,
            AuditKind::Rejected(outcome)
        );
        h.push(commit(
            6,
            Command::Rebuild {
                scope: scope(),
                reason: RebuildReason::Repair,
            },
        ));
        let repaired = replay(&kernel, "experiment", scope(), &h).unwrap();
        assert!(kernel.same_bits(repaired.state(), expected.state()));
        assert!(!repaired.needs_recompute());
    }
}

#[test]
fn immutable_prefix_check_bindings_and_missing_versions_fail_closed() {
    let kernel = Legacy::new();
    let h = history();
    let old = replay(&kernel, "experiment", scope(), &h[..4]).unwrap();
    let mut mutated = h.clone();
    mutated[0].id.push('!');
    assert!(matches!(
        step(&kernel, &old, &mutated),
        Err(Fault::History(_))
    ));
    for field in 0..3 {
        let mut stale = h.clone();
        check(&mut stale[4], scope(), CheckOutcome::Consistent);
        let bound = &mut stale[4].check.as_mut().unwrap().binding;
        match field {
            0 => bound.prefix.push('!'),
            1 => bound.lineage.push('!'),
            _ => bound.scope.law.push('!'),
        }
        assert!(matches!(
            step(&kernel, &old, &stale),
            Err(Fault::Binding(_))
        ));
    }
    let mut missing = h.clone();
    let mut new = scope();
    new.interpretation = "absent".into();
    missing.push(commit(
        6,
        Command::Rebuild {
            scope: new,
            reason: RebuildReason::Reinterpretation,
        },
    ));
    assert!(matches!(
        replay(&kernel, "experiment", scope(), &missing),
        Err(Fault::MissingInput(_))
    ));
    assert_eq!(old.state().theta.event_count(), 4);
}
