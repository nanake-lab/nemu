//! Original Python seeds/inputs and thresholds, exercising production statistics.
use super::*;
use std::{fs, path::PathBuf};

fn fixture(name: &str) -> Vec<u8> {
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../reports/P5-25/evidence/tests");
    if name == "theta1.bin" {
        // Lossless transport partitioning only; original byte stream/oracle unchanged.
        (0..19)
            .flat_map(|part| {
                fs::read(directory.join(format!("theta1.bin.part-{part:03}"))).unwrap()
            })
            .collect()
    } else {
        fs::read(directory.join(name)).unwrap()
    }
}
fn double(bytes: &mut &[u8]) -> f64 {
    let (head, tail) = bytes.split_at(8);
    *bytes = tail;
    f64::from_le_bytes(head.try_into().unwrap())
}
#[test]
fn theta_1_original_500_streams_zero_violations() {
    let data = fixture("theta1.bin");
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
                let excluded = stage(&s, Some(&e), &s.law, &fuses()).unwrap();
                identical(&s, excluded.state());
            }
            assert_eq!(s.values.baseline.connection.to_bits(), expected_b.to_bits());
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
        "THETA-1: 500 streams, {events} events, bitwise Python b/sigma/g oracle, 0 violations"
    );
}
fn feedback(k: f64, nb: f64, burst: bool, adaptive: bool, noise: &[f64]) -> (f64, u64) {
    let mut p = law();
    p.baseline_weight = nb;
    let mut s = DynamicsState::genesis(p).unwrap();
    let mut tail = 0.0;
    for (i, sample) in noise.iter().enumerate() {
        let a = if burst && (1000..1300).contains(&i) {
            -0.8 + 0.2 * sample
        } else {
            (-0.1 + k * s.values.baseline.connection) + 0.2 * sample
        }
        .clamp(-1.0, 1.0);
        s = if adaptive {
            advance(&s, &event(i as u64, a))
        } else {
            update::statistics(&s, [a; 4], i as u64 * 1_000_000_000, 1.0 - 0.99).unwrap()
        };
        if i >= 5500 {
            tail += s.values.baseline.connection;
        }
    }
    (tail / 500.0, s.change_count())
}
#[test]
fn theta_2_original_feedback_and_independent_adaptive_detector_check() {
    let data = fixture("theta2-noise.bin");
    let mut bytes = data.as_slice();
    let noise: Vec<_> = (0..6000).map(|_| double(&mut bytes)).collect();
    assert!(bytes.is_empty());
    let cap = 1.0 / (1.0 - 0.99);
    for adaptive in [false, true] {
        for (nb, ks) in [
            (10.0, &[0.0, 0.3, 0.6, 0.9, 1.2][..]),
            (cap, &[1.0, 1.5][..]),
        ] {
            for &k in ks {
                let share = cap / (nb + cap);
                let effective = k * share;
                let predicted = if effective < 1.0 {
                    (cap * -0.1) / (nb + cap) / (1.0 - effective)
                } else {
                    f64::NEG_INFINITY
                };
                let (plain, changes) = feedback(k, nb, false, adaptive, &noise);
                let (burst, burst_changes) = feedback(k, nb, true, adaptive, &noise);
                println!(
                    "THETA-2 adaptive={adaptive} nb={nb:.17} k={k} k_eff={effective:.17} predicted={predicted:.17} final={plain:.17} after={burst:.17} changes={changes}/{burst_changes}"
                );
                if predicted > -0.95 {
                    assert!((plain - predicted).abs() < 0.03);
                    assert!((burst - plain).abs() < 0.03);
                } else {
                    assert!(plain < -0.7);
                }
                if nb == cap {
                    assert_eq!(share, 0.5);
                }
            }
        }
    }
    println!("THETA-2: original and adaptive detector feedback cases, 0 violations");
}
