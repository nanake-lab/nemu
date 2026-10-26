use super::*;

#[test]
fn ac2_15_zero_delta_direct_event_map_is_nonexpansive() {
    let f = fixture();
    let mut rng = Rng(89);
    for trial in 0..10_000 {
        let e = effects_with(f.theta, trial % 4, &mut rng);
        let x = random_x(&e, &mut rng);
        let y = random_x(&e, &mut rng);
        let (ev, input) = random_event(&e, &mut rng);
        let drive = compute_drive(&e, &ev, &input, &f.law).unwrap();
        let mapped_x = apply_drive(&project_time(&x, &e, 0).unwrap(), &drive).unwrap();
        let mapped_y = apply_drive(&project_time(&y, &e, 0).unwrap(), &drive).unwrap();
        // No temporal contraction margin; only four binary64 rounding units
        // for translation/subtraction on the box. Existing tolerances unchanged.
        let bound = distance(&x, &y).unwrap() + 4.0 * f64::EPSILON;
        assert!(distance(&mapped_x, &mapped_y).unwrap() <= bound);
        let next = effects_with(f.theta, (trial + 1) % 4, &mut rng);
        assert!(
            distance(
                &f_out(&mapped_x, &next).unwrap(),
                &f_out(&mapped_y, &next).unwrap()
            )
            .unwrap()
                <= bound
        );
    }
}

#[test]
fn direct_event_map_has_independent_reference_with_saturation() {
    let e = effects_with(fixture().theta, 0, &mut Rng(97));
    let input = EventInput {
        appraisal: Some([0.5, -0.5, 0.25, -0.25]),
        empathy_weight: 1.0,
        class: EventClass::NewExogenous,
    };
    let evaluation = evaluate_effects(&e, &SituationMatch::default()).unwrap();
    let drive = compute_drive(&e, &evaluation, &input, &fixture().law).unwrap();
    let x = FastState::new([0.75, -0.75, 0.0, 0.0], vec![]).unwrap();
    let y = apply_drive(&project_time(&x, &e, 0).unwrap(), &drive).unwrap();
    assert_eq!(y.nature, [1.0, -1.0, 0.25, -0.25]);
}
