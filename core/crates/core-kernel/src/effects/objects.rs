//! Verification of the objects an effect contract references (O 3.1, 3.3;
//! S 6, 12). Every compile and admission path goes through these checks, so
//! a public `Scope` value is never trusted just because it exists.

use std::collections::BTreeSet;

use core_policy::canonical::{SCOPE_TAG, scope_bytes, scope_digest};
use core_policy::{Endpoint, Scope};
use core_types::generated::{EffectContract, ObjectRef};

use super::params::{EffectParameters, PARAMETERS_TAG};
use super::types::{CompileConfig, Digest, ThetaModuleRecord, VALIDITY_TAG};

/// The scope binds the contract and is a valid scope of its endpoint under
/// the configured interpreter: whitelist, depth, node, set and value-domain
/// limits are re-checked here, not only when a candidate is admitted.
pub(crate) fn check_scope(
    contract: &EffectContract,
    scope: &Scope,
    config: &CompileConfig,
) -> Result<(), String> {
    let endpoint = Endpoint::from(contract.endpoint);
    if scope.endpoint != endpoint {
        return Err("scope endpoint differs from contract endpoint".into());
    }
    if scope.catalog_version != config.catalog.version {
        return Err("scope interpreter version differs from configured catalog".into());
    }
    Scope::new(endpoint, scope.expr.clone(), &config.catalog)
        .map_err(|e| format!("scope fails validation: {e}"))?;
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
    Ok(())
}

/// Scope, identity and parameter checks; parameters must respect the
/// declared dimension and, by a conservative L1 bound, the amplitude bound.
pub(crate) fn check_objects(
    contract: &EffectContract,
    scope: &Scope,
    params: &EffectParameters,
    config: &CompileConfig,
) -> Result<(), String> {
    if contract.responsibility_id.is_empty() || contract.causal_revision.is_empty() {
        return Err("empty identity".into());
    }
    check_scope(contract, scope, config)?;
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
    let endpoint = Endpoint::from(contract.endpoint);
    if config.endpoint_dimensions.get(&endpoint) != Some(&params.dimension) {
        return Err("parameter dimension differs from endpoint dimension".into());
    }
    let bound = contract.amplitude_bound;
    if !bound.is_finite() || bound < 0.0 || params.l1_upper() > bound {
        return Err("parameters exceed amplitude bound".into());
    }
    Ok(())
}

/// The ValidityContract the compiler applies is exactly the object the
/// committed module references, and it covers the module's own scope (I22).
pub(crate) fn check_validity(
    rec: &ThetaModuleRecord,
    config: &CompileConfig,
) -> Result<(), String> {
    let v = &rec.validity;
    let bytes = v.canonical_bytes().map_err(|e| e.to_string())?;
    let d = v.digest().map_err(|e| e.to_string())?;
    if !binds(
        &rec.module.validity_contract,
        VALIDITY_TAG,
        &config.validity_resource_version,
        bytes.len(),
        &d,
    ) {
        return Err("validity contract reference does not bind the applied contract".into());
    }
    let sd = scope_digest(&rec.scope).map_err(|e| e.to_string())?;
    if v.scope_digest != sd {
        return Err("validity contract covers a different scope".into());
    }
    if v.valid_until.is_none() && v.conditions.is_empty() {
        return Err("no finite validity or verifiable condition".into());
    }
    Ok(())
}

pub(crate) fn binds(r: &ObjectRef, tag: &str, version: &str, len: usize, digest: &Digest) -> bool {
    r.schema_version == 1
        && r.object_type == tag
        && r.resource_version == version
        && r.byte_length.0 == len as u64
        && r.digest.0.as_slice() == digest.as_slice()
}

/// Digest of one reference; a malformed reference is an error, never dropped.
pub(crate) fn digest_of(r: &ObjectRef) -> Result<Digest, String> {
    if r.schema_version != 1 || r.object_type.is_empty() {
        return Err(format!("malformed reference of type {:?}", r.object_type));
    }
    <[u8; 32]>::try_from(r.digest.0.as_slice())
        .map_err(|_| format!("malformed digest in reference of type {:?}", r.object_type))
}

pub(crate) fn digests_of(refs: &[ObjectRef]) -> Result<BTreeSet<Digest>, String> {
    refs.iter().map(digest_of).collect()
}

/// Roots of the dependency closure (S 12): qualifications, the carrier (an
/// interpretation resource), data and action dependencies, plus any extra
/// assumptions. Scope and parameter objects are not roots: they are verified
/// by digest against the resolved objects. `ancestors` are revision IDs in
/// the IDL, not digest references, so they cannot be nodes of this digest
/// graph; mechanism invalidation reaches an effect through its qualification
/// and dependency references, whose upstream the closure follows.
pub(crate) fn closure_roots(
    contract: &EffectContract,
    extra: &BTreeSet<Digest>,
) -> Result<BTreeSet<Digest>, String> {
    let mut roots = digests_of(&contract.qualifications)?;
    roots.extend(digests_of(&contract.dependencies)?);
    roots.insert(digest_of(&contract.carrier)?);
    roots.extend(extra.iter().copied());
    Ok(roots)
}
