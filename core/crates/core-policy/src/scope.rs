//! Bounded scope predicate DSL (O 3.3; S 12).
//!
//! Content is limited to: versioned enumerations, Concept identities, finite
//! sets, closed numeric intervals and bounded AND/OR combinations. There is no
//! negation, no arbitrary code and no free-text trigger. Every field must be on
//! the whitelist of the endpoint the scope is declared for.

use std::collections::{BTreeMap, BTreeSet};

use crate::catalog::{Catalog, FieldId, FieldKind};
use crate::endpoint::Endpoint;
use crate::error::PolicyError;
use crate::numeric::in_units;

/// One field constraint.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Atom {
    /// Field value is one of the listed enum codes.
    EnumIn {
        field: FieldId,
        values: BTreeSet<u32>,
    },
    /// Field value is one of the listed Concept IDs.
    ConceptIn {
        field: FieldId,
        concepts: BTreeSet<String>,
    },
    /// Field value lies in the closed interval [lo, hi] (fixed-point units).
    NumIn { field: FieldId, lo: i64, hi: i64 },
}

impl Atom {
    pub fn field(&self) -> &str {
        match self {
            Self::EnumIn { field, .. }
            | Self::ConceptIn { field, .. }
            | Self::NumIn { field, .. } => field,
        }
    }
}

/// Scope expression. `True` is the unconditional scope.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeExpr {
    True,
    Atom(Atom),
    And(Vec<ScopeExpr>),
    Or(Vec<ScopeExpr>),
}

/// A scope validated against one catalog for one endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    pub endpoint: Endpoint,
    pub catalog_version: String,
    pub expr: ScopeExpr,
}

impl Scope {
    /// Validates `expr` for `endpoint` under `catalog`.
    pub fn new(
        endpoint: Endpoint,
        expr: ScopeExpr,
        catalog: &Catalog,
    ) -> Result<Self, PolicyError> {
        catalog.validate()?;
        let mut nodes = 0usize;
        validate_expr(&expr, endpoint, catalog, 1, &mut nodes)?;
        Ok(Self {
            endpoint,
            catalog_version: catalog.version.clone(),
            expr,
        })
    }

    /// Fields the scope reads.
    pub fn fields(&self) -> BTreeSet<&str> {
        let mut out = BTreeSet::new();
        collect_fields(&self.expr, &mut out);
        out
    }
}

fn validate_expr(
    expr: &ScopeExpr,
    endpoint: Endpoint,
    catalog: &Catalog,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), PolicyError> {
    let limits = catalog.limits;
    if depth > limits.max_depth {
        return Err(PolicyError::LimitExceeded("depth"));
    }
    *nodes += 1;
    if *nodes > limits.max_nodes {
        return Err(PolicyError::LimitExceeded("nodes"));
    }
    match expr {
        ScopeExpr::True => Ok(()),
        ScopeExpr::Atom(atom) => validate_atom(atom, endpoint, catalog),
        ScopeExpr::And(items) | ScopeExpr::Or(items) => {
            if items.is_empty() {
                return Err(PolicyError::EmptyCombination);
            }
            items
                .iter()
                .try_for_each(|item| validate_expr(item, endpoint, catalog, depth + 1, nodes))
        }
    }
}

fn validate_atom(atom: &Atom, endpoint: Endpoint, catalog: &Catalog) -> Result<(), PolicyError> {
    let field = atom.field();
    let spec = catalog.field(field)?;
    if !spec.endpoints.contains(&endpoint) {
        return Err(PolicyError::FieldNotAllowed {
            field: field.to_owned(),
            endpoint: endpoint.code(),
        });
    }
    let max_items = catalog.limits.max_set_items;
    match (atom, &spec.kind) {
        (Atom::EnumIn { values, .. }, FieldKind::Enum { domain }) => {
            if values.is_empty() {
                return Err(PolicyError::EmptySet(field.to_owned()));
            }
            if values.len() > max_items {
                return Err(PolicyError::LimitExceeded("set items"));
            }
            if !values.is_subset(domain) {
                return Err(PolicyError::OutOfDomain(field.to_owned()));
            }
            Ok(())
        }
        (Atom::ConceptIn { concepts, .. }, FieldKind::Concept { .. }) => {
            if concepts.is_empty() {
                return Err(PolicyError::EmptySet(field.to_owned()));
            }
            if concepts.len() > max_items {
                return Err(PolicyError::LimitExceeded("set items"));
            }
            if concepts.iter().any(String::is_empty) {
                return Err(PolicyError::OutOfDomain(field.to_owned()));
            }
            Ok(())
        }
        (Atom::NumIn { lo, hi, .. }, FieldKind::Numeric { min, max, .. }) => {
            if lo > hi {
                return Err(PolicyError::EmptyInterval(field.to_owned()));
            }
            if lo < min || hi > max {
                return Err(PolicyError::OutOfDomain(field.to_owned()));
            }
            Ok(())
        }
        _ => Err(PolicyError::KindMismatch(field.to_owned())),
    }
}

