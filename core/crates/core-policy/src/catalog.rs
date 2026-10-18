//! Field catalog for the scope DSL (O 3.3; S 12).
//!
//! The catalog is a versioned implementation resource: it is the
//! `scope_interpreter` part of Ω. It declares which fields exist, their
//! domains and precision, and which endpoints may use each field. This crate
//! does not decide which semantic fields exist; deployments supply a catalog
//! and tests use a structural fixture.

use std::collections::{BTreeMap, BTreeSet};

use crate::endpoint::Endpoint;
use crate::error::PolicyError;

/// Stable field identifier inside one catalog version.
pub type FieldId = String;

/// Domain of one field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldKind {
    /// Versioned enumeration with a finite declared domain of codes.
    Enum { domain: BTreeSet<u32> },
    /// Concept identities bound to one Registry version.
    Concept { registry_version: String },
    /// Bounded numeric field. Values are fixed-point integers in units of
    /// 10^-decimals; `min`/`max` bound the field domain in those units.
    Numeric { min: i64, max: i64, decimals: u8 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldSpec {
    pub kind: FieldKind,
    /// Endpoints whose whitelist contains this field.
    pub endpoints: BTreeSet<Endpoint>,
}

/// Hard limits on a scope expression and on overlap checking (O 3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScopeLimits {
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_set_items: usize,
    /// Maximum disjunctive-normal-form terms after normalisation.
    pub max_dnf_terms: usize,
    /// Maximum conjunct pairs examined by one overlap check.
    pub max_overlap_pairs: usize,
}

/// Largest fixed-point precision accepted by any catalog.
pub const MAX_DECIMALS: u8 = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    /// Interpreter/catalog version bound into every scope digest.
    pub version: String,
    pub fields: BTreeMap<FieldId, FieldSpec>,
    pub limits: ScopeLimits,
}

impl Catalog {
    /// Validates the catalog itself. A catalog that fails here must not be
    /// used to interpret any scope.
    pub fn validate(&self) -> Result<(), PolicyError> {
        if self.version.is_empty() {
            return Err(PolicyError::InvalidCatalog("empty version".into()));
        }
        let l = self.limits;
        if l.max_depth == 0
            || l.max_nodes == 0
            || l.max_set_items == 0
            || l.max_dnf_terms == 0
            || l.max_overlap_pairs == 0
        {
            return Err(PolicyError::InvalidCatalog("zero limit".into()));
        }
        for (id, spec) in &self.fields {
            if id.is_empty() {
                return Err(PolicyError::InvalidCatalog("empty field id".into()));
            }
            match &spec.kind {
                FieldKind::Enum { domain } => {
                    if domain.is_empty() {
                        return Err(PolicyError::InvalidCatalog(format!("{id}: empty domain")));
                    }
                }
                FieldKind::Concept { registry_version } => {
                    if registry_version.is_empty() {
                        return Err(PolicyError::InvalidCatalog(format!(
                            "{id}: empty registry version"
                        )));
                    }
                }
                FieldKind::Numeric { min, max, decimals } => {
                    let exact = -(1i64 << 53)..=(1i64 << 53);
                    if min > max
                        || *decimals > MAX_DECIMALS
                        || !exact.contains(min)
                        || !exact.contains(max)
                    {
                        return Err(PolicyError::InvalidCatalog(format!("{id}: numeric domain")));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn field(&self, id: &str) -> Result<&FieldSpec, PolicyError> {
        self.fields
            .get(id)
            .ok_or_else(|| PolicyError::UnknownField(id.to_owned()))
    }

    /// Fields an endpoint may read (its whitelist).
    pub fn whitelist(&self, endpoint: Endpoint) -> BTreeSet<&str> {
        self.fields
            .iter()
            .filter(|(_, s)| s.endpoints.contains(&endpoint))
            .map(|(id, _)| id.as_str())
            .collect()
    }
}
