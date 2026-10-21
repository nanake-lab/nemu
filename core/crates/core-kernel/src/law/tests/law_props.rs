//! Corollaries 1–7 of v2.0 §5.3 as property tests: a port of LAW-1 to LAW-10
//! in `reports/echo-core-v2-draft/check_law.py`. The trials are the same: the
//! same seeds, sizes and ranges, drawn with CPython's generator (`PyRandom`),
//! so the reported state counts can be compared with `results.txt`. The law
//! under test is this crate's recurrence (D4), not the script's.
//! LAW-8 to LAW-10 (groups, shares, attachment) are in `group_props.rs`.

use crate::law::recurrence::apply;
use crate::law::{Law, consistent_and_contrary, record};

use super::fixture::{law, tendency};
use super::pyrandom::PyRandom;

const SEED: u32 = 20_261_019;
const TOL: f64 = 1e-9;

/// The one-condition test tendency: c₀ = n₀ + κ.
fn base(l: &Law) -> f64 {
    l.base(1).unwrap()
}

fn e_c(t: &core_types::generated::Tendency, l: &Law) -> (f64, f64) {
    consistent_and_contrary(t, l).unwrap()
}

/// LAW-1, corollary 1: only consistent evidence, π > ½ ⟺ E > n₀ + κ.
pub fn law_1() -> (u64, u64) {
    let mut rng = PyRandom::new(SEED);
    let (mut bad, mut states) = (0, 0);
    for _ in 0..2000 {
        let rho = if rng.random() < 0.5 {
            1.0
        } else {
            rng.uniform(0.8, 1.0)
        };
        let (n0, kappa) = (rng.uniform(0.5, 6.0), rng.uniform(0.0, 4.0));
        let l = law(n0, kappa, 0.1, 10.0);
        let mut t = tendency(&l, "k", None);
        for _ in 0..rng.randint(1, 40) {
            apply(&mut t, &l, rho, rng.uniform(0.0, 3.0), 1.0).unwrap();
            let gap = e_c(&t, &l).0 - base(&l);
            if gap.abs() < TOL {
                continue;
            }
            states += 1;
            bad += u64::from((t.pi > 0.5) != (gap > 0.0));
        }
    }
    (bad, states)
}

#[test]
fn law_1_threshold_corollary_1() {
    let (bad, states) = law_1();
    println!("LAW-1: {states} states, {bad} violations");
    assert_eq!(bad, 0);
    assert_eq!(states, 41_607, "same trials as check_law.py");
}

/// LAW-2, corollary 2 and LP6: one group with W > n₀ + κ suffices; larger κ
/// needs more ordinary groups.
pub fn law_2() -> (u64, Vec<u32>) {
    let mut rng = PyRandom::new(SEED + 1);
    let mut bad = 0;
    for _ in 0..20_000 {
        let (n0, kappa, w) = (
            rng.uniform(0.5, 6.0),
            rng.uniform(0.0, 4.0),
            rng.uniform(0.0, 12.0),
        );
        if (w - n0 - kappa).abs() < TOL {
            continue;
        }
        let l = law(n0, kappa, 0.1, 10.0);
        let mut t = tendency(&l, "k", None);
        record(&mut t, &l, w, 1.0).unwrap();
        bad += u64::from((t.pi > 0.5) != (w > n0 + kappa));
    }
    let needed = (0..6)
        .map(|kappa| {
            let l = law(3.0, f64::from(kappa), 0.1, 10.0);
            let mut t = tendency(&l, "k", None);
            let mut n = 0;
            while t.pi <= 0.5 {
                apply(&mut t, &l, 1.0, 1.0, 1.0).unwrap();
                n += 1;
            }
            n
        })
        .collect();
    (bad, needed)
}

#[test]
fn law_2_one_shot_and_occam_corollary_2() {
    let (bad, needed) = law_2();
    println!(
        "LAW-2: 20000 single groups, {bad} violations; ordinary groups for kappa 0..5: {needed:?}"
    );
    assert_eq!(bad, 0);
    assert_eq!(needed, [4, 5, 6, 7, 8, 9]);
    assert!(needed.windows(2).all(|p| p[0] < p[1]));
}

