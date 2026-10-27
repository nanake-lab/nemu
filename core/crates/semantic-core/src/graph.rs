//! Structural subgraph extraction. The result is never an ACCEPTED IRL graph:
//! opaque spans and predicate operands still need their missing formal schemas.
use crate::{Fault, Result, wire};
use core_types::{WireMessage, contract::Limits, generated::*};
use std::collections::{BTreeMap, BTreeSet};

/// Authority-side read result, not candidate metadata. The caller must derive it
/// from an authenticated committed/staged event and current source-validity view.
/// No serialization is provided, and no production authority adapter exists yet.
pub struct SourceFacts {
    pub epistemic: EpistemicType,
    pub self_narrative: bool,
    pub speaker: Option<String>,
    pub verifiable: bool,
}
pub struct Snapshot<'a> {
    pub sources: &'a BTreeMap<String, SourceFacts>,
    pub concepts: &'a [ConceptRef],
    pub representation_version: &'a str,
}
pub struct Preflight {
    /// Canonical structural candidate only; no source span/object validation implied.
    pub graph: SemanticGraph,
    pub rejected: BTreeMap<String, Fault>,
    pub rejected_edges: Vec<usize>,
}

pub(crate) fn source<'a>(snapshot: &'a Snapshot<'_>, id: &str) -> Result<&'a SourceFacts> {
    snapshot.sources.get(id).ok_or(Fault::Source)
}
pub(crate) fn concept(snapshot: &Snapshot<'_>, value: &ConceptRef) -> Result<()> {
    if snapshot.concepts.contains(value) {
        Ok(())
    } else {
        Err(Fault::Concept)
    }
}
fn metadata(m: &SourceMetadata, snapshot: &Snapshot<'_>) -> Result<()> {
    if m.resolution_state == ResolutionState::PROVISIONAL {
        return Err(Fault::MissingContract);
    }
    if m.representation_version != snapshot.representation_version || m.source_refs.is_empty() {
        return Err(Fault::Source);
    }
    for id in &m.source_refs {
        let actual = source(snapshot, id)?;
        if actual.epistemic == EpistemicType::MODEL_INFERENCE
            && !matches!(
                m.epistemic_type,
                EpistemicType::MODEL_INFERENCE | EpistemicType::EPISTEMIC_UNKNOWN
            )
        {
            return Err(Fault::Epistemic);
        }
        // Inference and quotations can never be promoted to a user's assertion.
        let downgrade = m.epistemic_type == EpistemicType::EPISTEMIC_UNKNOWN
            || m.epistemic_type == EpistemicType::QUOTATION
            || (m.epistemic_type == EpistemicType::MODEL_INFERENCE
                && m.derivation == Derivation::INFERRED);
        if m.epistemic_type != actual.epistemic && !downgrade {
            return Err(Fault::Epistemic);
        }
        if m.derivation == Derivation::INFERRED
            && m.epistemic_type != EpistemicType::MODEL_INFERENCE
            && m.epistemic_type != EpistemicType::EPISTEMIC_UNKNOWN
        {
            return Err(Fault::Epistemic);
        }
        if actual.self_narrative
            && matches!(
                m.epistemic_type,
                EpistemicType::USER_ASSERTION
                    | EpistemicType::USER_SELF_REPORT
                    | EpistemicType::TOOL_OUTPUT
            )
        {
            return Err(Fault::Epistemic);
        }
    }
    Ok(())
}

pub fn preflight(
    graph: &SemanticGraph,
    snapshot: &Snapshot<'_>,
    limits: Limits,
) -> Result<Preflight> {
    // Codec checks the actual byte/item/nesting limits, including all nested refs.
    wire(graph, limits)?;
    let count = graph
        .refs
        .len()
        .saturating_add(graph.claims.len())
        .saturating_add(graph.acts.len())
        .saturating_add(graph.edges.len());
    if count > limits.max_items
        || snapshot.sources.len() > limits.max_items
        || snapshot.concepts.len() > limits.max_items
    {
        return Err(Fault::Budget);
    }
    let mut ids = BTreeSet::new();
    let mut rejected = BTreeMap::new();
    for id in graph
        .refs
        .iter()
        .map(|n| &n.id)
        .chain(graph.claims.iter().map(|n| &n.id))
        .chain(graph.acts.iter().map(|n| &n.id))
    {
        if !ids.insert(id.clone()) {
            return Err(Fault::Duplicate);
        }
    }
    for n in &graph.refs {
        let check = metadata(&n.metadata, snapshot).and_then(|()| {
            match (&n.referent, n.metadata.resolution_state) {
                (None, ResolutionState::UNRESOLVED) => Ok(()),
                (Some(c), ResolutionState::EXISTING) => concept(snapshot, c),
                // Provisional IDs cannot be authenticated without the staged registry contract.
                (_, ResolutionState::PROVISIONAL) => Err(Fault::MissingContract),
                _ => Err(Fault::Concept),
            }
        });
        if let Err(fault) = check {
            rejected.insert(n.id.clone(), fault);
        }
    }
    for n in &graph.claims {
        let check = metadata(&n.metadata, snapshot).and_then(|()| concept(snapshot, &n.predicate));
        if let Err(fault) = check {
            rejected.insert(n.id.clone(), fault);
        }
    }
    for n in &graph.acts {
        if let Err(fault) = metadata(&n.metadata, snapshot) {
            rejected.insert(n.id.clone(), fault);
        }
    }
    // Remove dependants to a fixed point, retaining independent legal components.
    loop {
        let before = rejected.len();
        for n in &graph.claims {
            if n.roles
                .iter()
                .any(|r| !ids.contains(&r.node_ref) || rejected.contains_key(&r.node_ref))
            {
                rejected.insert(n.id.clone(), Fault::Reference);
            }
        }
        if before == rejected.len() {
            break;
        }
    }
    let mut output = graph.clone();
    output.refs.retain(|n| !rejected.contains_key(&n.id));
    output.claims.retain(|n| !rejected.contains_key(&n.id));
    output.acts.retain(|n| !rejected.contains_key(&n.id));
    // Edges are an ordered list in the formal contract; never reorder them.
    // Unknown relations are rejected, not executed or reinterpreted as commands.
    const EDGES: &[&str] = &[
        "has_argument",
        "has_content",
        "targets",
        "corefers_to",
        "corrects",
        "quotes",
        "answers",
        "time_anchor",
    ];
    if output
        .edges
        .iter()
        .any(|e| !EDGES.contains(&e.relation.as_str()))
    {
        return Err(Fault::Reference);
    }
    let mut rejected_edges = Vec::new();
    output.edges = output
        .edges
        .into_iter()
        .enumerate()
        .filter_map(|(index, e)| {
            if ids.contains(&e.from_node)
                && ids.contains(&e.to_node)
                && !rejected.contains_key(&e.from_node)
                && !rejected.contains_key(&e.to_node)
            {
                Some(e)
            } else {
                rejected_edges.push(index);
                None
            }
        })
        .collect();
    // Round-trip applies CanonicalV1 set order rather than ordinary string sorting.
    let bytes = wire(&output, limits)?;
    output = SemanticGraph::from_canonical(&bytes, limits).map_err(|_| Fault::Schema)?;
    Ok(Preflight {
        graph: output,
        rejected,
        rejected_edges,
    })
}
