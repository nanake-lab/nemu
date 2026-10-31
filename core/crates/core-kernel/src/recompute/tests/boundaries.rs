use super::*;
use crate::dynamics::{Downtime, RunningTime};

#[test]
fn account_merge_rebuilds_groups_instead_of_editing_cached_counts() {
    let mut kernel = Legacy::new();
    for i in 1..=4 {
        kernel
            .source_aliases
            .get_mut("s2")
            .unwrap()
            .insert(format!("person-{i}"), "canonical-person".into());
    }
    let mut h: Vec<_> = (1..=4)
        .map(|i| {
            commit(
                i,
                Command::Event(experience(i, [0.6, -0.2, 0.3, 0.1], i == 1)),
            )
        })
        .collect();
    let original = h.clone();
    let before = replay(&kernel, "experiment", scope(), &h).unwrap();
    let mut merged = scope();
    merged.sources = "s2".into();
    let expected = replay(&kernel, "experiment", merged.clone(), &h).unwrap();
    h.push(commit(
        5,
        Command::Rebuild {
            scope: merged,
            reason: RebuildReason::SourceCorrection,
        },
    ));
    let result = step(&kernel, &before, &h).unwrap();
    assert!(kernel.same_bits(result.state(), expected.state()));
    assert_eq!(before.state().r.tendencies[0].group_count.0, 4);
    assert_eq!(result.state().r.tendencies[0].group_count.0, 1);
    assert_eq!(before.state().theta, result.state().theta);
    assert_eq!(&h[..4], &original);
}

#[test]
fn paused_source_and_law_changes_wait_for_explicit_resume() {
    let kernel = Legacy::new();
    let mut e = experience(1, [0.6, 0.0, 0.0, 0.0], true);
    if let Event::Experience(x) = &mut e {
        x.excluded_from.insert("s2".into());
    }
    let mut h = vec![commit(1, Command::Event(e)), commit(2, Command::Pause)];
    let frozen = replay(&kernel, "experiment", scope(), &h).unwrap();
    let mut new = scope();
    new.sources = "s2".into();
    h.push(commit(
        3,
        Command::Rebuild {
            scope: new.clone(),
            reason: RebuildReason::SourceCorrection,
        },
    ));
    new.law = "old-v2-test".into();
    h.push(commit(
        4,
        Command::Rebuild {
            scope: new.clone(),
            reason: RebuildReason::LawVersion,
        },
    ));
    h.push(commit(
        5,
        Command::Event(experience(5, [-0.6, 0.0, 0.0, 0.0], true)),
    ));
    let waiting = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert!(waiting.paused());
    assert!(waiting.needs_recompute());
    assert_eq!(waiting.target_scope(), &new);
    assert_eq!(waiting.applied_scope(), &scope());
    assert!(kernel.same_bits(waiting.state(), frozen.state()));
    h.push(commit(6, Command::Resume));
    let resumed = step(&kernel, &waiting, &h).unwrap();
    let expected = replay(&kernel, "experiment", new, &h[4..5]).unwrap();
    assert!(kernel.same_bits(resumed.state(), expected.state()));
    assert!(!resumed.paused());
    assert!(!resumed.needs_recompute());
}

#[test]
fn recorded_downtime_is_replayed_without_reading_a_clock() {
    let kernel = Legacy::new();
    let mut h = Vec::new();
    for i in [1, 5, 6, 7] {
        let mut e = experience(i, [if i < 6 { 0.6 } else { -0.4 }, 0.0, 0.0, 0.0], i == 1);
        if i >= 5
            && let Event::Experience(x) = &mut e
        {
            x.running = RunningTime::from_history(
                0,
                i * 1_000_000_000,
                &[Downtime {
                    start_ns: 1_000_000_000,
                    end_ns: 4_000_000_000,
                }],
            )
            .unwrap();
        }
        h.push(commit(i, Command::Event(e)));
    }
    let full = replay(&kernel, "experiment", scope(), &h).unwrap();
    let mut inc = start(&kernel, "experiment", scope()).unwrap();
    for end in 1..=h.len() {
        inc = step(&kernel, &inc, &h[..end]).unwrap();
    }
    assert!(kernel.same_bits(full.state(), inc.state()));
    // A different recorded stop history is a genuinely different C input.
    let mut no_stops = h.clone();
    for row in &mut no_stops {
        if let Command::Event(Event::Experience(x)) = &mut row.command {
            x.running = RunningTime::from_history(0, row.position * 1_000_000_000, &[]).unwrap();
        }
    }
    let other = replay(&kernel, "experiment", scope(), &no_stops).unwrap();
    assert_ne!(full.state().theta, other.state().theta);
}

