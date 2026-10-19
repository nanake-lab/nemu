//! Typed inputs and outputs of the two-phase effect functions (O 3.2; S 12).
//!
//! Each input is a narrow view that holds only the fields the compiler may
//! read. Nothing here is a general context map.

use std::collections::{BTreeMap, BTreeSet};

use core_policy::canonical::{Writer, digest};
use core_policy::{Catalog, Endpoint, PolicyError, Scope};
use core_types::generated::{CouplingPhase, EffectContract, ResponsibilityBinding, ThetaModule};

use super::params::EffectParameters;

/// Frozen type tag of a θ ValidityContract object. Like the scope and
/// parameter objects it is a Trusted Core internal encoding, not an IDL
/// message; `ThetaModule.validity_contract` references it by digest.
pub const VALIDITY_TAG: &str = "echo.theta-validity-contract";

/// SHA-256 digest of a content-addressed object.
pub type Digest = [u8; 32];

/// Processing time in nanoseconds on the committed time axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessTime(pub u64);

/// One stored R effect (an active-or-not coupling revision) together with the
/// resolved scope and parameter objects its contract references.
#[derive(Clone, Debug, PartialEq)]
pub struct REffectRecord {
    pub contract: EffectContract,
    pub phase: CouplingPhase,
    pub scope: Scope,
    pub parameters: EffectParameters,
    /// Envelope value q at the anchor, in [0, 1] (O 15).
    pub q_at_anchor: f64,
    /// Fixed decay rate γ per second, set at legal activation (O 15).
    pub gamma_per_second: f64,
    /// Committed time anchor of the envelope.
    pub anchor: ProcessTime,
    /// Responsibilities this coupling is declared to conflict with.
    pub conflict_set: BTreeSet<String>,
}

/// ValidityContract of one θ module (O 10.2; I22): the scope it covers, its
/// dependency assumptions, and a finite `valid_until` or verifiable
/// continuing-validity conditions (at least one of the two). The compiler
/// uses it only after checking that `ThetaModule.validity_contract` binds
/// exactly these bytes, so a side-loaded expiry or condition set cannot
/// replace the committed contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThetaValidity {
    /// Digest of the module's scope object.
    pub scope_digest: Digest,
    /// Dependency assumptions; they join the module's dependency closure.
    pub assumptions: BTreeSet<Digest>,
    pub valid_until: Option<ProcessTime>,
    /// Conditions that must all be present in `ValiditySnapshot::satisfied`.
    pub conditions: BTreeSet<Digest>,
}

impl ThetaValidity {
    /// CanonicalV1 bytes: scope digest, assumptions (sorted, counted),
    /// presence byte + u64 LE expiry, conditions (sorted, counted).
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, PolicyError> {
        let mut w = Writer::with_header(VALIDITY_TAG)?;
        w.bytes.extend_from_slice(&self.scope_digest);
        w.count(self.assumptions.len())?;
        self.assumptions
            .iter()
            .for_each(|d| w.bytes.extend_from_slice(d));
        match self.valid_until {
            Some(t) => {
                w.bytes.push(1);
                w.bytes.extend_from_slice(&t.0.to_le_bytes());
            }
            None => w.bytes.push(0),
        }
        w.count(self.conditions.len())?;
        self.conditions
            .iter()
            .for_each(|d| w.bytes.extend_from_slice(d));
        Ok(w.bytes)
    }

    pub fn digest(&self) -> Result<Digest, PolicyError> {
        digest(VALIDITY_TAG, &self.canonical_bytes()?)
    }
}

/// One stored θ module with its responsibility binding and resolved objects.
#[derive(Clone, Debug, PartialEq)]
pub struct ThetaModuleRecord {
    pub module: ThetaModule,
    pub binding: ResponsibilityBinding,
    pub scope: Scope,
    pub parameters: EffectParameters,
    pub validity: ThetaValidity,
}

/// Validity of one qualification certificate at compile time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CertificateValidity {
    pub qualified: bool,
    pub valid_until: Option<ProcessTime>,
}

