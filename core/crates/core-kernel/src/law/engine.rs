//! G for one committed event (D2): R′ = G(R, b, σ̂, the event's evidence; 𝔏_v).
//!
//! Stages, in this order:
//! 1. new tendencies: candidates after alias canonicalisation (D10), then the
//!    structural relation tendencies of people first involved (D6);
//! 2. evidence groups in H order (D3–D5; `group.rs`);
//! 3. the capacity fuse (D7);
//! 4. staged L entries (D11).
//!
//! R is held in the crate-internal pending structures until P5-32 merges
//! (`pending.rs`); G is therefore crate-internal too until then.

use std::collections::{BTreeMap, BTreeSet};

use core_types::generated::{RSnapshot, Tendency, TendencyChange};
use core_types::semantic::tendency_order;

use super::error::{LawError, LawResult};
use super::group;
use super::input::{GroupContent, LawContext, MatchTarget, RState, Step, ThetaView};
use super::pending::{PendingGroup, PendingSnapshot, PendingTendency};
use super::recurrence::{Origin, initial};
use super::relation::{relation_tendency, relation_tendency_id};
use super::replay::{Stored, check_pending, replay};

/// R′ and what the outer transaction stages with it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Output {
    pub(crate) r: PendingSnapshot,
    /// Group summaries created or changed in this step, by group_id order.
    pub(crate) groups: Vec<PendingGroup>,
    /// L payloads: one per tendency whose value changed (D11, V2-47).
    pub(crate) changes: Vec<TendencyChange>,
}

/// G. The signature carries no x, m, g or λ (LP8, I10).
pub(crate) fn g(
    r: RState<'_>,
    theta: &ThetaView,
    step: &Step<'_>,
    ctx: &LawContext<'_>,
) -> LawResult<Output> {
    validate(r, ctx)?;
    let mut work = Work::new(r);
    let remap = new_candidates(&mut work, step, ctx)?;
    new_relations(&mut work, step, ctx)?;
    for item in ordered_groups(step)? {
        group::process(&mut work, item, theta, &remap, ctx)?;
    }
    super::fuse::capacity(&mut work, ctx)?;
    work.finish(r, step, ctx)
}

fn canonical<'s>(aliases: &'s BTreeMap<String, String>, concept: &'s str) -> &'s str {
    aliases.get(concept).map_or(concept, String::as_str)
}

/// Every stored allocation of `id` with its group's factors.
fn stored_of<'a>(groups: impl Iterator<Item = &'a PendingGroup>, id: &str) -> Vec<Stored<'a>> {
    groups
        .flat_map(|g| {
            g.allocations
                .iter()
                .filter(move |a| a.base.tendency_id == id)
                .map(move |a| Stored {
                    group_id: &g.base.group_id,
                    allocation: a,
                    credibility: g.base.credibility,
                    verifiability: g.base.verifiability,
                })
        })
        .collect()
}

fn validate(r: RState<'_>, ctx: &LawContext<'_>) -> LawResult<()> {
    if r.snapshot.base.law_version != *ctx.law_version {
        return Err(LawError::InvalidInput(
            "R was computed under another law version; recompute it (v2.0 §5.4)".into(),
        ));
    }
    if !r.snapshot.base.tendencies.is_empty()
        || r.groups.iter().any(|g| !g.base.allocations.is_empty())
    {
        return Err(LawError::InconsistentState(
            "pending R carries its values outside the IDL lists".into(),
        ));
    }
    let ts = &r.snapshot.tendencies;
    if ts
        .windows(2)
        .any(|p| !tendency_order(p[0].id(), p[1].id()).is_lt())
    {
        return Err(LawError::InconsistentState(
            "R's tendencies are not in canonical order or repeat".into(),
        ));
    }
    let ids: BTreeSet<&str> = ts.iter().map(PendingTendency::id).collect();
    let mut groups = BTreeSet::new();
    for g in r.groups {
        if !groups.insert(g.base.group_id.as_str()) {
            return Err(LawError::InconsistentState(format!(
                "group {} repeats",
                g.base.group_id
            )));
        }
        let mut seen = BTreeSet::new();
        for a in &g.allocations {
            if !ids.contains(a.base.tendency_id.as_str()) || !seen.insert(&a.base.tendency_id) {
                return Err(LawError::InconsistentState(format!(
                    "group {} allocates to {} twice or outside R",
                    g.base.group_id, a.base.tendency_id
                )));
            }
        }
    }
    for t in ts {
        check_pending(t, &ctx.law, stored_of(r.groups.iter(), t.id()))?;
    }
    Ok(())
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

pub(super) struct Work {
    before: BTreeMap<String, PendingTendency>,
    pub(super) now: BTreeMap<String, PendingTendency>,
    groups_before: BTreeMap<String, PendingGroup>,
    pub(super) groups: BTreeMap<String, PendingGroup>,
    pub(super) touched: BTreeSet<String>,
    processed: BTreeMap<String, BTreeSet<String>>,
}

impl Work {
    fn new(r: RState<'_>) -> Self {
        let now: BTreeMap<String, PendingTendency> = r
            .snapshot
            .tendencies
            .iter()
            .map(|t| (t.id().to_owned(), t.clone()))
            .collect();
        let groups: BTreeMap<String, PendingGroup> = r
            .groups
            .iter()
            .map(|g| (g.base.group_id.clone(), g.clone()))
            .collect();
        Self {
            before: now.clone(),
            now,
            groups_before: groups.clone(),
            groups,
            touched: BTreeSet::new(),
            processed: BTreeMap::new(),
        }
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
            .map(|id| self.now[&id].base.clone())
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

    fn finish(self, r: RState<'_>, step: &Step<'_>, ctx: &LawContext<'_>) -> LawResult<Output> {
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
                pi_before: b.map(|t| t.base.pi),
                pi_after: a.map(|t| t.base.pi),
                evidence_groups: groups,
            });
        }
        let groups: Vec<PendingGroup> = self
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
            });
        }
        if step.revision.is_empty() {
            return Err(LawError::InvalidInput("R' needs a revision".into()));
        }
        let mut tendencies: Vec<PendingTendency> = self.now.into_values().collect();
        tendencies.sort_by(|a, b| tendency_order(a.id(), b.id()));
        let mut groups = groups;
        groups.sort_by(|a, b| tendency_order(&a.base.group_id, &b.base.group_id));
        Ok(Output {
            r: PendingSnapshot {
                base: RSnapshot {
                    schema_version: 1,
                    revision: step.revision.to_owned(),
                    law_version: ctx.law_version.clone(),
                    tendencies: Vec::new(),
                },
                tendencies,
            },
            groups,
            changes,
        })
    }
}

