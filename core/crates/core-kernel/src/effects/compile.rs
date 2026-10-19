//! CompileAuthorizedEffects (O 3.2, 9, 12, 15; S 6, 12).
//!
//! Pure: it reads the committed R/θ views and the validity snapshot, and
//! returns the minimal effective description at time `t`. It never writes
//! decay, expiry or quarantine back into stored R/θ (I21).

use std::collections::{BTreeMap, BTreeSet};

use core_policy::canonical::scope_digest;
use core_policy::{Endpoint, overlap};
use core_types::generated::{CouplingPhase, EffectContract, EffectOwner};

use super::bound::{add_up, mul_up};
use super::dag::{self, DagError};
use super::objects::{check_objects, check_scope, check_validity, closure_roots, digests_of};
use super::params::EffectParameters;
use super::takeover::{self, Relation, Takeover};
use super::types::*;

const NANOS_PER_SECOND: f64 = 1_000_000_000.0;

/// Compiles the authorised effect set at processing time `t`.
pub fn compile_authorized_effects(
    r_snapshot: &[REffectRecord],
    theta_snapshot: &[ThetaModuleRecord],
    validity: &ValiditySnapshot,
    t: ProcessTime,
    config: &CompileConfig,
) -> Result<(AuthorizedEffectSet, CompileAudit), StructuralFault> {
    check_config(config)?;
    let mut effects = Vec::new();
    let mut audit = CompileAudit::default();
    let mut expires_at: Option<ProcessTime> = None;
    let mut dependency_dag = BTreeMap::new();
    let mut takeovers: Vec<Takeover> = Vec::new();

    // θ first: its takeover registry is the suppression authority (I07).
    for rec in theta_snapshot {
        let contract = &rec.module.mechanism_effect;
        let theta_fault = |detail: &str| StructuralFault::ThetaInvalid {
            module_id: rec.module.module_id.clone(),
            detail: detail.to_owned(),
        };
        takeover::check_binding(rec).map_err(|d| theta_fault(&d))?;
        // The takeover sub-domain must be a verified scope even when the
        // module is disabled: disposal keeps the takeover record (O 3.1).
        check_scope(contract, &rec.scope, config).map_err(|d| theta_fault(&d))?;
        takeovers.push(Takeover {
            responsibility_id: contract.responsibility_id.clone(),
            endpoint: rec.scope.endpoint,
            scope: rec.scope.clone(),
        });
        if !rec.module.enabled {
            continue;
        }
        check_objects(contract, &rec.scope, &rec.parameters, config)
            .map_err(|d| theta_fault(&d))?;
        check_validity(rec, config).map_err(|d| theta_fault(&d))?;
        let v = &rec.validity;
        if let Some(until) = v.valid_until {
            if t >= until {
                return Err(theta_fault("validity contract expired"));
            }
            expires_at = earliest(expires_at, until);
        }
        if !v.conditions.is_subset(&validity.satisfied) {
            return Err(theta_fault("continuing-validity condition not satisfied"));
        }
        // A hard failure anywhere in the closure (qualifications and their
        // upstream included) quarantines the core; θ is never disabled
        // locally (O 3.2; I12, I22).
        let roots = closure_roots(contract, &v.assumptions).map_err(|d| theta_fault(&d))?;
        let dag = dag::resolve(&roots, &validity.dependency_graph, &validity.invalidated)
            .map_err(|e| theta_fault(&format!("dependency closure: {e:?}")))?;
        dependency_dag.extend(dag);
        effects.push(compiled(
            contract,
            rec.scope.clone(),
            &rec.parameters,
            Owner::Theta,
            1.0,
            roots,
            BTreeSet::new(),
        )?);
    }

    for rec in r_snapshot {
        let contract = &rec.contract;
        let rid = contract.responsibility_id.clone();
        let rev = contract.causal_revision.clone();
        let mut exclude = |reason| audit.excluded.push((rid.clone(), rev.clone(), reason));
        if contract.owner != EffectOwner::R_OWNER {
            return Err(integrity(&rid, "R record with non-R owner"));
        }
        if rec.phase != CouplingPhase::ACTIVE {
            exclude(ExclusionReason::NotActive);
            continue;
        }
        if rec.scope.catalog_version != config.catalog.version {
            exclude(ExclusionReason::InterpreterMismatch);
            continue;
        }
        check_objects(contract, &rec.scope, &rec.parameters, config)
            .map_err(|d| integrity(&rid, &d))?;
        match takeover::relation(&takeovers, &rid, &rec.scope, &config.catalog) {
            Relation::Free => {}
            Relation::Suppressed => {
                exclude(ExclusionReason::TakenOverByTheta);
                continue;
            }
            Relation::Ambiguous => {
                return Err(StructuralFault::AmbiguousTakeover {
                    responsibility_id: rid,
                });
            }
        }
        let quals = digests_of(&contract.qualifications).map_err(|d| integrity(&rid, &d))?;
        let roots = closure_roots(contract, &BTreeSet::new()).map_err(|d| integrity(&rid, &d))?;
        // Failure barrier first: a withdrawn qualification stops the effect
        // before anything else is looked at (I12).
        if quals.iter().any(|q| validity.invalidated.contains(q)) {
            exclude(ExclusionReason::Invalidated);
            continue;
        }
        let live: Vec<CertificateValidity> = quals
            .iter()
            .filter_map(|q| validity.certificates.get(q).copied())
            .filter(|c| c.qualified && c.valid_until.is_none_or(|u| t < u))
            .collect();
        if live.is_empty() {
            exclude(ExclusionReason::NoValidQualification);
            continue;
        }
        // The closure includes the qualifications themselves, so a withdrawn
        // upstream of a qualification blocks the effect too.
        let dag = match dag::resolve(&roots, &validity.dependency_graph, &validity.invalidated) {
            Ok(dag) => dag,
            Err(DagError::Invalidated(_)) => {
                exclude(ExclusionReason::Invalidated);
                continue;
            }
            Err(DagError::Unknown(_) | DagError::Cycle(_)) => {
                exclude(ExclusionReason::DependencyUnresolved);
                continue;
            }
        };
        let q = envelope(rec, t)?;
        if q == 0.0 {
            exclude(ExclusionReason::ZeroEnvelope);
            continue;
        }
        // Expiry of the longest-lived live certificate ends this effect.
        if live.iter().all(|c| c.valid_until.is_some())
            && let Some(until) = live.iter().filter_map(|c| c.valid_until).max()
        {
            expires_at = earliest(expires_at, until);
        }
        dependency_dag.extend(dag);
        effects.push(compiled(
            contract,
            rec.scope.clone(),
            &rec.parameters,
            Owner::R,
            q,
            roots,
            rec.conflict_set.clone(),
        )?);
    }

    effects.sort_by(|a, b| key(a).cmp(&key(b)));
    check_unique_owners(&effects, config)?;
    check_declared_conflicts(&effects, config)?;
    check_budgets(&effects, config)?;
    let set = AuthorizedEffectSet {
        at: t,
        catalog_version: config.catalog.version.clone(),
        effects,
        expires_at,
        dependency_dag,
    };
    Ok((set, audit))
}

