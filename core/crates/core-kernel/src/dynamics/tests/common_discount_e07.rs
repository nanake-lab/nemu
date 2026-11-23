use super::*;
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn e07_dynamic_same_h_birth_difference_rational_envelope() {
    // ADR 0035 §9(5): unlike the frozen-rho diagnostic, this invokes stage and
    // the real detector. It currently observes a (not b), so paired H shares rho.
    // Check each pair using independent exact rational N/T roundoff bounds.
    let make = |b0| {
        let mut p = law();
        p.version = "synthetic-ADR0035-dynamic-E07".into();
        p.baseline_prior = [b0; 4];
        p.baseline_weight = 2.0;
        DynamicsState::genesis_common_discount(p).unwrap()
    };
    let mut left = make(0.25);
    let mut right = make(-0.25);
    let mut trace = Vec::new();
    for i in 1..=128 {
        let e = event(i, if i % 29 < 14 { 0.75 } else { -0.5 });
        left = advance(&left, &e);
        right = advance(&right, &e);
        assert_eq!(left.h.to_bits(), right.h.to_bits());
        assert_eq!(left.prior_mass, right.prior_mass);
        assert_eq!(left.weight.to_bits(), right.weight.to_bits());
        assert_eq!(
            left.reaction_sum.map(f64::to_bits),
            right.reaction_sum.map(f64::to_bits)
        );
        trace.push(
            [
                left.prior_mass.unwrap(),
                left.weight,
                left.reaction_sum[0],
                0.25,
                left.values.baseline.connection,
                right.values.baseline.connection,
            ]
            .map(|x| x.to_bits().to_string()),
        );
    }
    let mut child = Command::new("python")
        .arg(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/dynamics/tests/d07_oracle.py"),
        )
        .arg("--trace")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("independent rational oracle");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&trace).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    println!(
        "D07-dynamic-birth bits={}",
        serde_json::to_string(&trace).unwrap()
    );
}
