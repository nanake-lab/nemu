//! Dependency DAG resolution (S 12): cycles, unknown edges and broken
//! references block authorisation; a withdrawn object anywhere in the closure
//! blocks the effect immediately (I12).

use std::collections::{BTreeMap, BTreeSet};

use super::types::Digest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DagError {
    /// Referenced object is not in the known dependency graph.
    Unknown(Digest),
    Cycle(Digest),
    /// An object in the closure is behind the failure barrier.
    Invalidated(Digest),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Open,
    Done,
}

type Graph = BTreeMap<Digest, BTreeSet<Digest>>;
type Frame = (Digest, Vec<Digest>, usize);

/// Returns the dependency subgraph reachable from `roots`.
pub fn resolve(
    roots: &BTreeSet<Digest>,
    graph: &Graph,
    invalidated: &BTreeSet<Digest>,
) -> Result<Graph, DagError> {
    let mut marks: BTreeMap<Digest, Mark> = BTreeMap::new();
    let mut out = Graph::new();
    for root in roots {
        if marks.get(root) == Some(&Mark::Done) {
            continue;
        }
        // Iterative depth-first search with explicit frames.
        let mut stack: Vec<Frame> = Vec::new();
        enter(*root, graph, invalidated, &mut marks, &mut stack)?;
        while let Some((node, children, pos)) = stack.last_mut() {
            if *pos < children.len() {
                let child = children[*pos];
                *pos += 1;
                match marks.get(&child) {
                    Some(Mark::Open) => return Err(DagError::Cycle(child)),
                    Some(Mark::Done) => {}
                    None => enter(child, graph, invalidated, &mut marks, &mut stack)?,
                }
            } else {
                let node = *node;
                marks.insert(node, Mark::Done);
                out.insert(node, graph.get(&node).cloned().unwrap_or_default());
                stack.pop();
            }
        }
    }
    Ok(out)
}

fn enter(
    node: Digest,
    graph: &Graph,
    invalidated: &BTreeSet<Digest>,
    marks: &mut BTreeMap<Digest, Mark>,
    stack: &mut Vec<Frame>,
) -> Result<(), DagError> {
    if invalidated.contains(&node) {
        return Err(DagError::Invalidated(node));
    }
    let children = graph.get(&node).ok_or(DagError::Unknown(node))?;
    marks.insert(node, Mark::Open);
    stack.push((node, children.iter().copied().collect(), 0));
    Ok(())
}