/// LAW-3, corollary 3: π < ½ ⟺ C > (E − n₀ − κ)/3, mixed signs, learned ρ.
pub fn law_3() -> (u64, u64) {
    let mut rng = PyRandom::new(SEED + 2);
    let (mut bad, mut states) = (0, 0);
    for _ in 0..2000 {
        let (n0, kappa, h0) = (
            rng.uniform(0.5, 6.0),
            rng.uniform(0.0, 4.0),
            rng.uniform(0.02, 0.3),
        );
        let l = law(n0, kappa, h0, 10.0);
        let mut t = tendency(&l, "k", None);
        for _ in 0..rng.randint(1, 60) {
            let (w, s) = (rng.uniform(0.0, 3.0), rng.uniform(-1.0, 1.0));
            record(&mut t, &l, w, s).unwrap();
            let (e, c) = e_c(&t, &l);
            let margin = c - (e - n0 - kappa) / 3.0;
            if margin.abs() < TOL || (t.pi - 0.5).abs() < TOL {
                continue;
            }
            states += 1;
            bad += u64::from((t.pi < 0.5) != (margin > 0.0));
        }
    }
    (bad, states)
}

#[test]
fn law_3_reversal_corollary_3() {
    let (bad, states) = law_3();
    println!("LAW-3: {states} states, {bad} violations");
    assert_eq!(bad, 0);
    assert_eq!(states, 61_604, "same trials as check_law.py");
}

/// LAW-4, corollary 4 and LP7: one group changes π by at most 2w/(B + w).
pub fn law_4() -> (u64, f64) {
    let mut rng = PyRandom::new(SEED + 3);
    let (mut bad, mut worst) = (0, 0.0f64);
    for _ in 0..30_000 {
        let n0 = rng.uniform(0.5, 6.0);
        let kappa = rng.uniform(0.0, 4.0);
        let drawn = rng.uniform(0.0, 50.0);
        let prior = rng.choice(&[0.0, drawn]);
        let l = law(n0, kappa, 0.1, 10.0);
        let mut t = tendency(&l, "k", (prior != 0.0).then_some(prior));
        for _ in 0..rng.randint(0, 30) {
            let (w, s) = (rng.uniform(0.0, 3.0), rng.uniform(-1.0, 1.0));
            apply(&mut t, &l, 1.0, w, s).unwrap();
        }
        let before = t.pi;
        let big_b = prior + t.evidence_b;
        let (w, s) = (rng.uniform(0.0, 10.0), rng.uniform(-1.0, 1.0));
        apply(&mut t, &l, 1.0, w, s).unwrap();
        let bound = 2.0 * w * s.abs() / (big_b + w * s.abs());
        let delta = (t.pi - before).abs();
        bad += u64::from(delta > bound + 1e-12);
        if bound > 0.0 {
            worst = worst.max(delta / bound);
        }
    }
    (bad, worst)
}

#[test]
fn law_4_single_group_bound_corollary_4() {
    let (bad, worst) = law_4();
    println!("LAW-4: 30000 states, {bad} violations, largest |dpi|/bound = {worst:.4}");
    assert_eq!(bad, 0);
    assert!(worst <= 1.0);
}

/// LAW-5, corollary 5, LP5, LP8: π depends on neither elapsed time nor x.
/// The fast state follows Φ_x and drives as in the script; the law never sees it.
pub fn law_5() -> u64 {
    fn run(events: &[(f64, f64, f64)], x0: [f64; 4], lam: [f64; 4]) -> f64 {
        let l = law(3.0, 1.0, 0.1, 10.0);
        let mut t = tendency(&l, "k", None);
        let mut x = x0;
        for &(dt, w, s) in events {
            for i in 0..4 {
                x[i] = (x[i] * libm::exp(-lam[i] * dt) + 0.3 * s).clamp(-1.0, 1.0);
            }
            record(&mut t, &l, w, s).unwrap();
        }
        std::hint::black_box(x);
        t.pi
    }
    let mut rng = PyRandom::new(SEED + 4);
    let mut bad = 0;
    for _ in 0..500 {
        let base: Vec<(f64, f64, f64)> = (0..30)
            .map(|_| {
                (
                    rng.uniform(0.0, 10.0),
                    rng.uniform(0.0, 3.0),
                    rng.uniform(-1.0, 1.0),
                )
            })
            .collect();
        let stretched: Vec<_> = base
            .iter()
            .map(|&(dt, w, s)| (dt * rng.uniform(1.0, 1e6), w, s))
            .collect();
        let lam: [f64; 4] = std::array::from_fn(|_| rng.uniform(0.01, 2.0));
        let xa: [f64; 4] = std::array::from_fn(|_| rng.uniform(-1.0, 1.0));
        let pi_a = run(&base, xa, lam);
        let xb: [f64; 4] = std::array::from_fn(|_| rng.uniform(-1.0, 1.0));
        let pi_b = run(&stretched, xb, lam);
        bad += u64::from(pi_a.to_bits() != pi_b.to_bits());
    }
    bad
}

