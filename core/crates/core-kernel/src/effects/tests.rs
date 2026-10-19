//! P5-09 structural tests. Acceptance IDs appear in test names: J06 and the
//! structural part of F06; invariants I06, I07, I08, I12, I21, I22.

mod fixture;
mod review;
mod review_numeric;
mod review_scope;

use std::collections::BTreeMap;

use core_policy::{Endpoint, FieldValue, ScopeExpr};
use core_types::generated::{CouplingPhase, EffectOwner};
use fixture::*;

use super::*;

fn jd(expr: ScopeExpr) -> core_policy::Scope {
    scope(Endpoint::JDirect, expr)
}

fn fe(expr: ScopeExpr) -> core_policy::Scope {
    scope(Endpoint::FEvent, expr)
}

#[test]
fn i21_compile_is_pure_and_repeated_reads_do_not_accumulate_decay() {
    let r = vec![r_effect(
        "resp-a",
        jd(ScopeExpr::True),
        params(&[(0, 0.5)]),
        1.0,
    )];
    let v = validity();
    let before = (r.clone(), v.clone());
    let (once, _) = compile(&r, &[], &v, 7 * SECOND).unwrap();
    let (again, _) = compile(&r, &[], &v, 7 * SECOND).unwrap();
    // Reading at an earlier time first must not change a later read.
    let _ = compile(&r, &[], &v, 3 * SECOND).unwrap();
    let (later, _) = compile(&r, &[], &v, 7 * SECOND).unwrap();
    assert_eq!(once, again);
    assert_eq!(once, later);
    assert_eq!(before, (r, v), "inputs must be untouched");
    let expected = libm::exp(-0.05 * 7.0);
    assert_eq!(once.effects[0].factor, expected);
}

#[test]
fn i06_only_active_r_effects_are_authorised() {
    for phase in [
        CouplingPhase::ZERO,
        CouplingPhase::SHADOW,
        CouplingPhase::QUALIFIED_SHADOW,
        CouplingPhase::SUSPENDED,
        CouplingPhase::SUPERSEDED,
    ] {
        let mut rec = r_effect("resp-a", jd(ScopeExpr::True), params(&[(0, 0.5)]), 1.0);
        rec.phase = phase;
        let (set, audit) = compile(&[rec], &[], &validity(), SECOND).unwrap();
        assert!(set.effects.is_empty(), "{phase:?}");
        assert_eq!(audit.excluded[0].2, ExclusionReason::NotActive);
    }
}

#[test]
fn i07_theta_takeover_suppresses_r_even_after_legal_disable() {
    let s = jd(ScopeExpr::True);
    let r = r_effect("resp-a", s.clone(), params(&[(0, 0.5)]), 1.0);
    let mut th = theta("mod-1", "resp-a", s, params(&[(0, 0.4)]), 1.0);
    let (set, audit) = compile(
        std::slice::from_ref(&r),
        std::slice::from_ref(&th),
        &validity(),
        SECOND,
    )
    .unwrap();
    assert_eq!(set.effects.len(), 1);
    assert_eq!(set.effects[0].owner, Owner::Theta);
    assert_eq!(audit.excluded[0].2, ExclusionReason::TakenOverByTheta);
    // C disposition disables the module: R must not resume (no double count).
    th.module.enabled = false;
    let (set, audit) = compile(&[r], &[th], &validity(), SECOND).unwrap();
    assert!(set.effects.is_empty());
    assert_eq!(audit.excluded[0].2, ExclusionReason::TakenOverByTheta);
}

/// Suppression is keyed by the stable responsibility, so a new causal
/// revision of the R entry does not release it. The R record has no separate
/// evidence-revision field yet; a true evidence-only path is a follow-up.
#[test]
fn i08_new_causal_revision_does_not_release_a_taken_over_responsibility() {
    let s = jd(ScopeExpr::True);
    let p = params(&[(0, 0.5)]);
    let mut r = r_effect("resp-a", s.clone(), p.clone(), 1.0);
    r.contract.causal_revision = "rev-2-new-evidence".into();
    let th = theta("mod-1", "resp-a", s, p, 1.0);
    let (set, _) = compile(&[r], &[th], &validity(), SECOND).unwrap();
    assert_eq!(set.effects.len(), 1);
    assert_eq!(set.effects[0].owner, Owner::Theta);
}

