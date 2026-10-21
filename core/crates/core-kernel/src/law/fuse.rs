//! Fuses of the tendency count (v2.0 §12; D7, D8, V2-43).
//!
//! Capacity: when R holds more tendencies than the caretaker's boundary
//! allows, the candidate (π ≤ ½) with the least evidence leaves R first. Its
//! evidence stays in H and the rule is deterministic, so recomputation gives
//! the same result. Personality (π > ½) is never removed.
//!
//! Single-group cap: one group's weight never exceeds w_max.

use std::cmp::Ordering;

use core_types::generated::Tendency;
use core_types::semantic::tendency_order;

use super::error::{LawError, LawResult};
use super::params::Law;
use super::recurrence::{evidence_amount, is_personality};

/// Index of the tendency to remove next: the candidate with the smallest
/// evidence amount V_k = (B_k − n₀ − κ_k) + A_att; ties go to the first in
/// canonical tendency-ID order. `None` when R holds no candidate.
pub fn eviction_choice(r: &[Tendency], law: &Law) -> LawResult<Option<usize>> {
    let mut best: Option<(usize, f64)> = None;
    for (i, t) in r.iter().enumerate() {
        if is_personality(t.pi) {
            continue;
        }
        let v = evidence_amount(t, law)?;
        let better = match best {
            None => true,
            Some((j, bv)) => match v.total_cmp(&bv) {
                Ordering::Less => true,
                Ordering::Greater => false,
                Ordering::Equal => tendency_order(&t.tendency_id, &r[j].tendency_id).is_lt(),
            },
        };
        if better {
            best = Some((i, v));
        }
    }
    Ok(best.map(|(i, _)| i))
}

/// The capacity fault when nothing can be removed.
pub fn exhausted(r: &[Tendency], capacity: u32) -> LawError {
    LawError::CapacityExhausted {
        capacity,
        personality: r.iter().filter(|t| is_personality(t.pi)).count(),
    }
}

/// D8: W ≤ w_max for every allocation.
pub fn check_weight_cap(w: f64, law: &Law, group_id: &str, tendency_id: &str) -> LawResult<()> {
    if w > law.w_max {
        return Err(LawError::WeightAboveCap {
            group_id: group_id.to_owned(),
            tendency_id: tendency_id.to_owned(),
        });
    }
    Ok(())
}
