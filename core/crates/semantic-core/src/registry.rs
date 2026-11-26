//! ADR 0048 G02. A successful IRL stage and its registry successor belong to
//! one outer transaction. Historical graphs continue to name their input registry.
use crate::{
    Fault, Result,
    irl::{IrlProduct, StagedIrl},
    objects::{check_object, object_ref},
    wire,
};
use core_types::{
    WireMessage,
    contract::Limits,
    generated::*,
    semantic::{
        SemanticReadView, check_registry_successor, check_semantic_evidence_message,
        resolve_location,
    },
};
use std::collections::{BTreeMap, BTreeSet};

/// Core-side authorization for this exact adoption (including its coverage).
/// The transaction reserves a fresh registry version and permanent IDs, and
/// approves the governance and link bases of this particular successor.
pub struct ApprovedRegistry<'a> {
    pub successor: &'a ObjectRef,
    pub new_ids: &'a [String],
}

/// Borrowed from the owning transaction. Implementations must bind the approval
/// to the event, producer output and governed result. No worker supplies this view.
pub trait RegistryTransaction: SemanticReadView {
    fn approved_registry(
        &self,
        adoption: &StageAdoption,
    ) -> core_types::contract::Result<ApprovedRegistry<'_>>;
}

/// The outer transaction materializes the listed revision objects from entries
/// at registry.version and publishes the whole bundle, or discards it.
pub struct StagedRegistry {
    pub irl: StagedIrl,
    pub registry: ConceptRegistry,
}

impl StagedIrl {
    /// Validate the proposed successor against the pinned registry and prepare
    /// the stage's publication manifest. Neither input registry is mutated.
    pub fn stage_registry(
        mut self,
        view: &impl RegistryTransaction,
        before: &ConceptRegistry,
        after: &ConceptRegistry,
        limits: Limits,
    ) -> Result<StagedRegistry> {
        let stage = &mut self.adoption;
        let IrlProduct::Graph(graph) = &self.product else {
            return Err(Fault::Binding);
        };
        if view.transaction() != Some(&stage.transaction_id)
            || view.branch() != stage.binding.branch_id
            || view.base_commit() != stage.binding.base_commit
            || stage.stage != InterpretationStage::INTERPRET_STAGE_IRL
            || stage.outcome == StageOutcome::STAGE_OUTCOME_FAILED
            || stage.registry_after.is_some()
            || !stage.staged_revisions.is_empty()
            || graph.registry != stage.registry_before
            || after.previous.as_ref() != Some(&stage.registry_before)
        {
            return Err(Fault::Binding);
        }
        check_object(&stage.interpretation.content, graph, limits)?;
        let approved = view.approved_registry(stage).map_err(|_| Fault::Binding)?;
        let successor = object_ref(after, &after.version, limits)?;
        if &successor != approved.successor {
            return Err(Fault::Binding);
        }
        // Compare canonical sets so a permutation cannot fabricate a revision.
        let canonical = |r: &ConceptRegistry| {
            ConceptRegistry::from_canonical(&wire(r, limits)?, limits).map_err(|_| Fault::Schema)
        };
        let before = canonical(before)?;
        let after = canonical(after)?;
        check_registry_successor(Some(&before), &after, limits).map_err(|_| Fault::Concept)?;
        let prior: BTreeMap<_, _> = before
            .entries
            .iter()
            .map(|e| (e.concept_id.as_str(), e))
            .collect();
        let mut assigned: BTreeSet<_> = approved.new_ids.iter().map(String::as_str).collect();
        if assigned.len() != approved.new_ids.len() {
            return Err(Fault::Duplicate);
        }
        use ConceptLifecycle::*;
        for entry in &after.entries {
            let old = prior.get(entry.concept_id.as_str()).copied();
            if old == Some(entry) {
                continue;
            }
            // D I §4.4: creation is provisional; stable cannot revert to
            // provisional and deprecation cannot be undone by another revision.
            match old {
                None if entry.lifecycle == CONCEPT_LIFE_PROVISIONAL
                    && assigned.remove(entry.concept_id.as_str()) => {}
                Some(old)
                    if old.lifecycle == entry.lifecycle
                        || old.lifecycle == CONCEPT_LIFE_PROVISIONAL
                        || entry.lifecycle == CONCEPT_LIFE_DEPRECATED => {}
                _ => return Err(Fault::Concept),
            }
            for loc in &entry.grounding {
                resolve_location(view, view.base_commit(), view.transaction(), loc, limits)
                    .map_err(|_| Fault::Source)?;
            }
            stage
                .staged_revisions
                .push(object_ref(entry, &after.version, limits)?);
        }
        if !assigned.is_empty() {
            return Err(Fault::Concept);
        }
        stage.registry_after = Some(successor);
        check_semantic_evidence_message(
            StageAdoption::NAME,
            &serde_json::to_value(stage).map_err(|_| Fault::Schema)?,
            limits,
        )
        .map_err(|_| Fault::Schema)?;
        Ok(StagedRegistry {
            irl: self,
            registry: after,
        })
    }
}