#[test]
fn i07_two_owners_for_one_responsibility_and_domain_is_a_structural_fault() {
    let p = params(&[(1, 0.2)]);
    let a = theta("mod-1", "resp-a", jd(x0(0, 100)), p.clone(), 0.5);
    let b = theta("mod-2", "resp-a", jd(x0(50, 150)), p.clone(), 0.5);
    let err = compile(&[], &[a.clone(), b], &validity(), SECOND).unwrap_err();
    assert_eq!(
        err,
        StructuralFault::DuplicateOwner {
            responsibility_id: "resp-a".into()
        }
    );
    // Provably disjoint sub-domains of one responsibility are allowed.
    let c = theta("mod-3", "resp-a", jd(x0(200, 300)), p.clone(), 0.5);
    assert!(compile(&[], &[a, c], &validity(), SECOND).is_ok());
    // Two active R revisions overlapping in one domain are also a fault.
    let r1 = r_effect("resp-b", jd(x0(0, 10)), p.clone(), 0.5);
    let mut r2 = r_effect("resp-b", jd(x0(5, 15)), p, 0.5);
    r2.contract.causal_revision = "rev-2".into();
    assert!(matches!(
        compile(&[r1, r2], &[], &validity(), SECOND),
        Err(StructuralFault::DuplicateOwner { .. })
    ));
}

#[test]
fn i22_theta_outside_validity_contract_quarantines_instead_of_local_disable() {
    let base = theta(
        "mod-1",
        "resp-a",
        jd(ScopeExpr::True),
        params(&[(0, 0.4)]),
        1.0,
    );
    let detail = |r: Result<_, StructuralFault>| match r {
        Err(StructuralFault::ThetaInvalid { detail, .. }) => detail,
        other => panic!("expected θ quarantine, got {other:?}"),
    };
    // Expired finite validity.
    let expired = compile(&[], std::slice::from_ref(&base), &validity(), 100 * SECOND);
    assert_eq!(detail(expired), "validity contract expired");
    // No finite expiry and no verifiable condition (contract legally bound,
    // so the rejection is about its content, not its reference).
    let mut open = base.clone();
    open.validity.valid_until = None;
    bind_validity(&mut open);
    assert_eq!(
        detail(compile(&[], &[open], &validity(), SECOND)),
        "no finite validity or verifiable condition"
    );
    // Continuing condition present but not currently verified.
    let mut cond = base.clone();
    cond.validity.valid_until = None;
    cond.validity.conditions = [d(0x77)].into_iter().collect();
    bind_validity(&mut cond);
    assert_eq!(
        detail(compile(
            &[],
            std::slice::from_ref(&cond),
            &validity(),
            SECOND
        )),
        "continuing-validity condition not satisfied"
    );
    let mut v = validity();
    v.satisfied.insert(d(0x77));
    assert!(compile(&[], &[cond], &v, SECOND).is_ok());
}

#[test]
fn i12_failure_barrier_blocks_effects_immediately() {
    let mut v = validity();
    v.invalidated.insert(d(0xD1));
    let th = theta(
        "mod-1",
        "resp-a",
        jd(ScopeExpr::True),
        params(&[(0, 0.4)]),
        1.0,
    );
    assert!(matches!(
        compile(&[], &[th], &v, SECOND),
        Err(StructuralFault::ThetaInvalid { .. })
    ));
    let r = r_effect("resp-b", jd(ScopeExpr::True), params(&[(0, 0.4)]), 1.0);
    let (set, audit) = compile(&[r], &[], &v, SECOND).unwrap();
    assert!(set.effects.is_empty());
    assert_eq!(audit.excluded[0].2, ExclusionReason::Invalidated);
}