fn collect_fields<'a>(expr: &'a ScopeExpr, out: &mut BTreeSet<&'a str>) {
    match expr {
        ScopeExpr::True => {}
        ScopeExpr::Atom(atom) => {
            out.insert(atom.field());
        }
        ScopeExpr::And(items) | ScopeExpr::Or(items) => {
            items.iter().for_each(|item| collect_fields(item, out));
        }
    }
}

/// A typed value supplied by an endpoint context.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    Enum(u32),
    Concept(String),
    /// Finite binary64; checked when the context is built.
    Number(f64),
}

/// Result of evaluating a scope against a typed context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeMatch {
    Hit,
    Miss,
    /// The scope needs a field that the context does not carry. This is
    /// `UnsupportedScope`; it is never treated as a hit (O 3.2).
    MissingField(String),
}

/// Evaluates `expr` against `values`. `catalog` supplies numeric precision.
pub fn evaluate(
    expr: &ScopeExpr,
    values: &BTreeMap<FieldId, FieldValue>,
    catalog: &Catalog,
) -> Result<ScopeMatch, PolicyError> {
    match expr {
        ScopeExpr::True => Ok(ScopeMatch::Hit),
        ScopeExpr::Atom(atom) => evaluate_atom(atom, values, catalog),
        ScopeExpr::And(items) => {
            let mut result = ScopeMatch::Hit;
            for item in items {
                match evaluate(item, values, catalog)? {
                    ScopeMatch::MissingField(f) => return Ok(ScopeMatch::MissingField(f)),
                    ScopeMatch::Miss => result = ScopeMatch::Miss,
                    ScopeMatch::Hit => {}
                }
            }
            Ok(result)
        }
        ScopeExpr::Or(items) => {
            let mut result = ScopeMatch::Miss;
            for item in items {
                match evaluate(item, values, catalog)? {
                    ScopeMatch::MissingField(f) => return Ok(ScopeMatch::MissingField(f)),
                    ScopeMatch::Hit => result = ScopeMatch::Hit,
                    ScopeMatch::Miss => {}
                }
            }
            Ok(result)
        }
    }
}

fn evaluate_atom(
    atom: &Atom,
    values: &BTreeMap<FieldId, FieldValue>,
    catalog: &Catalog,
) -> Result<ScopeMatch, PolicyError> {
    let field = atom.field();
    let Some(value) = values.get(field) else {
        return Ok(ScopeMatch::MissingField(field.to_owned()));
    };
    let hit = match (atom, value) {
        (Atom::EnumIn { values, .. }, FieldValue::Enum(code)) => values.contains(code),
        (Atom::ConceptIn { concepts, .. }, FieldValue::Concept(id)) => concepts.contains(id),
        (Atom::NumIn { lo, hi, .. }, FieldValue::Number(v)) => {
            let FieldKind::Numeric { decimals, .. } = catalog.field(field)?.kind else {
                return Err(PolicyError::KindMismatch(field.to_owned()));
            };
            // Quantised exactly to the field unit: the units overlap proves on.
            in_units(*v, *lo, *hi, decimals)
        }
        _ => return Err(PolicyError::KindMismatch(field.to_owned())),
    };
    Ok(if hit {
        ScopeMatch::Hit
    } else {
        ScopeMatch::Miss
    })
}
