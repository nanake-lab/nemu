use super::*;

#[test]
fn d07_adaptive_extremes_stay_within_derived_bound() {
    // ADR 0035 §3.3/3.4, §9(1,6): changed b drives the existing variance,
    // duration and gain paths too. All twelve components must satisfy the bound.
    for b0 in [-1.0, 0.0, 1.0] {
        for p0 in [0.25, 2.0, 100.0] {
            let mut resource = law();
            resource.version = "synthetic-ADR0035-extremes".into();
            resource.baseline_prior = [b0; 4];
            resource.baseline_weight = p0;
            let mut s = DynamicsState::genesis_common_discount(resource).unwrap();
            for i in 1..=128 {
                let e = event(i, if i % 3 == 0 { -1.0 } else { 1.0 });
                let result = stage(&s, Some(&e), &s.law, &fuses()).unwrap();
                let bounds = result.bounds.as_ref().unwrap();
                bounds
                    .check(&components(&s.values), &components(result.state().values()))
                    .unwrap();
                assert!(bounds.range_only.iter().all(|v| !v));
                assert!(result.state().prior_mass.unwrap() <= s.prior_mass.unwrap());
                s = result.into_state();
            }
        }
    }
}

#[test]
fn d07_outward_bound_covers_subnormal_quotient_and_range_fallback() {
    // §3.2/3.4: the q/2 witness is valid even when the interval must widen
    // through zero. Huge P0 may prevent tightening; only the proof falls back.
    let mut resource = law();
    resource.version = "synthetic-ADR0035-boundary".into();
    resource.baseline_prior = [1.0; 4];
    resource.baseline_weight = 1.0;
    let mut s = DynamicsState::genesis_common_discount(resource).unwrap();
    s.prior_mass = Some(f64::from_bits(2));
    s.weight = 2.0;
    s.reaction_sum = [1.0; 4];
    s.values.baseline = nature([0.5; 4]);
    let e = event(1, -0.5);
    let next = update::statistics(&s, [-0.5; 4], e.time_ns, 0.5).unwrap();
    assert_eq!(next.values.baseline.connection.to_bits(), 0);
    let bounds = bounds::derive(&s, &e, 0.5).unwrap();
    bounds
        .check(&components(&s.values), &components(&next.values))
        .unwrap();
    let mut resource = s.law.clone();
    resource.baseline_weight = f64::MAX;
    resource.baseline_prior = [0.0; 4];
    let huge = DynamicsState::genesis_common_discount(resource).unwrap();
    let next = stage(&huge, Some(&event(1, 0.5)), &huge.law, &fuses()).unwrap();
    assert!(next.state().values.baseline.connection.is_subnormal());
    assert!(next.bounds.unwrap().range_only[..4].iter().all(|v| *v));
}
