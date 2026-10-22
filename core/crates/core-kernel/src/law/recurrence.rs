//! One tendency's evidence count under the v2.0 §5.2 reference form (D4).
//!
//! State lives in the IDL `TendencyV2` (ADR 0016): A_k (`evidence_a`), B_k
//! (`evidence_b`, which includes n₀ + κ_k), ĥ_k, N_k, F_k, the last nonzero sign
//! and π_k. The typical-magnitude sums M_k, U_k and r_k are updated with them
//! in `replay::record_group`.
//! The recurrence is the canonical definition; the closed form Σ Πρ·W·S is
//! never used (LAW-7). Time is not an input: without a new group nothing
//! changes (LP5).

use core_types::generated::{
    ConceptRef, EvidenceSign, NatureVector, PersonRelation, TendencyV2, TrilingualText,
};
use core_types::scalar::DecimalU64;
use core_types::semantic;

use super::error::{LawError, LawResult};
use super::num::finite;
use super::params::Law;

/// Counts stay exactly representable in binary64, so ĥ is exact in N and F.
pub const MAX_GROUPS: u64 = 1 << 53;

/// π_k = (P + A_k)/(P + B_k); P is the attachment prior, 0 elsewhere (corollary 7).
pub fn pi(prior: f64, a: f64, b: f64) -> LawResult<f64> {
    finite((prior + a) / (prior + b), "pi")
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
pub fn prior(t: &TendencyV2) -> f64 {
    t.attachment_prior.unwrap_or(0.0)
}

/// Evidence amount V_k = (B_k − c₀) + P_k used by the capacity fuse (V2-43).
pub fn evidence_amount(t: &TendencyV2, law: &Law) -> LawResult<f64> {
    finite(
        t.evidence_b - law.base(t.predicate.len())? + prior(t),
        "evidence amount",
    )
}

/// What a new tendency starts with (D4.0, D6).
pub struct Origin<'a> {
    pub tendency_id: &'a str,
    pub concept: &'a ConceptRef,
    pub predicate: &'a [TrilingualText],
    pub direction: &'a NatureVector,
    pub relation: Option<&'a PersonRelation>,
    /// A_att on the caretaker's closeness; never multiplied by ρ.
    pub prior: Option<f64>,
    /// Initial evidence that decays like any other (the caretaker's trust, D6).
    pub initial_evidence: f64,
}

pub fn initial(law: &Law, o: &Origin<'_>) -> LawResult<TendencyV2> {
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
    let start_pi = pi(p, a, b)?;
    if !(start_pi > -1.0 && start_pi < 1.0) {
        return Err(LawError::PrecisionExhausted(format!(
            "{}: starting pi = {start_pi:e}",
            o.tendency_id
        )));
    }
    Ok(TendencyV2 {
        schema_version: 1,
        tendency_id: o.tendency_id.to_owned(),
        concept: o.concept.clone(),
        predicate: o.predicate.to_vec(),
        direction: o.direction.clone(),
        evidence_a: a,
        evidence_b: b,
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
pub fn check_stored(t: &TendencyV2, law: &Law) -> LawResult<()> {
    let bad = |m: &str| LawError::InconsistentState(format!("{}: {m}", t.tendency_id));
    semantic::check_tendency_v2(t).map_err(|e| bad(&e.to_string()))?;
    let (n, f) = (t.group_count.0, t.flip_count.0);
    if n > MAX_GROUPS {
        return Err(bad("group count beyond 2^53"));
    }
    if t.evidence_b < law.base(t.predicate.len())? {
        return Err(bad("B below n0 + kappa"));
    }
    if t.h_hat.to_bits() != law.h_hat(n, f)?.to_bits() {
        return Err(bad("h_hat differs from the law"));
    }
    if t.pi.to_bits() != pi(prior(t), t.evidence_a, t.evidence_b)?.to_bits() {
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
pub fn rho(t: &TendencyV2, law: &Law) -> LawResult<f64> {
    finite(1.0 - law.h_hat(t.group_count.0, t.flip_count.0)?, "rho")
}

/// Record one group with weight W ≥ 0 and support S ∈ [−1, 1] that brings no
/// projection (D4): multiply the old evidence by the current ρ_k, then add the
/// group. M_k and U_k only decay, as for a teaching group (ADR 0016). Returns
/// the ρ used. Experience groups go through `replay::record_group` with their
/// μ and p_c.
pub fn record(t: &mut TendencyV2, law: &Law, w: f64, s: f64) -> LawResult<f64> {
    super::replay::record_group(t, law, w, s, 0.0, None)
}

/// The recurrence for a given ρ. The canonical path is `record`; tests call
/// this with a constant ρ to check the corollaries that assume one.
pub(super) fn apply(t: &mut TendencyV2, law: &Law, rho: f64, w: f64, s: f64) -> LawResult<()> {
    if !w.is_finite() || w < 0.0 || !s.is_finite() || !(-1.0..=1.0).contains(&s) {
        return Err(LawError::InvalidInput(format!(
            "{}: group needs W >= 0 and S in [-1, 1]",
            t.tendency_id
        )));
    }
    let (n, f) = (t.group_count.0, t.flip_count.0);
    let c0 = law.base(t.predicate.len())?;
    let a = finite(rho * t.evidence_a + w * s, "A")?;
    let b = finite(c0 + rho * (t.evidence_b - c0) + w * s.abs(), "B")?;
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
    if groups > MAX_GROUPS {
        return Err(LawError::Overflow("group count"));
    }
    let new_pi = pi(prior(t), a, b)?;
    if a.abs() > b || !(new_pi > -1.0 && new_pi < 1.0) {
        return Err(LawError::PrecisionExhausted(format!(
            "{}: A = {a:e}, B = {b:e}, pi = {new_pi:e}",
            t.tendency_id
        )));
    }
    t.evidence_a = a;
    t.evidence_b = b;
    t.group_count = DecimalU64(groups);
    t.flip_count = DecimalU64(flips);
    t.last_sign = last;
    t.h_hat = law.h_hat(groups, flips)?;
    t.pi = new_pi;
    Ok(())
}

/// E = Σ_{S>0} ω·W·|S| and C = Σ_{S<0} ω·W·|S| (v2.0 §5.3), from the stored sums.
pub fn consistent_and_contrary(t: &TendencyV2, law: &Law) -> LawResult<(f64, f64)> {
    let total = t.evidence_b - law.base(t.predicate.len())?;
    Ok(((total + t.evidence_a) / 2.0, (total - t.evidence_a) / 2.0))
}
