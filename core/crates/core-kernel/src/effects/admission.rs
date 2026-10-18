//! Candidate admission and route audit (O 3.1, 3.3, 12.1; S 12).
//!
//! A new candidate is checked together with the existing authorised effects.
//! If it would exceed the budget or conflict, the candidate is rejected and
//! the existing effects stay unchanged; nothing is scaled down (J06).

use std::collections::BTreeSet;

use core_policy::{Endpoint, Scope, overlap};
use core_types::generated::EffectContract;

use super::compile::check_objects;
use super::params::EffectParameters;
use super::types::{AuthorizedEffectSet, CompileConfig};

/// A proposed effect awaiting admission.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateEffect {
    pub contract: EffectContract,
    pub scope: Scope,
    pub parameters: EffectParameters,
    /// Responsibilities the candidate is declared to conflict with.
    pub conflict_set: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdmissionRejection {
    /// Scope or parameters do not bind, use non-whitelisted fields, or
    /// exceed the candidate's own amplitude bound.
    Invalid(String),
    /// The same responsibility already acts on this endpoint domain (I07).
    DuplicateResponsibility(String),
    /// A declared conflict overlaps an existing effect; the qualification
    /// process must pick one, or compile a composite and requalify.
    Conflict(String),
    /// Σ|bound| over possibly overlapping effects would exceed β.
    BudgetExceeded { endpoint: u32 },
}

/// Decides whether `candidate` may join `existing`. Pure; `existing` is
/// not modified in any outcome.
pub fn admit_candidate(
    existing: &AuthorizedEffectSet,
    candidate: &CandidateEffect,
    config: &CompileConfig,
) -> Result<(), AdmissionRejection> {
    let catalog = &config.catalog;
    let contract = &candidate.contract;
    // Re-validating the scope enforces the endpoint whitelist ("超 scope").
    let endpoint = Endpoint::from(contract.endpoint);
    Scope::new(endpoint, candidate.scope.expr.clone(), catalog)
        .map_err(|e| AdmissionRejection::Invalid(e.to_string()))?;
    check_objects(contract, &candidate.scope, &candidate.parameters, config)
        .map_err(AdmissionRejection::Invalid)?;
    if existing.catalog_version != catalog.version {
        return Err(AdmissionRejection::Invalid(
            "interpreter version mismatch".into(),
        ));
    }
    let mut overlapping_total = contract.amplitude_bound;
    for e in existing.effects.iter().filter(|e| e.endpoint == endpoint) {
        // UNKNOWN_OVERLAP counts as overlapping.
        if !overlap(&e.scope, &candidate.scope, catalog).is_conflict() {
            continue;
        }
        if e.responsibility_id == contract.responsibility_id {
            return Err(AdmissionRejection::DuplicateResponsibility(
                e.responsibility_id.clone(),
            ));
        }
        if candidate.conflict_set.contains(&e.responsibility_id) {
            return Err(AdmissionRejection::Conflict(e.responsibility_id.clone()));
        }
        // Full bound, not the decayed factor: a decaying effect may still be
        // near its bound when the candidate first acts.
        overlapping_total += e.amplitude_bound;
    }
    let budget =
        config
            .endpoint_budgets
            .get(&endpoint)
            .ok_or(AdmissionRejection::BudgetExceeded {
                endpoint: endpoint.code(),
            })?;
    if overlapping_total > *budget {
        return Err(AdmissionRejection::BudgetExceeded {
            endpoint: endpoint.code(),
        });
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteFault {
    /// A drive proposal claims to have already encoded an R/θ effect. The
    /// Core assembles R/θ effects itself (S 5 FastInputView), so any such
    /// claim would implement the effect twice (F06).
    DuplicateRoute(String),
}

/// Routing audit for F_in inputs: A_x drive proposals must not carry any
/// R/θ responsibility, authorised or not.
pub fn audit_drive_route(
    existing: &AuthorizedEffectSet,
    drive_encoded_responsibilities: &[String],
) -> Result<(), RouteFault> {
    if let Some(rid) = drive_encoded_responsibilities.first() {
        let duplicate = existing.effects.iter().any(|e| &e.responsibility_id == rid);
        let detail = if duplicate {
            rid.clone()
        } else {
            format!("{rid} (not authorised)")
        };
        return Err(RouteFault::DuplicateRoute(detail));
    }
    Ok(())
}
