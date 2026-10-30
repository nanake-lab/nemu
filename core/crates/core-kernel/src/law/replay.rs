//! ADR 0019: append computes and saves q once; D5 folds stored q.
//! A/V/M/U all use that same q. Teaching adds no magnitude contribution.
//! M/U round separately: U=0 with M!=0 remains valid (ADR 0016).

use crate::law::state::{Allocation, Tendency};
use core_types::generated::RelationKind;

use super::error::{LawError, LawResult};
use super::fuse::check_weight_cap;
use super::num::{clip, finite};
use super::params::Law;
use super::recurrence::{Origin, actual_retention, check_stored, fold, initial, rho};
use super::relation::Caretaker;
use super::support::weight;

/// r_k = clip(M/U, 0, r_max); 0 while U = 0 (v2.0 §5.2).
pub(crate) fn typical_magnitude(law: &Law, sum: f64, weight: f64) -> LawResult<f64> {
    let sum = finite(sum, "M")?;
    let weight = finite(weight, "U")?;
    let cap = finite(law.r_max, "r_max")?;
    if weight == 0.0 {
        return Ok(0.0);
    }
    let ratio = finite(sum / weight, "magnitude ratio")?;
    finite(clip(ratio, 0.0, cap), "typical magnitude")
}

/// One group with weight W, support S, μ and p_c (None for teaching) recorded
/// into k (D4). Returns the actual q_k saved for historical replay.
pub(crate) fn record_group(
    t: &mut Tendency,
    law: &Law,
    w: f64,
    s: f64,
    mu: f64,
    projection: Option<f64>,
) -> LawResult<f64> {
    check_stored(t, law)?;
    let r = rho(t, law)?;
    let q = actual_retention(t, law, r, w, s)?;
    fold_group(t, law, q, w, s, mu, projection, true)?;
    Ok(q)
}

/// All scalar fields commit together, including faults in M/U after A/V.
#[allow(
    clippy::too_many_arguments,
    reason = "explicit scalar recurrence inputs, including stored q"
)]
pub(crate) fn fold_group(
    t: &mut Tendency,
    law: &Law,
    q: f64,
    w: f64,
    s: f64,
    mu: f64,
    projection: Option<f64>,
    strict: bool,
) -> LawResult<()> {
    let mu = finite(mu, "magnitude input")?;
    let projection = projection
        .map(|p| finite(p, "projection input"))
        .transpose()?;
    if !(0.0..=1.0).contains(&mu) {
        return Err(LawError::InvalidInput("invalid magnitude input".into()));
    }
    let mut next = t.clone();
    fold(&mut next, law, q, w, s, strict)?;
    let (sum, weight) = match projection {
        Some(p) => (
            finite(
                finite(q * t.magnitude_sum, "discount M")? + finite(mu * p, "projection product")?,
                "M",
            )?,
            finite(finite(q * t.magnitude_weight, "discount U")? + mu, "U")?,
        ),
        None => (
            finite(q * t.magnitude_sum, "discount M")?,
            finite(q * t.magnitude_weight, "discount U")?,
        ),
    };
    next.magnitude_sum = sum;
    next.magnitude_weight = weight;
    next.typical_magnitude = typical_magnitude(law, sum, weight)?;
    *t = next;
    Ok(())
}

/// Initial evidence of a structural tendency: A_att on u*'s trust, 0 elsewhere (D6).
fn initial_evidence(t: &Tendency, caretaker: Caretaker<'_>) -> f64 {
    match &t.relation {
        Some(r) if r.kind == RelationKind::TRUST && r.person_id == caretaker.person_id => {
            caretaker.attachment_prior
        }
        _ => 0.0,
    }
}

