use super::{records::canonical, *};
use crate::law::{Candidate, relation_tendency_id};
use core_types::semantic::tendency_order;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Catalog {
    pub remap: BTreeMap<String, String>,
    pub current: Vec<Candidate>,
    pub proposals: Vec<(usize, String)>,
    pub aliases: BTreeMap<String, String>,
}

impl Catalog {
    pub fn target(&self, id: &str, prefix: Prefix<'_>) -> Result<String> {
        self.remap
            .get(id)
            .cloned()
            .or_else(|| {
                prefix
                    .genesis_order
                    .iter()
                    .find(|known| known.as_str() == id)
                    .cloned()
            })
            .ok_or_else(|| Fault::MissingReference(id.to_owned()))
    }
}

pub(super) fn build(prefix: Prefix<'_>) -> Result<Catalog> {
    let mut out = Catalog {
        remap: BTreeMap::new(),
        current: Vec::new(),
        proposals: Vec::new(),
        aliases: BTreeMap::new(),
    };
    for id in prefix.concept_aliases.keys() {
        out.aliases
            .insert(id.clone(), canonical(prefix.concept_aliases, id)?);
    }
    let mut concepts: BTreeMap<String, Candidate> = BTreeMap::new();
    for (i, frame) in prefix.frames.iter().enumerate() {
        let mut candidates: Vec<_> = frame.candidates.iter().collect();
        candidates.sort_by(|a, b| {
            let alias = |c: &RegisteredCandidate| {
                out.aliases
                    .get(&c.concept.concept_id)
                    .is_some_and(|target| *target != c.concept.concept_id)
            };
            alias(a)
                .cmp(&alias(b))
                .then_with(|| tendency_order(&a.record.record_id, &b.record.record_id))
        });
        let mut proposed = BTreeSet::new();
        for registered in candidates {
            let record = &registered.record;
            let concept_id = canonical(prefix.concept_aliases, &registered.concept.concept_id)?;
            let is_alias = concept_id != registered.concept.concept_id;
            let mut candidate = Candidate {
                record_id: record.record_id.clone(),
                concept: registered.concept.clone(),
                predicate: record.predicate.clone(),
                direction: record.direction.clone(),
                relation: record.relation.clone(),
            };
            candidate.concept.concept_id = concept_id.clone();
            let identity = if let Some(rel) = &mut candidate.relation {
                rel.person_id = canonical(prefix.person_aliases, &rel.person_id)?;
                relation_tendency_id(rel.kind, &rel.person_id)
            } else if let Some(old) = concepts.get(&concept_id) {
                if !is_alias
                    && (old.predicate != candidate.predicate
                        || old.direction != candidate.direction
                        || old.concept.registry_version != candidate.concept.registry_version)
                {
                    return Err(Fault::RecomputeRequired(concept_id));
                }
                old.record_id.clone()
            } else {
                // An alias alone cannot choose an identity/structure for its missing
                // canonical candidate. This avoids order-dependent synonym priors.
                if is_alias {
                    return Err(Fault::Ambiguous(
                        "alias without registered canonical candidate".into(),
                    ));
                }
                concepts.insert(concept_id.clone(), candidate.clone());
                candidate.record_id.clone()
            };
            out.remap.insert(record.record_id.clone(), identity.clone());
            out.remap.insert(identity.clone(), identity.clone());
            if !is_alias && proposed.insert(identity.clone()) {
                out.proposals.push((i, identity.clone()));
                if i + 1 == prefix.frames.len() {
                    candidate.record_id = identity;
                    out.current.push(candidate);
                }
            }
        }
    }
    out.current
        .sort_by(|a, b| tendency_order(&a.record_id, &b.record_id));
    Ok(out)
}
