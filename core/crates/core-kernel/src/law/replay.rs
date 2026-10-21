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

use core_types::generated::{RelationKind, Tendency};

use super::error::{LawError, LawResult};
use super::fuse::check_weight_cap;
use super::num::{clip, finite};
use super::params::Law;
use super::pending::{PendingAllocation, PendingTendency};
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

/// One group with weight W, support S, μ and p_c (None for teaching) recorded into k (D4).
pub(crate) fn record_group(
    t: &mut PendingTendency,
    law: &Law,
    w: f64,
    s: f64,
    mu: f64,
    projection: Option<f64>,
) -> LawResult<()> {
    let r = rho(&t.base, law)?;
    apply(&mut t.base, law, r, w, s)?;
    let (sum, weight) = match projection {
        Some(p) => (r * t.magnitude_sum + mu * p, r * t.magnitude_weight + mu),
        None => (r * t.magnitude_sum, r * t.magnitude_weight),
    };
    t.magnitude_sum = finite(sum, "magnitude sum")?;
    t.magnitude_weight = finite(weight, "magnitude weight")?;
    t.base.typical_magnitude = typical_magnitude(law, t.magnitude_sum, t.magnitude_weight)?;
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
pub(crate) fn start(
    t: &Tendency,
    law: &Law,
    caretaker: Caretaker<'_>,
) -> LawResult<PendingTendency> {
    Ok(PendingTendency::fresh(initial(
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
    )?))
}

/// One stored allocation of k with the factors of its group.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Stored<'a> {
    pub(crate) group_id: &'a str,
    pub(crate) allocation: &'a PendingAllocation,
    pub(crate) credibility: f64,
    pub(crate) verifiability: f64,
}

/// The ordinals of k's allocations are exactly 0…N − 1 (ADR 0016 §3). Returns them sorted.
pub(crate) fn in_order<'a>(id: &str, mut list: Vec<Stored<'a>>) -> LawResult<Vec<Stored<'a>>> {
    list.sort_by_key(|s| s.allocation.ordinal);
    if list
        .iter()
        .zip(0u64..)
        .any(|(s, i)| s.allocation.ordinal != i)
    {
        return Err(LawError::InconsistentState(format!(
            "{id}: allocation ordinals are not 0..N-1"
        )));
    }
    Ok(list)
}

/// D5: k recomputed from its starting value and its allocations in ordinal order.
pub(crate) fn replay(
    t: &PendingTendency,
    allocations: Vec<Stored<'_>>,
    law: &Law,
    caretaker: Caretaker<'_>,
) -> LawResult<PendingTendency> {
    let mut out = start(&t.base, law, caretaker)?;
    for s in in_order(t.id(), allocations)? {
        let a = s.allocation;
        let w = weight(
            a.base.intensity,
            s.credibility,
            s.verifiability,
            a.base.share,
        )?;
        check_weight_cap(w, law, s.group_id, t.id())?;
        record_group(&mut out, law, w, a.base.support, a.mu, a.projection)?;
    }
    Ok(out)
}

/// A stored pending tendency agrees with the law: the IDL-level checks, the
/// typical magnitude as a function of M and U, and ordinals 0…N_k − 1.
pub(crate) fn check_pending(
    t: &PendingTendency,
    law: &Law,
    allocations: Vec<Stored<'_>>,
) -> LawResult<()> {
    let bad = |m: &str| LawError::InconsistentState(format!("{}: {m}", t.id()));
    check_stored(&t.base, law)?;
    if !t.magnitude_sum.is_finite() || !t.magnitude_weight.is_finite() || t.magnitude_weight < 0.0 {
        return Err(bad("magnitude sums must be finite, weight nonnegative"));
    }
    if t.magnitude_weight == 0.0 && t.magnitude_sum != 0.0 {
        return Err(bad("magnitude sum without magnitude weight"));
    }
    let r = typical_magnitude(law, t.magnitude_sum, t.magnitude_weight)?;
    if r.to_bits() != t.base.typical_magnitude.to_bits() {
        return Err(bad("typical magnitude differs from M/U"));
    }
    let n = u64::try_from(allocations.len()).map_err(|_| LawError::Overflow("allocations"))?;
    if n != t.base.group_count.0 {
        return Err(bad("allocation count differs from the group count"));
    }
    in_order(t.id(), allocations).map(|_| ())
}
