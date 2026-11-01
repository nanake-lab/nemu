use super::*;

fn signed_zero_state() -> FastState {
    FastState {
        nature: [-0.0, 0.0, -f64::from_bits(1), 1e-17],
        learned: vec![LearnedValue {
            tendency_id: "t0".into(),
            value: -0.0,
        }],
    }
}

fn assert_same_bits(x: &FastState, y: &FastState) {
    for (a, b) in x
        .nature
        .iter()
        .chain(x.learned.iter().map(|l| &l.value))
        .zip(y.nature.iter().chain(y.learned.iter().map(|l| &l.value)))
    {
        assert_eq!(a.to_bits(), b.to_bits());
    }
}

#[test]
fn f02_negative_zero_is_rejected_by_validator_and_normalized_at_construction() {
    let raw = signed_zero_state();
    assert_eq!(validate(&raw), Err(FastError::NonCanonicalZero));
    assert_eq!(serialize(&raw, "r", 0), Err(FastError::NonCanonicalZero));
    let mut learned_only = raw.clone();
    learned_only.nature[0] = 0.0;
    assert_eq!(validate(&learned_only), Err(FastError::NonCanonicalZero));
    let mut nature_only = raw.clone();
    nature_only.learned[0].value = 0.0;
    assert_eq!(validate(&nature_only), Err(FastError::NonCanonicalZero));
    let x = FastState::new(raw.nature, raw.learned).unwrap();
    assert_eq!(x.nature[0].to_bits(), 0);
    assert_eq!(x.learned[0].value.to_bits(), 0);
    assert_eq!(x.nature[2].to_bits(), (-f64::from_bits(1)).to_bits());
    assert!(FastState::new([2.0; 4], vec![]).is_err());
    assert!(FastState::new([f64::NAN; 4], vec![]).is_err());
}

#[test]
fn negative_zero_recovery_and_snapshot_preserve_canonical_zero_delta() {
    let e = effects_with(fixture().theta, 1, &mut Rng(83));
    let raw = signed_zero_state();
    assert_eq!(project_time(&raw, &e, 0), Err(FastError::NonCanonicalZero));
    let x = FastState::new(raw.nature, raw.learned).unwrap();
    let (reset, brk) = reset_to_init([-0.0; 4], &e).unwrap();
    assert_eq!(brk, ContinuityBreak::Reset);
    for point in [x, reset] {
        validate(&point).unwrap();
        let bytes = serialize(&point, "r", 123).unwrap();
        let (restored, rev, anchor) = deserialize(&bytes).unwrap();
        assert_eq!((rev.as_str(), anchor), ("r", 123));
        assert_same_bits(&point, &restored);
        assert_same_bits(&restored, &project_time(&restored, &e, 0).unwrap());
        assert_eq!(serialize(&restored, "r", 123).unwrap(), bytes);
    }
}

#[test]
fn snapshot_decoder_rejects_noncanonical_negative_zero_bytes() {
    // CanonicalV1 itself rejects -0 bytes. Verify both nature and learned
    // fields: decoding must never return a noncanonical FastState.
    for learned in [false, true] {
        let marker = 0.123_456_789_f64;
        let x = FastState::new(
            if learned {
                [0.0; 4]
            } else {
                [marker, 0.0, 0.0, 0.0]
            },
            vec![LearnedValue {
                tendency_id: "t0".into(),
                value: if learned { marker } else { 0.0 },
            }],
        )
        .unwrap();
        let mut bytes = serialize(&x, "r", 0).unwrap();
        let offsets: Vec<_> = bytes
            .windows(8)
            .enumerate()
            .filter_map(|(i, v)| (v == marker.to_le_bytes()).then_some(i))
            .collect();
        assert_eq!(offsets.len(), 1);
        bytes[offsets[0]..offsets[0] + 8].copy_from_slice(&(-0.0_f64).to_le_bytes());
        assert!(matches!(deserialize(&bytes), Err(FastError::Codec(_))));
    }
}