/// The starting value of k (D4.0), rebuilt from its definition.
pub(crate) fn start(t: &Tendency, law: &Law, caretaker: Caretaker<'_>) -> LawResult<Tendency> {
    initial(
        law,
        &Origin {
            tendency_id: &t.tendency_id,
            concept: &t.concept,
            predicate: &t.predicate,
            direction: &t.direction,
            relation: t.relation.as_ref(),
            prior: t.attachment_prior,
            initial_evidence: initial_evidence(t, caretaker),
        },
    )
}

/// One stored allocation of k with the factors of its group.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Stored<'a> {
    pub(crate) group_id: &'a str,
    pub(crate) allocation: &'a Allocation,
    pub(crate) credibility: f64,
    pub(crate) verifiability: f64,
}

/// The ordinals of k's allocations are exactly 0…N − 1 (ADR 0016 §3). Returns them sorted.
pub(crate) fn in_order<'a>(id: &str, mut list: Vec<Stored<'a>>) -> LawResult<Vec<Stored<'a>>> {
    list.sort_by_key(|s| s.allocation.ordinal.0);
    if list
        .iter()
        .zip(0u64..)
        .any(|(s, i)| s.allocation.ordinal.0 != i)
    {
        return Err(LawError::InconsistentState(format!(
            "{id}: allocation ordinals are not 0..N-1"
        )));
    }
    if list.iter().any(|s| {
        !s.allocation.actual_retention.is_finite()
            || s.allocation.actual_retention <= 0.0
            || s.allocation.actual_retention > 1.0
    }) {
        return Err(LawError::InconsistentState(format!(
            "{id}: invalid historical q"
        )));
    }
    Ok(list)
}

/// D5: k recomputed from its starting value and its allocations in ordinal order.
pub(crate) fn replay(
    t: &Tendency,
    allocations: Vec<Stored<'_>>,
    law: &Law,
    caretaker: Caretaker<'_>,
) -> LawResult<Tendency> {
    let mut out = start(t, law, caretaker)?;
    for s in in_order(t.tendency_id.as_str(), allocations)? {
        let a = s.allocation;
        let w = weight(a.intensity, s.credibility, s.verifiability, a.share)?;
        check_weight_cap(w, law, s.group_id, t.tendency_id.as_str())?;
        fold_group(
            &mut out,
            law,
            a.actual_retention,
            w,
            a.support,
            a.mu,
            a.projection,
            false,
        )?;
    }
    Ok(out)
}

/// A stored tendency agrees with the law: the IDL-level checks, the
/// typical magnitude as a function of M and U, and ordinals 0…N_k − 1
/// (ADR 0016 §3 as corrected after review).
pub(crate) fn check_with_allocations(
    t: &Tendency,
    law: &Law,
    allocations: Vec<Stored<'_>>,
) -> LawResult<()> {
    let bad = |m: &str| LawError::InconsistentState(format!("{}: {m}", t.tendency_id.as_str()));
    check_stored(t, law)?;
    if !t.magnitude_sum.is_finite() || !t.magnitude_weight.is_finite() || t.magnitude_weight < 0.0 {
        return Err(bad("magnitude sums must be finite, weight nonnegative"));
    }
    // M and U are rounded separately, so at the subnormal edge U can reach 0 while M
    // does not (review of PR #52); r_k is then 0. Only exact consequences are checked:
    // no sums before the first group, and r_k = clip(M/U, 0, r_max) or 0 while U = 0.
    if t.group_count.0 == 0 && (t.magnitude_sum != 0.0 || t.magnitude_weight != 0.0) {
        return Err(bad("magnitude sums before the first group"));
    }
    let r = typical_magnitude(law, t.magnitude_sum, t.magnitude_weight)?;
    if r.to_bits() != t.typical_magnitude.to_bits() {
        return Err(bad("typical magnitude differs from M/U"));
    }
    let n = u64::try_from(allocations.len()).map_err(|_| LawError::Overflow("allocations"))?;
    if n != t.group_count.0 {
        return Err(bad("allocation count differs from the group count"));
    }
    in_order(t.tendency_id.as_str(), allocations).map(|_| ())
}
