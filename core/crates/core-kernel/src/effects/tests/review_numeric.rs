//! Regression tests for the cross-review of PR #22, findings B6–B7: scope
//! membership and budget totals must agree with exact arithmetic. The first
//! test of each group is the reviewer's counterexample, kept as given.

use std::collections::{BTreeMap, BTreeSet};

use core_policy::scope::evaluate;
use core_policy::{Endpoint, FieldKind, FieldValue, Overlap, Scope, ScopeExpr, ScopeMatch};
use core_types::generated::EffectOwner;

use super::super::bound::{add_up, mul_up, sum_up};
use super::super::*;
use super::fixture::*;

fn jd(e: ScopeExpr) -> Scope {
    scope(Endpoint::JDirect, e)
}

// ---------------------------------------------------------------- B6 ----

#[test]
fn b6_review_disjoint_scopes_cannot_both_hit_one_context() {
    let mut cfg = config();
    cfg.catalog.fields.get_mut("x0").unwrap().kind = FieldKind::Numeric {
        min: 0,
        max: 1i64 << 53,
        decimals: 1,
    };
    let n = 9_007_199_254_740_952i64;
    let a = Scope::new(Endpoint::JDirect, x0(n, n), &cfg.catalog).unwrap();
    let b = Scope::new(Endpoint::JDirect, x0(n + 1, n + 1), &cfg.catalog).unwrap();
    assert_eq!(
        core_policy::overlap(&a, &b, &cfg.catalog),
        core_policy::Overlap::Disjoint
    );
    let ta = theta("m1", "same", a, params(&[(0, 0.25)]), 0.25);
    let tb = theta("m2", "same", b, params(&[(0, 0.25)]), 0.25);
    let (set, _) =
        compile_authorized_effects(&[], &[ta, tb], &validity(), ProcessTime(0), &cfg).unwrap();
    let context = TypedContext::new(
        Endpoint::JDirect,
        &cfg.catalog,
        BTreeMap::from([("x0".into(), FieldValue::Number(n as f64 / 10.0))]),
    )
    .unwrap();
    let out = evaluate_effects(&set, &context, &cfg.catalog).unwrap();
    assert!(
        out.receipt.len() <= 1,
        "two owners of same responsibility hit: {out:?}"
    );
}