fn key(e: &CompiledEffect) -> (u32, &str, Digest, &str) {
    (
        e.endpoint.code(),
        &e.responsibility_id,
        e.scope_digest,
        &e.causal_revision,
    )
}

fn earliest(current: Option<ProcessTime>, candidate: ProcessTime) -> Option<ProcessTime> {
    Some(current.map_or(candidate, |c| c.min(candidate)))
}

fn integrity(rid: &str, detail: &str) -> StructuralFault {
    StructuralFault::Integrity {
        responsibility_id: rid.to_owned(),
        detail: detail.to_owned(),
    }
}

fn check_config(config: &CompileConfig) -> Result<(), StructuralFault> {
    let bad = |d: &str| Err(StructuralFault::InvalidConfig(d.to_owned()));
    if config.catalog.validate().is_err() {
        return bad("catalog");
    }
    if config.parameter_resource_version.is_empty() {
        return bad("parameter resource version");
    }
    if config.validity_resource_version.is_empty() {
        return bad("validity resource version");
    }
    if config
        .endpoint_budgets
        .values()
        .any(|b| !b.is_finite() || *b < 0.0)
    {
        return bad("endpoint budget");
    }
    if config.endpoint_dimensions.values().any(|d| *d == 0) {
        return bad("endpoint dimension");
    }
    Ok(())
}

