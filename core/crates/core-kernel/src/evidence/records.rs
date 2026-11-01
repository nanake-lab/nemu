use super::*;
use core_types::{WireMessage, generated::*, semantic};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn wire<T: WireMessage>(value: &T, prefix: Prefix<'_>) -> Result<()> {
    value
        .canonical_bytes(prefix.limits)
        .map(|_| ())
        .map_err(Fault::InvalidRecord)
}

pub(super) fn canonical(map: &BTreeMap<String, String>, id: &str) -> Result<String> {
    let mut now = id;
    let mut seen = BTreeSet::new();
    while let Some(next) = map.get(now) {
        if next == now {
            break;
        }
        if next.is_empty() || !seen.insert(now) {
            return Err(Fault::AliasCycle);
        }
        now = next;
    }
    if now.is_empty() {
        return Err(Fault::InvalidRecord("empty identity".into()));
    }
    Ok(now.to_owned())
}

pub(super) struct Records<'a> {
    pub index: BTreeMap<String, usize>,
    pub appraisals: BTreeMap<usize, &'a NatureAppraisalRecord>,
    pub matches: BTreeMap<usize, BTreeMap<String, f64>>,
    pub teaching: Vec<&'a TeachingAnnotation>,
    pub same: Vec<(usize, usize)>,
    pub lies: BTreeSet<usize>,
}

