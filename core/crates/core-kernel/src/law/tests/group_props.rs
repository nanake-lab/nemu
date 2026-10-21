//! LAW-8 to LAW-10 of `check_law.py` (v2.0 §5.2 shares and support, §5.3
//! corollary 7), same seeds and sizes as the script (see `law_props.rs`).

use crate::law::shares::{ShareInput, shares};
use crate::law::{consistent_and_contrary, experience, record};

use super::fixture::{law, tendency};
use super::pyrandom::PyRandom;

const SEED: u32 = 20_261_019;

/// LAW-8, §5.2 shares, LP2, I25: each side's shares sum to 1 and the credit
/// given never exceeds the group.
pub fn law_8() -> u64 {
    let mut rng = PyRandom::new(SEED + 7);
    let mut bad = 0;
    for _ in 0..20_000 {
        let k = usize::try_from(rng.randint(1, 8)).unwrap();
        let pis: Vec<f64> = (0..k).map(|_| rng.uniform(-0.999, 0.999)).collect();
        let sup: Vec<f64> = (0..k)
            .map(|_| {
                let drawn = rng.uniform(-1.0, 1.0);
                rng.choice(&[0.0, drawn])
            })
            .collect();
        let ids: Vec<String> = (0..k).map(|i| format!("t{i}")).collect();
        let inputs: Vec<ShareInput<'_>> = (0..k)
            .map(|i| ShareInput {
                tendency_id: &ids[i],
                pi: pis[i],
                support: sup[i],
            })
            .collect();
        let sh = shares(&inputs).unwrap();
        let w_base = rng.uniform(0.0, 5.0);
        for positive in [true, false] {
            let idx: Vec<usize> = (0..k)
                .filter(|&i| if positive { sup[i] > 0.0 } else { sup[i] < 0.0 })
                .collect();
            if !idx.is_empty() {
                let total = idx.iter().fold(0.0, |a, &i| a + sh[i]);
                bad += u64::from((total - 1.0).abs() > 1e-12);
                let credit = idx
                    .iter()
                    .fold(0.0, |a, &i| a + w_base * sh[i] * sup[i].abs());
                bad += u64::from(credit > w_base * (1.0 + 1e-12));
            }
        }
        bad += u64::from(sh.iter().any(|v| !(0.0..=1.0).contains(v)));
        bad += u64::from((0..k).any(|i| sup[i] == 0.0 && sh[i] != 0.0));
    }
    bad
}

#[test]
fn law_8_shares_sum_to_one_per_side_i25() {
    let bad = law_8();
    println!("LAW-8: 20000 random groups with 1-8 matching tendencies, {bad} violations");
    assert_eq!(bad, 0);
}

/// LAW-9: §5.2 support and strength, LP3, LP6.
pub fn law_9() -> (u64, u64, (f64, f64), (f64, f64)) {
    let mut rng = PyRandom::new(SEED + 8);
    let (mut bad_tol, mut bad_bits) = (0, 0);
    let run =
        |group: &[[f64; 4]], base: &[f64; 4], d: &[f64; 4], sigma: &[f64; 4], r: f64, mu: f64| {
            let devs: Vec<[f64; 4]> = group
                .iter()
                .map(|a| std::array::from_fn(|i| a[i] - base[i]))
                .collect();
            let g = experience(&devs, d, sigma, r, mu, 8.0).unwrap();
            (g.support, g.intensity)
        };
    for _ in 0..5000 {
        let base: [f64; 4] = std::array::from_fn(|_| rng.uniform(-0.5, 0.5));
        let raw: [f64; 4] = std::array::from_fn(|_| rng.uniform(-1.0, 1.0));
        let norm = raw.iter().fold(0.0, |s, v| s + v * v).sqrt();
        let norm = if norm == 0.0 { 1.0 } else { norm };
        let d: [f64; 4] = std::array::from_fn(|i| raw[i] / norm);
        let sigma: [f64; 4] = std::array::from_fn(|_| rng.uniform(0.05, 0.5));
        let members = rng.randint(1, 6);
        let group: Vec<[f64; 4]> = (0..members)
            .map(|_| std::array::from_fn(|_| rng.uniform(-1.0, 1.0)))
            .collect();
        let (r, mu) = (rng.uniform(0.05, 1.0), rng.uniform(0.0, 1.0));
        let once = run(&group, &base, &d, &sigma, r, mu);
        let k = usize::try_from(rng.randint(2, 9)).unwrap();
        let repeated: Vec<[f64; 4]> = group
            .iter()
            .copied()
            .cycle()
            .take(group.len() * k)
            .collect();
        let again = run(&repeated, &base, &d, &sigma, r, mu);
        bad_tol += u64::from((once.0 - again.0).abs() > 1e-12 || (once.1 - again.1).abs() > 1e-12);
        bad_bits += u64::from(
            once.0.to_bits() != again.0.to_bits() || once.1.to_bits() != again.1.to_bits(),
        );
    }
    let zero = [0.0; 4];
    let e1 = [1.0, 0.0, 0.0, 0.0];
    let silent = run(&[zero], &zero, &e1, &[0.2; 4], 0.2, 1.0);
    let striking = run(&[e1], &zero, &e1, &[0.2; 4], 0.2, 1.0);
    (bad_tol, bad_bits, silent, striking)
}