/// v2 §7. Copy a governed tendency candidate's names, definitions and ordered
/// examples into a provisional entry using the permanent ID reserved by core.
/// Registration does not evaluate the tendency or give it any behavioral effect.
pub fn provisional_tendency(
    allocated_id: String,
    concept: TendencyConcept,
    governance: ObjectRef,
    grounding: Vec<SourceLocation>,
) -> ConceptRevision {
    ConceptRevision {
        schema_version: 1,
        concept_id: allocated_id,
        lifecycle: ConceptLifecycle::CONCEPT_LIFE_PROVISIONAL,
        previous: None,
        governance,
        grounding,
        label: Some(concept.name),
        definition: Some(concept.definition),
        positive_examples: concept.positive_examples,
        negative_examples: concept.negative_examples,
        roles: vec![],
        allows_reported_text: false,
        constraints: vec![],
    }
}

/// Pure lookup over the supplied object, including during staging. A caller reading an older
/// ConceptRef must supply its original registry, even after a merge or rename.
pub fn canonical_concept(
    registry: &ConceptRegistry,
    pinned: &ObjectRef,
    concept: &ConceptRef,
    limits: Limits,
) -> Result<ConceptRef> {
    wire(concept, limits)?;
    if pinned.resource_version != registry.version || concept.registry_version != registry.version {
        return Err(Fault::Binding);
    }
    check_object(pinned, registry, limits)?;
    check_semantic_evidence_message(
        ConceptRegistry::NAME,
        &serde_json::to_value(registry).map_err(|_| Fault::Schema)?,
        limits,
    )
    .map_err(|_| Fault::Concept)?;
    if !registry
        .entries
        .iter()
        .any(|entry| entry.concept_id == concept.concept_id)
    {
        return Err(Fault::Reference);
    }
    // The formal registry validator above proved every edge is in this version,
    // single-valued and acyclic. Follow just the requested chain.
    let edges: BTreeMap<_, _> = registry
        .aliases
        .iter()
        .map(|a| (a.origin.concept_id.as_str(), a.target.concept_id.as_str()))
        .collect();
    let mut terminal = concept.concept_id.as_str();
    while let Some(next) = edges.get(terminal) {
        terminal = next;
    }
    Ok(ConceptRef {
        concept_id: terminal.into(),
        ..concept.clone()
    })
}

/// The runtime resolves the exact, published registry at this branch prefix
/// (current or historical). Return None for an unpublished object, and an error
/// if a published manifest is missing its bytes.
pub trait CommittedRegistryPrefix: SemanticReadView {
    fn published_registry(&self, pinned: &ObjectRef)
    -> core_types::contract::Result<Option<&[u8]>>;
}

/// Read a versioned identity after publication. Possession of successor bytes
/// or a transaction receipt does not make them visible in this prefix.
pub fn committed_concept(
    prefix: &impl CommittedRegistryPrefix,
    pinned: &ObjectRef,
    concept: &ConceptRef,
    limits: Limits,
) -> Result<Option<ConceptRef>> {
    if prefix.transaction().is_some() || pinned.resource_version != concept.registry_version {
        return Err(Fault::Binding);
    }
    let Some(bytes) = prefix
        .published_registry(pinned)
        .map_err(|_| Fault::Source)?
    else {
        return Ok(None);
    };
    let registry = ConceptRegistry::from_canonical(bytes, limits).map_err(|_| Fault::Object)?;
    if wire(&registry, limits)? != bytes {
        return Err(Fault::Object);
    }
    canonical_concept(&registry, pinned, concept, limits).map(Some)
}
