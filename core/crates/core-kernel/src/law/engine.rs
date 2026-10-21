//! G for one committed event (D2): R′ = G(R, b, σ̂, the event's evidence; 𝔏_v).
//!
//! Stages, in this order:
//! 1. new tendencies: candidates after alias canonicalisation (D10), then the
//!    structural relation tendencies of people first involved (D6);
//! 2. evidence groups in H order (D3–D5);
//! 3. the capacity fuse (D7);
//! 4. staged L entries (D11).
//!
//! Stage 2 and the re-sharing after a capacity removal need R fields that the
//! IDL lacks (V2-48); they return `LawError::ContractGap` instead of computing
//! with state that would not survive storage.

use std::collections::{BTreeMap, BTreeSet};

use core_types::generated::{EvidenceGroup, RSnapshot, Tendency, TendencyChange};
use core_types::semantic::tendency_order;

use super::error::{LawError, LawResult};
use super::fuse::{eviction_choice, exhausted};
use super::input::{GroupContent, LawContext, MatchTarget, RState, Step, ThetaView};
use super::recurrence::{Origin, check_stored, initial};
use super::relation::{relation_tendency, relation_tendency_id};

/// R′ and what the outer transaction stages with it.
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub r: RSnapshot,
    /// Group summaries created or changed in this step, by group_id order.
    pub groups: Vec<EvidenceGroup>,
    /// L payloads: one per tendency whose value changed (D11, V2-47).
    pub changes: Vec<TendencyChange>,
}

/// G. The signature carries no x, m, g or λ (LP8, I10).
pub fn g(
    r: RState<'_>,
    theta: &ThetaView,
    step: &Step<'_>,
    ctx: &LawContext<'_>,
) -> LawResult<Output> {
    validate(r, ctx)?;
    let mut work = Work::new(r);
    let remap = new_candidates(&mut work, step, ctx)?;
    new_relations(&mut work, step, ctx)?;
    groups(&mut work, theta, step, &remap)?;
    capacity(&mut work, ctx)?;
    work.finish(r, step, ctx)
}

fn canonical<'s>(aliases: &'s BTreeMap<String, String>, concept: &'s str) -> &'s str {
    aliases.get(concept).map_or(concept, String::as_str)
}

fn validate(r: RState<'_>, ctx: &LawContext<'_>) -> LawResult<()> {
    if r.snapshot.law_version != *ctx.law_version {
        return Err(LawError::InvalidInput(
            "R was computed under another law version; recompute it (v2.0 §5.4)".into(),
        ));
    }
    let ts = &r.snapshot.tendencies;
    if ts
        .windows(2)
        .any(|p| !tendency_order(&p[0].tendency_id, &p[1].tendency_id).is_lt())
    {
        return Err(LawError::InconsistentState(
            "R's tendencies are not in canonical order or repeat".into(),
        ));
    }
    for t in ts {
        check_stored(t, &ctx.law)?;
    }
    let ids: BTreeSet<&str> = ts.iter().map(|t| t.tendency_id.as_str()).collect();
    let mut groups = BTreeSet::new();
    for g in r.groups {
        if !groups.insert(g.group_id.as_str()) {
            return Err(LawError::InconsistentState(format!(
                "group {} repeats",
                g.group_id
            )));
        }
        if let Some(a) = g
            .allocations
            .iter()
            .find(|a| !ids.contains(a.tendency_id.as_str()))
        {
            return Err(LawError::InconsistentState(format!(
                "group {} allocates to {}, which is not in R",
                g.group_id, a.tendency_id
            )));
        }
    }
    Ok(())
}

struct Work {
    before: BTreeMap<String, Tendency>,
    now: BTreeMap<String, Tendency>,
    groups: BTreeMap<String, EvidenceGroup>,
    touched: BTreeSet<String>,
    processed: BTreeMap<String, BTreeSet<String>>,
}

