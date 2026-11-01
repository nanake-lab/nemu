//! Every scalar addition and share operation of legacy+extra, same input bits.
use super::fixture::{law, tendency};
use super::reference_support::{ReferenceChild, fault_code};
use super::state_bits::tendency_bits_eq;
use crate::law::recurrence::actual_retention;
use crate::law::replay::fold_group;
use crate::law::{LawError, ShareInput, shares};
use core_types::generated::EvidenceSign;
use core_types::scalar::DecimalU64;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

fn float(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s, 16).unwrap())
}
fn sign(s: &str) -> Option<EvidenceSign> {
    match s {
        "1" => Some(EvidenceSign::SIGN_POSITIVE),
        "-1" => Some(EvidenceSign::SIGN_NEGATIVE),
        "0" => None,
        _ => panic!("invalid sign {s}"),
    }
}

#[test]
fn adr0019_python_legacy_and_extra_same_input_all_scalar_bits() {
    let python = std::env::var("P523_PYTHON").unwrap_or_else(|_| "python".into());
    let script =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/law/tests/reference_vectors.py");
    let mut child = ReferenceChild(
        Command::new(python)
            .arg("-B")
            .arg(script)
            .env("PYTHONUTF8", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let mut scalar = 0u64;
    let mut share_count = 0u64;
    let mut faults = 0u64;
    for (line_number, line) in BufReader::new(stdout).lines().enumerate() {
        let line = line.unwrap();
        if let Some(result) = line.strip_prefix("R ") {
            println!("REFERENCE {result}");
            continue;
        }
        let v: Vec<&str> = line.split_whitespace().collect();
        match v[0] {
            "A" => {
                let l = law(float(v[1]), float(v[2]), float(v[4]), float(v[5]));
                let p = float(v[3]);
                let mut t = tendency(&l, "k", if p == 0.0 { None } else { Some(p) });
                t.evidence_a = float(v[6]);
                t.evidence_absolute = float(v[7]);
                t.evidence_b = l.base(1).unwrap() + t.evidence_absolute;
                t.magnitude_sum = float(v[8]);
                t.magnitude_weight = float(v[9]);
                t.group_count = DecimalU64(v[10].parse().unwrap());
                t.flip_count = DecimalU64(v[11].parse().unwrap());
                t.last_sign = sign(v[12]);
                t.pi = crate::law::pi(p, t.evidence_a, l.base(1).unwrap(), t.evidence_absolute)
                    .unwrap();
                t.h_hat = l.h_hat(t.group_count.0, t.flip_count.0).unwrap();
                t.typical_magnitude =
                    crate::law::replay::typical_magnitude(&l, t.magnitude_sum, t.magnitude_weight)
                        .unwrap();
                let before = t.clone();
                let r = if v[13] == "none" {
                    crate::law::rho(&t, &l).unwrap()
                } else {
                    float(v[13])
                };
                let (w, s, mu) = (float(v[14]), float(v[15]), float(v[16]));
                let projection = if v[17] == "none" {
                    None
                } else {
                    Some(float(v[17]))
                };
                let result = (|| {
                    let q = actual_retention(&t, &l, r, w, s)?;
                    fold_group(&mut t, &l, q, w, s, mu, projection, true)?;
                    Ok::<f64, LawError>(q)
                })();
                if v[18] == "fault" {
                    faults += 1;
                    assert_eq!(v.len(), 20, "fault vector {line_number}");
                    let error = result.expect_err("reference fault accepted");
                    assert_eq!(
                        fault_code(&error),
                        v[19],
                        "fault category at vector {line_number}: {error}; {line}"
                    );
                    assert!(
                        tendency_bits_eq(&t, &before),
                        "fault changed state at {line_number}"
                    );
                } else {
                    assert_eq!(v[18], "ok");
                    assert_eq!(v.len(), 31, "scalar vector {line_number}");
                    let q =
                        result.unwrap_or_else(|err| panic!("vector {line_number}: {err}; {line}"));
                    let got = [
                        q,
                        t.evidence_a,
                        t.evidence_absolute,
                        t.pi,
                        t.magnitude_sum,
                        t.magnitude_weight,
                        t.h_hat,
                        t.evidence_b,
                        t.typical_magnitude,
                    ];
                    for (field, (got, want)) in got.iter().zip(&v[19..28]).enumerate() {
                        assert_eq!(
                            got.to_bits(),
                            float(want).to_bits(),
                            "vector {line_number} field {field}: {line}"
                        );
                    }
                    let mut expected = before;
                    expected.evidence_a = float(v[20]);
                    expected.evidence_absolute = float(v[21]);
                    expected.pi = float(v[22]);
                    expected.magnitude_sum = float(v[23]);
                    expected.magnitude_weight = float(v[24]);
                    expected.h_hat = float(v[25]);
                    expected.evidence_b = float(v[26]);
                    expected.typical_magnitude = float(v[27]);
                    expected.group_count = DecimalU64(v[28].parse().unwrap());
                    expected.flip_count = DecimalU64(v[29].parse().unwrap());
                    expected.last_sign = sign(v[30]);
                    assert!(
                        tendency_bits_eq(&t, &expected),
                        "full scalar state at {line_number}: {line}"
                    );
                    scalar += 1;
                }
            }
            "S" => {
                let n: usize = v[1].parse().unwrap();
                let ids: Vec<String> = (0..n).map(|i| format!("{i:08}")).collect();
                let input: Vec<ShareInput<'_>> = (0..n)
                    .map(|i| ShareInput {
                        tendency_id: &ids[i],
                        pi: float(v[2 + i]),
                        support: float(v[2 + n + i]),
                    })
                    .collect();
                let result = shares(&input);
                if v[2 + 2 * n] == "fault" {
                    assert_eq!(v.len(), 4 + 2 * n, "share fault {line_number}");
                    let error = result.expect_err("reference share fault accepted");
                    assert_eq!(
                        fault_code(&error),
                        v[3 + 2 * n],
                        "share fault {line_number}: {error}"
                    );
                    faults += 1;
                } else {
                    assert_eq!(v[2 + 2 * n], "ok");
                    assert_eq!(v.len(), 3 + 3 * n, "share vector {line_number}");
                    let got = result.unwrap();
                    assert_eq!(got.len(), n);
                    for (got, want) in got.iter().zip(&v[3 + 2 * n..]) {
                        assert_eq!(
                            got.to_bits(),
                            float(want).to_bits(),
                            "share vector {line_number}"
                        );
                    }
                    share_count += 1;
                }
            }
            _ => panic!("unexpected reference output {line}"),
        }
    }
    assert!(child.0.wait().unwrap().success());
    assert!(scalar > 1_000_000 && share_count > 20_000 && faults > 0);
    println!(
        "Same-input parity: {scalar} scalar states, {share_count} share vectors, {faults} explicit faults; zero bit mismatches"
    );
}
