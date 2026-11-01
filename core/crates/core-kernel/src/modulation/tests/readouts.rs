use super::*;

#[test]
fn body_clamps_all_three_axes_and_has_no_learned_component() {
    let body = BodyReadoutConfig::new(
        "saturating-fixture".into(),
        [
            [4.0, 0.0, 0.0, 0.0],
            [0.0, 4.0, 0.0, 0.0],
            [0.0, 0.0, 4.0, 0.0],
        ],
    )
    .unwrap();
    for (values, expected) in [([1.0; 4], (1.0, 1.0, 1.0)), ([-1.0; 4], (0.0, -1.0, 0.0))] {
        let x = state(values, &[]);
        let r = StoredR::default();
        let result = compile_numeric(
            JStateView {
                snapshot_id: "body",
                anchor_time: ProcessTime(0),
                theta: &theta(),
                x: &x,
                r: &r,
                process_time: ProcessTime(0),
                validity: &Validity,
            },
            &config(),
            &body,
        )
        .unwrap();
        assert_eq!(
            (result.body.facing, result.body.tension, result.body.energy),
            expected
        );
    }
    let r = StoredR {
        tendencies: vec![tendency("a")],
    };
    let a = compile(&state([0.5; 4], &[("a", -1.0)]), &r).unwrap();
    let b = compile(&state([0.5; 4], &[("a", 1.0)]), &r).unwrap();
    assert_eq!(a.body, b.body);
    assert_ne!(a.learned_coefficients, b.learned_coefficients);
}

#[test]
fn body_nonfinite_or_overflow_is_not_hidden_by_clipping() {
    assert!(BodyReadoutConfig::new(String::new(), [[0.0; 4]; 3]).is_err());
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(BodyReadoutConfig::new("invalid".into(), [[bad; 4]; 3]).is_err());
    }
    let body = BodyReadoutConfig::new("overflow".into(), [[f64::MAX; 4]; 3]).unwrap();
    let x = state([1.0; 4], &[]);
    let r = StoredR::default();
    assert_eq!(
        compile_numeric(
            JStateView {
                snapshot_id: "body",
                anchor_time: ProcessTime(0),
                theta: &theta(),
                x: &x,
                r: &r,
                process_time: ProcessTime(0),
                validity: &Validity
            },
            &config(),
            &body
        ),
        Err(JFault::NonFinite("body_readout_sum"))
    );
}

#[test]
fn f_disabled_is_explicitly_rehearsal_only_and_matches_zero_readout() {
    let cfg = config();
    let zero = compile(&state([0.0; 4], &[]), &StoredR::default()).unwrap();
    let rehearsal =
        rehearsal_without_fast(Lineage::REHEARSAL, "early", ProcessTime(9), &cfg, &body()).unwrap();
    assert_eq!(rehearsal.nature_coefficients, zero.nature_coefficients);
    assert_eq!(rehearsal.body, zero.body);
    assert!(rehearsal.learned_coefficients.is_empty());
    assert_eq!(
        rehearsal.receipt.mode,
        CompilationMode::RehearsalWithoutFast
    );
    for lineage in [Lineage::LIFE, Lineage::EXPERIMENT] {
        assert_eq!(
            rehearsal_without_fast(lineage, "early", ProcessTime(9), &cfg, &body()),
            Err(JFault::RehearsalOnly)
        );
    }
}

#[test]
fn numerical_zero_modulation_keeps_body_neutral_even_for_nonzero_x() {
    let cfg =
        CarrierConfig::new(resources(), [0.0; 4].map(direction), vec![], binding(0.0)).unwrap();
    let x = state([1.0; 4], &[]);
    let r = StoredR::default();
    let result = compile_numeric(
        JStateView {
            snapshot_id: "zero-beta",
            anchor_time: ProcessTime(0),
            theta: &theta(),
            x: &x,
            r: &r,
            process_time: ProcessTime(0),
            validity: &Validity,
        },
        &cfg,
        &body(),
    )
    .unwrap();
    assert_eq!(
        (result.body.facing, result.body.tension, result.body.energy),
        (0.5, 0.0, 0.5)
    );
    assert_eq!(result.total_bound.to_bits(), 0);
    assert_eq!(nature_values(&result).map(f64::to_bits), [0; 4]);
}
