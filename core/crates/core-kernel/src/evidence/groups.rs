use super::{
    candidates::Catalog,
    eligibility,
    records::{Records, canonical},
    *,
};
use crate::law::{GroupContent, GroupItem, Match, MatchTarget, relation_tendency_id};
use core_types::{generated::EpistemicType, semantic::tendency_order};
use std::collections::{BTreeMap, BTreeSet};

fn key(parts: &[&str]) -> String {
    parts.iter().map(|s| format!("{}:{s}", s.len())).collect()
}

fn target_id(m: &Match) -> String {
    match &m.target {
        MatchTarget::Tendency(id) => id.clone(),
        MatchTarget::Relation { person_id, kind } => relation_tendency_id(*kind, person_id),
    }
}

fn root(parents: &[usize], mut i: usize) -> usize {
    while parents[i] != i {
        i = parents[i];
    }
    i
}
fn union(parents: &mut [usize], a: usize, b: usize) {
    let (a, b) = (root(parents, a), root(parents, b));
    parents[a.max(b)] = a.min(b);
}

pub(super) fn build(
    prefix: Prefix<'_>,
    records: &Records<'_>,
    catalog: &Catalog,
    tainted: &BTreeSet<usize>,
) -> Result<Vec<GroupItem>> {
    let mut parents: Vec<usize> = (0..prefix.frames.len()).collect();
    let mut keys = BTreeMap::new();
    for (i, frame) in prefix.frames.iter().enumerate() {
        if !eligibility::connectable(i, prefix, tainted) {
            continue;
        }
        let attr = frame.attributes.as_ref().expect("validated");
        let sem = frame.semantics.as_ref().expect("validated");
        let mut identities = vec![(
            "source",
            canonical(prefix.person_aliases, &attr.source_key)?,
        )];
        if let Some(id) = &sem.conversation {
            identities.push(("conversation", id.clone()));
        }
        for identity in identities {
            if let Some(j) = keys.insert(identity, i) {
                union(&mut parents, i, j);
            }
        }
    }
    for &(a, b) in &records.same {
        if eligibility::connectable(a, prefix, tainted)
            && eligibility::connectable(b, prefix, tainted)
        {
            union(&mut parents, a, b);
        }
    }
    let mut members: BTreeMap<usize, Vec<(usize, Vec<Match>)>> = BTreeMap::new();
    for i in 0..prefix.frames.len() {
        if !eligibility::usable(i, prefix, records, tainted) {
            continue;
        }
        let matches = eligibility::matches(i, prefix, records, catalog, tainted)?;
        if !matches.is_empty() {
            members
                .entry(root(&parents, i))
                .or_default()
                .push((i, matches));
        }
    }
    let mut out = Vec::new();
    for rows in members.values() {
        let first = &prefix.frames[rows[0].0];
        let mut group = GroupItem {
            group_id: key(&["experience", &first.event.id]),
            position: first.event.raw_log_position.0,
            source_key: canonical(
                prefix.person_aliases,
                &first.attributes.as_ref().expect("validated").source_key,
            )?,
            members: Vec::new(),
            verifiable: first.attributes.as_ref().expect("validated").verifiable,
            content: GroupContent::Experience {
                appraisals: Vec::new(),
                matches: Vec::new(),
            },
        };
        let mut combined: BTreeMap<String, Match> = BTreeMap::new();
        for (i, matches) in rows {
            let frame = &prefix.frames[*i];
            // No approved reduction exists for mixed verification claims or μ.
            // Fail explicitly rather than inventing mean/max/first/AND policy.
            if frame.attributes.as_ref().expect("validated").verifiable != group.verifiable {
                return Err(Fault::Ambiguous("mixed group verifiability".into()));
            }
            group.members.push(frame.event.id.clone());
            if let GroupContent::Experience { appraisals, .. } = &mut group.content {
                appraisals.push(records.appraisals[i].appraisal.clone());
            }
            for m in matches {
                let id = target_id(m);
                if let Some(old) = combined.get(&id) {
                    if old.mu.to_bits() != m.mu.to_bits() {
                        return Err(Fault::Ambiguous(format!("mixed group mu for {id}")));
                    }
                    // A structural target must stay structural so G creates it.
                    if matches!(old.target, MatchTarget::Relation { .. }) {
                        continue;
                    }
                }
                combined.insert(id, m.clone());
            }
        }
        let targets: BTreeSet<_> = combined.keys().cloned().collect();
        if rows
            .iter()
            .any(|(_, matches)| matches.iter().map(target_id).collect::<BTreeSet<_>>() != targets)
        {
            // G shares one appraisal vector across a group's matches. Unioning
            // heterogeneous eligible sets would leak an excluded outcome into
            // another tendency's mean. Splitting the group would violate EV2.
            return Err(Fault::Ambiguous(
                "group needs target-specific appraisal eligibility".into(),
            ));
        }
        let mut ids: Vec<_> = combined.keys().collect();
        ids.sort_by(|a, b| tendency_order(a, b));
        if let GroupContent::Experience { matches, .. } = &mut group.content {
            *matches = ids.into_iter().map(|id| combined[id].clone()).collect();
        }
        out.push(group);
    }
    teaching(prefix, records, catalog, tainted, &mut out)?;
    out.sort_by(|a, b| {
        a.position
            .cmp(&b.position)
            .then_with(|| tendency_order(&a.group_id, &b.group_id))
    });
    Ok(out)
}