impl Work {
    fn new(r: RState<'_>) -> Self {
        let now: BTreeMap<String, Tendency> = r
            .snapshot
            .tendencies
            .iter()
            .map(|t| (t.tendency_id.clone(), t.clone()))
            .collect();
        Self {
            before: now.clone(),
            now,
            groups: r
                .groups
                .iter()
                .map(|g| (g.group_id.clone(), g.clone()))
                .collect(),
            touched: BTreeSet::new(),
            processed: BTreeMap::new(),
        }
    }

    /// Tendency IDs in canonical order.
    fn ordered_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.now.keys().cloned().collect();
        ids.sort_by(|a, b| tendency_order(a, b));
        ids
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
                pi_before: b.map(|t| t.pi),
                pi_after: a.map(|t| t.pi),
                evidence_groups: groups,
            });
        }
        if changes.is_empty() && self.touched.is_empty() {
            return Ok(Output {
                r: r.snapshot.clone(),
                groups: Vec::new(),
                changes,
            });
        }
        if step.revision.is_empty() {
            return Err(LawError::InvalidInput("R' needs a revision".into()));
        }
        let mut tendencies: Vec<Tendency> = self.now.into_values().collect();
        tendencies.sort_by(|a, b| tendency_order(&a.tendency_id, &b.tendency_id));
        let mut groups: Vec<EvidenceGroup> = self
            .touched
            .iter()
            .filter_map(|id| self.groups.get(id).cloned())
            .collect();
        groups.sort_by(|a, b| tendency_order(&a.group_id, &b.group_id));
        Ok(Output {
            r: RSnapshot {
                schema_version: 1,
                revision: step.revision.to_owned(),
                law_version: ctx.law_version.clone(),
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
                work.now.insert(id.clone(), t);
            }
            remap.insert(c.record_id.clone(), id);
            continue;
        }
        let canon = canonical(step.aliases, &c.concept.concept_id);
        let existing = work
            .ordered_ids()
            .into_iter()
            .find(|id| canonical(step.aliases, &work.now[id].concept.concept_id) == canon);
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
        work.now.insert(c.record_id.clone(), t);
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
            slot.insert(relation_tendency(
                &ctx.law,
                ctx.relation_concepts,
                ctx.caretaker,
                person,
                kind,
            )?);
        }
    }
    Ok(())
}

/// Stage 2 (D3–D5): every group updates r_k and may need a replay, which
/// needs the R fields of V2-48. Until the contract has them this is an
/// explicit fault, never an approximation.
fn groups(
    _work: &mut Work,
    _theta: &ThetaView,
    step: &Step<'_>,
    _remap: &BTreeMap<String, String>,
) -> LawResult<()> {
    if step.groups.is_empty() {
        return Ok(());
    }
    Err(LawError::ContractGap(
        "evidence groups need the typical-magnitude sums and allocation ordinals",
    ))
}

/// Stage 3 (D7): remove the weakest candidates until R fits its capacity.
fn capacity(work: &mut Work, ctx: &LawContext<'_>) -> LawResult<()> {
    let cap = usize::try_from(ctx.r_capacity).map_err(|_| LawError::Overflow("capacity"))?;
    while work.now.len() > cap {
        let ids = work.ordered_ids();
        let ts: Vec<Tendency> = ids.iter().map(|id| work.now[id].clone()).collect();
        let Some(i) = eviction_choice(&ts, &ctx.law)? else {
            return Err(exhausted(&ts, ctx.r_capacity));
        };
        let gone = ids[i].clone();
        for (gid, g) in &mut work.groups {
            let Some(pos) = g.allocations.iter().position(|a| a.tendency_id == gone) else {
                continue;
            };
            let side = g.allocations[pos].support.signum();
            let shared = g
                .allocations
                .iter()
                .any(|a| a.tendency_id != gone && a.support != 0.0 && a.support.signum() == side);
            if shared && g.allocations[pos].support != 0.0 {
                return Err(LawError::ContractGap(
                    "re-sharing a group after a capacity removal needs allocation ordinals",
                ));
            }
            g.allocations.remove(pos);
            work.touched.insert(gid.clone());
        }
        work.now.remove(&gone);
    }
    Ok(())
}
