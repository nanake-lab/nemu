//! ADR 0019 signal-limited recurrence, finite binary64 in explicit order.
//! V is independent evidence mass; B retains its legacy meaning c+V.
//! P/c never decay. First insertions compute q; revisions fold stored q.

use crate::law::pending::Tendency;
use core_types::generated::{
    ConceptRef, EvidenceSign, NatureVector, PersonRelation, TrilingualText,
};
use core_types::scalar::DecimalU64;
use core_types::semantic;

use super::error::{LawError, LawResult};
use super::num::finite;
use super::params::Law;

/// Counts stay exactly representable in binary64, so ĥ is exact in N and F.
pub const MAX_GROUPS: u64 = 1 << 53;

/// π_k = (P + A_k)/(P + B_k); P is the attachment prior, 0 elsewhere (corollary 7).
pub fn pi(prior: f64, a: f64, c: f64, v: f64) -> LawResult<f64> {
    let numerator = finite(prior + a, "pi numerator")?;
    let anchor = finite(prior + c, "pi anchor")?;
    finite(numerator / finite(anchor + v, "pi denominator")?, "pi")
}

/// s_k = max(0, 2π_k − 1) (v2.0 §5.2).
pub fn strength(pi: f64) -> f64 {
    (2.0 * pi - 1.0).max(0.0)
}

/// π_k > ½: k is personality (R⁺); at ½ or below it is a candidate (v2.0 §7; I06).
pub fn is_personality(pi: f64) -> bool {
    pi > 0.5
}

/// The prior P_k of a stored tendency.
pub fn prior(t: &Tendency) -> f64 {
    t.attachment_prior.unwrap_or(0.0)
}

/// Capacity score V_k + P_k (V2-43); V itself excludes the fixed prior.
pub fn evidence_amount(t: &Tendency, _law: &Law) -> LawResult<f64> {
    finite(t.evidence_v + prior(t), "evidence amount")
}

/// What a new tendency starts with (D4.0, D6).
pub struct Origin<'a> {
    pub tendency_id: &'a str,
    pub concept: &'a ConceptRef,
    pub predicate: &'a [TrilingualText],
    pub direction: &'a NatureVector,
    pub relation: Option<&'a PersonRelation>,
    /// A_att on the caretaker's closeness; never multiplied by q.
    pub prior: Option<f64>,
    /// Initial evidence that decays like any other (the caretaker's trust, D6).
    pub initial_evidence: f64,
}

pub fn initial(law: &Law, o: &Origin<'_>) -> LawResult<Tendency> {
    semantic::check_direction(o.direction).map_err(|e| LawError::InvalidInput(e.to_string()))?;
    if o.predicate.is_empty() {
        return Err(LawError::InvalidInput(
            "a tendency needs a situation predicate".into(),
        ));
    }
    let init = finite(o.initial_evidence, "initial evidence")?;
    if init < 0.0 || o.prior.is_some_and(|p| !p.is_finite() || p < 0.0) {
        return Err(LawError::InvalidInput(
            "prior and initial evidence are nonnegative".into(),
        ));
    }
    let a = init;
    let b = finite(law.base(o.predicate.len())? + init, "B")?;
    let p = o.prior.unwrap_or(0.0);
    let start_pi = pi(p, a, law.base(o.predicate.len())?, init)?;
    if !(start_pi > -1.0 && start_pi < 1.0) {
        return Err(LawError::PrecisionExhausted(format!(
            "{}: starting pi = {start_pi:e}",
            o.tendency_id
        )));
    }
    Ok(Tendency {
        schema_version: 1,
        tendency_id: o.tendency_id.to_owned(),
        concept: o.concept.clone(),
        predicate: o.predicate.to_vec(),
        direction: o.direction.clone(),
        evidence_a: a,
        evidence_b: b,
        evidence_v: init,
        h_hat: law.h_hat(0, 0)?,
        group_count: DecimalU64(0),
        flip_count: DecimalU64(0),
        last_sign: None,
        pi: start_pi,
        typical_magnitude: 0.0,
        relation: o.relation.cloned(),
        attachment_prior: o.prior,
        magnitude_sum: 0.0,
        magnitude_weight: 0.0,
    })
}