/// Stage 1a (D10, D6): create candidates; aliased ones map onto the existing
/// tendency. Returns candidate record ID → the tendency ID it stands for.
fn new_candidates(
    work: &mut Work,
    step: &Step<'_>,
    ctx: &LawContext<'_>,
) -> LawResult<BTreeMap<String, String>> {
    let mut order: Vec<usize> = (0..step.candidates.len()).collect();
    order.sort_by(|&a, &b| {
        tendency_order(&step.candidates[a].record_id, &step.candidates[b].record_id)
    });
    let mut remap = BTreeMap::new();
    for i in order {
        let c = &step.candidates[i];
        if remap.contains_key(&c.record_id) {
            return Err(LawError::InvalidInput(format!(
                "candidate {} repeats",
                c.record_id
            )));
        }
        if let Some(rel) = &c.relation {
            let id = relation_tendency_id(rel.kind, &rel.person_id);
            if !work.now.contains_key(&id) {
                let t = relation_tendency(
                    &ctx.law,
                    ctx.relation_concepts,
                    ctx.caretaker,
                    &rel.person_id,
                    rel.kind,
                )?;
                work.now.insert(id.clone(), PendingTendency::fresh(t));
            }
            remap.insert(c.record_id.clone(), id);
            continue;
        }
        let canon = canonical(step.aliases, &c.concept.concept_id);
        let existing = work
            .ordered_ids()
            .into_iter()
            .find(|id| canonical(step.aliases, &work.now[id].base.concept.concept_id) == canon);
        if let Some(id) = existing {
            remap.insert(c.record_id.clone(), id);
            continue;
        }
        if work.now.contains_key(&c.record_id) {
            return Err(LawError::InvalidInput(format!(
                "candidate {} collides with a tendency ID",
                c.record_id
            )));
        }
        let t = initial(
            &ctx.law,
            &Origin {
                tendency_id: &c.record_id,
                concept: &c.concept,
                predicate: &c.predicate,
                direction: &c.direction,
                relation: None,
                prior: None,
                initial_evidence: 0.0,
            },
        )?;
        work.now
            .insert(c.record_id.clone(), PendingTendency::fresh(t));
        remap.insert(c.record_id.clone(), c.record_id.clone());
    }
    Ok(remap)
}

/// Stage 1b (D6): structural relation tendencies of people first involved.
fn new_relations(work: &mut Work, step: &Step<'_>, ctx: &LawContext<'_>) -> LawResult<()> {
    let mut wanted = Vec::new();
    for g in step.groups {
        if let GroupContent::Experience { matches, .. } = &g.content {
            for m in matches {
                if let MatchTarget::Relation { person_id, kind } = &m.target {
                    wanted.push((relation_tendency_id(*kind, person_id), person_id, *kind));
                }
            }
        }
    }
    wanted.sort_by(|a, b| tendency_order(&a.0, &b.0));
    wanted.dedup_by(|a, b| a.0 == b.0);
    for (id, person, kind) in wanted {
        if let std::collections::btree_map::Entry::Vacant(slot) = work.now.entry(id) {
            slot.insert(PendingTendency::fresh(relation_tendency(
                &ctx.law,
                ctx.relation_concepts,
                ctx.caretaker,
                person,
                kind,
            )?));
        }
    }
    Ok(())
}
