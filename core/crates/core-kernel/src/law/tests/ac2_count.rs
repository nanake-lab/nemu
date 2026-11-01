//! AC2-01 full-step LP1 under ADR 0019, AC2-06, AC2-07 and exact I25.
//! The original counterexample is retained in prior evidence; the new law
//! must strictly increase on the same four supporting inputs.

use crate::law::recurrence::apply;
use crate::law::shares::{ShareInput, shares};
use crate::law::state::Tendency;
use crate::law::{Law, experience, record};

use super::fixture::{law, tendency};
use super::pyrandom::PyRandom;

/// A random history of `n` mixed groups under the learned ρ.
fn history(rng: &mut PyRandom, l: &Law, n: i64) -> Tendency {
    let mut t = tendency(l, "k", None);
    for _ in 0..n {
        let (w, s) = (rng.uniform(0.0, 3.0), rng.uniform(-1.0, 1.0));
        record(&mut t, l, w, s).unwrap();
    }
    t
}

#[test]
fn ac2_01_consistent_evidence_raises_pi_and_contrary_lowers_it() {
    let mut rng = PyRandom::new(2301);
    let (mut checked, mut bad_plain, mut bad_marginal) = (0u32, 0u32, 0u32);
    for _ in 0..4000 {
        let l = law(
            rng.uniform(0.5, 6.0),
            rng.uniform(0.0, 4.0),
            rng.uniform(0.02, 0.3),
            10.0,
        );
        let n = rng.randint(0, 40);
        let t = history(&mut rng, &l, n);
        let (w, s) = (rng.uniform(0.01, 3.0), rng.uniform(-1.0, 1.0));
        if s == 0.0 {
            continue;
        }
        checked += 1;
        // Without recency weight.
        let mut plain = t.clone();
        apply(&mut plain, &l, 1.0, w, s).unwrap();
        bad_plain += u32::from(if s > 0.0 {
            plain.pi <= t.pi
        } else {
            plain.pi >= t.pi
        });
        // Learned nominal rho selects q; compare the complete committed step.
        let base = t.pi;
        let mut learned = t.clone();
        record(&mut learned, &l, w, s).unwrap();
        bad_marginal += u32::from(if s > 0.0 {
            learned.pi <= base
        } else {
            learned.pi >= base
        });
    }
    println!(
        "AC2-01 full before/after check: {checked} groups, {bad_plain} violations at rho = 1, {bad_marginal} over the committed pre-state"
    );
    assert!(checked > 3900);
    assert_eq!((bad_plain, bad_marginal), (0, 0));
}

#[test]
fn ac2_01_signal_limited_form_keeps_weak_support_monotone() {
    // ADR 0018's old failure, now tested with the adopted ADR 0019 recurrence.
    let l = law(1.0, 0.0, 0.3, 10.0);
    let mut t = tendency(&l, "k", None);
    for _ in 0..3 {
        record(&mut t, &l, 3.0, 1.0).unwrap();
    }
    let before = t.pi;
    record(&mut t, &l, 0.01, 1.0).unwrap();
    let status = if t.pi > before { "passed" } else { "failed" };
    println!(
        "AC2-01_RESULT={{\"id\":\"AC2-01\",\"status\":\"{status}\",\"predicate\":\"pi_after > pi_before for W > 0 and S = 1\",\"pi_before\":{before:.17},\"pi_after\":{:.17},\"support\":1.0,\"weight\":0.01}}",
        t.pi
    );
    assert!(t.pi > before, "{} vs {before}", t.pi);
}

#[test]
fn ac2_06_complex_tendencies_need_more_and_one_salient_group_suffices() {
    let mut rng = PyRandom::new(2306);
    for _ in 0..2000 {
        let (n0, k1) = (rng.uniform(0.5, 6.0), rng.uniform(0.01, 3.0));
        let l = Law::new(&super::fixture::parameters(n0, k1, 0.1, 10.0)).unwrap();
        let w = rng.uniform(0.2, 2.0);
        let mut needed = Vec::new();
        for conditions in 1..=4 {
            assert!(l.base(conditions + 1).unwrap() > l.base(conditions).unwrap());
            let mut t = tendency(&l, "k", None);
            t.predicate = vec![super::fixture::text("c"); conditions];
            t.evidence_b = l.base(conditions).unwrap();
            t.pi = 0.0;
            let mut n = 0u32;
            while t.pi <= 0.5 {
                apply(&mut t, &l, 1.0, w, 1.0).unwrap();
                n += 1;
            }
            needed.push(n);
        }
        assert!(needed.windows(2).all(|p| p[0] <= p[1]), "{needed:?}");
        // One group heavier than n₀ + κ makes personality at once.
        let mut t = tendency(&l, "k", None);
        record(&mut t, &l, l.base(1).unwrap() * 1.01, 1.0).unwrap();
        assert!(t.pi > 0.5);
    }
    // Salience: a 5σ reaction weighs 5 and so clears n₀ + κ = 4 alone.
    let l = law(3.0, 1.0, 0.1, 10.0);
    let g = experience(
        &[[1.0, 0.0, 0.0, 0.0]],
        &[1.0, 0.0, 0.0, 0.0],
        &[0.2; 4],
        0.2,
        1.0,
        8.0,
    )
    .unwrap();
    let mut t = tendency(&l, "k", None);
    record(&mut t, &l, g.intensity, g.support).unwrap();
    assert!(t.pi > 0.5, "{}", t.pi);
}