#[test]
fn law_9_repeats_add_nothing_and_missing_reaction_counts() {
    let (bad_tol, bad_bits, silent, striking) = law_9();
    println!(
        "LAW-9: 5000 groups repeated 2-9 times: {bad_tol} changes beyond 1e-12, {bad_bits} bitwise; silent {silent:?}, striking {striking:?}"
    );
    assert_eq!(bad_tol, 0);
    assert_eq!(
        bad_bits, 0,
        "the exact mean makes repeats bitwise invariant"
    );
    assert_eq!(silent, (-1.0, 1.0));
    assert_eq!(striking, (1.0, 5.0));
}

/// LAW-10, corollary 7: attachment is hard to move but finite; alternating
/// contrary evidence cannot move it.
pub fn law_10() -> (Option<u32>, Option<u32>, Option<u32>, bool, f64) {
    let l = law(3.0, 1.0, 0.1, 10.0);
    let history = |prior: f64| {
        let mut t = tendency(&l, "k", (prior != 0.0).then_some(prior));
        for _ in 0..200 {
            record(&mut t, &l, 1.0, 1.0).unwrap();
        }
        t
    };
    let flip_after = |t: &mut core_types::generated::Tendency, pattern: &dyn Fn(u32) -> f64| {
        for n in 1..=100_000u32 {
            record(t, &l, 1.0, pattern(n)).unwrap();
            if t.pi < 0.5 {
                return Some(n);
            }
        }
        None
    };
    let n_ordinary = flip_after(&mut history(0.0), &|_| -1.0);
    let n_att = flip_after(&mut history(60.0), &|_| -1.0);
    let n_att_big = flip_after(&mut history(600.0), &|_| -1.0);
    let mut t = history(60.0);
    let flipped = flip_after(&mut t, &|_| -1.0);
    let (e, c) = consistent_and_contrary(&t, &l).unwrap();
    let exact = flipped.is_some() && c > (60.0 + e - 3.0 - 1.0) / 3.0;
    let mut alternating = history(60.0);
    let mut lowest = 1.0f64;
    for n in 0..5000 {
        record(
            &mut alternating,
            &l,
            1.0,
            if n % 2 == 0 { -1.0 } else { 1.0 },
        )
        .unwrap();
        lowest = lowest.min(alternating.pi);
    }
    (n_ordinary, n_att, n_att_big, exact, lowest)
}

#[test]
fn law_10_attachment_corollary_7() {
    let (ordinary, att, att_big, exact, lowest) = law_10();
    println!(
        "LAW-10: contrary groups to bring pi below 1/2 after 200 supporting: ordinary {ordinary:?}, A_att=60 {att:?}, A_att=600 {att_big:?}; exact {exact}; alternating keeps pi >= {lowest:.3}"
    );
    let (o, a, b) = (ordinary.unwrap(), att.unwrap(), att_big.unwrap());
    assert!(o < a && a < b);
    assert_eq!((o, a, b), (29, 51, 421), "same scenario as check_law.py");
    assert!(exact);
    assert!(lowest > 0.5);
}
