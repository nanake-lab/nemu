//! CompileAuthorizedEffects (O 3.2, 12, 15; S 12).
//!
//! Pure: it reads the committed R/θ views and the validity snapshot, and
//! returns the minimal effective description at time `t`. It never writes
//! decay, expiry or quarantine back into stored R/θ (I21).

use std::collections::{BTreeMap, BTreeSet};

use core_policy::canonical::{SCOPE_TAG, scope_bytes, scope_digest};
use core_policy::{Endpoint, overlap};
use core_types::generated::{CouplingPhase, EffectContract, EffectOwner, ObjectRef};

use super::dag::{self, DagError};
use super::params::{EffectParameters, PARAMETERS_TAG};
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
    let mut taken_over = BTreeSet::new();

    // θ first: its takeover registry is the suppression authority (I07).
    for rec in theta_snapshot {
        let contract = &rec.module.mechanism_effect;
        let rid = contract.responsibility_id.clone();
        let theta_fault = |detail: &str| StructuralFault::ThetaInvalid {
            module_id: rec.module.module_id.clone(),
            detail: detail.to_owned(),
        };
        if rec.binding.responsibility_id != rid
            || rec.binding.module_id != rec.module.module_id
            || rec.binding.owner != EffectOwner::THETA_OWNER
            || contract.owner != EffectOwner::THETA_OWNER
        {
            return Err(theta_fault("binding does not match module"));
        }
        taken_over.insert(rid.clone());
        // A legal C disposition keeps the takeover record; it never frees R.
        if !rec.module.enabled {
            continue;
        }
        check_objects(contract, &rec.scope, &rec.parameters, config)
            .map_err(|d| theta_fault(&d))?;
        let v = &rec.validity;
        if v.valid_until.is_none() && v.conditions.is_empty() {
            return Err(theta_fault("no finite validity or verifiable condition"));
        }
        if let Some(until) = v.valid_until {
            if t >= until {
                return Err(theta_fault("validity contract expired"));
            }
            expires_at = earliest(expires_at, until);
        }
        if !v.conditions.is_subset(&validity.satisfied) {
            return Err(theta_fault("continuing-validity condition not satisfied"));
        }
        let roots = digests(&contract.dependencies);
        let dag = dag::resolve(&roots, &validity.dependency_graph, &validity.invalidated)
            .map_err(|e| theta_fault(&format!("dependency: {e:?}")))?;
        dependency_dag.extend(dag);
        effects.push(compiled(
            contract,
            rec.scope.clone(),
            &rec.parameters,
            Owner::Theta,
            1.0,
            roots,
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
        if taken_over.contains(&rid) {
            exclude(ExclusionReason::TakenOverByTheta);
            continue;
        }
        if rec.scope.catalog_version != config.catalog.version {
            exclude(ExclusionReason::InterpreterMismatch);
            continue;
        }
        check_objects(contract, &rec.scope, &rec.parameters, config)
            .map_err(|d| integrity(&rid, &d))?;
        let quals = digests(&contract.qualifications);
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
        let roots = digests(&contract.dependencies);
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
        )?);
    }

    effects.sort_by(|a, b| key(a).cmp(&key(b)));
    check_unique_owners(&effects, config)?;
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

fn digests(refs: &[ObjectRef]) -> BTreeSet<Digest> {
    refs.iter()
        .filter_map(|r| <[u8; 32]>::try_from(r.digest.0.as_slice()).ok())
        .collect()
}

fn check_config(config: &CompileConfig) -> Result<(), StructuralFault> {
    let bad = |d: &str| Err(StructuralFault::InvalidConfig(d.to_owned()));
    if config.catalog.validate().is_err() {
        return bad("catalog");
    }
    if config.parameter_resource_version.is_empty() {
        return bad("parameter resource version");
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

/// Verifies that the contract's scope and parameter references bind exactly
/// the resolved objects, and that parameters respect the declared bounds.
pub(crate) fn check_objects(
    contract: &EffectContract,
    scope: &core_policy::Scope,
    params: &EffectParameters,
    config: &CompileConfig,
) -> Result<(), String> {
    let endpoint = Endpoint::from(contract.endpoint);
    if scope.endpoint != endpoint {
        return Err("scope endpoint differs from contract endpoint".into());
    }
    if contract.responsibility_id.is_empty() || contract.causal_revision.is_empty() {
        return Err("empty identity".into());
    }
    let sb = scope_bytes(scope).map_err(|e| e.to_string())?;
    let sd = scope_digest(scope).map_err(|e| e.to_string())?;
    if !binds(
        &contract.scope,
        SCOPE_TAG,
        &config.catalog.version,
        sb.len(),
        &sd,
    ) {
        return Err("scope reference does not bind scope object".into());
    }
    params.validate()?;
    let pb = params.canonical_bytes().map_err(|e| e.to_string())?;
    let pd = params.digest().map_err(|e| e.to_string())?;
    if !binds(
        &contract.parameters,
        PARAMETERS_TAG,
        &config.parameter_resource_version,
        pb.len(),
        &pd,
    ) {
        return Err("parameter reference does not bind parameter object".into());
    }
    if config.endpoint_dimensions.get(&endpoint) != Some(&params.dimension) {
        return Err("parameter dimension differs from endpoint dimension".into());
    }
    let bound = contract.amplitude_bound;
    if !bound.is_finite() || bound < 0.0 || params.l1() > bound {
        return Err("parameters exceed amplitude bound".into());
    }
    Ok(())
}

fn binds(r: &ObjectRef, tag: &str, version: &str, len: usize, digest: &Digest) -> bool {
    r.schema_version == 1
        && r.object_type == tag
        && r.resource_version == version
        && r.byte_length.0 == len as u64
        && r.digest.0.as_slice() == digest.as_slice()
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

/// Conservative absolute budget per endpoint: Σ factor·bound ≤ β. Signs are
/// ignored, so opposite strong effects cannot cancel their way through.
fn check_budgets(
    effects: &[CompiledEffect],
    config: &CompileConfig,
) -> Result<(), StructuralFault> {
    let mut used: BTreeMap<Endpoint, f64> = BTreeMap::new();
    for e in effects {
        *used.entry(e.endpoint).or_insert(0.0) += e.factor * e.amplitude_bound;
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
