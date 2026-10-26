use super::types::{components, nature};
use super::*;

fn law() -> DynamicsLaw {
    DynamicsLaw {
        version: "P5-25-v2-93-98-libm-0.2.16".into(),
        baseline_prior: [0.0; 4],
        gain_prior: [1.0; 4],
        sigma_prior: [0.3; 4],
        tau_prior_s: [10.0; 4],
        baseline_weight: 100.0,
        variance_weight: 10.0,
        duration_weight: 10.0,
        change_prior: 0.05,
        change_prior_weight: 10.0,
        change_min: 0.01,
        gain_min: 0.2,
        gain_max: 3.0,
        recovery_min: 0.001,
        recovery_max: 2.0,
        detector_shift_sigma: 1.0,
        detector_threshold: 8.0,
    }
}
fn fuses() -> DynamicsFuses {
    DynamicsFuses {
        jump_notice: 100.0,
        drift_run: 1_000_000,
        drift_margin: 0.01,
    }
}
fn event(i: u64, a: f64) -> ReactionEvent {
    ReactionEvent {
        id: format!("synthetic-{i}"),
        ordinal: i,
        time_ns: i * 1_000_000_000,
        running_time: RunningTime::from_history(0, i * 1_000_000_000, &[]).unwrap(),
        source: ReactionSource::OtherPersonReply,
        evaluation_unmodulated: true,
        reaction: nature([a; 4]),
    }
}
// PartialEq checks every field; Debug's round-trip float representation additionally
// distinguishes signed zeros, so this checks the entire finite state, not only theta.
fn identical(a: &DynamicsState, b: &DynamicsState) {
    assert_eq!(a, b);
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
}
fn advance(s: &DynamicsState, e: &ReactionEvent) -> DynamicsState {
    let staged = stage(s, Some(e), &s.law, &fuses()).unwrap();
    staged.into_state()
}

#[test]
fn ac2_14_i13_external_changes_and_all_self_sources_are_whole_state_identity() {
    let initial = DynamicsState::genesis(law()).unwrap();
    let changed = advance(&initial, &event(1, 0.7));
    assert_ne!(changed.values, initial.values); // kills identity-C implementation
    assert_eq!(changed.event_count, 1);
    identical(
        stage(&changed, None, &changed.law, &fuses())
            .unwrap()
            .state(),
        &changed,
    );
    for source in [
        ReactionSource::SelfAction,
        ReactionSource::TriggeredProgramResult,
        ReactionSource::SelfNarrative,
        ReactionSource::DeceptionInducedReply,
    ] {
        let mut e = event(2, -1.0);
        e.source = source;
        let result = stage(&changed, Some(&e), &changed.law, &fuses()).unwrap();
        assert!(result.change.is_none());
        identical(result.state(), &changed);
    }
    let mut world = event(2, -0.6);
    world.source = ReactionSource::ExternalWorld;
    assert_ne!(advance(&changed, &world), changed);
}

#[test]
fn ac2_05_replay_every_prefix_including_detector_open_runs_and_drift() {
    let initial = DynamicsState::genesis(law()).unwrap();
    let events: Vec<_> = (0..128)
        .map(|i| {
            let mut e = event(i, if i % 29 < 14 { 0.6 } else { -0.4 });
            if i % 5 == 0 {
                e.source = ReactionSource::SelfAction;
            }
            e
        })
        .collect();
    let mut incremental = initial.clone();
    for (end, e) in events.iter().enumerate() {
        incremental = advance(&incremental, e);
        let replay = events[..=end]
            .iter()
            .fold(initial.clone(), |s, e| advance(&s, e));
        identical(&incremental, &replay);
    }
}

#[test]
fn ac2_09_has_only_stored_unmodulated_reaction_and_stages_existing_idl() {
    let s = DynamicsState::genesis(law()).unwrap();
    let e = event(1, 0.8);
    let result = stage(&s, Some(&e), &s.law, &fuses()).unwrap();
    let change = result.change.unwrap();
    assert_eq!(change.before, s.values);
    assert_eq!(change.after, result.state.values);
    assert_eq!(change.events, vec![e.id.clone()]);
    let mut contaminated = e;
    contaminated.evaluation_unmodulated = false;
    assert_eq!(
        stage(&s, Some(&contaminated), &s.law, &fuses()),
        Err(Fault::InvalidEvent)
    );
    // There is no x/m/R/model argument or ambient read: provenance validation stays at ingress.
}

#[test]
fn variance_uses_baseline_before_event_and_duration_completes_same_sign_run() {
    let s = DynamicsState::genesis(law()).unwrap();
    let first = advance(&s, &event(0, 0.6));
    let expected_var = (10.0 * (0.3 * 0.3) + 0.6 * 0.6) / 11.0;
    assert_eq!(first.sigma[0].to_bits(), libm::sqrt(expected_var).to_bits());
    assert_eq!(first.tau, s.tau); // open run is right-censored
    let next = advance(&first, &event(5, -0.6));
    assert_eq!(next.duration_log_sum, [libm::log(5.0); 4]);
    assert_eq!(next.duration_mass, [1.0; 4]);
    assert_eq!(
        next.tau,
        [libm::exp((10.0 * libm::log(10.0) + libm::log(5.0)) / 11.0); 4]
    );
    assert_eq!(next.runs[0].start_ns, 5_000_000_000);
}

