use super::{bound_object, check_semantic_evidence_message};
use crate::contract::{Limits, Result, require};
use crate::generated::*;
use crate::scalar::DecimalU64;
use crate::semantic::{SemanticReadView, resolve_location};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn revision(r: &ConceptRevision) -> Result<()> {
    if r.lifecycle == ConceptLifecycle::CONCEPT_LIFE_STABLE {
        require(
            r.label.is_some() && r.definition.is_some() && !r.grounding.is_empty(),
            "stable concept lacks definition or grounding",
        )?;
    }
    Ok(())
}

fn member<'a>(r: &'a ConceptRegistry, id: &ConceptRef) -> Result<&'a ConceptRevision> {
    require(id.registry_version == r.version, "concept registry version")?;
    r.entries
        .iter()
        .find(|e| e.concept_id == id.concept_id)
        .ok_or("unknown concept".into())
}

pub(super) fn registry(r: &ConceptRegistry) -> Result<()> {
    let mut edges = BTreeMap::new();
    for e in &r.entries {
        for role in &e.roles {
            if let Some(domain) = &role.domain {
                member(r, domain)?;
            }
        }
    }
    for link in &r.links {
        member(r, &link.origin)?;
        member(r, &link.target)?;
        require(
            r.relation_kinds.contains(&link.kind),
            "unknown concept relation",
        )?;
    }
    for alias in &r.aliases {
        member(r, &alias.origin)?;
        member(r, &alias.target)?;
        require(
            alias.kind == "canonical"
                && edges
                    .insert(&alias.origin.concept_id, &alias.target.concept_id)
                    .is_none(),
            "ambiguous canonical mapping",
        )?;
    }
    let mut visited = BTreeSet::new();
    for origin in edges.keys() {
        let mut path = BTreeSet::new();
        let mut current = *origin;
        while let Some(next) = edges.get(current) {
            if visited.contains(current) {
                break;
            }
            require(path.insert(current), "canonical cycle")?;
            current = next;
        }
        visited.extend(path);
    }
    Ok(())
}

fn bind_registry(r: &ConceptRegistry, reference: &ObjectRef, limits: Limits) -> Result<()> {
    let value = serde_json::to_value(r).map_err(|e| e.to_string())?;
    check_semantic_evidence_message("ConceptRegistry", &value, limits)?;
    require(
        reference.resource_version == r.version,
        "registry version mismatch",
    )?;
    bound_object(
        crate::codec_schema("ConceptRegistry")?,
        "ConceptRegistry",
        &value,
        reference,
        limits,
    )
}

/// IDs survive revisions and deprecation. A changed entry must point to its
/// exact predecessor. The caller owns ID allocation and atomic publication.
pub fn check_registry_successor(
    old: Option<&ConceptRegistry>,
    new: &ConceptRegistry,
    limits: Limits,
) -> Result<()> {
    let value = serde_json::to_value(new).map_err(|e| e.to_string())?;
    check_semantic_evidence_message("ConceptRegistry", &value, limits)?;
    let Some(old) = old else {
        return require(
            new.previous.is_none() && new.entries.iter().all(|e| e.previous.is_none()),
            "unexpected predecessor",
        );
    };
    require(old.version != new.version, "registry version reused")?;
    bind_registry(
        old,
        new.previous.as_ref().ok_or("missing previous registry")?,
        limits,
    )?;
    let mut remaining: BTreeMap<_, _> = new.entries.iter().map(|e| (&e.concept_id, e)).collect();
    for prior in &old.entries {
        let current = remaining
            .remove(&prior.concept_id)
            .ok_or("concept removed")?;
        if current != prior {
            let reference = current
                .previous
                .as_ref()
                .ok_or("missing previous revision")?;
            // Revision objects are addressed under the registry version in which
            // the trusted reader materialized that occurrence.
            require(
                reference.resource_version == old.version,
                "wrong revision version",
            )?;
            bound_object(
                crate::codec_schema("ConceptRevision")?,
                "ConceptRevision",
                &serde_json::to_value(prior).map_err(|e| e.to_string())?,
                reference,
                limits,
            )?;
        }
    }
    require(
        remaining.values().all(|r| r.previous.is_none()),
        "new ID with predecessor",
    )
}

pub(super) fn entry(e: &TextAdapterEntry) -> Result<()> {
    require(
        matches!(
            e.payload_tag.as_str(),
            "echo.user-text" | "echo.user-utterance"
        ) && e.payload_schema == 1
            && matches!(e.path.as_slice(), [f] if f.field_number == 2 && f.element.is_none()),
        "unsupported text adapter entry",
    )
}

pub(super) fn table(t: &TextAdapterTable) -> Result<()> {
    require(
        t.registry.object_type == "echo.concept-registry"
            && t.entries.len() == 2
            && t.entries
                .iter()
                .all(|e| e.predicate.registry_version == t.registry.resource_version)
            && t.entries[0].payload_tag != t.entries[1].payload_tag,
        "text and utterance mappings required",
    )
}