/// Deterministic xorshift64* stream for property tests (no extra crate).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[test]
fn b6_property_disjoint_implies_no_shared_hit_at_every_precision() {
    let limit = 1i64 << 53;
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for decimals in 0..=12u8 {
        let mut catalog = catalog();
        catalog.fields.get_mut("x0").unwrap().kind = FieldKind::Numeric {
            min: -limit,
            max: limit,
            decimals,
        };
        let scale = 10f64.powi(i32::from(decimals));
        for round in 0..400 {
            // Large magnitudes near ±2^53 and small ones near zero.
            let centre = match round % 3 {
                0 => limit - 64,
                1 => -limit + 64,
                _ => 0,
            };
            let lo_a = centre - 32 + rng.below(32) as i64;
            let hi_a = lo_a + rng.below(4) as i64;
            let lo_b = hi_a + 1 + rng.below(3) as i64;
            let hi_b = lo_b + rng.below(4) as i64;
            let a = Scope::new(Endpoint::JDirect, x0(lo_a, hi_a), &catalog).unwrap();
            let b = Scope::new(Endpoint::JDirect, x0(lo_b, hi_b), &catalog).unwrap();
            assert_eq!(core_policy::overlap(&a, &b, &catalog), Overlap::Disjoint);
            let mut probes = Vec::new();
            for bound in [lo_a, hi_a, lo_b, hi_b] {
                let v = bound as f64 / scale;
                probes.extend([v, v.next_up(), v.next_down()]);
                probes.push((bound as f64 + 0.5) / scale);
            }
            for v in probes {
                let ctx = BTreeMap::from([("x0".to_owned(), FieldValue::Number(v))]);
                let hit = |s: &Scope| evaluate(&s.expr, &ctx, &catalog).unwrap() == ScopeMatch::Hit;
                assert!(
                    !(hit(&a) && hit(&b)),
                    "d={decimals} [{lo_a},{hi_a}] [{lo_b},{hi_b}] both hit by {v:e}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------- B7 ----

#[test]
fn b7_review_absolute_budget_rounding_must_be_conservative() {
    let mut cfg = config();
    cfg.endpoint_budgets.insert(Endpoint::JDirect, 1.0);
    let small = 2.0f64.powi(-53);
    let rs = [
        r_effect("a", jd(ScopeExpr::True), params(&[(0, 1.0)]), 1.0),
        r_effect("b", jd(ScopeExpr::True), params(&[(1, small)]), small),
        r_effect("c", jd(ScopeExpr::True), params(&[(2, small)]), small),
    ];
    assert!(
        compile_authorized_effects(&rs, &[], &validity(), ProcessTime(0), &cfg).is_err(),
        "exact absolute budget is 1+2^-52 but floating fold admits it"
    );
}

#[test]
fn b7_exact_totals_on_the_budget_still_pass() {
    let mut cfg = config();
    cfg.endpoint_budgets.insert(Endpoint::JDirect, 1.0);
    let rs = [
        r_effect("a", jd(ScopeExpr::True), params(&[(0, 0.5)]), 0.5),
        r_effect("b", jd(ScopeExpr::True), params(&[(1, 0.25)]), 0.25),
        r_effect("c", jd(ScopeExpr::True), params(&[(2, 0.25)]), 0.25),
    ];
    assert!(compile_authorized_effects(&rs, &[], &validity(), ProcessTime(0), &cfg).is_ok());
}

#[test]
fn b7_admission_and_amplitude_bounds_are_conservative() {
    let small = 2.0f64.powi(-53);
    let mut cfg = config();
    cfg.endpoint_budgets.insert(Endpoint::JDirect, 1.0);
    let existing = [
        r_effect("a", jd(ScopeExpr::True), params(&[(0, 1.0)]), 1.0),
        r_effect("b", jd(ScopeExpr::True), params(&[(1, small)]), small),
    ];
    let (set, _) =
        compile_authorized_effects(&existing[..1], &[], &validity(), ProcessTime(0), &cfg).unwrap();
    let s = jd(ScopeExpr::True);
    let p = params(&[(2, small)]);
    let c = CandidateEffect {
        contract: contract("c", "1", EffectOwner::R_OWNER, &s, &p, small),
        scope: s,
        parameters: p,
        conflict_set: BTreeSet::new(),
    };
    assert!(matches!(
        admit_candidate(&set, &c, &cfg),
        Err(AdmissionRejection::BudgetExceeded { .. })
    ));
    // L1 of (1, 2^-53, 2^-53) is 1+2^-52 > bound 1, though a plain fold
    // rounds it to 1.
    let over = r_effect(
        "d",
        jd(ScopeExpr::True),
        params(&[(0, 1.0), (1, small), (2, small)]),
        1.0,
    );
    assert!(matches!(
        compile(&[over], &[], &validity(), 0),
        Err(StructuralFault::Integrity { .. })
    ));
}

/// Little-endian 64-bit limbs of an exact non-negative integer.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Big(Vec<u64>);

impl Big {
    fn shifted(value: u128, shift: u32) -> Self {
        let mut limbs = vec![0u64; (shift / 64) as usize + 3];
        let base = (shift / 64) as usize;
        let bits = shift % 64;
        let wide = [value as u64, (value >> 64) as u64];
        for (i, w) in wide.into_iter().enumerate() {
            let lo = u128::from(w) << bits;
            limbs[base + i] |= lo as u64;
            limbs[base + i + 1] |= (lo >> 64) as u64;
        }
        Self(limbs)
    }
    fn add(&self, other: &Self) -> Self {
        let n = self.0.len().max(other.0.len()) + 1;
        let mut out = vec![0u64; n];
        let mut carry = 0u128;
        for (i, slot) in out.iter_mut().enumerate() {
            let a = u128::from(*self.0.get(i).unwrap_or(&0));
            let b = u128::from(*other.0.get(i).unwrap_or(&0));
            let s = a + b + carry;
            *slot = s as u64;
            carry = s >> 64;
        }
        Self(out)
    }
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let n = self.0.len().max(other.0.len());
        for i in (0..n).rev() {
            let a = self.0.get(i).unwrap_or(&0);
            let b = other.0.get(i).unwrap_or(&0);
            if a != b {
                return a.cmp(b);
            }
        }
        std::cmp::Ordering::Equal
    }
}

/// Fixed scale: every test value is an integer multiple of 2^-SCALE.
const SCALE: i32 = 300;

/// v = mantissa · 2^exponent with an integer mantissa.
fn parts(v: f64) -> (u128, i32) {
    let bits = v.to_bits();
    let e = ((bits >> 52) & 0x7ff) as i32;
    let f = u128::from(bits & ((1u64 << 52) - 1));
    (f | (1 << 52), e - 1075)
}

fn exact(v: f64) -> Big {
    let (m, e) = parts(v);
    Big::shifted(m, (e + SCALE) as u32)
}

fn exact_product(a: f64, b: f64) -> Big {
    let (ma, ea) = parts(a);
    let (mb, eb) = parts(b);
    Big::shifted(ma * mb, (ea + eb + SCALE) as u32)
}

fn random_value(rng: &mut Rng, min_exp: i32, max_exp: i32) -> f64 {
    let mantissa = (1u64 << 52) | (rng.next() >> 12);
    let span = (max_exp - min_exp + 1) as u64;
    let exp = min_exp + rng.below(span) as i32;
    libm::ldexp(mantissa as f64, exp - 52)
}

#[test]
fn b7_upper_bounds_never_fall_below_an_independent_exact_sum() {
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    for _ in 0..2_000 {
        let n = 1 + rng.below(12) as usize;
        let pairs: Vec<(f64, f64)> = (0..n)
            .map(|_| {
                (
                    random_value(&mut rng, -40, 0),
                    random_value(&mut rng, -60, 20),
                )
            })
            .collect();
        let mut exact_total = Big(vec![0]);
        for (f, b) in &pairs {
            exact_total = exact_total.add(&exact_product(*f, *b));
        }
        let upper = sum_up(pairs.iter().map(|(f, b)| mul_up(*f, *b)));
        assert_ne!(
            exact(upper).cmp(&exact_total),
            std::cmp::Ordering::Less,
            "upper bound below exact total for {pairs:?}"
        );
        let plain: f64 = pairs.iter().map(|(f, b)| f * b).sum();
        assert!(upper >= plain);
    }
    // Exact operations are not inflated.
    assert_eq!(add_up(0.5, 0.25), 0.75);
    assert_eq!(mul_up(0.5, 0.25), 0.125);
    assert_eq!(add_up(1.0, 2f64.powi(-53)), 1.0f64.next_up());
}