#[test]
fn gaussian_detector_uses_magnitude_not_reaction_sign_changes() {
    let p = law();
    let mut d = types::Detector::default();
    for i in 0..500 {
        d = detector::observe(&d, [if i % 2 == 0 { 0.05 } else { -0.05 }; 4], &p).unwrap();
    }
    assert_eq!(d.changes, 0);
    for _ in 0..20 {
        d = detector::observe(&d, [0.75; 4], &p).unwrap();
    }
    assert!(d.changes > 0);
    let mut old = types::Detector {
        segment_count: 1,
        ..Default::default()
    };
    old.mean = [0.4; 4];
    let observed = detector::observe(&old, [0.7; 4], &p).unwrap();
    // Independent Gaussian log-density ratio; tolerance only for this diagnostic
    // equivalence, never for a production decision or an acceptance inequality.
    let r: f64 = 0.7 - 0.4;
    let shift: f64 = 0.3;
    let var: f64 = 0.3 * 0.3;
    let log_ratio = -(r - shift).powi(2) / (2.0 * var) + r.powi(2) / (2.0 * var);
    assert!((observed.positive[0] - log_ratio).abs() < 1e-14);
}

#[test]
fn adaptive_h_respects_floor_and_ignores_wall_clock_and_excluded_events() {
    let p = law();
    let mut s = DynamicsState::genesis(p).unwrap();
    for i in 0..1000 {
        s = advance(&s, &event(i, if i % 2 == 0 { 0.05 } else { -0.05 }));
    }
    assert_eq!(s.change_count(), 0);
    assert_eq!(s.h.to_bits(), s.law.change_min.to_bits());
    let mut e = event(1000, 0.8);
    e.time_ns = u64::MAX;
    e.source = ReactionSource::SelfAction;
    identical(&s, stage(&s, Some(&e), &s.law, &fuses()).unwrap().state());
    assert!(s.weight <= 1.0 / s.law.change_min);
}

#[test]
fn jump_and_directional_drift_apply_and_stage_notices_before_boundary() {
    let s = DynamicsState::genesis(law()).unwrap();
    let e = event(1, 1.0);
    let proposed = advance(&s, &e);
    let largest = components(&proposed.values)
        .iter()
        .zip(components(&s.values))
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    let mut f = fuses();
    f.jump_notice = largest;
    let equal = stage(&s, Some(&e), &s.law, &f).unwrap();
    assert!(!equal.signals.contains(&ReviewSignal::Jump));
    identical(equal.state(), &proposed);
    f.jump_notice = largest / 2.0;
    let notice = stage(&s, Some(&e), &s.law, &f).unwrap();
    assert!(notice.signals.contains(&ReviewSignal::Jump));
    assert!(notice.change.is_some());
    identical(notice.state(), &proposed);
    let mut p = law();
    p.baseline_weight = 1.0;
    let mut s = DynamicsState::genesis(p).unwrap();
    f = fuses();
    f.drift_run = 5;
    f.drift_margin = 0.2;
    let mut seen = false;
    for i in 0..200 {
        let staged = stage(&s, Some(&event(i, 1.0)), &s.law, &f).unwrap();
        if !staged.signals.is_empty() {
            assert!(staged.signals.contains(&ReviewSignal::SlowDrift {
                component: 0,
                direction: 1
            }));
            assert!(staged.state().values.baseline.connection < 1.0);
            assert_ne!(staged.state(), &s);
            assert!(staged.change.is_some());
            seen = true;
            break;
        }
        s = staged.into_state();
    }
    assert!(seen);
}

#[test]
fn numerical_and_ingress_faults_do_not_publish_partial_state() {
    let s = DynamicsState::genesis(law()).unwrap();
    for a in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            stage(&s, Some(&event(1, a)), &s.law, &fuses()),
            Err(Fault::NonFinite)
        );
    }
    assert_eq!(
        stage(&s, Some(&event(1, 1.1)), &s.law, &fuses()),
        Err(Fault::OutOfRange)
    );
    let updated = advance(&s, &event(1, 0.5));
    assert_eq!(
        stage(&updated, Some(&event(1, -0.5)), &updated.law, &fuses()),
        Err(Fault::EventOrder)
    );
    let mut p = law();
    p.baseline_weight = 101.0;
    assert_eq!(
        stage(&s, Some(&event(1, 0.5)), &p, &fuses()),
        Err(Fault::LawMismatch)
    );
    assert_eq!(
        numeric::mul(f64::MIN_POSITIVE, f64::MIN_POSITIVE),
        Err(Fault::Underflow)
    );
    assert_eq!(numeric::count(1 << 53), Err(Fault::CounterExhausted));
    assert_eq!(numeric::div(1.0, 0.0), Err(Fault::ZeroDenominator));
    assert_eq!(numeric::finite(-0.0).unwrap().to_bits(), 0.0f64.to_bits());
    let zero = advance(&s, &event(1, -0.0));
    assert_eq!(zero.values.baseline.connection.to_bits(), 0.0f64.to_bits());
    let mut long = event(2, -0.5);
    long.time_ns = (1 << 53) + 1_000_000_000;
    long.running_time = RunningTime::from_history(0, long.time_ns, &[]).unwrap();
    assert_eq!(
        stage(&updated, Some(&long), &updated.law, &fuses()),
        Err(Fault::TimePrecision)
    );
    identical(&s, &DynamicsState::genesis(law()).unwrap());
}

mod batch4;
mod binary64;
mod reference;