/// q(t) = q(anchor) · exp(−γ Δ). Pure projection from the committed anchor:
/// repeated reads never accumulate decay (O 15; I21). Uses the pure-Rust
/// `libm` exponential so the result does not depend on the platform libm.
fn envelope(rec: &REffectRecord, t: ProcessTime) -> Result<f64, StructuralFault> {
    let rid = &rec.contract.responsibility_id;
    let q0 = rec.q_at_anchor;
    let gamma = rec.gamma_per_second;
    if !q0.is_finite() || !(0.0..=1.0).contains(&q0) || !gamma.is_finite() || gamma < 0.0 {
        return Err(integrity(rid, "invalid envelope"));
    }
    let delta =
        t.0.checked_sub(rec.anchor.0)
            .ok_or_else(|| StructuralFault::TimeBeforeAnchor {
                responsibility_id: rid.clone(),
            })?;
    let exponent = gamma * (delta as f64 / NANOS_PER_SECOND);
    Ok(q0 * libm::exp(-exponent))
}

fn compiled(
    contract: &EffectContract,
    scope: core_policy::Scope,
    params: &EffectParameters,
    owner: Owner,
    factor: f64,
    dependencies: BTreeSet<Digest>,
    conflicts: BTreeSet<String>,
) -> Result<CompiledEffect, StructuralFault> {
    let scope_digest =
        scope_digest(&scope).map_err(|e| integrity(&contract.responsibility_id, &e.to_string()))?;
    Ok(CompiledEffect {
        responsibility_id: contract.responsibility_id.clone(),
        causal_revision: contract.causal_revision.clone(),
        owner,
        endpoint: Endpoint::from(contract.endpoint),
        scope,
        scope_digest,
        parameters: params.clone(),
        factor,
        amplitude_bound: contract.amplitude_bound,
        dependencies,
        conflicts,
    })
}

/// I07: one responsibility has at most one owner per endpoint domain. Two
/// authorised entries of one responsibility on one endpoint are a fault
/// unless their scopes are provably disjoint.
fn check_unique_owners(
    effects: &[CompiledEffect],
    config: &CompileConfig,
) -> Result<(), StructuralFault> {
    for (i, a) in effects.iter().enumerate() {
        for b in &effects[i + 1..] {
            if a.responsibility_id == b.responsibility_id
                && a.endpoint == b.endpoint
                && overlap(&a.scope, &b.scope, &config.catalog).is_conflict()
            {
                return Err(StructuralFault::DuplicateOwner {
                    responsibility_id: a.responsibility_id.clone(),
                });
            }
        }
    }
    Ok(())
}

/// O 3.3, 12.1: a conflict declared by either side, between effects whose
/// scopes may overlap, cannot be executed together.
fn check_declared_conflicts(
    effects: &[CompiledEffect],
    config: &CompileConfig,
) -> Result<(), StructuralFault> {
    for (i, a) in effects.iter().enumerate() {
        for b in &effects[i + 1..] {
            let declared = a.conflicts.contains(&b.responsibility_id)
                || b.conflicts.contains(&a.responsibility_id);
            if declared
                && a.responsibility_id != b.responsibility_id
                && overlap(&a.scope, &b.scope, &config.catalog).is_conflict()
            {
                return Err(StructuralFault::DeclaredConflict {
                    first: a.responsibility_id.clone(),
                    second: b.responsibility_id.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Conservative absolute budget per endpoint: an upper bound of
/// Σ factor·bound must not exceed β (S 6). Signs are ignored, so opposite
/// strong effects cannot cancel their way through, and rounding can only
/// raise the total.
fn check_budgets(
    effects: &[CompiledEffect],
    config: &CompileConfig,
) -> Result<(), StructuralFault> {
    let mut used: BTreeMap<Endpoint, f64> = BTreeMap::new();
    for e in effects {
        let total = used.entry(e.endpoint).or_insert(0.0);
        *total = add_up(*total, mul_up(e.factor, e.amplitude_bound));
    }
    for (endpoint, total) in used {
        let budget = config.endpoint_budgets.get(&endpoint).ok_or_else(|| {
            StructuralFault::InvalidConfig(format!("no budget for endpoint {}", endpoint.code()))
        })?;
        if total > *budget {
            return Err(StructuralFault::BudgetExceeded {
                endpoint: endpoint.code(),
            });
        }
    }
    Ok(())
}