/// Resolve an authenticated event against the pinned table and registry. None
/// is an unsupported source. The returned role and predicate are proposals.
pub fn text_adapter<'a>(
    t: &'a TextAdapterTable,
    registry: &ConceptRegistry,
    view: &impl SemanticReadView,
    event: &RecordVersion,
    limits: Limits,
) -> Result<Option<(&'a TextAdapterEntry, SourceLocation)>> {
    check_semantic_evidence_message(
        "TextAdapterTable",
        &serde_json::to_value(t).map_err(|e| e.to_string())?,
        limits,
    )?;
    bind_registry(registry, &t.registry, limits)?;
    let Some(mapping) = t
        .entries
        .iter()
        .find(|e| e.payload_tag == event.content.object_type)
    else {
        return Ok(None);
    };
    require(
        matches!(view.content(event)?, super::VisibleContent::Event(raw, _)
        if raw.id == event.record_id && raw.payload_ref == event.content),
        "text adapter requires the authenticated ingress event",
    )?;
    let predicate = member(registry, &mapping.predicate)?;
    require(
        predicate.lifecycle == ConceptLifecycle::CONCEPT_LIFE_STABLE
            && predicate.allows_reported_text
            && predicate
                .roles
                .iter()
                .any(|r| r.role == mapping.role && r.node_kind == SemanticNodeKind::SEM_NODE_REF),
        "unregistered text signature",
    )?;
    let mut loc = SourceLocation {
        schema_version: 1,
        source: event.clone(),
        path: mapping.path.clone(),
        text: None,
    };
    let text = resolve_location(view, view.base_commit(), view.transaction(), &loc, limits)?;
    let bytes = text.as_str().ok_or("adapter requires text")?.len();
    require(bytes > 0, "empty text has no reported span")?;
    loc.text = Some(Utf8Range {
        schema_version: 1,
        start: DecimalU64(0),
        end: DecimalU64(bytes as u64),
    });
    Ok(Some((mapping, loc)))
}

type GraphNodes<'a> = BTreeMap<&'a str, (SemanticNodeKind, Option<&'a ConceptRef>)>;

pub(super) fn nodes(g: &SemanticGraphV2) -> Result<GraphNodes<'_>> {
    use SemanticNodeKind::*;
    let mut map = BTreeMap::new();
    for (id, kind, domain) in g
        .references
        .iter()
        .map(|r| (&r.id, SEM_NODE_REF, r.referent.as_ref()))
        .chain(
            g.statements
                .iter()
                .map(|c| (&c.id, SEM_NODE_CLAIM, Some(&c.predicate))),
        )
        .chain(g.actions.iter().map(|a| (&a.id, SEM_NODE_ACT, None)))
    {
        require(
            map.insert(id.as_str(), (kind, domain)).is_none(),
            "duplicate graph node",
        )?;
    }
    for e in &g.edges {
        require(
            map.contains_key(e.from_node.as_str()) && map.contains_key(e.to_node.as_str()),
            "dangling graph edge",
        )?;
    }
    for c in &g.statements {
        require(
            c.roles
                .iter()
                .all(|r| map.contains_key(r.node_ref.as_str())),
            "dangling role target",
        )?;
    }
    Ok(map)
}

/// Verify the graph against the exact registry object named by the graph.
pub fn check_graph_registry(
    g: &SemanticGraphV2,
    r: &ConceptRegistry,
    limits: Limits,
) -> Result<()> {
    let value = serde_json::to_value(g).map_err(|e| e.to_string())?;
    check_semantic_evidence_message("SemanticGraphV2", &value, limits)?;
    bind_registry(r, &g.registry, limits)?;
    let nodes = nodes(g)?;
    for (_, domain) in nodes.values() {
        if let Some(concept) = domain {
            member(r, concept)?;
        }
    }
    require(
        g.edges.iter().all(|e| r.edge_kinds.contains(&e.relation))
            && g.actions
                .iter()
                .all(|a| r.interaction_kinds.contains(&a.interaction_kind)),
        "unknown graph vocabulary",
    )?;
    for c in &g.statements {
        let p = member(r, &c.predicate)?;
        require(
            p.lifecycle == ConceptLifecycle::CONCEPT_LIFE_STABLE && p.allows_reported_text,
            "predicate operand domain",
        )?;
        for binding in &c.roles {
            let role = p
                .roles
                .iter()
                .find(|r| r.role == binding.role)
                .ok_or("unknown role")?;
            let (kind, domain) = nodes[binding.node_ref.as_str()];
            require(
                kind == role.node_kind && (role.domain.is_none() || role.domain.as_ref() == domain),
                "role domain",
            )?;
        }
        require(
            p.roles
                .iter()
                .filter(|r| r.mandatory)
                .all(|r| c.roles.iter().any(|b| b.role == r.role)),
            "missing mandatory role",
        )?;
    }
    Ok(())
}