#[test]
fn r_certificate_expiry_and_dependency_faults_withhold_authorisation() {
    let r = r_effect("resp-a", jd(ScopeExpr::True), params(&[(0, 0.4)]), 1.0);
    let mut v = validity();
    v.certificates.insert(
        d(0xA1),
        CertificateValidity {
            qualified: true,
            valid_until: Some(ProcessTime(5 * SECOND)),
        },
    );
    let (set, _) = compile(std::slice::from_ref(&r), &[], &v, SECOND).unwrap();
    assert_eq!(
        set.expires_at,
        Some(ProcessTime(5 * SECOND)),
        "cache must end at expiry (AT09)"
    );
    let (set, audit) = compile(std::slice::from_ref(&r), &[], &v, 5 * SECOND).unwrap();
    assert!(set.effects.is_empty());
    assert_eq!(audit.excluded[0].2, ExclusionReason::NoValidQualification);
    // Unknown dependency and a cycle both block the effect.
    let mut unknown = validity();
    unknown.dependency_graph.clear();
    let mut cycle = validity();
    cycle
        .dependency_graph
        .insert(d(0xD1), [d(0xD2)].into_iter().collect());
    cycle
        .dependency_graph
        .insert(d(0xD2), [d(0xD1)].into_iter().collect());
    for snap in [unknown, cycle] {
        let (set, audit) = compile(std::slice::from_ref(&r), &[], &snap, SECOND).unwrap();
        assert!(set.effects.is_empty());
        assert_eq!(audit.excluded[0].2, ExclusionReason::DependencyUnresolved);
    }
}

#[test]
fn integrity_and_time_faults_are_structural() {
    let mut tampered = r_effect("resp-a", jd(ScopeExpr::True), params(&[(0, 0.4)]), 1.0);
    tampered.parameters = params(&[(0, 0.9)]);
    assert!(matches!(
        compile(&[tampered], &[], &validity(), SECOND),
        Err(StructuralFault::Integrity { .. })
    ));
    let mut late = r_effect("resp-a", jd(ScopeExpr::True), params(&[(0, 0.4)]), 1.0);
    late.anchor = ProcessTime(10 * SECOND);
    assert!(matches!(
        compile(&[late], &[], &validity(), SECOND),
        Err(StructuralFault::TimeBeforeAnchor { .. })
    ));
    let over = r_effect(
        "resp-a",
        jd(ScopeExpr::True),
        params(&[(0, 0.7), (1, -0.7)]),
        1.0,
    );
    assert!(matches!(
        compile(&[over], &[], &validity(), SECOND),
        Err(StructuralFault::Integrity { .. })
    ));
}

#[test]
fn j06_out_of_scope_and_unauthorised_effects_contribute_zero() {
    let cfg = config();
    let hit = r_effect("resp-a", jd(x0(0, 100)), params(&[(0, 0.5)]), 1.0);
    let mut shadow = r_effect("resp-b", jd(ScopeExpr::True), params(&[(1, 0.5)]), 1.0);
    shadow.phase = CouplingPhase::SHADOW;
    let (set, _) = compile(&[hit, shadow], &[], &validity(), 0).unwrap();
    let outside = TypedContext::new(
        Endpoint::JDirect,
        &cfg.catalog,
        [("x0".into(), FieldValue::Number(0.5))].into(),
    )
    .unwrap();
    let out = evaluate_effects(&set, &outside, &cfg.catalog).unwrap();
    assert!(out.values.is_empty() && out.receipt.is_empty());
    let inside = TypedContext::new(
        Endpoint::JDirect,
        &cfg.catalog,
        [("x0".into(), FieldValue::Number(0.05))].into(),
    )
    .unwrap();
    let out = evaluate_effects(&set, &inside, &cfg.catalog).unwrap();
    assert_eq!(
        out.values,
        BTreeMap::from([(0, 0.5)]),
        "shadow index 1 must stay zero"
    );
    assert_eq!(out.receipt.len(), 1);
}

