//! θ takeover registry and R suppression (O 3.1, 9, 10.1; S 5; I07, I08).
//!
//! A takeover covers one responsibility on one endpoint inside one decidable
//! scope sub-domain. It suppresses only the R entries of that responsibility
//! that provably lie inside the sub-domain. R entries on other endpoints, or
//! with a provably disjoint scope, remain R's ("未覆盖的责任仍由 R 承担").
//! An entry that is neither inside nor disjoint means the split was not safe;
//! that is a structural fault, never a wider suppression. A disabled module
//! keeps its takeover record, so disposal never frees the R effect.

use core_policy::{Catalog, Endpoint, Scope, contains, overlap};
use core_types::generated::EffectOwner;

use super::types::ThetaModuleRecord;

/// One registered takeover sub-domain.
#[derive(Clone, Debug)]
pub(crate) struct Takeover {
    pub responsibility_id: String,
    pub endpoint: Endpoint,
    pub scope: Scope,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Relation {
    /// No takeover of this responsibility touches the entry's domain.
    Free,
    /// The entry lies inside a takeover sub-domain.
    Suppressed,
    /// The entry overlaps a takeover sub-domain without lying inside it.
    Ambiguous,
}

/// The binding is the suppression authority (S 5), so it must agree with the
/// module and its mechanism contract on every identity and domain field.
pub(crate) fn check_binding(rec: &ThetaModuleRecord) -> Result<(), String> {
    let module = &rec.module;
    let contract = &module.mechanism_effect;
    let binding = &rec.binding;
    let mismatches = [
        (
            binding.responsibility_id != contract.responsibility_id,
            "responsibility",
        ),
        (binding.module_id != module.module_id, "module id"),
        (binding.owner != EffectOwner::THETA_OWNER, "binding owner"),
        (contract.owner != EffectOwner::THETA_OWNER, "contract owner"),
        (
            binding.causal_revision != contract.causal_revision,
            "causal revision",
        ),
        (binding.endpoint != contract.endpoint, "endpoint"),
        (binding.scope != contract.scope, "binding scope"),
        (module.scope != contract.scope, "module scope"),
        (module.parameters != contract.parameters, "parameters"),
        (module.carrier != contract.carrier, "carrier"),
        (
            binding.source_slice_id != module.source_slice_id,
            "source slice",
        ),
    ];
    match mismatches.iter().find(|(bad, _)| *bad) {
        Some((_, field)) => Err(format!("binding does not match module: {field}")),
        None => Ok(()),
    }
}

/// How the takeover registry applies to one R entry of `responsibility_id`.
pub(crate) fn relation(
    takeovers: &[Takeover],
    responsibility_id: &str,
    scope: &Scope,
    catalog: &Catalog,
) -> Relation {
    let mut suppressed = false;
    for t in takeovers
        .iter()
        .filter(|t| t.responsibility_id == responsibility_id && t.endpoint == scope.endpoint)
    {
        if !overlap(&t.scope, scope, catalog).is_conflict() {
            continue;
        }
        if contains(&t.scope, scope, catalog) {
            suppressed = true;
        } else {
            return Relation::Ambiguous;
        }
    }
    if suppressed {
        Relation::Suppressed
    } else {
        Relation::Free
    }
}
