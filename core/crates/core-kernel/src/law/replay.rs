//! The typical-magnitude recurrence and the replay of one tendency (D4, D5; ADR 0016).
//!
//! Recording a group multiplies the old evidence by ρ_k and adds the group to A, B
//! (`recurrence::apply`) and to the typical-magnitude sums:
//!
//! ```text
//! M ← ρ·M + μ·p_c,  U ← ρ·U + μ          (experience groups)
//! M ← ρ·M,          U ← ρ·U              (teaching groups: they decay r_k's weights, add nothing)
//! r_k = clip(M/U, 0, r_max), 0 while U = 0
//! ```
//!
//! A replay starts from the tendency's starting value (D4.0, D6) and records its
//! allocations again in ordinal order, from the stored S, strength, share,
//! credibility, verifiability, μ and p_c only. With unchanged values it reproduces
//! the stored tendency bit for bit.

use core_types::generated::{EvidenceAllocationV2, RelationKind, TendencyV2};

use super::error::{LawError, LawResult};
use super::fuse::check_weight_cap;
use super::num::{clip, finite};
use super::params::Law;
use super::recurrence::{Origin, apply, check_stored, initial, rho};
use super::relation::Caretaker;
use super::support::weight;

/// r_k = clip(M/U, 0, r_max); 0 while U = 0 (v2.0 §5.2).
pub(crate) fn typical_magnitude(law: &Law, sum: f64, weight: f64) -> LawResult<f64> {
    if weight == 0.0 {
        return Ok(0.0);
    }
    finite(clip(sum / weight, 0.0, law.r_max), "typical magnitude")
}

/// One group with weight W, support S, μ and p_c (None for teaching) recorded
/// into k (D4). Returns the ρ_k used.
pub(crate) fn record_group(
    t: &mut TendencyV2,
    law: &Law,
    w: f64,
    s: f64,
    mu: f64,
    projection: Option<f64>,
) -> LawResult<f64> {
    let r = rho(t, law)?;
    apply(t, law, r, w, s)?;
    let (sum, weight) = match projection {
        Some(p) => (r * t.magnitude_sum + mu * p, r * t.magnitude_weight + mu),
        None => (r * t.magnitude_sum, r * t.magnitude_weight),
    };
    t.magnitude_sum = finite(sum, "magnitude sum")?;
    t.magnitude_weight = finite(weight, "magnitude weight")?;
    t.typical_magnitude = typical_magnitude(law, t.magnitude_sum, t.magnitude_weight)?;
    Ok(r)
}

/// Initial evidence of a structural tendency: A_att on u*'s trust, 0 elsewhere (D6).
fn initial_evidence(t: &TendencyV2, caretaker: Caretaker<'_>) -> f64 {
    match &t.relation {
        Some(r) if r.kind == RelationKind::TRUST && r.person_id == caretaker.person_id => {
            caretaker.attachment_prior
        }
        _ => 0.0,
    }
}

/// The starting value of k (D4.0), rebuilt from its definition.
pub(crate) fn start(t: &TendencyV2, law: &Law, caretaker: Caretaker<'_>) -> LawResult<TendencyV2> {
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
    pub(crate) allocation: &'a EvidenceAllocationV2,
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
    Ok(list)
}

/// D5: k recomputed from its starting value and its allocations in ordinal order.
pub(crate) fn replay(
    t: &TendencyV2,
    allocations: Vec<Stored<'_>>,
    law: &Law,
    caretaker: Caretaker<'_>,
) -> LawResult<TendencyV2> {
    let mut out = start(t, law, caretaker)?;
    for s in in_order(t.tendency_id.as_str(), allocations)? {
        let a = s.allocation;
        let w = weight(a.intensity, s.credibility, s.verifiability, a.share)?;
        check_weight_cap(w, law, s.group_id, t.tendency_id.as_str())?;
        record_group(&mut out, law, w, a.support, a.mu, a.projection)?;
    }
    Ok(out)
}

/// A stored tendency agrees with the law: the IDL-level checks, the
/// typical magnitude as a function of M and U, and ordinals 0…N_k − 1
/// (ADR 0016 §3 as corrected after review).
pub(crate) fn check_with_allocations(
    t: &TendencyV2,
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
