//! Validate a complete graph first. On failure, independently check each node
//! with all its role dependencies so a rejected claim cannot poison its inputs.
use crate::{
    Fault, Result,
    irl::{IrlDispatch, IrlReadView},
    wire,
};
use core_types::{
    WireMessage,
    contract::Limits,
    generated::*,
    semantic::{check_graph_registry, check_graph_sources},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn ids(g: &SemanticGraphV2) -> impl Iterator<Item = &str> {
    g.references
        .iter()
        .map(|n| n.id.as_str())
        .chain(g.statements.iter().map(|n| n.id.as_str()))
        .chain(g.actions.iter().map(|n| n.id.as_str()))
}

fn select(g: &SemanticGraphV2, keep: &BTreeSet<&str>) -> SemanticGraphV2 {
    SemanticGraphV2 {
        schema_version: g.schema_version,
        id: g.id.clone(),
        registry: g.registry.clone(),
        references: g
            .references
            .iter()
            .filter(|n| keep.contains(n.id.as_str()))
            .cloned()
            .collect(),
        statements: g
            .statements
            .iter()
            .filter(|n| keep.contains(n.id.as_str()))
            .cloned()
            .collect(),
        actions: g
            .actions
            .iter()
            .filter(|n| keep.contains(n.id.as_str()))
            .cloned()
            .collect(),
        edges: vec![],
    }
}

// Follow simple paths only. Cyclic mentions do not recurse forever. Bound
// expansion by the supplied inspection budget, including highly branching DAGs.
fn check_depth(links: &BTreeMap<&str, Vec<&str>>, limits: Limits) -> Result<()> {
    for &root in links.keys() {
        let mut path = BTreeSet::from([root]);
        let mut stack = vec![(root, 0)];
        let mut work = 0;
        while let Some((node, i)) = stack.last_mut() {
            let Some(&next) = links.get(node).and_then(|children| children.get(*i)) else {
                path.remove(node);
                stack.pop();
                continue;
            };
            *i += 1;
            work += 1;
            if work > limits.max_items {
                return Err(Fault::Budget);
            }
            if path.insert(next) {
                if path.len() > limits.max_depth {
                    return Err(Fault::Budget);
                }
                stack.push((next, 0));
            }
        }
    }
    Ok(())
}

pub(super) fn govern(
    d: &IrlDispatch<'_>,
    view: &impl IrlReadView,
    g: &SemanticGraphV2,
    registry: &ConceptRegistry,
    adapter: (&TextAdapterEntry, &SourceLocation),
    limits: Limits,
) -> Result<(SemanticGraphV2, BTreeMap<String, Fault>)> {
    let count = ids(g).count();
    if count.saturating_add(g.edges.len()) > limits.max_items {
        return Err(Fault::Budget);
    }
    let all: BTreeSet<_> = ids(g).collect();
    if all.len() != count {
        return Err(Fault::Duplicate);
    }
    if all.is_empty() {
        return Err(Fault::Reference);
    }
    let claims: BTreeMap<_, _> = g.statements.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut links = BTreeMap::new();
    for c in &g.statements {
        links.insert(
            c.id.as_str(),
            c.roles
                .iter()
                .map(|r| r.node_ref.as_str())
                .collect::<Vec<_>>(),
        );
    }
    for e in &g.edges {
        links
            .entry(e.from_node.as_str())
            .or_default()
            .push(e.to_node.as_str());
    }
    check_depth(&links, limits)?;

    let validate = |part: &SemanticGraphV2| -> Result<()> {
        check_graph_registry(part, registry, limits).map_err(|_| Fault::Concept)?;
        for c in &part.statements {
            if c.reported_text.source == *d.event
                && (c.reported_text.path != adapter.1.path
                    || c.predicate != adapter.0.predicate
                    || !c.roles.iter().any(|r| r.role == adapter.0.role))
            {
                return Err(Fault::Source);
            }
        }
        for n in &part.references {
            if n.referent.is_some() != (n.source.resolution_state == ResolutionState::EXISTING) {
                return Err(Fault::Concept);
            }
        }
        for meta in part
            .references
            .iter()
            .map(|n| &n.source)
            .chain(part.statements.iter().map(|n| &n.source))
            .chain(part.actions.iter().map(|n| &n.source))
        {
            if meta.representation_version != d.representation_version {
                return Err(Fault::Binding);
            }
            // Pin the epistemic ceiling to the authenticated origins. The model
            // cannot turn an inference or a quotation into a user's self-report.
            for root in &meta.roots {
                if root.branch_id != d.event.branch_id
                    || root != d.event
                        && !d.facts.items.iter().any(|i| {
                            i.availability == QueryStatus::OK
                                && i.source_id == root.record_id
                                && i.content == root.content
                        })
                {
                    return Err(Fault::Source);
                }
                let facts = view.source_facts(root).map_err(|_| Fault::Source)?;
                crate::graph::epistemic(meta.epistemic_type, meta.derivation, &facts)?;
            }
            if meta.resolution_state == ResolutionState::PROVISIONAL {
                // Creation/atomic publication of new IDs is in the registry PR.
                return Err(Fault::Concept);
            }
        }
        check_graph_sources(
            part,
            view,
            &d.binding.base_commit,
            Some(d.transaction),
            d.time_facts,
            limits,
        )
        .map_err(|_| Fault::Source)?;
        Ok(())
    };
    let mut rejected = BTreeMap::new();
    let output = if validate(g).is_ok() {
        g.clone()
    } else {
        for &id in &all {
            let mut needed = BTreeSet::new();
            let mut stack = vec![id];
            while let Some(node) = stack.pop() {
                if needed.insert(node)
                    && let Some(c) = claims.get(node)
                {
                    stack.extend(c.roles.iter().map(|r| r.node_ref.as_str()));
                }
            }
            if let Err(fault) = validate(&select(g, &needed)) {
                rejected.insert(id.to_owned(), fault);
            }
        }
        let kept: BTreeSet<_> = all
            .into_iter()
            .filter(|id| !rejected.contains_key(*id))
            .collect();
        if kept.is_empty() {
            return Err(rejected
                .values()
                .next()
                .copied()
                .unwrap_or(Fault::Reference));
        }
        let mut legal = select(g, &kept);
        legal.edges = g
            .edges
            .iter()
            .filter(|e| {
                kept.contains(e.from_node.as_str())
                    && kept.contains(e.to_node.as_str())
                    && registry.edge_kinds.contains(&e.relation)
            })
            .cloned()
            .collect();
        validate(&legal)?;
        legal
    };
    if !output
        .references
        .iter()
        .map(|n| &n.source)
        .chain(output.statements.iter().map(|n| &n.source))
        .chain(output.actions.iter().map(|n| &n.source))
        .any(|s| s.roots.contains(d.event) || s.locations.iter().any(|loc| loc.source == *d.event))
    {
        return Err(Fault::Source);
    }
    let canonical = SemanticGraphV2::from_canonical(&wire(&output, limits)?, limits)
        .map_err(|_| Fault::Schema)?;
    Ok((canonical, rejected))
}