fn teaching(
    prefix: Prefix<'_>,
    records: &Records<'_>,
    catalog: &Catalog,
    tainted: &BTreeSet<usize>,
    out: &mut Vec<GroupItem>,
) -> Result<()> {
    let mut rows = records.teaching.clone();
    rows.sort_by(|a, b| {
        records.index[&a.event_id]
            .cmp(&records.index[&b.event_id])
            .then_with(|| tendency_order(&a.record_id, &b.record_id))
    });
    let mut seen = BTreeSet::new();
    for r in rows {
        let i = records.index[&r.event_id];
        let f = &prefix.frames[i];
        let sem = f.semantics.as_ref().expect("validated");
        let attr = f.attributes.as_ref().expect("validated");
        if tainted.contains(&i)
            || attr.self_narrative
            || !attr.exogenous
            || sem.kind != EventUse::Experience
            || sem.utterance != Utterance::TeachingAboutHer
            || sem.epistemic != EpistemicType::USER_ASSERTION
        {
            continue;
        }
        let speaker = canonical(prefix.person_aliases, &r.speaker)?;
        if sem
            .speaker
            .as_ref()
            .map(|s| canonical(prefix.person_aliases, s))
            .transpose()?
            .as_ref()
            != Some(&speaker)
        {
            return Err(Fault::InvalidRecord("teaching speaker".into()));
        }
        let id = catalog.target(&r.tendency_id, prefix)?;
        let group_id = key(&[
            "teaching",
            &speaker,
            &id,
            if r.supports { "+" } else { "-" },
        ]);
        if !seen.insert(group_id.clone()) {
            continue;
        }
        // First member only: repeats cannot alter position, credibility,
        // verifiability, membership or contents (EV7, P5-23 V2-45).
        out.push(GroupItem {
            group_id,
            position: f.event.raw_log_position.0,
            source_key: speaker.clone(),
            members: vec![f.event.id.clone()],
            verifiable: attr.verifiable,
            content: GroupContent::Teaching {
                speaker,
                tendency_id: id,
                supports: r.supports,
            },
        });
    }
    Ok(())
}

pub(super) fn incremental_extension(old: &GroupItem, new: &GroupItem) -> bool {
    if old.source_key != new.source_key
        || old.position != new.position
        || old.verifiable != new.verifiable
    {
        return false;
    }
    match (&old.content, &new.content) {
        (
            GroupContent::Experience {
                appraisals: a,
                matches: m,
            },
            GroupContent::Experience {
                appraisals: b,
                matches: n,
            },
        ) => {
            old.members.iter().zip(a).all(|(member, appraisal)| {
                new.members
                    .iter()
                    .position(|id| id == member)
                    .is_some_and(|i| b.get(i) == Some(appraisal))
            }) && m.iter().all(|item| n.contains(item))
        }
        _ => old == new,
    }
}
