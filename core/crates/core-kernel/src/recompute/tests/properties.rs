use super::*;

#[test]
fn ac2_08_i23_random_histories_every_prefix_zero_bit_differences() {
    let kernel = Legacy::new();
    let mut comparisons = 0;
    for seed in 1..=16_u64 {
        let mut rng = seed;
        let mut h = vec![];
        let mut current = scope();
        let mut incremental = start(&kernel, "experiment", scope()).unwrap();
        for i in 1..=24 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let command = match i {
                5 | 17 => Command::Pause,
                8 | 20 => Command::Resume,
                10 => {
                    current.law = "old-v2-test".into();
                    Command::Rebuild {
                        scope: current.clone(),
                        reason: RebuildReason::LawVersion,
                    }
                }
                13 => {
                    current.sources = "s2".into();
                    Command::Rebuild {
                        scope: current.clone(),
                        reason: RebuildReason::SourceCorrection,
                    }
                }
                16 => Command::Review {
                    alarm_id: format!("alarm-{seed}"),
                    binding: Binding {
                        lineage: "experiment".into(),
                        prefix: "prefix-15".into(),
                        scope: current.clone(),
                    },
                    verdict: if seed % 2 == 0 {
                        ReviewVerdict::LawCorrect
                    } else {
                        ReviewVerdict::LawIncorrect
                    },
                },
                _ => {
                    let a = std::array::from_fn(|c| {
                        (((rng >> (c * 12)) & 1023) as f64 / 1024.0) * 1.5 - 0.75
                    });
                    let mut e = experience(i, a, i == 1);
                    if i == 3
                        && let Event::Experience(x) = &mut e
                    {
                        x.excluded_from.insert("s2".into());
                    }
                    Command::Event(e)
                }
            };
            h.push(commit(i, command));
            if i == 15 {
                let binding = Binding {
                    lineage: "experiment".into(),
                    prefix: h.last().unwrap().prefix.clone(),
                    scope: current.clone(),
                };
                h.last_mut().unwrap().check = Some(CheckRecord {
                    alarm_id: format!("alarm-{seed}"),
                    binding,
                    triggers: vec![Trigger::SlowDrift {
                        component: 0,
                        direction: 1,
                    }],
                    outcome: match seed % 4 {
                        0 => CheckOutcome::Consistent,
                        1 => CheckOutcome::Unavailable(Unavailable::TimedOut),
                        2 => CheckOutcome::Inconsistent,
                        _ => CheckOutcome::ComputationFault("recorded-fault".into()),
                    },
                });
            }
            incremental = step(&kernel, &incremental, &h).unwrap();
            let full = replay(&kernel, "experiment", scope(), &h).unwrap();
            assert!(
                kernel.same_bits(incremental.state(), full.state()),
                "seed={seed} prefix={i}"
            );
            assert_eq!(incremental.paused(), full.paused());
            assert_eq!(incremental.needs_recompute(), full.needs_recompute());
            assert_eq!(incremental.unresolved_fault(), full.unresolved_fault());
            assert_eq!(incremental.applied_scope(), full.applied_scope());
            assert_eq!(incremental.target_scope(), full.target_scope());
            assert_eq!(incremental.audits().len(), full.audits().len());
            for (a, b) in incremental.audits().iter().zip(full.audits()) {
                assert_eq!(a.binding, b.binding);
                assert_eq!(a.kind, b.kind);
                assert_eq!(a.check, b.check);
                assert_eq!(a.unresolved_fault, b.unresolved_fault);
                assert_eq!(a.changed, b.changed);
            }
            comparisons += 1;
        }
    }
    println!(
        "AC2-08/I23: {comparisons} complete-state prefix comparisons; 0 differences; fixed seeds 1..=16"
    );
    assert_eq!(comparisons, 384);
}

