//! D3: one evidence group in one step.
//!
//! The group's values are computed first, against R as it stands before the
//! group (π for shares, r̃ for support): new allocations are recorded with D4,
//! changed ones are stored in place and their tendencies replayed (D5).
//! What is recomputed when (V2-39, V2-40, V2-41):
//! - a new allocation: S, strength, p_c from this step's b, σ̂ and r̃_k;
//! - new evidence (experience group): every allocation's S, strength and p_c;
//!   whole-group repetitions only append member IDs, preserving old values;
//! - a side whose membership changed, or any side of a group whose members
//!   changed in evidence content: its shares, from this step's π;
//! - otherwise nothing: the same group arriving again changes nothing (AC2-03).

use crate::law::state::{Allocation, EvidenceGroup};
use std::collections::{BTreeMap, BTreeSet};

use core_types::scalar::DecimalU64;
use core_types::semantic::tendency_order;

use super::engine::Work;
use super::error::{LawError, LawResult};
use super::fuse::check_weight_cap;
use super::input::{GroupContent, GroupItem, LawContext, ThetaView};
use super::plan::{Plan, experience_plan, find, teaching_plan};
use super::replay::record_group;
use super::shares::{ShareInput, shares};
use super::support::weight;

/// Shares of every allocation of the planned group (V2-40).
pub(super) fn plan_shares(
    work: &Work,
    plan: &Plan,
    stored: Option<&EvidenceGroup>,
) -> LawResult<BTreeMap<String, f64>> {
    let side_of = |s: f64| {
        if s > 0.0 {
            1
        } else if s < 0.0 {
            -1
        } else {
            0
        }
    };
    let mut out: BTreeMap<String, f64> = plan.values.keys().map(|k| (k.clone(), 0.0)).collect();
    for side in [1, -1] {
        let now: BTreeSet<&String> = plan
            .values
            .iter()
            .filter(|(_, v)| side_of(v.support) == side)
            .map(|(k, _)| k)
            .collect();
        let before: BTreeSet<&String> = stored
            .into_iter()
            .flat_map(|g| &g.allocations)
            .filter(|a| side_of(a.support) == side)
            .map(|a| &a.tendency_id)
            .collect();
        if plan.reshare_all || now != before {
            let inputs: Vec<ShareInput<'_>> = now
                .iter()
                .map(|id| ShareInput {
                    tendency_id: id,
                    pi: work.now[id.as_str()].pi,
                    support: plan.values[id.as_str()].support,
                })
                .collect();
            for (id, s) in now.iter().zip(shares(&inputs)?) {
                out.insert((*id).clone(), s);
            }
        } else {
            for id in now {
                let a = find(stored, id).expect("side unchanged, so the allocation is stored");
                out.insert(id.clone(), a.share);
            }
        }
    }
    Ok(out)
}

/// Process one group (D3). New allocations are recorded; changed ones are
/// stored and their tendencies replayed before the next group.
pub(super) fn prepare(
    work: &mut Work,
    item: &GroupItem,
    theta: &ThetaView,
    remap: &BTreeMap<String, String>,
    ctx: &LawContext<'_>,
) -> LawResult<Vec<FirstAllocation>> {
    let mut members = item.members.clone();
    members.sort_by(|a, b| tendency_order(a, b));
    if members.is_empty() || members.windows(2).any(|p| p[0] == p[1]) {
        return Err(LawError::InvalidInput(format!(
            "group {} needs distinct members",
            item.group_id
        )));
    }
    let stored = work.groups.get(&item.group_id).cloned();
    if let Some(g) = &stored
        && g.source_key != item.source_key
    {
        return Err(LawError::InconsistentState(format!(
            "group {} changed its source key",
            item.group_id
        )));
    }
    // A group keeps its kind: experience allocations carry p_c, teaching ones do not.
    let teaching_group = matches!(item.content, GroupContent::Teaching { .. });
    if let Some(g) = &stored
        && g.allocations
            .iter()
            .any(|a| a.projection.is_none() != teaching_group)
    {
        return Err(LawError::InconsistentState(format!(
            "group {} changed between experience and teaching",
            item.group_id
        )));
    }
    let plan = match item.content {
        GroupContent::Experience { .. } => {
            experience_plan(work, item, stored.as_ref(), &members, theta, remap, ctx)?
        }
        GroupContent::Teaching { .. } => teaching_plan(work, item, stored.as_ref(), remap, ctx)?,
    };
    store_plan(work, item, stored, members, plan, ctx)
}

