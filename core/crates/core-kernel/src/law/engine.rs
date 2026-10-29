//! G for one committed event (D2): R′ = G(R, b, σ̂, the event's evidence; 𝔏_v).
//!
//! Stages, in this order:
//! 1. new tendencies: candidates after alias canonicalisation (D10), then the
//!    structural relation tendencies of people first involved (D6);
//! 2. all old-allocation revisions, then cf, then all first allocations;
//! 3. the capacity fuse (D7);
//! 4. staged L entries (D11).
//!
//! q/V are carried by the published P5-34 V3 messages.

use crate::law::state::{EvidenceGroup, Snapshot, Tendency};
use std::collections::{BTreeMap, BTreeSet};

use core_types::generated::TendencyChange;
use core_types::semantic::tendency_order;

use super::candidates::{new_candidates, new_relations};
use super::error::{LawError, LawResult};
use super::group;
use super::input::{LawContext, RState, Step, ThetaView};
use super::replay::{Stored, replay};
use super::validation::validate;

/// R′ and what the outer transaction stages with it.
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub r: Snapshot,
    /// Group summaries created or changed in this step, by group_id order.
    pub groups: Vec<EvidenceGroup>,
    /// L payloads: one per tendency whose value changed (D11, V2-47).
    pub changes: Vec<TendencyChange>,
    /// Unpublishable audit view: all old revisions, no first allocations.
    pub counterfactual: BTreeMap<String, f64>,
    pub after_additions: BTreeMap<String, f64>,
    pub capacity_audits: Vec<CapacityAudit>,
    /// Staged notification for the outer transaction, never a growth pause.
    pub capacity_notice: Option<CapacityNotice>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CapacityNotice {
    pub capacity: u32,
    pub retained: usize,
    pub protected_candidates: u32,
    /// These proposals stay in H; a later re-proposal must collect all EV5 evidence.
    pub deferred_candidates: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CapacityAudit {
    pub removed: String,
    pub before: BTreeMap<String, f64>,
    pub after: BTreeMap<String, f64>,
}

/// G. The signature carries no x, m, g or λ (LP8, I10).
pub fn g(
    r: RState<'_>,
    theta: &ThetaView,
    step: &Step<'_>,
    ctx: &LawContext<'_>,
) -> LawResult<Output> {
    execute(r, theta, step, ctx, |work, item, remap| {
        group::prepare(work, item, theta, remap, ctx)
    })
}

/// Shared event phase machine; the test adapter supplies precomputed W/S plans
/// to compare against the scalar Python fixture without a model/appraisal step.
pub(super) fn execute<P>(
    r: RState<'_>,
    _theta: &ThetaView,
    step: &Step<'_>,
    ctx: &LawContext<'_>,
    mut prepare: P,
) -> LawResult<Output>
where
    P: FnMut(
        &mut Work,
        &super::input::GroupItem,
        &BTreeMap<String, String>,
    ) -> LawResult<Vec<group::FirstAllocation>>,
{
    validate(r, ctx)?;
    // A saturated R still consumes evidence. No new hypothesis participates in
    // its shares or q sequence. If growth in this event exhausts the available
    // slots, recompute the same pure event without new candidates before publish.
    let saturated = r
        .snapshot
        .tendencies
        .iter()
        .filter(|t| super::is_personality(t.pi))
        .count()
        >= ctx.r_capacity as usize;
    let first = execute_once(r, step, ctx, &mut prepare, !saturated)?;
    if !saturated
        && first.capacity_notice.is_some()
        && first.r.tendencies.iter().any(|t| {
            !r.snapshot
                .tendencies
                .iter()
                .any(|old| old.tendency_id == t.tendency_id)
        })
    {
        return execute_once(r, step, ctx, &mut prepare, false);
    }
    Ok(first)
}

fn execute_once<P>(
    r: RState<'_>,
    step: &Step<'_>,
    ctx: &LawContext<'_>,
    prepare: &mut P,
    admit_new: bool,
) -> LawResult<Output>
where
    P: FnMut(
        &mut Work,
        &super::input::GroupItem,
        &BTreeMap<String, String>,
    ) -> LawResult<Vec<group::FirstAllocation>>,
{
    let mut work = Work::new(r);
    let remap = new_candidates(&mut work, step, ctx)?;
    new_relations(&mut work, step, ctx)?;
    if !admit_new {
        // Canonicalise aliases before withholding the new identities. Otherwise
        // a deferred canonical proposal could make its synonyms look ambiguous.
        work.defer_new();
    }
    let items = ordered_groups(step)?;
    let initial = work.clone();
    let mut first = Vec::new();
    for item in &items {
        if initial.groups.contains_key(&item.group_id) {
            first.extend(prepare(&mut work, item, &remap)?);
        }
    }
    super::audit::revisions(&initial, &work, &work.revised, ctx)?;
    for id in work.ordered_ids() {
        if work.revised.contains(&id) {
            work.replay(&id, ctx)?;
        }
    }
    let counterfactual = work.now.iter().map(|(id, t)| (id.clone(), t.pi)).collect();
    // These allocations are appended at this committed event, not their
    // source group's historical first-member position (ADR 0019 §6).
    first.sort_by(|a, b| {
        tendency_order(&a.group_id, &b.group_id)
            .then_with(|| tendency_order(&a.allocation.tendency_id, &b.allocation.tendency_id))
    });
    for row in first {
        group::insert(&mut work, row, ctx)?;
    }
    for item in items {
        if !initial.groups.contains_key(&item.group_id) {
            let rows = prepare(&mut work, item, &remap)?;
            for row in rows {
                group::insert(&mut work, row, ctx)?;
            }
        }
    }
    let after_additions = work.now.iter().map(|(id, t)| (id.clone(), t.pi)).collect();
    super::fuse::capacity(&mut work, step, ctx)?;
    work.finish(r, step, ctx, counterfactual, after_additions)
}

/// Every stored allocation of `id` with its group's factors.
pub(super) fn stored_of<'a>(
    groups: impl Iterator<Item = &'a EvidenceGroup>,
    id: &str,
) -> Vec<Stored<'a>> {
    groups
        .flat_map(|g| {
            g.allocations
                .iter()
                .filter(move |a| a.tendency_id == id)
                .map(move |a| Stored {
                    group_id: &g.group_id,
                    allocation: a,
                    credibility: g.credibility,
                    verifiability: g.verifiability,
                })
        })
        .collect()
}

