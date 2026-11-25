//! Ephemeral index over an explicitly supplied directed alias relation.
//! Not a registry persistence format; does not allocate IDs or mutate old state.
use crate::{Fault, Result};
use core_types::contract::Limits;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug)]
pub struct Aliases {
    representatives: BTreeMap<String, Arc<str>>,
}
impl Aliases {
    /// Each alias has one explicit successor. The terminal ID is its representative;
    /// input order and language never choose or invent a representative.
    pub fn build(ids: &[String], edges: &[(String, String)], limits: Limits) -> Result<Self> {
        let bytes = ids
            .iter()
            .map(String::len)
            .chain(edges.iter().flat_map(|(a, b)| [a.len(), b.len()]))
            .fold(0usize, usize::saturating_add);
        if ids.len().saturating_add(edges.len()) > limits.max_items || bytes > limits.max_bytes {
            return Err(Fault::Budget);
        }
        let known: BTreeSet<_> = ids.iter().cloned().collect();
        if known.len() != ids.len() || known.contains("") {
            return Err(Fault::Duplicate);
        }
        let mut next = BTreeMap::new();
        for (from, to) in edges {
            if !known.contains(from) || !known.contains(to) {
                return Err(Fault::Reference);
            }
            if next.insert(from.clone(), to.clone()).is_some() {
                return Err(Fault::Duplicate);
            }
        }
        let mut representatives: BTreeMap<String, Arc<str>> = BTreeMap::new();
        for id in &known {
            let mut cursor = id;
            let mut path = BTreeSet::new();
            let terminal = loop {
                if let Some(done) = representatives.get(cursor) {
                    break done.clone();
                }
                if !path.insert(cursor.clone()) {
                    return Err(Fault::AliasCycle);
                }
                match next.get(cursor) {
                    Some(to) => cursor = to,
                    None => break Arc::<str>::from(cursor.as_str()),
                }
            };
            for member in path {
                representatives.insert(member, terminal.clone());
            }
        }
        Ok(Self { representatives })
    }
    pub fn representative(&self, id: &str) -> Result<&str> {
        self.representatives
            .get(id)
            .map(AsRef::as_ref)
            .ok_or(Fault::Reference)
    }
}