#[test]
fn ac2_07_one_group_moves_pi_at_most_by_corollary_4() {
    let mut rng = PyRandom::new(2307);
    let (mut bad_plain, mut bad_marginal) = (0u32, 0u32);
    for _ in 0..20_000 {
        let l = law(
            rng.uniform(0.5, 6.0),
            rng.uniform(0.0, 4.0),
            rng.uniform(0.02, 0.3),
            10.0,
        );
        let n = rng.randint(0, 30);
        let t = history(&mut rng, &l, n);
        let (w, s) = (rng.uniform(0.0, 10.0), rng.uniform(-1.0, 1.0));
        let ws = w * s.abs();
        let mut plain = t.clone();
        apply(&mut plain, &l, 1.0, w, s).unwrap();
        bad_plain += u32::from((plain.pi - t.pi).abs() > 2.0 * ws / (t.evidence_b + ws) + 1e-12);
        let base = t.pi;
        let nominal = crate::law::rho(&t, &l).unwrap();
        let q = crate::law::recurrence::actual_retention(&t, &l, nominal, w, s).unwrap();
        let c = l.base(1).unwrap();
        let drift = -base * c;
        let b = c + q * t.evidence_absolute;
        let mut learned = t.clone();
        record(&mut learned, &l, w, s).unwrap();
        bad_marginal += u32::from(
            (learned.pi - base).abs() > ((1.0 - q) * drift.abs() + 2.0 * ws) / (b + ws) + 1e-12,
        );
    }
    println!(
        "AC2-07: 20000 groups, {bad_plain} violations at rho = 1, {bad_marginal} over the committed pre-state"
    );
    assert_eq!((bad_plain, bad_marginal), (0, 0));
}

#[test]
fn ac2_02_i25_each_side_sums_to_one_and_zero_support_gets_nothing() {
    let mut rng = PyRandom::new(2302);
    for _ in 0..20_000 {
        let k = usize::try_from(rng.randint(1, 10)).unwrap();
        let ids: Vec<String> = (0..k).map(|i| format!("k{}", 9 - i)).collect();
        let pis: Vec<f64> = (0..k).map(|_| rng.uniform(-0.9999, 0.9999)).collect();
        let sup: Vec<f64> = (0..k)
            .map(|_| {
                let drawn = rng.uniform(-1.0, 1.0);
                rng.choice(&[0.0, 1.0, -1.0, drawn])
            })
            .collect();
        let inputs: Vec<ShareInput<'_>> = (0..k)
            .map(|i| ShareInput {
                tendency_id: &ids[i],
                pi: pis[i],
                support: sup[i],
            })
            .collect();
        let sh = shares(&inputs).unwrap();
        for positive in [true, false] {
            let side: Vec<usize> = (0..k)
                .filter(|&i| if positive { sup[i] > 0.0 } else { sup[i] < 0.0 })
                .collect();
            if !side.is_empty() {
                let total: f64 = side.iter().map(|&i| sh[i]).sum();
                assert_eq!(total, 1.0);
            }
        }
        assert!((0..k).all(|i| (sup[i] != 0.0 || sh[i] == 0.0) && (0.0..=1.0).contains(&sh[i])));
        // The shares do not depend on the order the tendencies are listed in.
        let mut rev = inputs.clone();
        rev.reverse();
        let mut back = shares(&rev).unwrap();
        back.reverse();
        assert_eq!(
            back.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            sh.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn shares_reject_duplicates_and_out_of_range_values() {
    let a = ShareInput {
        tendency_id: "a",
        pi: 0.0,
        support: 1.0,
    };
    assert!(shares(&[a, a]).is_err());
    assert!(shares(&[ShareInput { pi: 1.0, ..a }]).is_err());
    assert!(shares(&[ShareInput { support: 1.5, ..a }]).is_err());
    // More credible explanations get the larger share (proportional to (1 + π)/2).
    let b = ShareInput {
        tendency_id: "b",
        pi: 0.5,
        support: 0.3,
    };
    let sh = shares(&[a, b]).unwrap();
    let units = (1u64 << 52) as f64;
    assert_eq!(
        sh,
        vec![
            1_801_439_850_948_198.0 / units,
            2_702_159_776_422_298.0 / units
        ]
    );
    assert_eq!(sh.iter().sum::<f64>(), 1.0);
    assert!(sh[1] > sh[0]);
}
