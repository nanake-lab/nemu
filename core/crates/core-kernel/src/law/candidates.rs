//! Candidate canonicalization and structural relations (D6/D10).
use super::engine::Work;
use super::input::{GroupContent, LawContext, MatchTarget, Step};
use super::recurrence::{Origin, initial};
use super::relation::{relation_tendency, relation_tendency_id};
use super::{LawError, LawResult};
use core_types::semantic::tendency_order;
use std::collections::BTreeMap;

fn canonical<'s>(aliases: &'s BTreeMap<String, String>, concept: &'s str) -> &'s str {
    aliases.get(concept).map_or(concept, String::as_str)
}

/// Stage 1a (D10, D6): create candidates; aliased ones map onto the existing
/// tendency. Returns candidate record ID → the tendency ID it stands for.
pub(super) fn new_candidates(
    work: &mut Work,
    step: &Step<'_>,
    ctx: &LawContext<'_>,
) -> LawResult<BTreeMap<String, String>> {
    let mut order: Vec<usize> = (0..step.candidates.len()).collect();
    order.sort_by(|&a, &b| {
        let ca = &step.candidates[a];
        let cb = &step.candidates[b];
        let is_alias = |c: &super::input::Candidate| {
            canonical(step.aliases, &c.concept.concept_id) != c.concept.concept_id
        };
        // ADR 0014: extra synonym records cannot replace the actual canonical
        // candidate (and thereby turn the direct mu into a lower-priority alias).
        is_alias(ca)
            .cmp(&is_alias(cb))
            .then_with(|| tendency_order(&ca.record_id, &cb.record_id))
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
                work.admit(id.clone(), t);
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
        if canon != c.concept.concept_id
            && step.candidates.iter().any(|other| {
                other.relation.is_none()
                    && other.record_id != c.record_id
                    && canonical(step.aliases, &other.concept.concept_id) == canon
            })
        {
            // A sole alias can supply an unambiguous initial candidate. With
            // competing aliases and no canonical candidate, there is no direct
            // source to prefer; ID order must not decide the structure or mu.
            return Err(LawError::InvalidInput(format!(
                "competing aliases need a canonical candidate for {canon}"
            )));
        }
        if work.now.contains_key(&c.record_id) {
            return Err(LawError::InvalidInput(format!(
                "candidate {} collides with a tendency ID",
                c.record_id
            )));
        }
        let mut concept = c.concept.clone();
        concept.concept_id = canon.to_owned();
        let t = initial(
            &ctx.law,
            &Origin {
                tendency_id: &c.record_id,
                concept: &concept,
                predicate: &c.predicate,
                direction: &c.direction,
                relation: None,
                prior: None,
                initial_evidence: 0.0,
            },
        )?;
        work.admit(c.record_id.clone(), t);
        remap.insert(c.record_id.clone(), c.record_id.clone());
    }
    Ok(remap)
}

/// Stage 1b (D6): structural relation tendencies of people first involved.
pub(super) fn new_relations(
    work: &mut Work,
    step: &Step<'_>,
    ctx: &LawContext<'_>,
) -> LawResult<()> {
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
        if !work.now.contains_key(&id) {
            let tendency =
                relation_tendency(&ctx.law, ctx.relation_concepts, ctx.caretaker, person, kind)?;
            work.admit(id, tendency);
        }
    }
    Ok(())
}
