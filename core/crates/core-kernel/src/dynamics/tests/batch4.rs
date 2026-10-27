use super::*;

#[test]
fn v2_95_recorded_downtime_is_subtracted_and_replay_is_bitwise() {
    let initial = DynamicsState::genesis(law()).unwrap();
    let first = event(0, 0.8);
    let mut close = event(120, -0.8);
    close.running_time = RunningTime::from_history(
        0,
        close.time_ns,
        &[Downtime {
            start_ns: 10_000_000_000,
            end_ns: 110_000_000_000,
        }],
    )
    .unwrap();
    assert_eq!(close.running_time.running_ns(), 20_000_000_000);
    let opened = advance(&initial, &first);
    let finished = advance(&opened, &close);
    assert_eq!(finished.duration_log_sum, [libm::log(20.0); 4]);
    assert_eq!(finished.duration_mass, [1.0; 4]);
    identical(&finished, &advance(&advance(&initial, &first), &close));
    // Same reactions and running times, with no stop, give identical estimators.
    let no_stop = advance(&opened, &event(20, -0.8));
    assert_eq!(finished.tau, no_stop.tau);
    assert_eq!(finished.values, no_stop.values);
}

#[test]
fn v2_95_rejects_invalid_or_inconsistent_recorded_time() {
    for stops in [
        vec![Downtime {
            start_ns: 4,
            end_ns: 3,
        }],
        vec![Downtime {
            start_ns: 1,
            end_ns: 11,
        }],
        vec![
            Downtime {
                start_ns: 1,
                end_ns: 5,
            },
            Downtime {
                start_ns: 4,
                end_ns: 6,
            },
        ],
    ] {
        assert_eq!(
            RunningTime::from_history(0, 10, &stops),
            Err(Fault::InvalidRunningTime)
        );
    }
    let mut start = event(10, 0.8);
    start.running_time = RunningTime::from_history(
        0,
        start.time_ns,
        &[Downtime {
            start_ns: 0,
            end_ns: 5_000_000_000,
        }],
    )
    .unwrap();
    let old = advance(&DynamicsState::genesis(law()).unwrap(), &start);
    let snapshot = old.clone();
    // A later projection cannot silently forget previously counted downtime.
    assert_eq!(
        stage(&old, Some(&event(11, -0.8)), &old.law, &fuses()),
        Err(Fault::InvalidRunningTime)
    );
    let mut wrong = event(11, -0.8);
    wrong.time_ns += 1;
    assert_eq!(
        stage(&old, Some(&wrong), &old.law, &fuses()),
        Err(Fault::InvalidRunningTime)
    );
    identical(&old, &snapshot);
}

#[test]
fn v2_95_zero_duration_counts_separately_and_open_run_is_censored() {
    let initial = DynamicsState::genesis(law()).unwrap();
    let mut old = advance(&initial, &event(0, 0.8));
    let mut zero = event(1, -0.8);
    zero.time_ns = 0;
    zero.running_time = RunningTime::from_history(0, 0, &[]).unwrap();
    old = advance(&old, &zero);
    assert_eq!(old.zero_duration_count(), [1; 4]);
    assert_eq!(old.duration_mass, [0.0; 4]);
    assert_eq!(old.duration_log_sum, [0.0; 4]);
    for i in 2..102 {
        old = advance(&old, &event(i, -0.8));
    }
    assert_eq!(old.duration_mass, [0.0; 4]);
    assert_eq!(old.tau, initial.tau);
    assert_eq!(old.zero_duration_count(), [1; 4]);
}

#[test]
fn v2_95_geometric_mean_uses_each_external_event_retention() {
    let initial = DynamicsState::genesis(law()).unwrap();
    let a = update::statistics(&initial, [0.8; 4], 0, 0.1).unwrap();
    let b = update::statistics(&a, [-0.8; 4], 10_000_000_000, 0.1).unwrap();
    let c = update::statistics(&b, [-0.8; 4], 15_000_000_000, 0.2).unwrap();
    let d = update::statistics(&c, [0.8; 4], 30_000_000_000, 0.3).unwrap();
    let sum = 0.7 * (0.8 * libm::log(10.0)) + libm::log(20.0);
    let mass = 0.7 * (0.8 * 1.0) + 1.0;
    let tau = libm::exp((10.0 * libm::log(10.0) + sum) / (10.0 + mass));
    assert_eq!(d.duration_log_sum, [sum; 4]);
    assert_eq!(d.duration_mass, [mass; 4]);
    assert_eq!(d.tau, [tau; 4]);
    // A 1 ns observation is valid; zero is never replaced by epsilon.
    let tiny = update::statistics(&a, [-0.8; 4], 1, 0.1).unwrap();
    assert_eq!(tiny.duration_log_sum, [libm::log(1.0 / 1_000_000_000.0); 4]);
}