/// A stored tendency must satisfy the semantic checks (V2-32, ADR 0016 §3) and agree
/// bit for bit with the law: ĥ and π are functions of the counts and sums.
pub fn check_stored(t: &Tendency, law: &Law) -> LawResult<()> {
    let bad = |m: &str| LawError::InconsistentState(format!("{}: {m}", t.tendency_id));
    semantic::check_tendency_v2(&t.legacy()).map_err(|e| bad(&e.to_string()))?;
    let (n, f) = (t.group_count.0, t.flip_count.0);
    if n >= MAX_GROUPS {
        return Err(bad("group count beyond 2^53"));
    }
    if t.evidence_b < law.base(t.predicate.len())? {
        return Err(bad("B below n0 + kappa"));
    }
    if t.h_hat.to_bits() != law.h_hat(n, f)?.to_bits() {
        return Err(bad("h_hat differs from the law"));
    }
    let c = law.base(t.predicate.len())?;
    if !t.evidence_v.is_finite()
        || t.evidence_v < t.evidence_a.abs()
        || t.evidence_b.to_bits() != finite(c + t.evidence_v, "B")?.to_bits()
    {
        return Err(bad("V inconsistent with evidence or B"));
    }
    if t.pi.to_bits() != pi(prior(t), t.evidence_a, c, t.evidence_v)?.to_bits() {
        return Err(bad("pi differs from A and B"));
    }
    if !(0.0..=law.r_max).contains(&t.typical_magnitude) {
        return Err(bad("typical magnitude outside [0, r_max]"));
    }
    Ok(())
}

fn sign_of(s: f64) -> Option<EvidenceSign> {
    if s > 0.0 {
        Some(EvidenceSign::SIGN_POSITIVE)
    } else if s < 0.0 {
        Some(EvidenceSign::SIGN_NEGATIVE)
    } else {
        None
    }
}

/// ρ_k = 1 − ĥ_k from the counts before the group (D4).
pub fn rho(t: &Tendency, law: &Law) -> LawResult<f64> {
    finite(1.0 - law.h_hat(t.group_count.0, t.flip_count.0)?, "rho")
}

/// Record one group with weight W ≥ 0 and support S ∈ [−1, 1] that brings no
/// projection (D4): multiply the old evidence by the actual q_k, then add the
/// group. M_k and U_k only decay, as for a teaching group (ADR 0016). Returns
/// q used. Experience groups go through `replay::record_group` with their
/// μ and p_c.
pub fn record(t: &mut Tendency, law: &Law, w: f64, s: f64) -> LawResult<f64> {
    super::replay::record_group(t, law, w, s, 0.0, None)
}

/// Test adapter: fixed nominal rho still goes through the signal-limited q.
#[cfg(test)]
pub(super) fn apply(t: &mut Tendency, law: &Law, rho: f64, w: f64, s: f64) -> LawResult<()> {
    let q = actual_retention(t, law, rho, w, s)?;
    super::replay::fold_group(t, law, q, w, s, 0.0, None, true)
}

/// ADR 0019 §3.2, §8: compute q exactly once, before the first insertion.
pub(super) fn actual_retention(
    t: &Tendency,
    law: &Law,
    rho: f64,
    w: f64,
    s: f64,
) -> LawResult<f64> {
    use super::num::nonnegative_product;
    if !(rho > 0.0 && rho <= 1.0) {
        return Err(LawError::NumericFault("PrecisionExhaustedRetention"));
    }
    let w = finite(w, "evidence weight")?;
    let s = finite(s, "evidence support")?;
    if w < 0.0 || !(-1.0..=1.0).contains(&s) {
        return Err(LawError::InvalidInput("invalid evidence".into()));
    }
    let before = pi(
        prior(t),
        t.evidence_a,
        law.base(t.predicate.len())?,
        t.evidence_v,
    )?;
    let h = finite(1.0 - rho, "h")?;
    let anchor = finite(prior(t) + law.base(t.predicate.len())?, "anchor")?;
    let drift = finite(
        prior(t) - finite(before * anchor, "drift product")?,
        "drift",
    )?;
    let signal = nonnegative_product(w, s.abs(), "UnderflowEvidence")?;
    let sign = if s > 0.0 {
        1.0
    } else if s < 0.0 {
        -1.0
    } else {
        0.0
    };
    let mut q = rho;
    if signal == 0.0 {
        q = 1.0;
    } else if sign * drift < 0.0 {
        let directional = finite(
            signal * finite(1.0 - sign * before, "signal direction")?,
            "directional signal",
        )?;
        let opposing = finite(h * drift.abs(), "opposing drift")?;
        let divisor = finite(
            directional + finite(2.0 * opposing, "twice drift")?,
            "retention divisor",
        )?;
        if directional <= 0.0 || divisor <= 0.0 {
            return Err(LawError::NumericFault("UnderflowGain"));
        }
        let fraction = finite(directional / divisor, "discount fraction")?;
        let discount = finite(h * fraction, "discount")?;
        q = finite(1.0 - discount, "actual retention")?;
    }
    if !(rho <= q && q <= 1.0) {
        return Err(LawError::NumericFault("PrecisionExhaustedRetention"));
    }
    Ok(q)
}

