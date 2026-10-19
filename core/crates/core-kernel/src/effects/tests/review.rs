//! Regression tests for the cross-review of PR #22, findings B1–B3 (owner,
//! dependency closure, ValidityContract). The first test of each group is
//! the reviewer's counterexample, kept as given; the others cover the
//! positive and negative cases the review asked for. B4–B5 are in
//! `review_scope.rs`, B6–B7 in `review_numeric.rs`.

use core_policy::{Endpoint, Scope, ScopeExpr};

use super::super::*;
use super::fixture::*;

fn jd(e: ScopeExpr) -> Scope {
    scope(Endpoint::JDirect, e)
}

fn theta_detail(r: Result<(AuthorizedEffectSet, CompileAudit), StructuralFault>) -> String {
    match r {
        Err(StructuralFault::ThetaInvalid { detail, .. }) => detail,
        other => panic!("expected θ quarantine, got {other:?}"),
    }
}

fn excluded(r: &[REffectRecord], v: &ValiditySnapshot) -> Vec<ExclusionReason> {
    let (set, audit) = compile(r, &[], v, 0).unwrap();
    assert!(set.effects.is_empty());
    audit.excluded.into_iter().map(|(_, _, why)| why).collect()
}

// ---------------------------------------------------------------- B1 ----

#[test]
fn b1_review_partial_takeover_preserves_other_endpoint() {
    let r = r_effect(
        "same",
        scope(Endpoint::FEvent, ScopeExpr::True),
        params(&[(0, 0.25)]),
        0.25,
    );
    let th = theta("m", "same", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    let (set, _) = compile(&[r], &[th], &validity(), 0).unwrap();
    assert!(
        set.effects.iter().any(|e| e.endpoint == Endpoint::FEvent),
        "untransferred F_event disappeared"
    );
}

#[test]
fn b1_review_partial_takeover_preserves_disjoint_scope() {
    let r = r_effect("same", jd(x0(200, 300)), params(&[(0, 0.25)]), 0.25);
    let th = theta("m", "same", jd(x0(0, 100)), params(&[(0, 0.25)]), 0.25);
    let (set, _) = compile(&[r], &[th], &validity(), 0).unwrap();
    assert!(
        set.effects.iter().any(|e| e.owner == Owner::R),
        "untransferred R domain disappeared"
    );
}

#[test]
fn b1_r_inside_the_taken_sub_domain_is_suppressed() {
    let r = r_effect("same", jd(x0(10, 20)), params(&[(0, 0.25)]), 0.25);
    let th = theta("m", "same", jd(x0(0, 100)), params(&[(0, 0.25)]), 0.25);
    let (set, audit) = compile(&[r], &[th], &validity(), 0).unwrap();
    assert_eq!(set.effects.len(), 1);
    assert_eq!(set.effects[0].owner, Owner::Theta);
    assert_eq!(audit.excluded[0].2, ExclusionReason::TakenOverByTheta);
}

#[test]
fn b1_unsafe_split_is_a_fault_not_a_wider_suppression() {
    // R [50, 150] neither lies inside θ's [0, 100] nor is disjoint from it.
    let r = r_effect("same", jd(x0(50, 150)), params(&[(0, 0.25)]), 0.25);
    let th = theta("m", "same", jd(x0(0, 100)), params(&[(0, 0.25)]), 0.25);
    assert_eq!(
        compile(&[r], &[th], &validity(), 0).unwrap_err(),
        StructuralFault::AmbiguousTakeover {
            responsibility_id: "same".into()
        }
    );
}

#[test]
fn b1_disabled_tombstone_keeps_suppression_only_inside_its_sub_domain() {
    let inside = r_effect("same", jd(x0(10, 20)), params(&[(0, 0.25)]), 0.25);
    let mut outside = r_effect("same", jd(x0(200, 300)), params(&[(1, 0.25)]), 0.25);
    outside.contract.causal_revision = "rev-2".into();
    let mut th = theta("m", "same", jd(x0(0, 100)), params(&[(0, 0.25)]), 0.25);
    th.module.enabled = false;
    let (set, audit) = compile(&[inside, outside], &[th], &validity(), 0).unwrap();
    assert_eq!(set.effects.len(), 1, "remaining R domain must stay");
    assert_eq!(set.effects[0].causal_revision, "rev-2");
    assert_eq!(audit.excluded[0].2, ExclusionReason::TakenOverByTheta);
}

#[test]
fn b1_binding_must_agree_with_module_on_every_domain_field() {
    let base = theta("m", "same", jd(x0(0, 100)), params(&[(0, 0.25)]), 0.25);
    let other_scope = theta("m", "same", jd(x0(0, 50)), params(&[(0, 0.25)]), 0.25);
    type Tamper = fn(&mut ThetaModuleRecord, &ThetaModuleRecord);
    let cases: [(&str, Tamper); 6] = [
        ("endpoint", |r, _| {
            r.binding.endpoint = Endpoint::FEvent.into();
        }),
        ("binding scope", |r, o| {
            r.binding.scope = o.binding.scope.clone()
        }),
        ("module scope", |r, o| {
            r.module.scope = o.module.scope.clone()
        }),
        ("carrier", |r, _| r.module.carrier = plain(0xC9)),
        ("causal revision", |r, _| {
            r.binding.causal_revision = "other".into();
        }),
        ("source slice", |r, _| {
            r.binding.source_slice_id = "other".into()
        }),
    ];
    for (field, tamper) in cases {
        let mut rec = base.clone();
        tamper(&mut rec, &other_scope);
        let detail = theta_detail(compile(&[], &[rec], &validity(), 0));
        assert!(detail.ends_with(field), "{field}: {detail}");
    }
}

// ---------------------------------------------------------------- B2 ----

#[test]
fn b2_review_theta_invalidated_qualification_quarantines() {
    let th = theta("m", "a", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    let mut v = validity();
    v.invalidated.insert(d(0xA1));
    assert!(
        compile(&[], &[th], &v, 0).is_err(),
        "invalidated theta qualification still authorised"
    );
}

#[test]
fn b2_review_r_qualification_dependency_barrier() {
    let r = r_effect("a", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    let mut v = validity();
    v.dependency_graph.insert(d(0xA1), [d(0xD2)].into());
    v.dependency_graph.insert(d(0xD2), Default::default());
    v.invalidated.insert(d(0xD2));
    let (set, _) = compile(&[r], &[], &v, 0).unwrap();
    assert!(
        set.effects.is_empty(),
        "invalidated qualification ancestor still authorised"
    );
}

#[test]
fn b2_theta_qualification_upstream_failure_quarantines() {
    let th = theta("m", "a", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    let mut v = validity();
    v.dependency_graph.insert(d(0xA1), [d(0xD2)].into());
    v.dependency_graph.insert(d(0xD2), Default::default());
    v.invalidated.insert(d(0xD2));
    assert!(theta_detail(compile(&[], &[th], &v, 0)).starts_with("dependency closure"));
}

#[test]
fn b2_unknown_or_cyclic_qualification_and_carrier_block_authorisation() {
    let r = r_effect("a", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    let th = theta("m", "b", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    let mut unknown_qual = validity();
    unknown_qual.dependency_graph.remove(&d(0xA1));
    let mut unknown_carrier = validity();
    unknown_carrier.dependency_graph.remove(&d(0xC0));
    let mut cycle = validity();
    cycle.dependency_graph.insert(d(0xA1), [d(0xD2)].into());
    cycle.dependency_graph.insert(d(0xD2), [d(0xA1)].into());
    for v in [unknown_qual, unknown_carrier, cycle] {
        assert_eq!(
            excluded(std::slice::from_ref(&r), &v),
            [ExclusionReason::DependencyUnresolved]
        );
        assert!(
            theta_detail(compile(&[], std::slice::from_ref(&th), &v, 0))
                .starts_with("dependency closure")
        );
    }
}

#[test]
fn b2_malformed_references_are_rejected_not_dropped() {
    let mut bad = plain(0xA1);
    bad.digest.0.pop();
    let mut r = r_effect("a", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    r.contract.dependencies.push(bad.clone());
    assert!(matches!(
        compile(&[r], &[], &validity(), 0),
        Err(StructuralFault::Integrity { .. })
    ));
    let mut th = theta("m", "a", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    th.module.mechanism_effect.qualifications.push(bad);
    assert!(theta_detail(compile(&[], &[th], &validity(), 0)).starts_with("malformed"));
}

#[test]
fn b2_valid_qualification_upstream_and_assumptions_are_authorised() {
    let r = r_effect("a", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    let mut th = theta("m", "b", jd(ScopeExpr::True), params(&[(1, 0.25)]), 0.25);
    th.validity.assumptions.insert(d(0xE5));
    bind_validity(&mut th);
    let mut v = validity();
    v.dependency_graph.insert(d(0xA1), [d(0xD2)].into());
    v.dependency_graph.insert(d(0xD2), Default::default());
    v.dependency_graph.insert(d(0xE5), Default::default());
    let (set, _) = compile(&[r], std::slice::from_ref(&th), &v, 0).unwrap();
    assert_eq!(set.effects.len(), 2);
    assert!(set.dependency_dag.contains_key(&d(0xD2)));
    assert!(set.dependency_dag.contains_key(&d(0xE5)));
    // A withdrawn or unknown assumption of the ValidityContract quarantines.
    v.invalidated.insert(d(0xE5));
    assert!(theta_detail(compile(&[], std::slice::from_ref(&th), &v, 0)).starts_with("dependency"));
    v.invalidated.clear();
    v.dependency_graph.remove(&d(0xE5));
    assert!(theta_detail(compile(&[], &[th], &v, 0)).starts_with("dependency"));
}

// ---------------------------------------------------------------- B3 ----

#[test]
fn b3_review_unknown_theta_validity_reference_rejected() {
    let mut th = theta("m", "a", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    th.module.validity_contract = plain(0xEE);
    assert!(
        compile(&[], &[th], &validity(), 0).is_err(),
        "unresolved validity reference ignored"
    );
}

#[test]
fn b3_side_loaded_expiry_or_conditions_cannot_replace_the_committed_contract() {
    let base = theta("m", "a", jd(ScopeExpr::True), params(&[(0, 0.25)]), 0.25);
    let unbound = "validity contract reference does not bind the applied contract";
    let mut longer = base.clone();
    longer.validity.valid_until = Some(ProcessTime(1_000 * SECOND));
    assert_eq!(
        theta_detail(compile(&[], &[longer], &validity(), 0)),
        unbound
    );
    let mut fewer = base.clone();
    fewer.validity.valid_until = None;
    fewer.validity.conditions = [d(0x01)].into();
    assert_eq!(
        theta_detail(compile(&[], &[fewer], &validity(), 0)),
        unbound
    );
    let mut wrong_type = base.clone();
    wrong_type.module.validity_contract.object_type = "echo.fixture-object".into();
    assert_eq!(
        theta_detail(compile(&[], &[wrong_type], &validity(), 0)),
        unbound
    );
    // A contract that is bound but covers another scope is rejected too.
    let mut other_scope = base.clone();
    other_scope.validity.scope_digest = d(0x55);
    bind_validity(&mut other_scope);
    assert_eq!(
        theta_detail(compile(&[], &[other_scope], &validity(), 0)),
        "validity contract covers a different scope"
    );
    assert!(compile(&[], &[base], &validity(), 0).is_ok());
}