#[test]
fn law_5_time_and_state_independence_corollary_5() {
    let bad = law_5();
    println!("LAW-5: 500 sequences, {bad} bitwise differences in pi");
    assert_eq!(bad, 0);
}

/// LAW-6, corollary 6: with constant ρ, E and C stay below w_max/(1 − ρ).
pub fn law_6() -> u64 {
    let mut rng = PyRandom::new(SEED + 5);
    let mut bad = 0;
    for _ in 0..3000 {
        let (rho, w_max) = (rng.uniform(0.5, 0.999), rng.uniform(0.5, 5.0));
        let l = law(3.0, 1.0, 0.1, 10.0);
        let mut t = tendency(&l, "k", None);
        let cap = w_max / (1.0 - rho);
        for _ in 0..rng.randint(1, 400) {
            let w = rng.uniform(0.0, w_max);
            let drawn = rng.uniform(-1.0, 1.0);
            let s = rng.choice(&[1.0, -1.0, drawn]);
            apply(&mut t, &l, rho, w, s).unwrap();
            let (e, c) = e_c(&t, &l);
            bad += u64::from(e > cap * (1.0 + 1e-12) || c > cap * (1.0 + 1e-12));
        }
    }
    bad
}

#[test]
fn law_6_memory_bound_corollary_6() {
    let bad = law_6();
    println!("LAW-6: 3000 sequences, {bad} violations");
    assert_eq!(bad, 0);
}

/// LAW-7, §5.2 calculation order and §2.2: replaying the recurrence is
/// bitwise identical; the mathematically equal closed form is not.
pub fn law_7() -> (u64, u64, f64) {
    let mut rng = PyRandom::new(SEED + 6);
    let (mut bitwise_bad, mut closed_differs, mut max_rel) = (0, 0, 0.0f64);
    for _ in 0..1000 {
        let n = rng.randint(1, 200);
        let groups: Vec<(f64, f64)> = (0..n)
            .map(|_| (rng.uniform(0.0, 3.0), rng.uniform(-1.0, 1.0)))
            .collect();
        let l = law(3.0, 1.0, 0.1, 10.0);
        let mut t = tendency(&l, "k", None);
        let rhos: Vec<f64> = groups
            .iter()
            .map(|&(w, s)| record(&mut t, &l, w, s).unwrap())
            .collect();
        let mut replay = tendency(&l, "k", None);
        for &(w, s) in &groups {
            record(&mut replay, &l, w, s).unwrap();
        }
        bitwise_bad += u64::from(replay != t);
        let mut closed = 0.0;
        for (i, &(w, s)) in groups.iter().enumerate() {
            closed += rhos[i + 1..].iter().fold(1.0, |p, r| p * r) * w * s;
        }
        closed_differs += u64::from(closed != t.evidence_a);
        max_rel = max_rel.max((closed - t.evidence_a).abs() / t.evidence_a.abs().max(1.0));
    }
    (bitwise_bad, closed_differs, max_rel)
}

#[test]
fn law_7_recurrence_is_canonical() {
    let (bitwise_bad, closed_differs, max_rel) = law_7();
    println!(
        "LAW-7: replay differs bitwise in {bitwise_bad}; closed form differs in {closed_differs} (max relative {max_rel:.1e})"
    );
    assert_eq!(bitwise_bad, 0);
    assert!(max_rel < 1e-12);
    // A_k follows the same operations as the script's `a`, so the closed form
    // differs in exactly the same trials.
    assert_eq!(closed_differs, 410);
}
