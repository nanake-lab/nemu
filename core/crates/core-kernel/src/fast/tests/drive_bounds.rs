use super::*;

fn event() -> EventInput {
    EventInput {
        appraisal: None,
        empathy_weight: 1.0,
        class: EventClass::NewExogenous,
    }
}

#[test]
fn f02_inconsistent_sum_is_rejected_even_when_within_bound() {
    let f = fixture();
    let mut rng = Rng(61);
    for n in [0, 2] {
        let e = effects_with(f.theta, n, &mut rng);
        let ev = evaluate_effects(&e, &SituationMatch::default()).unwrap();
        assert!(compute_drive(&e, &ev, &event(), &f.law).is_ok());
        for value in [100.0, 0.125, f64::from_bits(1)] {
            let mut bad = ev.clone();
            bad.f_sum[0] = value;
            assert_eq!(
                compute_drive(&e, &bad, &event(), &f.law),
                Err(FastError::InconsistentSum { channel: 0 })
            );
        }
    }
}

#[test]
fn f02_learned_out_of_range_or_nonfinite_is_rejected_before_weighting() {
    let f = fixture();
    let e = effects_with(f.theta, 1, &mut Rng(67));
    let ev = evaluate_effects(&e, &SituationMatch::default()).unwrap();
    for class in [
        EventClass::NewExogenous,
        EventClass::InternalRecord { n: u64::MAX },
    ] {
        let input = EventInput { class, ..event() };
        for value in [
            -f64::from_bits(1),
            -1.0,
            1.0 + f64::EPSILON,
            100.0,
            f64::MAX,
        ] {
            let mut bad = ev.clone();
            bad.drives[0].drive = value;
            assert_eq!(
                compute_drive(&e, &bad, &input, &f.law),
                Err(FastError::DriveOverBound { channel: 4 })
            );
        }
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut bad = ev.clone();
            bad.drives[0].drive = value;
            assert_eq!(
                compute_drive(&e, &bad, &input, &f.law),
                Err(FastError::NonFinite("learned_drive"))
            );
        }
    }
}

#[test]
fn f02_nonfinite_eta_contributions_and_sum_are_rejected() {
    let f = fixture();
    let e = effects_with(f.theta, 2, &mut Rng(71));
    let ev = evaluate_effects(&e, &SituationMatch::default()).unwrap();
    let huge = DriveLaw {
        g_max: f64::MAX,
        ..f.law
    };
    assert_eq!(
        compute_drive(&e, &ev, &event(), &huge),
        Err(FastError::NonFinite("eta"))
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut bad = ev.clone();
        bad.drives[0].f[0] = value;
        assert_eq!(
            compute_drive(&e, &bad, &event(), &f.law),
            Err(FastError::NonFinite("contribution"))
        );
        bad = ev.clone();
        bad.f_sum[0] = value;
        assert_eq!(
            compute_drive(&e, &bad, &event(), &f.law),
            Err(FastError::NonFinite("f_sum"))
        );
    }
    // Cancellation and even a zero class weight cannot launder oversized terms.
    let mut bad = ev.clone();
    bad.drives[0].f[0] = f64::MAX;
    bad.drives[1].f[0] = -f64::MAX;
    assert_eq!(
        compute_drive(
            &e,
            &bad,
            &EventInput {
                class: EventClass::InternalRecord { n: u64::MAX },
                ..event()
            },
            &f.law
        ),
        Err(FastError::DriveOverBound { channel: 0 })
    );
}

#[test]
fn f02_contributions_are_summed_in_canonical_order() {
    let f = fixture();
    let e = effects_with(f.theta, 3, &mut Rng(73));
    let mut ev = evaluate_effects(&e, &SituationMatch::default()).unwrap();
    for (d, value) in ev.drives.iter_mut().zip([1.0, -1.0, 1e-17]) {
        d.f[0] = value;
    }
    ev.f_sum[0] = 1e-17;
    let d = compute_drive(&e, &ev, &event(), &f.law).unwrap();
    assert_eq!(d.nature()[0].to_bits(), 1e-17_f64.to_bits());
    assert!(d.learned().iter().all(|v| (0.0..=1.0).contains(&v.value)));
    let mut unordered = e.clone();
    unordered.effects.swap(0, 1);
    ev.drives.swap(0, 1);
    assert_eq!(
        compute_drive(&unordered, &ev, &event(), &f.law),
        Err(FastError::Misaligned)
    );
}