/// D2: groups by position, then group_id; a group appears at most once per step.
fn ordered_groups<'a>(step: &Step<'a>) -> LawResult<Vec<&'a super::input::GroupItem>> {
    let mut items: Vec<&super::input::GroupItem> = step.groups.iter().collect();
    items.sort_by(|a, b| {
        a.position
            .cmp(&b.position)
            .then_with(|| tendency_order(&a.group_id, &b.group_id))
    });
    let mut ids: Vec<&str> = items.iter().map(|g| g.group_id.as_str()).collect();
    ids.sort_unstable();
    if ids.windows(2).any(|p| p[0] == p[1]) {
        return Err(LawError::InvalidInput(
            "a group appears twice in one step".into(),
        ));
    }
    Ok(items)
}

#[derive(Clone)]
pub(super) struct Work {
    before: BTreeMap<String, Tendency>,
    pub(super) now: BTreeMap<String, Tendency>,
    groups_before: BTreeMap<String, EvidenceGroup>,
    pub(super) groups: BTreeMap<String, EvidenceGroup>,
    pub(super) touched: BTreeSet<String>,
    /// Old allocations changed in phase one. Unchanged sides are not replayed.
    pub(super) revised: BTreeSet<String>,
    processed: BTreeMap<String, BTreeSet<String>>,
    pub(super) capacity_audits: Vec<CapacityAudit>,
    pub(super) deferred_candidates: BTreeSet<String>,
    pub(super) capacity_notice: Option<CapacityNotice>,
}

impl Work {
    fn new(r: RState<'_>) -> Self {
        let now: BTreeMap<String, Tendency> = r
            .snapshot
            .tendencies
            .iter()
            .map(|t| (t.tendency_id.clone(), t.clone()))
            .collect();
        let groups: BTreeMap<String, EvidenceGroup> = r
            .groups
            .iter()
            .map(|g| (g.group_id.clone(), g.clone()))
            .collect();
        Self {
            before: now.clone(),
            now,
            groups_before: groups.clone(),
            groups,
            touched: BTreeSet::new(),
            revised: BTreeSet::new(),
            processed: BTreeMap::new(),
            capacity_audits: Vec::new(),
            deferred_candidates: BTreeSet::new(),
            capacity_notice: None,
        }
    }