#[test]
fn ac2_09_real_g_c_inputs_cannot_read_fast_state_or_modulation() {
    let mut kernel = Legacy::new();
    let h: Vec<_> = (1..=12)
        .map(|i| {
            commit(
                i,
                Command::Event(experience(
                    i,
                    [if i % 2 == 0 { 0.8 } else { -0.4 }, 0.2, -0.1, 0.3],
                    i == 1,
                )),
            )
        })
        .collect();
    let original = replay(&kernel, "experiment", scope(), &h).unwrap();
    // Change actual C's g/lambda priors/ranges: b/sigma and the G inputs must
    // remain unchanged. x/m are absent from BOTH public kernel signatures.
    let resource = kernel.versions.get_mut("old-v1").unwrap();
    resource.c.gain_prior = [0.7, 1.5, 2.0, 0.4];
    resource.c.recovery_min = 0.03;
    resource.c.recovery_max = 0.04;
    let changed = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert_eq!(original.state().r, changed.state().r);
    assert_eq!(original.state().groups, changed.state().groups);
    assert_ne!(
        original.state().theta.values().gain,
        changed.state().theta.values().gain
    );
    assert_ne!(
        original.state().theta.values().recovery_per_s,
        changed.state().theta.values().recovery_per_s
    );
}

#[test]
fn capacity_zero_does_not_pause_c_and_reproposal_recovers_h_evidence() {
    let kernel = Legacy::new();
    let mut h = vec![commit(
        1,
        Command::Event(Event::Capacity {
            capacity: 0,
            protected: 1,
        }),
    )];
    for i in 2..=4 {
        h.push(commit(
            i,
            Command::Event(experience(i, [0.8, 0.0, 0.0, 0.0], i == 2)),
        ));
    }
    let saturated = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert!(saturated.state().r.tendencies.is_empty());
    assert_eq!(saturated.state().theta.event_count(), 3);
    assert!(!saturated.paused());
    h.push(commit(
        5,
        Command::Event(Event::Capacity {
            capacity: 2,
            protected: 1,
        }),
    ));
    let mut again = experience(6, [0.8, 0.0, 0.0, 0.0], true);
    if let Event::Experience(e) = &mut again {
        e.interpretations.get_mut("i1").unwrap().candidates[0]
            .record
            .record_id = "k-reproposal".into();
    }
    h.push(commit(6, Command::Event(again)));
    let admitted = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert_eq!(admitted.state().r.tendencies.len(), 1);
    assert_eq!(admitted.state().r.tendencies[0].group_count.0, 4);
    let mut inc = start(&kernel, "experiment", scope()).unwrap();
    for end in 1..=h.len() {
        inc = step(&kernel, &inc, &h[..end]).unwrap();
    }
    assert!(kernel.same_bits(admitted.state(), inc.state()));
}

#[test]
fn independent_comparison_binds_all_resources_and_does_not_mislabel_absence() {
    let kernel = Legacy::new();
    let state = kernel.genesis(&scope()).unwrap();
    let binding = Binding {
        lineage: "experiment".into(),
        prefix: "h0".into(),
        scope: scope(),
    };
    assert_eq!(
        verify_candidate(&kernel, &binding, "production", &state, None).unwrap(),
        CheckOutcome::Unavailable(Unavailable::Missing)
    );
    let mut other = IndependentResult {
        binding: binding.clone(),
        implementation: "test-only-second".into(),
        result: Ok(state.clone()),
    };
    assert_eq!(
        verify_candidate(&kernel, &binding, "production", &state, Some(&other)).unwrap(),
        CheckOutcome::Consistent
    );
    // Test plumbing only: this value is not evidence of numerical independence.
    other.binding.prefix = "wrong".into();
    assert_eq!(
        verify_candidate(&kernel, &binding, "production", &state, Some(&other)).unwrap(),
        CheckOutcome::Unavailable(Unavailable::Failed)
    );
    other.binding = binding.clone();
    other.implementation = "production".into();
    assert_eq!(
        verify_candidate(&kernel, &binding, "production", &state, Some(&other)).unwrap(),
        CheckOutcome::Unavailable(Unavailable::Failed)
    );
    other.implementation = "test-only-second".into();
    let mut corrupted = state.clone();
    corrupted.r.revision = "corrupt".into();
    other.result = Ok(corrupted);
    assert_eq!(
        verify_candidate(&kernel, &binding, "production", &state, Some(&other)).unwrap(),
        CheckOutcome::Inconsistent
    );
    for failure in [Unavailable::Failed, Unavailable::TimedOut] {
        other.result = Err(failure);
        assert_eq!(
            verify_candidate(&kernel, &binding, "production", &state, Some(&other)).unwrap(),
            CheckOutcome::Unavailable(failure)
        );
    }
}
