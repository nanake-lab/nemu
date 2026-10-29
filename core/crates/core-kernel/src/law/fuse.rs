//! Fuses of the tendency count (v2.0 §12; V2-80/105).
//!
//! Capacity: when R holds more tendencies than the caretaker's boundary
//! allows, the candidate (π ≤ ½) with the least evidence leaves R first. Its
//! evidence stays in H and the rule is deterministic, so recomputation gives
//! the same result. Personality (π > ½) is never removed.
//!
//! Single-group cap: one group's weight never exceeds w_max.

use crate::law::state::{Allocation, Tendency};
use std::cmp::Ordering;
use std::collections::BTreeSet;

use core_types::semantic::tendency_order;

use super::engine::Work;
use super::error::{LawError, LawResult};
use super::input::{LawContext, Step};
use super::params::Law;
use super::recurrence::{evidence_amount, is_personality};
use super::shares::{ShareInput, shares};

/// Index of the tendency to remove next: the candidate with the smallest
/// evidence score V_k + P_k; ties go to the first in
/// canonical tendency-ID order. `None` when R holds no candidate.
pub fn eviction_choice(r: &[Tendency], law: &Law) -> LawResult<Option<usize>> {
    eviction_unprotected(r, law, &BTreeSet::new())
}

fn eviction_unprotected(
    r: &[Tendency],
    law: &Law,
    protected: &BTreeSet<String>,
) -> LawResult<Option<usize>> {
    let mut best: Option<(usize, f64)> = None;
    for (i, t) in r.iter().enumerate() {
        if is_personality(t.pi) || protected.contains(&t.tendency_id) {
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

/// Protect the newest K candidates using the caller's H-derived order.
/// Missing/duplicate order data is a structural input error, never guessed from
/// ID strings, evidence counts, memory, or wall-clock timestamps.
pub fn protected_candidates(
    r: &[Tendency],
    order: &[String],
    k: u32,
) -> LawResult<BTreeSet<String>> {
    if k == 0 {
        return Ok(BTreeSet::new());
    }
    let candidates: BTreeSet<&str> = r
        .iter()
        .filter(|t| !is_personality(t.pi))
        .map(|t| t.tendency_id.as_str())
        .collect();
    let supplied: BTreeSet<&str> = order.iter().map(String::as_str).collect();
    if supplied.len() != order.len() || !candidates.is_subset(&supplied) {
        return Err(LawError::InvalidInput(
            "candidate proposal order is incomplete or repeats".into(),
        ));
    }
    Ok(order
        .iter()
        .rev()
        .filter(|id| candidates.contains(id.as_str()))
        .take(k as usize)
        .cloned()
        .collect())
}

/// Stage 3: remove the weakest unprotected candidates until R fits its capacity.
/// The removed tendency's allocations leave their groups; the remaining
/// tendencies on the same side are re-shared with the current π and replayed.
pub(super) fn capacity(work: &mut Work, step: &Step<'_>, ctx: &LawContext<'_>) -> LawResult<()> {
    let cap = usize::try_from(ctx.r_capacity).map_err(|_| LawError::Overflow("capacity"))?;
    while work.now.len() > cap {
        let before_revision = work.clone();
        let ids = work.ordered_ids();
        let bases = work.bases();
        let protected =
            protected_candidates(&bases, step.proposal_order, ctx.protected_candidates)?;
        let Some(i) = eviction_unprotected(&bases, &ctx.law, &protected)? else {
            break;
        };
        let gone = ids[i].clone();
        work.now.remove(&gone);
        let mut dirty = BTreeSet::new();
        let mut removed_from = Vec::new();
        let group_ids: Vec<String> = work.groups.keys().cloned().collect();
        for gid in group_ids {
            let g = work.groups.get_mut(&gid).expect("listed");
            let Some(pos) = g.allocations.iter().position(|a| a.tendency_id == gone) else {
                continue;
            };
            let removed = g.allocations.remove(pos);
            removed_from.push(gid.clone());
            work.touched.insert(gid.clone());
            let side = removed.support;
            if side == 0.0 {
                continue;
            }
            let same: Vec<usize> = (0..g.allocations.len())
                .filter(|&k| {
                    let s = g.allocations[k].support;
                    s != 0.0 && s.signum() == side.signum()
                })
                .collect();
            if same.is_empty() {
                continue;
            }
            let inputs: Vec<ShareInput<'_>> = same
                .iter()
                .map(|&k| {
                    let a: &Allocation = &g.allocations[k];
                    ShareInput {
                        tendency_id: &a.tendency_id,
                        pi: work.now[&a.tendency_id].pi,
                        support: a.support,
                    }
                })
                .collect();
            let new_shares = shares(&inputs)?;
            let mut marked = Vec::with_capacity(same.len());
            for (&k, s) in same.iter().zip(new_shares) {
                let a = &mut g.allocations[k];
                if a.share.to_bits() != s.to_bits() {
                    a.share = s;
                    dirty.insert(a.tendency_id.clone());
                }
                marked.push(a.tendency_id.clone());
            }
            for id in marked {
                work.mark(&id, &gid);
            }
        }
        // Include every deleted allocation, including S = 0 and an empty
        // remaining side. They are evidence processed for the removed tendency.
        for gid in removed_from {
            work.mark(&gone, &gid);
        }
        super::audit::revisions(&before_revision, work, &dirty, ctx)?;
        for id in dirty {
            work.replay(&id, ctx)?;
        }
        work.capacity_audits.push(super::engine::CapacityAudit {
            removed: gone,
            before: before_revision
                .now
                .iter()
                .map(|(id, t)| (id.clone(), t.pi))
                .collect(),
            after: work.now.iter().map(|(id, t)| (id.clone(), t.pi)).collect(),
        });
    }
    if work.now.len() > cap || !work.deferred_candidates.is_empty() {
        let mut deferred: Vec<_> = work.deferred_candidates.iter().cloned().collect();
        deferred.sort_by(|a, b| tendency_order(a, b));
        work.capacity_notice = Some(super::engine::CapacityNotice {
            capacity: ctx.r_capacity,
            retained: work.now.len(),
            protected_candidates: ctx.protected_candidates,
            deferred_candidates: deferred,
        });
    }
    Ok(())
}
