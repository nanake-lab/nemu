//! Every scalar addition and share operation of legacy+extra, same input bits.
use super::fixture::{law, tendency};
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
    let mut child = Command::new(python)
        .arg("-B")
        .arg(script)
        .env("PYTHONUTF8", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
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
                t.evidence_v = float(v[7]);
                t.evidence_b = l.base(1).unwrap() + t.evidence_v;
                t.magnitude_sum = float(v[8]);
                t.magnitude_weight = float(v[9]);
                t.group_count = DecimalU64(v[10].parse().unwrap());
                t.flip_count = DecimalU64(v[11].parse().unwrap());
                t.last_sign = sign(v[12]);
                t.pi = crate::law::pi(p, t.evidence_a, l.base(1).unwrap(), t.evidence_v).unwrap();
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
                    assert!(
                        result.is_err(),
                        "reference fault {} accepted at vector {line_number}",
                        v[19]
                    );
                    assert_eq!(t, before, "fault changed state at {line_number}");
                } else {
                    let q =
                        result.unwrap_or_else(|err| panic!("vector {line_number}: {err}; {line}"));
                    let got = [
                        q,
                        t.evidence_a,
                        t.evidence_v,
                        t.pi,
                        t.magnitude_sum,
                        t.magnitude_weight,
                        t.h_hat,
                    ];
                    for (field, (got, want)) in got.iter().zip(&v[19..26]).enumerate() {
                        assert_eq!(
                            got.to_bits(),
                            float(want).to_bits(),
                            "vector {line_number} field {field}: {line}"
                        );
                    }
                    assert_eq!(t.group_count.0, v[26].parse::<u64>().unwrap());
                    assert_eq!(t.flip_count.0, v[27].parse::<u64>().unwrap());
                    assert_eq!(t.last_sign, sign(v[28]));
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
                    assert!(result.is_err());
                    faults += 1;
                } else {
                    let got = result.unwrap();
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
    assert!(child.wait().unwrap().success());
    assert!(scalar > 1_000_000 && share_count > 20_000 && faults > 0);
    println!(
        "Same-input parity: {scalar} scalar states, {share_count} share vectors, {faults} explicit faults; zero bit mismatches"
    );
}
