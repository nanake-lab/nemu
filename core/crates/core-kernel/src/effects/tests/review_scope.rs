//! Regression tests for the cross-review of PR #22, findings B4–B5: the
//! compiler re-validates every scope, and declared conflicts are kept and
//! checked from both sides. The first test of each group is the reviewer's
//! counterexample, kept as given.

use core_policy::{Endpoint, Scope, ScopeExpr};
use core_types::generated::EffectOwner;

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

// ---------------------------------------------------------------- B4 ----

#[test]
fn b4_review_compiler_revalidates_scope() {
    let mut forged = scope(Endpoint::FEvent, kind(&[1]));
    forged.endpoint = Endpoint::JDirect;
    let r = r_effect("a", forged, params(&[(0, 0.25)]), 0.25);
    assert!(
        compile(&[r], &[], &validity(), 0).is_err(),
        "J scope containing event_kind compiled"
    );
}

/// Scopes that `Scope::new` refuses, built directly through the public
/// fields, as a loader or replay path could.
fn forged_scopes() -> Vec<(&'static str, Scope)> {
    let raw = |expr| Scope {
        endpoint: Endpoint::JDirect,
        catalog_version: CATALOG.into(),
        expr,
    };
    let mut deep = x0(0, 1);
    for _ in 0..10 {
        deep = ScopeExpr::And(vec![deep]);
    }
    vec![
        ("foreign field", raw(kind(&[1]))),
        ("over-limit depth", raw(deep)),
        ("empty combination", raw(ScopeExpr::Or(vec![]))),
        ("out-of-domain value", raw(x0(0, 2_000_000))),
    ]
}

#[test]
fn b4_both_compile_paths_reject_scopes_that_fail_validation() {
    for (case, s) in forged_scopes() {
        let r = r_effect("a", s.clone(), params(&[(0, 0.25)]), 0.25);
        assert!(
            matches!(
                compile(&[r], &[], &validity(), 0),
                Err(StructuralFault::Integrity { .. })
            ),
            "R path accepted {case}"
        );
        let th = theta("m", "a", s, params(&[(0, 0.25)]), 0.25);
        assert!(
            theta_detail(compile(&[], &[th], &validity(), 0)).starts_with("scope fails validation"),
            "θ path accepted {case}"
        );
    }
    // A θ scope interpreted under another catalog version is rejected; an R
    // one is withheld as an interpreter mismatch.
    let mut stale = jd(ScopeExpr::True);
    stale.catalog_version = "older-catalog".into();
    let th = theta("m", "a", stale.clone(), params(&[(0, 0.25)]), 0.25);
    assert!(theta_detail(compile(&[], &[th], &validity(), 0)).starts_with("scope interpreter"));
    let r = r_effect("a", stale, params(&[(0, 0.25)]), 0.25);
    assert_eq!(
        excluded(&[r], &validity()),
        [ExclusionReason::InterpreterMismatch]
    );
}

#[test]
fn b4_disabled_module_still_needs_a_verifiable_takeover_domain() {
    let mut th = theta(
        "m",
        "a",
        forged_scopes().remove(0).1,
        params(&[(0, 0.25)]),
        0.25,
    );
    th.module.enabled = false;
    assert!(theta_detail(compile(&[], &[th], &validity(), 0)).starts_with("scope fails"));
}

// ---------------------------------------------------------------- B5 ----

fn candidate(rid: &str, s: Scope, conflicts: &[&str]) -> CandidateEffect {
    let p = params(&[(1, 0.25)]);
    CandidateEffect {
        contract: contract(rid, "1", EffectOwner::R_OWNER, &s, &p, 0.25),
        scope: s,
        parameters: p,
        conflict_set: conflicts.iter().map(|c| (*c).into()).collect(),
    }
}

fn existing_with_conflict(declared: &[&str]) -> AuthorizedEffectSet {
    let mut r = r_effect("a", jd(x0(0, 100)), params(&[(0, 0.25)]), 0.25);
    r.conflict_set = declared.iter().map(|c| (*c).into()).collect();
    compile(&[r], &[], &validity(), 0).unwrap().0
}

#[test]
fn b5_review_existing_conflict_is_not_erased() {
    let set = existing_with_conflict(&["b"]);
    assert!(
        admit_candidate(&set, &candidate("b", jd(ScopeExpr::True), &[]), &config()).is_err(),
        "old a->b conflict lost"
    );
}

#[test]
fn b5_conflicts_are_checked_from_both_sides_only_where_scopes_overlap() {
    let conflict = |r| matches!(r, Err(AdmissionRejection::Conflict(_)));
    let cfg = config();
    let overlapping = jd(x0(50, 150));
    let disjoint = jd(x0(200, 300));
    // Only the old effect declares, only the candidate declares, or both.
    assert!(conflict(admit_candidate(
        &existing_with_conflict(&["b"]),
        &candidate("b", overlapping.clone(), &[]),
        &cfg
    )));
    assert!(conflict(admit_candidate(
        &existing_with_conflict(&[]),
        &candidate("b", overlapping.clone(), &["a"]),
        &cfg
    )));
    assert!(conflict(admit_candidate(
        &existing_with_conflict(&["b"]),
        &candidate("b", overlapping, &["a"]),
        &cfg
    )));
    // Disjoint scopes never act together, so a declared conflict is moot.
    assert!(
        admit_candidate(
            &existing_with_conflict(&["b"]),
            &candidate("b", disjoint.clone(), &["a"]),
            &cfg
        )
        .is_ok()
    );
    // A requalified composite is its own responsibility with no conflict.
    let mut composite = existing_with_conflict(&[]);
    composite.effects.clear();
    assert!(
        admit_candidate(
            &composite,
            &candidate("a+b", jd(ScopeExpr::True), &[]),
            &cfg
        )
        .is_ok()
    );
}

#[test]
fn b5_committed_effects_with_a_declared_conflict_cannot_compile_together() {
    let mut a = r_effect("a", jd(x0(0, 100)), params(&[(0, 0.25)]), 0.25);
    a.conflict_set = ["b".into()].into();
    let b = r_effect("b", jd(x0(50, 150)), params(&[(1, 0.25)]), 0.25);
    assert_eq!(
        compile(&[a.clone(), b], &[], &validity(), 0).unwrap_err(),
        StructuralFault::DeclaredConflict {
            first: "a".into(),
            second: "b".into()
        }
    );
    let apart = r_effect("b", jd(x0(200, 300)), params(&[(1, 0.25)]), 0.25);
    assert_eq!(
        compile(&[a, apart], &[], &validity(), 0)
            .unwrap()
            .0
            .effects
            .len(),
        2
    );
}
