use super::*;
use crate::law::{GroupContent, GroupItem, MatchTarget, relation_tendency_id};
use std::collections::BTreeSet;

pub(super) fn relation_ids(groups: &[GroupItem]) -> BTreeSet<String> {
    groups
        .iter()
        .flat_map(|g| match &g.content {
            GroupContent::Experience { matches, .. } => matches
                .iter()
                .filter_map(|m| match &m.target {
                    MatchTarget::Relation { person_id, kind } => {
                        Some(relation_tendency_id(*kind, person_id))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            _ => vec![],
        })
        .collect()
}

pub(super) fn route(all: &Assembly, mut out: Assembly) -> Result<Assembly> {
    let Some(admissions) = &all.structural_admissions else {
        if !relation_ids(&out.groups).is_empty() {
            return Err(Fault::MissingSemantics(
                "recorded structural admission intent required".into(),
            ));
        }
        return Ok(out);
    };
    let requested: BTreeSet<_> = admissions.iter().cloned().collect();
    if requested.len() != admissions.len() || !requested.is_subset(&relation_ids(&all.groups)) {
        return Err(Fault::InvalidRecord(
            "structural admission identities".into(),
        ));
    }
    // All historical groups and matches for sharing; never select by sign.
    // The C sequence remains the checked incremental suffix.
    if !requested.is_empty() {
        out.groups = all.groups.clone();
    }
    for group in &mut out.groups {
        if let GroupContent::Experience { matches, .. } = &mut group.content {
            for m in matches {
                if let MatchTarget::Relation { person_id, kind } = &m.target {
                    let id = relation_tendency_id(*kind, person_id);
                    // Update retained R, without historical backfill itself
                    // auto-creating unrelated evicted structural relations.
                    if !requested.contains(&id) {
                        m.target = MatchTarget::Tendency(id);
                    }
                }
            }
        }
    }
    Ok(out)
}