    pub(super) fn admit(&mut self, id: String, tendency: Tendency) {
        self.now.insert(id, tendency);
    }

    fn defer_new(&mut self) {
        self.now.retain(|id, _| {
            if self.before.contains_key(id) {
                true
            } else {
                self.deferred_candidates.insert(id.clone());
                false
            }
        });
    }

    /// Tendency IDs in canonical order.
    pub(super) fn ordered_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.now.keys().cloned().collect();
        ids.sort_by(|a, b| tendency_order(a, b));
        ids
    }

    /// The tendencies of R in canonical order, as IDL values.
    pub(super) fn bases(&self) -> Vec<Tendency> {
        self.ordered_ids()
            .into_iter()
            .map(|id| self.now[&id].clone())
            .collect()
    }

    /// Records that this step processed `group` for `id` (the L entry's evidence groups).
    pub(super) fn mark(&mut self, id: &str, group: &str) {
        self.processed
            .entry(id.to_owned())
            .or_default()
            .insert(group.to_owned());
    }

    /// D5: recompute `id` from its starting value and its allocations by ordinal.
    pub(super) fn replay(&mut self, id: &str, ctx: &LawContext<'_>) -> LawResult<()> {
        let t = self.now[id].clone();
        let fresh = replay(
            &t,
            stored_of(self.groups.values(), id),
            &ctx.law,
            ctx.caretaker,
        )?;
        self.now.insert(id.to_owned(), fresh);
        Ok(())
    }

    fn finish(
        self,
        r: RState<'_>,
        step: &Step<'_>,
        ctx: &LawContext<'_>,
        counterfactual: BTreeMap<String, f64>,
        after_additions: BTreeMap<String, f64>,
    ) -> LawResult<Output> {
        let mut ids: Vec<&String> = self.before.keys().chain(self.now.keys()).collect();
        ids.sort_by(|a, b| tendency_order(a, b));
        ids.dedup();
        let mut changes = Vec::new();
        for id in ids {
            let (b, a) = (self.before.get(id), self.now.get(id));
            if b == a {
                continue;
            }
            let mut groups: Vec<String> = self
                .processed
                .get(id)
                .into_iter()
                .flatten()
                .cloned()
                .collect();
            groups.sort_by(|x, y| tendency_order(x, y));
            changes.push(TendencyChange {
                schema_version: 1,
                tendency_id: id.clone(),
                pi_before: b.map(|t| t.pi),
                pi_after: a.map(|t| t.pi),
                evidence_groups: groups,
            });
        }
        let groups: Vec<EvidenceGroup> = self
            .touched
            .iter()
            .filter(|id| self.groups.get(*id) != self.groups_before.get(*id))
            .filter_map(|id| self.groups.get(id).cloned())
            .collect();
        if changes.is_empty() && groups.is_empty() {
            return Ok(Output {
                r: r.snapshot.clone(),
                groups,
                changes,
                counterfactual,
                after_additions,
                capacity_audits: self.capacity_audits,
                capacity_notice: self.capacity_notice,
            });
        }
        if step.revision.is_empty() {
            return Err(LawError::InvalidInput("R' needs a revision".into()));
        }
        let mut tendencies: Vec<Tendency> = self.now.into_values().collect();
        tendencies.sort_by(|a, b| tendency_order(&a.tendency_id, &b.tendency_id));
        let mut groups = groups;
        groups.sort_by(|a, b| tendency_order(&a.group_id, &b.group_id));
        let next = Snapshot {
            schema_version: 1,
            revision: step.revision.to_owned(),
            law_version: ctx.law_version.clone(),
            tendencies,
        };
        let all_groups: Vec<_> = self.groups.values().cloned().collect();
        validate(
            RState {
                snapshot: &next,
                groups: &all_groups,
            },
            ctx,
        )?;
        Ok(Output {
            r: next,
            groups,
            changes,
            counterfactual,
            after_additions,
            capacity_audits: self.capacity_audits,
            capacity_notice: self.capacity_notice,
        })
    }
}
