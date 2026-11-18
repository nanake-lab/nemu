//! Canonical record closure computation over frontier relations (ADR 0042 §1.3; Step 5.4).

use std::collections::{BTreeMap, BTreeSet};

use super::types::{Eligibility, HPos, RecordId, RecordTable, Relation, RelationKind, Unavailable};

/// Canonicalized relation index built purely from `BTreeMap` and `BTreeSet` (ADR 0042 §1.3).
pub(crate) struct RelationIndex {
    /// `Unknown` edges indexed on both endpoints (`from == x || to == x`, Step 0 Q3 Reading A).
    unknown_incident: BTreeMap<RecordId, BTreeSet<(RecordId, RecordId)>>,
    /// `BodySource` edges indexed by `from -> {to}`.
    body_sources_from: BTreeMap<RecordId, BTreeSet<RecordId>>,
    /// `Correction` edges indexed by `to -> {from}`.
    corrections_to: BTreeMap<RecordId, BTreeSet<RecordId>>,
    /// `Continuation` edges indexed by `from -> {to}` (Step 0 Q5 Reading A).
    continuations_from: BTreeMap<RecordId, BTreeSet<RecordId>>,
}

impl RelationIndex {
    /// Builds a deterministic, input-order-independent relation index (ADR 0042 §1.3).
    pub(crate) fn new(relations: &[Relation]) -> Self {
        let mut unknown_incident: BTreeMap<RecordId, BTreeSet<(RecordId, RecordId)>> =
            BTreeMap::new();
        let mut body_sources_from: BTreeMap<RecordId, BTreeSet<RecordId>> = BTreeMap::new();
        let mut corrections_to: BTreeMap<RecordId, BTreeSet<RecordId>> = BTreeMap::new();
        let mut continuations_from: BTreeMap<RecordId, BTreeSet<RecordId>> = BTreeMap::new();

        for rel in relations {
            match rel.kind {
                RelationKind::BodySource => {
                    body_sources_from
                        .entry(rel.from.clone())
                        .or_default()
                        .insert(rel.to.clone());
                }
                RelationKind::Continuation => {
                    continuations_from
                        .entry(rel.from.clone())
                        .or_default()
                        .insert(rel.to.clone());
                }
                RelationKind::Correction => {
                    corrections_to
                        .entry(rel.to.clone())
                        .or_default()
                        .insert(rel.from.clone());
                }
                RelationKind::LocalOnly => {}
                RelationKind::Unknown => {
                    let edge = (rel.from.clone(), rel.to.clone());
                    unknown_incident
                        .entry(rel.from.clone())
                        .or_default()
                        .insert(edge.clone());
                    unknown_incident
                        .entry(rel.to.clone())
                        .or_default()
                        .insert(edge);
                }
            }
        }

        Self {
            unknown_incident,
            body_sources_from,
            corrections_to,
            continuations_from,
        }
    }

    /// Returns direct `Continuation` target IDs from `id`, deduplicated in byte order
    /// (ADR 0042 §1.3, §2.1; Step 0 Q5 Reading A).
    pub(crate) fn continuation_refs(&self, id: &RecordId) -> Vec<RecordId> {
        self.continuations_from
            .get(id)
            .map(|targets| targets.iter().cloned().collect())
            .unwrap_or_default()
    }
}

/// Output of `compute_closure` (ADR 0042 §1.3; Step 5.4).
pub(crate) struct ClosureOutput {
    /// Complete closure records sorted by `(h_pos, id)` ascending.
    pub ordered: Vec<(HPos, RecordId)>,
    /// Records reached via `BodySource.to` or `Correction.from` (including seed hits; Step 0 Q1A).
    pub expanded_targets: BTreeSet<(HPos, RecordId)>,
    /// Earliest `Ineligible` record in `(h_pos, id)` order and its reason, if any.
    pub first_ineligible: Option<(RecordId, String)>,
}

/// Computes the canonical closure over `BodySource`, `Correction`, and `Unknown` edges
/// starting from `seeds` (ADR 0042 §1.3; Step 5.4; Step 0 Q3A, Q6B, Q9A).
pub(crate) fn compute_closure(
    seeds: &BTreeSet<RecordId>,
    records: &RecordTable,
    rels: &RelationIndex,
) -> Result<ClosureOutput, Unavailable> {
    let mut visited: BTreeSet<(HPos, RecordId)> = BTreeSet::new();
    let mut queue: BTreeSet<(HPos, RecordId)> = BTreeSet::new();
    let mut expanded_targets: BTreeSet<(HPos, RecordId)> = BTreeSet::new();

    // Step 5.4(1) & Step 0 Q6B / Q9A(1): seed lookup in canonical byte order.
    for seed_id in seeds {
        let Some(rec) = records.get(seed_id) else {
            return Err(Unavailable::MissingRecord(seed_id.clone()));
        };
        let key = (rec.h_pos, rec.id.clone());
        if visited.insert(key.clone()) {
            queue.insert(key);
        }
    }

    // Step 5.4(2)-(3) & Step 0 Q9A(2): pop earliest `(h_pos, id)`; Unknown takes precedence
    // over missing BodySource targets, then missing Correction sources, all in byte order.
    while let Some((_, x)) = queue.pop_first() {
        if let Some(unknown_edges) = rels.unknown_incident.get(&x)
            && let Some((from, to)) = unknown_edges.iter().next()
        {
            return Err(Unavailable::UnknownRelation {
                from: from.clone(),
                to: to.clone(),
            });
        }

        if let Some(targets) = rels.body_sources_from.get(&x) {
            for to_id in targets {
                let Some(to_rec) = records.get(to_id) else {
                    return Err(Unavailable::MissingRecord(to_id.clone()));
                };
                let key = (to_rec.h_pos, to_rec.id.clone());
                expanded_targets.insert(key.clone());
                if visited.insert(key.clone()) {
                    queue.insert(key);
                }
            }
        }

        if let Some(sources) = rels.corrections_to.get(&x) {
            for from_id in sources {
                let Some(from_rec) = records.get(from_id) else {
                    return Err(Unavailable::MissingRecord(from_id.clone()));
                };
                let key = (from_rec.h_pos, from_rec.id.clone());
                expanded_targets.insert(key.clone());
                if visited.insert(key.clone()) {
                    queue.insert(key);
                }
            }
        }
    }

    // Step 5.4(4) & Step 0 Q9A(3): check `Undetermined` in `(h_pos, id)` ascending order.
    for (_, id) in &visited {
        if let Some(rec) = records.get(id)
            && matches!(rec.eligibility, Eligibility::Undetermined)
        {
            return Err(Unavailable::EligibilityUndetermined(id.clone()));
        }
    }

    // Step 5.4(5): find first `Ineligible` record in `(h_pos, id)` ascending order.
    let mut first_ineligible: Option<(RecordId, String)> = None;
    for (_, id) in &visited {
        if let Some(rec) = records.get(id)
            && let Eligibility::Ineligible { reason } = &rec.eligibility
        {
            first_ineligible = Some((id.clone(), reason.clone()));
            break;
        }
    }

    Ok(ClosureOutput {
        ordered: visited.into_iter().collect(),
        expanded_targets,
        first_ineligible,
    })
}
