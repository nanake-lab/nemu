//! V2-93 normative oracle, separate from the retained failing **2 comparison.
use super::*;

fn double(bytes: &mut &[u8]) -> f64 {
    let (head, tail) = bytes.split_at(8);
    *bytes = tail;
    f64::from_le_bytes(head.try_into().unwrap())
}

#[test]
fn theta_1_binary64_500_original_streams_zero_bit_differences() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../reports/P5-25/batch4-20261026/reference/theta1-mul.bin");
    let data = std::fs::read(path).unwrap();
    let mut bytes = data.as_slice();
    let mut events = 0;
    for _ in 0..500 {
        let mut p = law();
        p.baseline_weight = double(&mut bytes);
        p.variance_weight = double(&mut bytes);
        let rho = double(&mut bytes);
        p.baseline_prior = [double(&mut bytes); 4];
        let (head, tail) = bytes.split_at(4);
        let count = u32::from_le_bytes(head.try_into().unwrap());
        bytes = tail;
        let mut s = DynamicsState::genesis(p).unwrap();
        for i in 0..count {
            let a = double(&mut bytes);
            let exo = bytes[0] == 1;
            bytes = &bytes[1..];
            let expected_b = double(&mut bytes);
            let expected_var = double(&mut bytes);
            let expected_g = double(&mut bytes);
            if exo {
                s = update::statistics(&s, [a; 4], u64::from(i) * 1_000_000_000, 1.0 - rho)
                    .unwrap();
            } else {
                let mut e = event(u64::from(i), a);
                e.source = ReactionSource::SelfAction;
                identical(&s, stage(&s, Some(&e), &s.law, &fuses()).unwrap().state());
            }
            assert_eq!(s.values.baseline.connection.to_bits(), expected_b.to_bits());
            assert_eq!(s.variance[0].to_bits(), expected_var.to_bits());
            assert_eq!(s.sigma[0].to_bits(), libm::sqrt(expected_var).to_bits());
            assert_eq!(s.values.gain.connection.to_bits(), expected_g.to_bits());
            assert!((-1.0..=1.0).contains(&s.values.baseline.connection));
            assert!((0.2..=3.0).contains(&s.values.gain.connection));
            assert!(
                (s.law.recovery_min..=s.law.recovery_max)
                    .contains(&s.values.recovery_per_s.connection)
            );
            events += 1;
        }
    }
    assert!(bytes.is_empty());
    assert_eq!(events, 72822);
    println!(
        "THETA-1 V2-93: 500 original streams, {events} events, strict b/variance/sigma/g bits, 0 differences"
    );
}