/// Read-only validity snapshot bound to the same committed snapshot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValiditySnapshot {
    pub certificates: BTreeMap<Digest, CertificateValidity>,
    /// Failure barrier: objects whose validity has been withdrawn.
    pub invalidated: BTreeSet<Digest>,
    /// Continuing-validity conditions currently verified.
    pub satisfied: BTreeSet<Digest>,
    /// Known dependency objects and their outgoing edges.
    pub dependency_graph: BTreeMap<Digest, BTreeSet<Digest>>,
}

/// Immutable compile configuration (part of the bound profile and Ω).
#[derive(Clone, Debug, PartialEq)]
pub struct CompileConfig {
    pub catalog: Catalog,
    /// Absolute amplitude budget per endpoint. A missing entry means the
    /// endpoint has no configured budget and no effect may be authorised.
    pub endpoint_budgets: BTreeMap<Endpoint, f64>,
    /// Declared parameter dimension per endpoint.
    pub endpoint_dimensions: BTreeMap<Endpoint, u32>,
    /// Resource version of parameter objects (carrier contract version).
    pub parameter_resource_version: String,
    /// Resource version of θ ValidityContract objects.
    pub validity_resource_version: String,
}

/// Effective owner of a responsibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Owner {
    R,
    Theta,
}

/// One authorised, compiled effect.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledEffect {
    pub responsibility_id: String,
    pub causal_revision: String,
    pub owner: Owner,
    pub endpoint: Endpoint,
    pub scope: Scope,
    pub scope_digest: Digest,
    pub parameters: EffectParameters,
    /// Current envelope factor: q(t) for R, 1 for θ.
    pub factor: f64,
    pub amplitude_bound: f64,
    pub dependencies: BTreeSet<Digest>,
    /// Responsibilities this effect is declared to conflict with. Kept so
    /// that a later candidate is checked against both sides (O 3.3, 12.1).
    pub conflicts: BTreeSet<String>,
}

/// Minimal effective description at one processing time. It carries no
/// candidate text, confidence narrative or evidence count.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorizedEffectSet {
    pub at: ProcessTime,
    pub catalog_version: String,
    /// Sorted by endpoint, responsibility, scope digest, revision.
    pub effects: Vec<CompiledEffect>,
    /// Earliest known expiry; a cache of this set is invalid from here on.
    pub expires_at: Option<ProcessTime>,
    /// Dependency DAG restricted to the authorised effects.
    pub dependency_dag: BTreeMap<Digest, BTreeSet<Digest>>,
}

/// Why a stored R effect did not enter the effective set. Kept separate
/// from the set so that it never reaches effect consumers.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExclusionReason {
    NotActive,
    TakenOverByTheta,
    NoValidQualification,
    Invalidated,
    DependencyUnresolved,
    InterpreterMismatch,
    ZeroEnvelope,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompileAudit {
    pub excluded: Vec<(String, String, ExclusionReason)>,
}

/// Structural fault: the stored state cannot be compiled. In strict mode a
/// θ fault is a whole-core quarantine request, never a local disable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructuralFault {
    InvalidConfig(String),
    Integrity {
        responsibility_id: String,
        detail: String,
    },
    /// I07: more than one owner for one responsibility in one domain.
    DuplicateOwner {
        responsibility_id: String,
    },
    /// O 9: a stored R entry of a taken-over responsibility is neither
    /// provably inside the θ sub-domain nor provably disjoint from it, so the
    /// split is not safe and no owner can be chosen.
    AmbiguousTakeover {
        responsibility_id: String,
    },
    /// O 3.3, 12.1: two authorised effects with a declared conflict overlap.
    /// Qualification must pick one, or a composite must be requalified.
    DeclaredConflict {
        first: String,
        second: String,
    },
    /// θ outside its ValidityContract or behind a failure barrier (I22).
    ThetaInvalid {
        module_id: String,
        detail: String,
    },
    TimeBeforeAnchor {
        responsibility_id: String,
    },
    BudgetExceeded {
        endpoint: u32,
    },
}
