use super::*;

#[test]
fn j06_budget_rounding_matches_independent_exact_rational_vectors() {
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("bound-vectors.json")).unwrap();
    let vectors = vectors.as_array().unwrap();
    assert_eq!(vectors.len(), 308);
    let read = |v: &serde_json::Value| {
        f64::from_bits(u64::from_str_radix(v.as_str().unwrap(), 16).unwrap())
    };
    for vector in vectors {
        let left = read(&vector["left"]);
        let right = read(&vector["right"]);
        for (a, b) in [(left, right), (right, left)] {
            let result = super::super::numeric::add_bound(a, b);
            if vector["upper"].is_null() {
                assert!(result.is_err(), "{vector}");
            } else {
                assert_eq!(
                    result.unwrap().to_bits(),
                    read(&vector["upper"]).to_bits(),
                    "{vector}"
                );
            }
        }
    }
}

#[test]
fn j06_cancellation_and_zero_state_do_not_evade_configuration_budget() {
    let config = CarrierConfig::new(
        resources(),
        nature(),
        vec![learned("a"), learned("b")],
        binding(2.0),
    )
    .unwrap();
    let r = StoredR {
        tendencies: vec![tendency("a"), tendency("b")],
    };
    for values in [[1.0, -1.0], [0.0, 0.0]] {
        let x = state([0.0; 4], &[("a", values[0]), ("b", values[1])]);
        let result = compile_numeric(
            JStateView {
                snapshot_id: "over-budget",
                anchor_time: ProcessTime(0),
                theta: &theta(),
                x: &x,
                r: &r,
                process_time: ProcessTime(0),
                validity: &Validity,
            },
            &config,
            &body(),
        );
        assert_eq!(result, Err(JFault::TotalBudgetExceeded));
    }
    // Removing the extra term restores the earlier, unscaled coefficient.
    let x = state([0.0; 4], &[("a", 1.0)]);
    let r = StoredR {
        tendencies: vec![tendency("a")],
    };
    let result = compile_numeric(
        JStateView {
            snapshot_id: "within-budget",
            anchor_time: ProcessTime(0),
            theta: &theta(),
            x: &x,
            r: &r,
            process_time: ProcessTime(0),
            validity: &Validity,
        },
        &config,
        &body(),
    )
    .unwrap();
    assert_eq!(result.total_bound, 2.0);
    assert_eq!(result.learned_coefficients[0].coefficient, 0.125);
}

#[test]
fn j06_outward_sum_catches_sub_ulp_over_budget_without_inflating_exact_sum() {
    assert_eq!(super::super::numeric::add_bound(0.5, 0.5).unwrap(), 1.0);
    assert_eq!(
        super::super::numeric::add_bound(1.0, f64::from_bits(1)).unwrap(),
        1.0_f64.next_up()
    );
    assert!(super::super::numeric::add_bound(f64::MAX, f64::from_bits(1)).is_err());
    let mut items = [1.0, f64::from_bits(1), 0.0, 0.0].map(direction);
    assert_eq!(
        CarrierConfig::new(resources(), items.clone(), vec![], binding(1.0)),
        Err(JFault::TotalBudgetExceeded)
    );
    items[1].beta = 0.0;
    assert!(CarrierConfig::new(resources(), items, vec![], binding(1.0)).is_ok());
}

#[test]
fn j06_individual_beta_and_nonfinite_inputs_are_rejected() {
    for invalid in [
        -0.25,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        2.0_f64.next_up(),
    ] {
        let mut items = nature();
        items[0].beta = invalid;
        assert!(CarrierConfig::new(resources(), items, vec![], binding(4.0)).is_err());
    }
    let mut items = nature();
    items[0].binding.max_beta = f64::NAN;
    assert!(CarrierConfig::new(resources(), items, vec![], binding(4.0)).is_err());
    for invalid in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(CarrierConfig::new(resources(), nature(), vec![], binding(invalid)).is_err());
    }
}

#[test]
fn validation_cannot_be_reused_for_another_model_tokenizer_carrier_or_direction_set() {
    for field in 0..4 {
        let mut items = nature();
        let resource = match field {
            0 => &mut items[0].binding.resources.model,
            1 => &mut items[0].binding.resources.tokenizer,
            2 => &mut items[0].binding.resources.carrier,
            _ => &mut items[0].binding.resources.direction_set,
        };
        resource.resource_version = "different".into();
        assert_eq!(
            CarrierConfig::new(resources(), items, vec![], binding(4.0)),
            Err(JFault::ResourceMismatch)
        );
    }
    let mut combination = binding(4.0);
    combination.resources.direction_set.digest.0[0] ^= 1;
    assert_eq!(
        CarrierConfig::new(resources(), nature(), vec![], combination),
        Err(JFault::ResourceMismatch)
    );
}

#[test]
fn malformed_references_duplicate_ids_and_empty_concepts_are_refused() {
    let mut invalid = resources();
    invalid.model.digest = HexBytes(vec![0; 31]);
    assert!(matches!(
        CarrierConfig::new(invalid, nature(), vec![], binding(4.0)),
        Err(JFault::Codec(_))
    ));
    assert_eq!(
        CarrierConfig::new(
            resources(),
            nature(),
            vec![learned("a"), learned("a")],
            binding(4.0)
        ),
        Err(JFault::DuplicateDirection("a".into()))
    );
    let mut bad = learned("a");
    bad.concept_id.clear();
    assert_eq!(
        CarrierConfig::new(resources(), nature(), vec![bad], binding(4.0)),
        Err(JFault::InvalidConfig("learned_identifier"))
    );
}

#[test]
fn changed_concept_and_misaligned_x_are_not_silently_repaired() {
    let x = state([0.0; 4], &[("a", 1.0)]);
    let mut t = tendency("a");
    t.concept = "different-concept".into();
    assert_eq!(
        compile(
            &x,
            &StoredR {
                tendencies: vec![t]
            }
        ),
        Err(JFault::ConceptMismatch("a".into()))
    );
    assert_eq!(
        compile(&x, &StoredR::default()),
        Err(JFault::Fast(crate::fast::FastError::Misaligned))
    );
}

#[test]
fn invalid_state_and_reversed_time_fail_without_reusing_a_previous_result() {
    let r = StoredR::default();
    let x = state([0.5; 4], &[]);
    let before = compile(&x, &r).unwrap();
    for invalid in [f64::NAN, f64::INFINITY, 1.1, -0.0] {
        let mut bad = x.clone();
        bad.nature[0] = invalid;
        assert!(matches!(compile(&bad, &r), Err(JFault::Fast(_))));
    }
    let view = JStateView {
        snapshot_id: "snapshot",
        anchor_time: ProcessTime(1),
        theta: &theta(),
        x: &x,
        r: &r,
        process_time: ProcessTime(0),
        validity: &Validity,
    };
    assert_eq!(
        compile_numeric(view, &config(), &body()),
        Err(JFault::TimeBeforeAnchor)
    );
    assert_eq!(
        compile_numeric(
            JStateView {
                snapshot_id: "",
                process_time: ProcessTime(1),
                ..view
            },
            &config(),
            &body()
        ),
        Err(JFault::EmptySnapshotId)
    );
    let mut bad_theta = theta();
    bad_theta.g[0] = f64::NAN;
    assert!(matches!(
        compile_numeric(
            JStateView {
                theta: &bad_theta,
                process_time: ProcessTime(1),
                ..view
            },
            &config(),
            &body()
        ),
        Err(JFault::Effects(_))
    ));
    assert_eq!(bits(&compile(&x, &r).unwrap()), bits(&before));
}