// Fault injection only, not a surrogate G/C or an independent implementation.
struct FailingKernel {
    broken: bool,
}
impl Kernel<u64> for FailingKernel {
    type State = [u64; 2];
    type Input = u64;
    fn same_event(&self, a: &u64, b: &u64) -> bool {
        a == b
    }
    fn same_input(&self, a: &u64, b: &u64) -> bool {
        a == b
    }
    fn genesis(&self, _: &Scope) -> Result<Self::State> {
        Ok([0, 0])
    }
    fn project(&self, h: &[Commit<u64>], _: &Scope) -> Result<Vec<u64>> {
        Ok(h.iter()
            .filter_map(|r| match r.command {
                Command::Event(n) => Some(n),
                _ => None,
            })
            .collect())
    }
    fn advance(&self, old: &Self::State, n: &u64, _: &Scope) -> Result<Self::State> {
        let mut staged = *old;
        staged[0] += n;
        if self.broken && *n == 2 {
            return Err(Fault::Computation("injected-fault".into()));
        }
        staged[1] += 1;
        Ok(staged)
    }
    fn same_bits(&self, a: &Self::State, b: &Self::State) -> bool {
        a == b
    }
}

#[test]
fn local_fault_requires_record_and_never_leaks_part_of_candidate() {
    let broken = FailingKernel { broken: true };
    let mut h = vec![commit(1, Command::Event(1))];
    let old = replay(&broken, "experiment", scope(), &h).unwrap();
    h.push(commit(2, Command::Event(2)));
    assert!(matches!(
        step(&broken, &old, &h),
        Err(Fault::Computation(_))
    ));
    assert_eq!(old.state(), &[1, 1]);
    h[1].check = Some(CheckRecord {
        alarm_id: "fault-2".into(),
        binding: Binding {
            lineage: "experiment".into(),
            prefix: h[1].prefix.clone(),
            scope: scope(),
        },
        triggers: vec![Trigger::ComputationFault],
        outcome: CheckOutcome::ComputationFault("injected-fault".into()),
    });
    let rejected = step(&broken, &old, &h).unwrap();
    assert_eq!(rejected.state(), &[1, 1]);
    assert!(rejected.needs_recompute());
    assert!(!rejected.paused());
    // R1: even with the implementation still broken, a later event is retained
    // without rerunning the known-bad recurrence or requiring another check.
    h.push(commit(3, Command::Event(3)));
    let held = step(&broken, &rejected, &h).unwrap();
    assert_eq!(held.state(), rejected.state());
    let fixed = FailingKernel { broken: false };
    // Structural replay honors the historical rejection even with a fixed implementation.
    let historical = replay(&fixed, "experiment", scope(), &h).unwrap();
    assert_eq!(historical.state(), rejected.state());
    h.push(commit(
        4,
        Command::Rebuild {
            scope: scope(),
            reason: RebuildReason::Repair {
                alarm_id: "fault-2".into(),
                binding: h[1].check.as_ref().unwrap().binding.clone(),
            },
        },
    ));
    let repaired = step(&fixed, &historical, &h).unwrap();
    assert_eq!(repaired.state(), &[6, 3]);
    assert!(!repaired.needs_recompute());
}

#[test]
fn review_cannot_target_an_unrecorded_alarm_or_other_prefix() {
    let kernel = FailingKernel { broken: false };
    let mut h = vec![commit(1, Command::Event(1))];
    let old = replay(&kernel, "experiment", scope(), &h).unwrap();
    h.push(commit(
        2,
        Command::Review {
            alarm_id: "unknown".into(),
            binding: Binding {
                lineage: "experiment".into(),
                prefix: "prefix-1".into(),
                scope: scope(),
            },
            verdict: ReviewVerdict::LawIncorrect,
        },
    ));
    assert!(matches!(step(&kernel, &old, &h), Err(Fault::Binding(_))));
    assert_eq!(old.state(), &[1, 1]);
}
