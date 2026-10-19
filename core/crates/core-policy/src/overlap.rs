//! Deterministic scope intersection (O 3.3; S 12).
//!
//! Scopes are normalised into disjunctive normal form over per-field
//! constraints. Two scopes are reported `Disjoint` only when every pair of
//! normal-form terms has some field whose constraints provably do not
//! intersect. Anything that cannot be proven disjoint is `Overlapping`;
//! exceeding a resource limit is `Unknown` (UNKNOWN_OVERLAP), which callers
//! must treat as a conflict. No model is consulted.

use std::collections::{BTreeMap, BTreeSet};

use crate::catalog::{Catalog, FieldId, ScopeLimits};
use crate::scope::{Atom, Scope, ScopeExpr};

/// Constraint on one field inside a normal-form term.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Constraint {
    Enum(BTreeSet<u32>),
    Concept(BTreeSet<String>),
    Num(i64, i64),
}

impl Constraint {
    fn from_atom(atom: &Atom) -> Self {
        match atom {
            Atom::EnumIn { values, .. } => Self::Enum(values.clone()),
            Atom::ConceptIn { concepts, .. } => Self::Concept(concepts.clone()),
            Atom::NumIn { lo, hi, .. } => Self::Num(*lo, *hi),
        }
    }

    /// Intersection, or `None` when provably empty. Mismatched kinds cannot
    /// occur for validated scopes under one catalog; they are reported as
    /// non-empty so that the result stays conservative.
    fn intersect(&self, other: &Self) -> Option<Self> {
        match (self, other) {
            (Self::Enum(a), Self::Enum(b)) => {
                let i: BTreeSet<u32> = a.intersection(b).copied().collect();
                (!i.is_empty()).then_some(Self::Enum(i))
            }
            (Self::Concept(a), Self::Concept(b)) => {
                let i: BTreeSet<String> = a.intersection(b).cloned().collect();
                (!i.is_empty()).then_some(Self::Concept(i))
            }
            (Self::Num(a0, a1), Self::Num(b0, b1)) => {
                let lo = *a0.max(b0);
                let hi = *a1.min(b1);
                (lo <= hi).then_some(Self::Num(lo, hi))
            }
            _ => Some(self.clone()),
        }
    }

    /// Whether every value allowed by `self` is allowed by `outer`.
    /// Mismatched kinds are never proven contained.
    fn within(&self, outer: &Self) -> bool {
        match (self, outer) {
            (Self::Enum(a), Self::Enum(b)) => a.is_subset(b),
            (Self::Concept(a), Self::Concept(b)) => a.is_subset(b),
            (Self::Num(a0, a1), Self::Num(b0, b1)) => b0 <= a0 && a1 <= b1,
            _ => false,
        }
    }
}

/// One conjunction of field constraints. A field absent from the map is
/// unconstrained (its full validated domain, which is non-empty).
pub type Term = BTreeMap<FieldId, Constraint>;

/// Result of an overlap check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overlap {
    Disjoint,
    Overlapping,
    /// UNKNOWN_OVERLAP: a resource limit was reached. Treat as a conflict.
    Unknown,
}

impl Overlap {
    /// Only a proof of disjointness clears a conflict.
    pub fn is_conflict(self) -> bool {
        !matches!(self, Self::Disjoint)
    }
}

/// Normalises a scope expression. `None` means a limit was reached.
pub fn normal_form(expr: &ScopeExpr, limits: ScopeLimits) -> Option<Vec<Term>> {
    match expr {
        ScopeExpr::True => Some(vec![Term::new()]),
        ScopeExpr::Atom(atom) => {
            let mut term = Term::new();
            term.insert(atom.field().to_owned(), Constraint::from_atom(atom));
            Some(vec![term])
        }
        ScopeExpr::Or(items) => {
            let mut out = Vec::new();
            for item in items {
                out.extend(normal_form(item, limits)?);
                if out.len() > limits.max_dnf_terms {
                    return None;
                }
            }
            Some(out)
        }
        ScopeExpr::And(items) => {
            let mut acc = vec![Term::new()];
            for item in items {
                let next = normal_form(item, limits)?;
                let mut product = Vec::new();
                for left in &acc {
                    for right in &next {
                        if let Some(term) = conjoin(left, right) {
                            product.push(term);
                            if product.len() > limits.max_dnf_terms {
                                return None;
                            }
                        }
                    }
                }
                acc = product;
            }
            Some(acc)
        }
    }
}

/// Conjunction of two terms, or `None` when provably unsatisfiable.
fn conjoin(left: &Term, right: &Term) -> Option<Term> {
    let mut out = left.clone();
    for (field, constraint) in right {
        let merged = match out.get(field) {
            Some(existing) => existing.intersect(constraint)?,
            None => constraint.clone(),
        };
        out.insert(field.clone(), merged);
    }
    Some(out)
}

/// Checks whether two validated scopes may overlap.
///
/// Scopes for different endpoints never share an effect domain and are
/// `Disjoint`. Scopes interpreted under different catalog versions cannot be
/// compared and are `Unknown`.
pub fn overlap(a: &Scope, b: &Scope, catalog: &Catalog) -> Overlap {
    if a.endpoint != b.endpoint {
        return Overlap::Disjoint;
    }
    if a.catalog_version != catalog.version || b.catalog_version != catalog.version {
        return Overlap::Unknown;
    }
    let limits = catalog.limits;
    let (Some(ta), Some(tb)) = (normal_form(&a.expr, limits), normal_form(&b.expr, limits)) else {
        return Overlap::Unknown;
    };
    if ta.len().saturating_mul(tb.len()) > limits.max_overlap_pairs {
        return Overlap::Unknown;
    }
    for left in &ta {
        for right in &tb {
            if conjoin(left, right).is_some() {
                return Overlap::Overlapping;
            }
        }
    }
    Overlap::Disjoint
}

/// Whether `inner ⊆ outer` is proven (O 9: a takeover sub-domain must be
/// decidable). Sound but not complete: each normal-form term of `inner` must
/// lie inside one single term of `outer`. A term covered only by a union of
/// several `outer` terms, a limit hit or a version mismatch is "not proven",
/// and callers must then treat the domains as not safely split.
pub fn contains(outer: &Scope, inner: &Scope, catalog: &Catalog) -> bool {
    if outer.endpoint != inner.endpoint
        || outer.catalog_version != catalog.version
        || inner.catalog_version != catalog.version
    {
        return false;
    }
    let limits = catalog.limits;
    let (Some(to), Some(ti)) = (
        normal_form(&outer.expr, limits),
        normal_form(&inner.expr, limits),
    ) else {
        return false;
    };
    if to.len().saturating_mul(ti.len()) > limits.max_overlap_pairs {
        return false;
    }
    ti.iter().all(|t| to.iter().any(|o| term_within(t, o)))
}

/// A field `outer` constrains must be constrained at least as tightly by
/// `inner`; a field `outer` leaves free imposes nothing.
fn term_within(inner: &Term, outer: &Term) -> bool {
    outer
        .iter()
        .all(|(field, oc)| inner.get(field).is_some_and(|ic| ic.within(oc)))
}