#[test]
fn j06_candidate_over_budget_or_cancelling_is_rejected_and_existing_kept() {
    let cfg = config();
    let existing = r_effect("resp-a", jd(ScopeExpr::True), params(&[(0, 1.5)]), 1.5);
    let (set, _) = compile(&[existing], &[], &validity(), 0).unwrap();
    let snapshot = set.clone();
    let cand =
        |rid: &str, s: core_policy::Scope, p: EffectParameters, bound: f64| CandidateEffect {
            contract: contract(rid, "c-1", EffectOwner::R_OWNER, &s, &p, bound),
            scope: s,
            parameters: p,
            conflict_set: Default::default(),
        };
    // Opposite sign would cancel numerically; absolute budget still rejects.
    let cancel = cand("resp-b", jd(ScopeExpr::True), params(&[(0, -1.0)]), 1.0);
    assert_eq!(
        admit_candidate(&set, &cancel, &cfg),
        Err(AdmissionRejection::BudgetExceeded { endpoint: 3 })
    );
    assert_eq!(set, snapshot, "existing effects are not scaled down");
    // Within budget is admitted.
    let small = cand("resp-c", jd(ScopeExpr::True), params(&[(2, 0.4)]), 0.4);
    assert_eq!(admit_candidate(&set, &small, &cfg), Ok(()));
    // Same responsibility again is a duplicate, even with a new revision.
    let dup = cand("resp-a", jd(x0(0, 10)), params(&[(2, 0.1)]), 0.1);
    assert!(matches!(
        admit_candidate(&set, &dup, &cfg),
        Err(AdmissionRejection::DuplicateResponsibility(_))
    ));
    // Declared conflict overlapping an existing effect is rejected.
    let mut conflict = cand("resp-d", jd(ScopeExpr::True), params(&[(3, 0.1)]), 0.1);
    conflict.conflict_set.insert("resp-a".into());
    assert!(matches!(
        admit_candidate(&set, &conflict, &cfg),
        Err(AdmissionRejection::Conflict(_))
    ));
}

#[test]
fn j06_candidate_with_non_whitelisted_field_is_rejected() {
    let cfg = config();
    let (set, _) = compile(&[], &[], &validity(), 0).unwrap();
    // A J_direct scope may not read event features; build it as F_event and
    // relabel to bypass the constructor, as a forged proposal would.
    let mut forged = fe(kind(&[1]));
    forged.endpoint = Endpoint::JDirect;
    let p = params(&[(0, 0.1)]);
    let cand = CandidateEffect {
        contract: contract("resp-x", "c-1", EffectOwner::R_OWNER, &forged, &p, 0.1),
        scope: forged,
        parameters: p,
        conflict_set: Default::default(),
    };
    assert!(matches!(
        admit_candidate(&set, &cand, &cfg),
        Err(AdmissionRejection::Invalid(_))
    ));
}

#[test]
fn compile_budget_overflow_in_stored_state_is_a_fault() {
    let a = r_effect("resp-a", jd(ScopeExpr::True), params(&[(0, 1.5)]), 1.5);
    let b = r_effect("resp-b", jd(ScopeExpr::True), params(&[(1, -1.5)]), 1.5);
    assert_eq!(
        compile(&[a, b], &[], &validity(), 0).unwrap_err(),
        StructuralFault::BudgetExceeded { endpoint: 3 }
    );
}

#[test]
fn f06_drive_proposal_may_not_reencode_an_r_or_theta_effect() {
    let r = r_effect("resp-a", fe(kind(&[1])), params(&[(0, 0.3)]), 0.3);
    let (set, _) = compile(&[r], &[], &validity(), 0).unwrap();
    assert_eq!(audit_drive_route(&set, &[]), Ok(()));
    assert_eq!(
        audit_drive_route(&set, &["resp-a".into()]),
        Err(RouteFault::DuplicateRoute("resp-a".into()))
    );
    assert!(audit_drive_route(&set, &["resp-zzz".into()]).is_err());
}