pub(super) fn store_plan(
    work: &mut Work,
    item: &GroupItem,
    stored: Option<EvidenceGroup>,
    members: Vec<String>,
    plan: Plan,
    ctx: &LawContext<'_>,
) -> LawResult<Vec<FirstAllocation>> {
    let law = &ctx.law;
    let share = plan_shares(work, &plan, stored.as_ref())?;
    let factors_changed = stored.as_ref().is_some_and(|g| {
        g.credibility != plan.credibility || g.verifiability != plan.verifiability
    });
    let mut ids: Vec<&String> = plan.values.keys().collect();
    ids.sort_by(|a, b| tendency_order(a, b));
    let mut allocations = Vec::with_capacity(ids.len());
    let mut first = Vec::new();
    for id in ids {
        let v = plan.values[id];
        let w = weight(v.intensity, plan.credibility, plan.verifiability, share[id])?;
        let allocation = |ordinal: u64, q: f64| Allocation {
            schema_version: 1,
            tendency_id: id.clone(),
            support: v.support,
            intensity: v.intensity,
            share: share[id],
            ordinal: DecimalU64(ordinal),
            mu: v.mu,
            projection: v.projection,
            actual_retention: q,
        };
        check_weight_cap(w, law, &item.group_id, id)?;
        match find(stored.as_ref(), id) {
            None => {
                first.push(FirstAllocation {
                    group_id: item.group_id.clone(),
                    allocation: allocation(0, 1.0),
                });
            }
            Some(old) => {
                let new = allocation(old.ordinal.0, old.actual_retention);
                if new != *old || factors_changed {
                    work.revised.insert(id.clone());
                    work.mark(id, &item.group_id);
                }
                allocations.push(new);
            }
        }
    }
    let group = EvidenceGroup {
        schema_version: 1,
        group_id: item.group_id.clone(),
        members,
        source_key: item.source_key.clone(),
        credibility: plan.credibility,
        verifiability: plan.verifiability,
        allocations,
    };
    if stored.as_ref() != Some(&group) {
        work.touched.insert(item.group_id.clone());
    }
    work.groups.insert(item.group_id.clone(), group);
    Ok(first)
}

/// New evidence, including first allocations into a historical group.
pub(super) struct FirstAllocation {
    pub(super) group_id: String,
    pub(super) allocation: Allocation,
}

pub(super) fn insert(
    work: &mut Work,
    mut row: FirstAllocation,
    ctx: &LawContext<'_>,
) -> LawResult<()> {
    let id = row.allocation.tendency_id.clone();
    let g = &work.groups[&row.group_id];
    let a = &mut row.allocation;
    let w = weight(a.intensity, g.credibility, g.verifiability, a.share)?;
    let t = work.now.get_mut(&id).expect("planned tendency");
    a.ordinal = DecimalU64(t.group_count.0);
    a.actual_retention = record_group(t, &ctx.law, w, a.support, a.mu, a.projection)?;
    let g = work.groups.get_mut(&row.group_id).expect("planned group");
    g.allocations.push(row.allocation);
    g.allocations
        .sort_by(|a, b| tendency_order(&a.tendency_id, &b.tendency_id));
    work.mark(&id, &row.group_id);
    work.touched.insert(row.group_id);
    Ok(())
}