/// Fold stored q, without recalculating it or applying append-only LP1 to revisions.
pub(super) fn fold(
    t: &mut Tendency,
    law: &Law,
    q: f64,
    w: f64,
    s: f64,
    strict: bool,
) -> LawResult<()> {
    let w = finite(w, "evidence weight")?;
    let s = finite(s, "evidence support")?;
    if w < 0.0 || !(-1.0..=1.0).contains(&s) {
        return Err(LawError::InvalidInput(format!(
            "{}: group needs W >= 0 and S in [-1, 1]",
            t.tendency_id
        )));
    }
    let (n, f) = (t.group_count.0, t.flip_count.0);
    let c0 = law.base(t.predicate.len())?;
    if !(q > 0.0 && q <= 1.0) {
        return Err(LawError::NumericFault("InvalidRetention"));
    }
    let before = pi(prior(t), t.evidence_a, c0, t.evidence_v)?;
    let signed = finite(w * s, "signed signal")?;
    let signal = super::num::nonnegative_product(w, s.abs(), "UnderflowEvidence")?;
    let a = finite(finite(q * t.evidence_a, "discount A")? + signed, "A")?;
    let v = finite(finite(q * t.evidence_v, "discount V")? + signal, "V")?;
    let b = finite(c0 + v, "B")?;
    let mut flips = f;
    let mut last = t.last_sign;
    if let Some(sign) = sign_of(s) {
        if last.is_some_and(|l| l != sign) {
            flips = flips
                .checked_add(1)
                .ok_or(LawError::Overflow("flip count"))?;
        }
        last = Some(sign);
    }
    let groups = n.checked_add(1).ok_or(LawError::Overflow("group count"))?;
    if groups >= MAX_GROUPS {
        return Err(LawError::Overflow("group count"));
    }
    if a.abs() > v {
        return Err(LawError::NumericFault("PrecisionExhaustedEvidence"));
    }
    let new_pi = pi(prior(t), a, c0, v)?;
    if !(new_pi > -1.0 && new_pi < 1.0) {
        return Err(LawError::PrecisionExhausted(format!(
            "{}: A = {a:e}, B = {b:e}, pi = {new_pi:e}",
            t.tendency_id
        )));
    }
    if strict && signal > 0.0 && !((s > 0.0 && new_pi > before) || (s < 0.0 && new_pi < before)) {
        return Err(LawError::NumericFault("PrecisionExhaustedLP1"));
    }
    if strict && signal == 0.0 && new_pi.to_bits() != before.to_bits() {
        return Err(LawError::NumericFault("PrecisionExhaustedZero"));
    }
    let h_hat = law.h_hat(groups, flips)?;
    t.evidence_a = a;
    t.evidence_b = b;
    t.evidence_v = v;
    t.group_count = DecimalU64(groups);
    t.flip_count = DecimalU64(flips);
    t.last_sign = last;
    t.h_hat = h_hat;
    t.pi = new_pi;
    Ok(())
}

/// E = Σ_{S>0} ω·W·|S| and C = Σ_{S<0} ω·W·|S| (v2.0 §5.3), from the stored sums.
pub fn consistent_and_contrary(t: &Tendency, law: &Law) -> LawResult<(f64, f64)> {
    let _ = law;
    let total = t.evidence_v;
    Ok(((total + t.evidence_a) / 2.0, (total - t.evidence_a) / 2.0))
}