#[test]
fn evaluate_rejects_missing_inputs_foreign_fields_and_version_mismatch() {
    let cfg = config();
    let r = r_effect(
        "resp-a",
        fe(ScopeExpr::And(vec![kind(&[2]), x0(0, 100)])),
        params(&[(0, 0.3)]),
        0.3,
    );
    let (set, _) = compile(&[r], &[], &validity(), 0).unwrap();
    let partial = TypedContext::new(
        Endpoint::FEvent,
        &cfg.catalog,
        [("event_kind".into(), FieldValue::Enum(2))].into(),
    )
    .unwrap();
    assert_eq!(
        evaluate_effects(&set, &partial, &cfg.catalog),
        Err(EvaluateError::UnsupportedScope {
            responsibility_id: "resp-a".into(),
            field: "x0".into()
        })
    );
    // J_direct contexts cannot carry event features at all.
    assert!(
        TypedContext::new(
            Endpoint::JDirect,
            &cfg.catalog,
            [("event_kind".into(), FieldValue::Enum(2))].into()
        )
        .is_err()
    );
    assert!(
        TypedContext::new(
            Endpoint::JDirect,
            &cfg.catalog,
            [("x0".into(), FieldValue::Number(f64::NAN))].into()
        )
        .is_err()
    );
    let mut other = cfg.catalog.clone();
    other.version = "fixture-catalog-2".into();
    let ctx = TypedContext::new(Endpoint::FEvent, &other, BTreeMap::new()).unwrap();
    assert!(matches!(
        evaluate_effects(&set, &ctx, &other),
        Err(EvaluateError::DependencyFault(_))
    ));
}

#[test]
fn effects_sum_in_canonical_order_with_receipts() {
    let cfg = config();
    let a = r_effect(
        "resp-b",
        jd(ScopeExpr::True),
        params(&[(0, 0.25), (2, 0.5)]),
        0.75,
    );
    let b = theta(
        "mod-1",
        "resp-a",
        jd(ScopeExpr::True),
        params(&[(0, 0.5)]),
        0.5,
    );
    let (set, _) = compile(&[a], &[b], &validity(), 0).unwrap();
    let order: Vec<&str> = set
        .effects
        .iter()
        .map(|e| e.responsibility_id.as_str())
        .collect();
    assert_eq!(order, ["resp-a", "resp-b"]);
    let ctx = TypedContext::new(Endpoint::JDirect, &cfg.catalog, BTreeMap::new()).unwrap();
    let out = evaluate_effects(&set, &ctx, &cfg.catalog).unwrap();
    assert_eq!(out.values, BTreeMap::from([(0, 0.75), (2, 0.5)]));
    let owners: Vec<Owner> = out.receipt.iter().map(|r| r.owner).collect();
    assert_eq!(owners, [Owner::Theta, Owner::R]);
    assert_eq!(out.receipt[1].magnitude, 0.75);
}

#[test]
fn parameter_canonical_bytes_golden_layout() {
    let p = params(&[(1, -0.0), (0, 0.5)]);
    let bytes = p.canonical_bytes().unwrap();
    let mut want = b"NCV1".to_vec();
    want.extend(1u32.to_le_bytes());
    want.extend((PARAMETERS_TAG.len() as u32).to_le_bytes());
    want.extend(PARAMETERS_TAG.as_bytes());
    want.extend(4u32.to_le_bytes());
    want.extend(2u32.to_le_bytes());
    want.extend(0u32.to_le_bytes());
    want.extend(0.5f64.to_bits().to_le_bytes());
    want.extend(1u32.to_le_bytes());
    want.extend(0.0f64.to_bits().to_le_bytes());
    assert_eq!(
        bytes, want,
        "negative zero normalised, keys in canonical order"
    );
}