pub(super) fn read(prefix: Prefix<'_>) -> Result<Records<'_>> {
    if prefix.attribute_version.is_empty()
        || prefix.semantic_version.is_empty()
        || prefix.frames.len() > prefix.limits.max_items
    {
        return Err(Fault::InvalidRecord("version or prefix budget".into()));
    }
    for map in [prefix.concept_aliases, prefix.person_aliases] {
        if map.len() > prefix.limits.max_items {
            return Err(Fault::InvalidRecord("alias budget".into()));
        }
        for id in map.keys() {
            canonical(map, id)?;
        }
    }
    let mut out = Records {
        index: BTreeMap::new(),
        appraisals: BTreeMap::new(),
        matches: BTreeMap::new(),
        teaching: Vec::new(),
        same: Vec::new(),
        lies: BTreeSet::new(),
    };
    let mut ids = BTreeSet::new();
    let mut last = None;
    let mut branch = None;
    for (i, frame) in prefix.frames.iter().enumerate() {
        let e = &frame.event;
        wire(e, prefix)?;
        if last.is_some_and(|p| p >= e.raw_log_position.0) || out.index.contains_key(&e.id) {
            return Err(Fault::HistoryOrder);
        }
        for parent in &e.parents {
            if !out.index.contains_key(parent) {
                return Err(Fault::MissingReference(parent.clone()));
            }
        }
        // Applying effective source corrections belongs to the H/recompute owner.
        if !e.correction_of.is_empty() {
            return Err(Fault::RecomputeRequired(e.id.clone()));
        }
        last = Some(e.raw_log_position.0);
        out.index.insert(e.id.clone(), i);
        let attr = frame
            .attributes
            .as_ref()
            .ok_or_else(|| Fault::MissingAttributes(e.id.clone()))?;
        wire(attr, prefix)?;
        if attr.event_id != e.id
            || attr.rule_table_version != prefix.attribute_version
            || (attr.self_narrative && attr.exogenous)
        {
            return Err(Fault::InvalidRecord("attribute binding/partition".into()));
        }
        let sem = frame
            .semantics
            .as_ref()
            .ok_or_else(|| Fault::MissingSemantics(e.id.clone()))?;
        validate_semantics(sem, attr, &out.index, i, prefix)?;
        let mut check = |id: &str, target: &str, b: &ProposalBindingV2| -> Result<usize> {
            if !ids.insert(id.to_owned()) || ids.len() > prefix.limits.max_items {
                return Err(Fault::InvalidRecord(
                    "duplicate record or record budget".into(),
                ));
            }
            if b.event_id != e.id
                || b.raw_log_position != e.raw_log_position
                || branch.as_ref().is_some_and(|old| *old != b.branch_id)
            {
                return Err(Fault::InvalidRecord("record commit binding".into()));
            }
            branch = Some(b.branch_id.clone());
            out.index
                .get(target)
                .copied()
                .ok_or_else(|| Fault::MissingReference(target.into()))
        };
        for r in &frame.appraisals {
            wire(r, prefix)?;
            let target = check(&r.record_id, &r.event_id, &r.binding)?;
            if out.appraisals.insert(target, r).is_some() {
                return Err(Fault::RecomputeRequired(r.event_id.clone()));
            }
        }
        for r in &frame.situations {
            wire(r, prefix)?;
            semantic::check_situation_match(r).map_err(Fault::InvalidRecord)?;
            let target = check(&r.record_id, &r.event_id, &r.binding)?;
            let matches = out.matches.entry(target).or_default();
            for m in &r.matches {
                if let Some(old) = matches.insert(m.tendency_id.clone(), m.mu)
                    && old.to_bits() != m.mu.to_bits()
                {
                    return Err(Fault::RecomputeRequired(r.event_id.clone()));
                }
            }
        }
        for r in &frame.teachings {
            wire(r, prefix)?;
            check(&r.record_id, &r.event_id, &r.binding)?;
            out.teaching.push(r);
        }
        for r in &frame.same_matters {
            wire(r, prefix)?;
            let target = check(&r.record_id, &r.event_id, &r.binding)?;
            for id in &r.same_as {
                let other = *out
                    .index
                    .get(id)
                    .ok_or_else(|| Fault::MissingReference(id.clone()))?;
                out.same.push((target, other));
            }
        }
        for r in &frame.lies {
            wire(r, prefix)?;
            check(&r.record_id, &r.event_id, &r.binding)?;
            let statement = *out
                .index
                .get(&r.statement)
                .ok_or_else(|| Fault::MissingReference(r.statement.clone()))?;
            if !out.index.contains_key(&r.contradicts) {
                return Err(Fault::MissingReference(r.contradicts.clone()));
            }
            let source = &prefix.frames[statement];
            if source.attributes.as_ref().is_some_and(|a| a.exogenous) {
                return Err(Fault::InvalidRecord(
                    "lie annotation must identify her statement/action".into(),
                ));
            }
            out.lies.insert(statement);
        }
        for r in &frame.candidates {
            wire(&r.record, prefix)?;
            wire(&r.concept, prefix)?;
            semantic::check_candidate(&r.record).map_err(Fault::InvalidRecord)?;
            check(&r.record.record_id, &r.record.event_id, &r.record.binding)?;
        }
    }
    Ok(out)
}

fn validate_semantics(
    sem: &Semantics,
    attr: &EventAttributesRecord,
    index: &BTreeMap<String, usize>,
    i: usize,
    prefix: Prefix<'_>,
) -> Result<()> {
    if sem.version != prefix.semantic_version
        || sem.conversation.as_ref().is_some_and(String::is_empty)
        || (sem.kind == EventUse::OwnNarrative) != attr.self_narrative
        || (sem.kind == EventUse::Maintenance && attr.exogenous)
    {
        return Err(Fault::InvalidRecord(
            "semantic projection binding/partition".into(),
        ));
    }
    let people: BTreeSet<_> = sem
        .participants
        .iter()
        .map(|p| canonical(prefix.person_aliases, p))
        .collect::<Result<_>>()?;
    if people.len() > prefix.limits.max_items || sem.match_basis.len() > prefix.limits.max_items {
        return Err(Fault::InvalidRecord("semantic projection budget".into()));
    }
    for person in sem.relied_on.iter().chain(sem.speaker.iter()) {
        if !people.contains(&canonical(prefix.person_aliases, person)?) {
            return Err(Fault::InvalidRecord("role is not a participant".into()));
        }
    }
    for basis in sem.match_basis.values() {
        if let MatchBasis::OwnActionOutcome {
            context,
            variant,
            action_event,
        } = basis
        {
            if context.is_empty()
                || variant.as_ref().is_some_and(String::is_empty)
                || variant.is_some() != action_event.is_some()
            {
                return Err(Fault::InvalidRecord("incomplete action comparison".into()));
            }
            if let Some(id) = action_event {
                let p = *index
                    .get(id)
                    .ok_or_else(|| Fault::MissingReference(id.clone()))?;
                if p >= i || prefix.frames[p].event.kind != EventKind::ACTION_REQUEST {
                    return Err(Fault::InvalidRecord(
                        "comparison needs an earlier action record".into(),
                    ));
                }
            }
        }
    }
    Ok(())
}
