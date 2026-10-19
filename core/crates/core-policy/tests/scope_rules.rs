//! Scope DSL validation, evaluation and canonical encoding.

mod support;

use std::collections::BTreeMap;

use core_policy::canonical::{SCOPE_TAG, scope_bytes, scope_digest};
use core_policy::scope::evaluate;
use core_policy::{Endpoint, FieldValue, PolicyError, Scope, ScopeExpr, ScopeLimits, ScopeMatch};
use support::*;

fn new(endpoint: Endpoint, expr: ScopeExpr) -> Result<Scope, PolicyError> {
    Scope::new(endpoint, expr, &catalog())
}

#[test]
fn whitelist_rejects_fields_outside_endpoint() {
    // J_direct may only use state projections, never event concepts.
    let err = new(Endpoint::JDirect, c(&["a"])).unwrap_err();
    assert!(matches!(err, PolicyError::FieldNotAllowed { .. }), "{err}");
    let err = new(Endpoint::JDirect, e("event_kind", &[1])).unwrap_err();
    assert!(matches!(err, PolicyError::FieldNotAllowed { .. }), "{err}");
    // F_time has no whitelisted field in the fixture: only True is legal.
    assert!(new(Endpoint::FTime, n("x0", 0, 1)).is_err());
    assert!(new(Endpoint::FTime, ScopeExpr::True).is_ok());
}

#[test]
fn malformed_scopes_are_rejected() {
    let bad = [
        (n("nope", 0, 1), "unknown field"),
        (e("x0", &[1]), "kind mismatch"),
        (e("event_kind", &[9]), "enum outside domain"),
        (n("x0", 0, 2_000_000), "number outside domain"),
        (n("x0", 5, 4), "empty interval"),
        (and(vec![]), "empty and"),
        (or(vec![]), "empty or"),
        (e("event_kind", &[]), "empty set"),
        (c(&[""]), "empty concept id"),
    ];
    for (expr, why) in bad {
        assert!(new(Endpoint::FEvent, expr).is_err(), "{why}");
    }
}

#[test]
fn depth_node_and_set_limits_are_enforced() {
    let mut deep = e("event_kind", &[1]);
    for _ in 0..LIMITS.max_depth {
        deep = and(vec![deep]);
    }
    assert_eq!(
        new(Endpoint::FEvent, deep).unwrap_err(),
        PolicyError::LimitExceeded("depth")
    );

    let wide = or((0..LIMITS.max_nodes)
        .map(|i| n("x0", i as i64, i as i64))
        .collect());
    assert_eq!(
        new(Endpoint::FEvent, wide).unwrap_err(),
        PolicyError::LimitExceeded("nodes")
    );

    let tight = ScopeLimits {
        max_set_items: 2,
        ..LIMITS
    };
    let err = Scope::new(
        Endpoint::FEvent,
        e("event_kind", &[1, 2, 3]),
        &catalog_with(tight, "v"),
    );
    assert_eq!(err.unwrap_err(), PolicyError::LimitExceeded("set items"));
}

fn ctx(pairs: &[(&str, FieldValue)]) -> BTreeMap<String, FieldValue> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.clone()))
        .collect()
}

#[test]
fn evaluation_is_closed_interval_and_never_defaults_to_hit() {
    let cat = catalog();
    let scope = and(vec![e("event_kind", &[2]), n("x0", 10, 20)]);
    // 10 units at 3 decimals = 0.010; both ends are inclusive.
    for (x, want) in [
        (0.010, ScopeMatch::Hit),
        (0.020, ScopeMatch::Hit),
        (0.0205, ScopeMatch::Miss),
        (0.009, ScopeMatch::Miss),
    ] {
        let values = ctx(&[
            ("event_kind", FieldValue::Enum(2)),
            ("x0", FieldValue::Number(x)),
        ]);
        assert_eq!(evaluate(&scope, &values, &cat).unwrap(), want, "x0={x}");
    }
    let values = ctx(&[("event_kind", FieldValue::Enum(2))]);
    assert_eq!(
        evaluate(&scope, &values, &cat).unwrap(),
        ScopeMatch::MissingField("x0".into())
    );
    // A missing field inside OR still blocks; a hit elsewhere cannot hide it.
    let either = or(vec![e("event_kind", &[2]), n("x1", 0, 1)]);
    assert_eq!(
        evaluate(&either, &values, &cat).unwrap(),
        ScopeMatch::MissingField("x1".into())
    );
}

#[test]
fn canonical_bytes_are_stable_and_order_independent_for_sets() {
    let a = ev(and(vec![e("event_kind", &[3, 1, 2]), c(&["b", "a"])]));
    let b = ev(and(vec![e("event_kind", &[1, 2, 3]), c(&["a", "b"])]));
    assert_eq!(scope_bytes(&a).unwrap(), scope_bytes(&b).unwrap());
    assert_eq!(scope_digest(&a).unwrap(), scope_digest(&b).unwrap());
    let bytes = scope_bytes(&a).unwrap();
    assert_eq!(&bytes[..4], b"NCV1");
    assert_eq!(&bytes[8..12], &(SCOPE_TAG.len() as u32).to_le_bytes());
    // Endpoint and catalog version are part of identity.
    let j = at(Endpoint::GMeta, e("event_kind", &[1, 2, 3]));
    let f = ev(e("event_kind", &[1, 2, 3]));
    assert_ne!(scope_digest(&j).unwrap(), scope_digest(&f).unwrap());
}

#[test]
fn scope_digest_golden_vector() {
    // Hand-checked layout: NCV1 | 01000000 | tag | endpoint=1 | catalog | True.
    let scope = ev(ScopeExpr::True);
    let mut want = b"NCV1".to_vec();
    want.extend(1u32.to_le_bytes());
    want.extend((SCOPE_TAG.len() as u32).to_le_bytes());
    want.extend(SCOPE_TAG.as_bytes());
    want.extend(1u32.to_le_bytes());
    want.extend(("fixture-catalog-1".len() as u32).to_le_bytes());
    want.extend(b"fixture-catalog-1");
    want.extend(0u32.to_le_bytes());
    assert_eq!(scope_bytes(&scope).unwrap(), want);
}
