//! Structural fixture catalog for logic tests only. It carries no semantic
//! meaning and must not be used to claim behavioural validity (AT12).
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use core_policy::{Atom, Catalog, Endpoint, FieldKind, FieldSpec, Scope, ScopeExpr, ScopeLimits};

pub const LIMITS: ScopeLimits = ScopeLimits {
    max_depth: 8,
    max_nodes: 64,
    max_set_items: 16,
    max_dnf_terms: 32,
    max_overlap_pairs: 256,
};

pub fn catalog() -> Catalog {
    catalog_with(LIMITS, "fixture-catalog-1")
}

pub fn catalog_with(limits: ScopeLimits, version: &str) -> Catalog {
    let mut fields = BTreeMap::new();
    let all = |eps: &[Endpoint]| eps.iter().copied().collect::<BTreeSet<_>>();
    fields.insert(
        "event_kind".to_owned(),
        FieldSpec {
            kind: FieldKind::Enum {
                domain: (1..=8).collect(),
            },
            endpoints: all(&[Endpoint::FEvent, Endpoint::GMeta]),
        },
    );
    fields.insert(
        "concept".to_owned(),
        FieldSpec {
            kind: FieldKind::Concept {
                registry_version: "fixture-registry-1".into(),
            },
            endpoints: all(&[Endpoint::FEvent]),
        },
    );
    for i in 0..4 {
        fields.insert(
            format!("x{i}"),
            FieldSpec {
                kind: FieldKind::Numeric {
                    min: -1_000_000,
                    max: 1_000_000,
                    decimals: 3,
                },
                endpoints: all(&[Endpoint::FEvent, Endpoint::JDirect, Endpoint::GMeta]),
            },
        );
    }
    Catalog {
        version: version.to_owned(),
        fields,
        limits,
    }
}

pub fn e(field: &str, codes: &[u32]) -> ScopeExpr {
    ScopeExpr::Atom(Atom::EnumIn {
        field: field.into(),
        values: codes.iter().copied().collect(),
    })
}

pub fn c(ids: &[&str]) -> ScopeExpr {
    ScopeExpr::Atom(Atom::ConceptIn {
        field: "concept".into(),
        concepts: ids.iter().map(|s| (*s).to_owned()).collect(),
    })
}

pub fn n(field: &str, lo: i64, hi: i64) -> ScopeExpr {
    ScopeExpr::Atom(Atom::NumIn {
        field: field.into(),
        lo,
        hi,
    })
}

pub fn and(items: Vec<ScopeExpr>) -> ScopeExpr {
    ScopeExpr::And(items)
}

pub fn or(items: Vec<ScopeExpr>) -> ScopeExpr {
    ScopeExpr::Or(items)
}

pub fn ev(expr: ScopeExpr) -> Scope {
    Scope::new(Endpoint::FEvent, expr, &catalog()).expect("valid fixture scope")
}

pub fn at(endpoint: Endpoint, expr: ScopeExpr) -> Scope {
    Scope::new(endpoint, expr, &catalog()).expect("valid fixture scope")
}