#[test]
fn v2_96_prior_decay_changes_baseline_even_when_reaction_equals_current() {
    let mut p = law();
    p.baseline_prior = [0.6; 4];
    let initial = DynamicsState::genesis(p).unwrap();
    let old = advance(&initial, &event(0, -0.5));
    let e = event(1, old.values.baseline.connection);
    let result = stage(&old, Some(&e), &old.law, &fuses()).unwrap();
    let delta = result.state().values.baseline.connection - old.values.baseline.connection;
    assert!(delta > 0.0);
    assert!(delta <= result.bounds.as_ref().unwrap().single_event[0]);
    println!(
        "V2-96 prior-decay: delta={delta:e} bound={:e}",
        result.bounds.unwrap().single_event[0]
    );
}

#[test]
fn v2_96_equality_is_allowed_and_strictly_above_bound_is_a_fault() {
    let old = DynamicsState::genesis(law()).unwrap();
    let e = event(0, 0.8);
    let staged = stage(&old, Some(&e), &old.law, &fuses()).unwrap();
    let bounds = staged.bounds.unwrap();
    let limits = bounds.limits();
    bounds.check(&[0.0; 12], &limits).unwrap();
    for i in 0..12 {
        let mut beyond = limits;
        beyond[i] = beyond[i].next_up();
        assert_eq!(
            bounds.check(&[0.0; 12], &beyond),
            Err(Fault::ComputationBound { component: i })
        );
    }
}

#[test]
fn v2_97_computation_fault_rejects_whole_candidate_without_clipping() {
    let old = DynamicsState::genesis(law()).unwrap();
    let snapshot = old.clone();
    let e = event(0, 0.8);
    let next = stage(&old, Some(&e), &old.law, &fuses())
        .unwrap()
        .into_state();
    let mut faulty = next.clone();
    faulty.values.baseline.connection = 0.5; // in range, but impossible for this step
    assert_eq!(
        update::finish(&old, faulty, &e, &fuses()),
        Err(Fault::ComputationBound { component: 0 })
    );
    let mut faulty = next;
    faulty.values.gain.curiosity = f64::NAN;
    assert_eq!(
        update::finish(&old, faulty, &e, &fuses()),
        Err(Fault::NonFinite)
    );
    identical(&old, &snapshot);
}

#[test]
fn v2_96_legal_extremes_and_changing_retention_stay_inside_derived_bounds() {
    for prior in [-1.0, 0.0, 1.0] {
        let mut p = law();
        p.baseline_prior = [prior; 4];
        let mut s = DynamicsState::genesis(p).unwrap();
        for i in 0..500 {
            let e = event(i, if i % 3 == 0 { -1.0 } else { 1.0 });
            let result = stage(&s, Some(&e), &s.law, &fuses()).unwrap();
            assert!(
                result
                    .bounds
                    .as_ref()
                    .unwrap()
                    .range_only
                    .iter()
                    .all(|x| !x)
            );
            result
                .bounds
                .as_ref()
                .unwrap()
                .check(&components(&s.values), &components(result.state().values()))
                .unwrap();
            s = result.into_state();
        }
    }
}

#[test]
fn v2_96_legal_subnormal_variance_uses_proven_range_not_a_false_fault() {
    let mut p = law();
    p.sigma_prior = [libm::sqrt(f64::from_bits(1)); 4];
    let old = DynamicsState::genesis(p).unwrap();
    let e = event(0, 0.0);
    // The specified recurrence is finite. An outward enclosure can nevertheless
    // straddle zero, or underflow in the inverse-square-root derivative's product.
    let expected = update::statistics(&old, [0.0; 4], 0, old.h).unwrap();
    let result = stage(&old, Some(&e), &old.law, &fuses()).unwrap();
    assert_eq!(result.state().values, expected.values);
    let bounds = result.bounds.unwrap();
    assert!(bounds.range_only[4..8].iter().all(|x| *x));
    for i in 4..8 {
        assert_eq!(bounds.single_event[i], bounds.range[i]);
    }
    let mut impossible = result.state.values.clone();
    impossible.gain.connection = old.values.gain.connection + bounds.range[4].next_up();
    assert!(
        bounds
            .check(&components(&old.values), &components(&impossible))
            .is_err()
    );
}
