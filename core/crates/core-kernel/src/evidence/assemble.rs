use super::{candidates, groups, records, *};
use crate::law::{GroupContent, MatchTarget, relation_tendency_id};
use std::collections::{BTreeMap, BTreeSet};

/// Rebuild a complete evidence view from this exact prefix. No retained mutable
/// accumulator exists, so incremental callers and replay read the same records.
/// This is a conditional pure kernel, not the missing P5-06 authenticated adapter.
pub fn assemble(prefix: Prefix<'_>) -> Result<Assembly> {
    let recorded = records::read(prefix)?;
    let catalog = candidates::build(prefix)?;
    let tainted = deception(prefix, &recorded);
    let groups = groups::build(prefix, &recorded, &catalog, &tainted)?;
    let mut external = Vec::new();
    for (i, frame) in prefix.frames.iter().enumerate() {
        let attrs = frame.attributes.as_ref().expect("validated attributes");
        let sem = frame.semantics.as_ref().expect("validated semantics");
        if !tainted.contains(&i)
            && attrs.exogenous
            && !attrs.self_narrative
            && sem.kind == EventUse::Experience
            && let Some(a) = recorded.appraisals.get(&i)
        {
            external.push(ExternalEvent {
                event_id: frame.event.id.clone(),
                position: frame.event.raw_log_position.0,
                appraisal_record: a.record_id.clone(),
                appraisal: a.appraisal.clone(),
            });
        }
    }
    let mut ops: BTreeMap<usize, Vec<(String, bool)>> = BTreeMap::new();
    for (i, id) in &catalog.proposals {
        ops.entry(*i).or_default().push((id.clone(), true));
    }
    // First involvement is the actual recorded participant position, not the
    // group's earliest member (which may precede this person's arrival).
    for (i, frame) in prefix.frames.iter().enumerate() {
        let sem = frame.semantics.as_ref().expect("validated");
        if tainted.contains(&i)
            || sem.kind != EventUse::Experience
            || !recorded.appraisals.contains_key(&i)
        {
            continue;
        }
        for (people, kind) in [
            (
                &sem.participants,
                core_types::generated::RelationKind::CLOSENESS,
            ),
            (&sem.relied_on, core_types::generated::RelationKind::TRUST),
        ] {
            for person in people {
                let person = records::canonical(prefix.person_aliases, person)?;
                let id = relation_tendency_id(kind, &person);
                if groups.iter().any(|g| matches!(&g.content, GroupContent::Experience { matches, .. }
                    if matches.iter().any(|m| matches!(&m.target, MatchTarget::Relation { person_id, kind: k }
                        if *person_id == person && *k == kind)))) {
                    ops.entry(i).or_default().push((id, false));
                }
            }
        }
    }
    let mut order = Vec::new();
    let known_relations = super::delta::relation_ids(&groups);
    let mut first_seen = BTreeMap::new();
    for (i, rows) in &ops {
        for (id, _) in rows {
            first_seen.entry(id.clone()).or_insert(*i);
        }
    }
    for (i, frame) in prefix.frames.iter().enumerate() {
        if let Some(admissions) = &frame.structural_admissions {
            let unique: BTreeSet<_> = admissions.iter().cloned().collect();
            if admissions.len() > prefix.limits.max_items
                || unique.len() != admissions.len()
                || !unique.is_subset(&known_relations)
                || admissions
                    .iter()
                    .any(|id| first_seen.get(id).is_none_or(|first| *first > i))
            {
                return Err(Fault::InvalidRecord(
                    "structural admission identities".into(),
                ));
            }
            for id in admissions {
                ops.entry(i).or_default().push((id.clone(), true));
            }
        }
    }
    for id in prefix.genesis_order {
        if id.is_empty() || order.contains(id) {
            return Err(Fault::InvalidRecord("genesis proposal order".into()));
        }
        order.push(id.clone());
    }
    for (i, rows) in ops {
        let mut unique = BTreeMap::new();
        for (id, refresh) in rows {
            if refresh || !order.contains(&id) {
                unique
                    .entry(id)
                    .and_modify(|r| *r |= refresh)
                    .or_insert(refresh);
            }
        }
        let ids = match &prefix.frames[i].simultaneous_proposals {
            Some(ids) => {
                let supplied: BTreeSet<_> = ids.iter().cloned().collect();
                if supplied.len() != ids.len() || supplied != unique.keys().cloned().collect() {
                    return Err(Fault::InvalidRecord("simultaneous proposal order".into()));
                }
                ids.clone()
            }
            None if unique.len() <= 1 => unique.keys().cloned().collect(),
            None => {
                return Err(Fault::Ambiguous(
                    "missing simultaneous proposal order".into(),
                ));
            }
        };
        for id in ids {
            if unique[&id] {
                order.retain(|old| old != &id);
            }
            if !order.contains(&id) {
                order.push(id);
            }
        }
    }
    Ok(Assembly {
        structural_admissions: prefix
            .frames
            .last()
            .and_then(|f| f.structural_admissions.clone()),
        groups,
        external,
        candidates: catalog.current,
        proposal_order: order,
        aliases: catalog.aliases,
    })
}

fn deception(prefix: Prefix<'_>, recorded: &records::Records<'_>) -> BTreeSet<usize> {
    let mut tainted = BTreeSet::new();
    let mut conversations = BTreeSet::new();
    for (i, frame) in prefix.frames.iter().enumerate() {
        let conversation = &frame.semantics.as_ref().expect("validated").conversation;
        if recorded.lies.contains(&i)
            || frame
                .event
                .parents
                .iter()
                .any(|id| tainted.contains(&recorded.index[id]))
            || conversation
                .as_ref()
                .is_some_and(|id| conversations.contains(id))
        {
            tainted.insert(i);
            if let Some(id) = conversation {
                conversations.insert(id.clone());
            }
        }
    }
    tainted
}
