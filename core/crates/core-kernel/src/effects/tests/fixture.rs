//! Structural fixture builders (logic tests only; AT12). Dimensions, budgets
//! and rates here are arbitrary test values, not deployment parameters.

use std::collections::{BTreeMap, BTreeSet};

use core_policy::canonical::{SCOPE_TAG, scope_bytes, scope_digest};
use core_policy::{Atom, Catalog, Endpoint, FieldKind, FieldSpec, Scope, ScopeExpr, ScopeLimits};
use core_types::generated::{
    CouplingPhase, EffectContract, EffectOwner, ObjectRef, ResponsibilityBinding, ThetaModule,
};
use core_types::scalar::{DecimalU64, HexBytes};

use crate::effects::*;

pub const CATALOG: &str = "fixture-catalog-1";
pub const PARAMS: &str = "fixture-carrier-1";
pub const VALIDITY: &str = "fixture-validity-1";
pub const SECOND: u64 = 1_000_000_000;

pub fn catalog() -> Catalog {
    let mut fields = BTreeMap::new();
    let eps = |e: &[Endpoint]| e.iter().copied().collect::<BTreeSet<_>>();
    fields.insert(
        "event_kind".to_owned(),
        FieldSpec {
            kind: FieldKind::Enum {
                domain: (1..=8).collect(),
            },
            endpoints: eps(&[Endpoint::FEvent]),
        },
    );
    for i in 0..4 {
        fields.insert(
            format!("x{i}"),
            FieldSpec {
                kind: FieldKind::Numeric {
                    min: -1_000_000,
                    max: 1_000_000,
                    decimals: 3,
                },
                endpoints: eps(&[Endpoint::FEvent, Endpoint::JDirect]),
            },
        );
    }
    let limits = ScopeLimits {
        max_depth: 8,
        max_nodes: 64,
        max_set_items: 16,
        max_dnf_terms: 32,
        max_overlap_pairs: 256,
    };
    Catalog {
        version: CATALOG.into(),
        fields,
        limits,
    }
}

pub fn config() -> CompileConfig {
    CompileConfig {
        catalog: catalog(),
        endpoint_budgets: [(Endpoint::FEvent, 2.0), (Endpoint::JDirect, 2.0)]
            .into_iter()
            .collect(),
        endpoint_dimensions: [(Endpoint::FEvent, 4), (Endpoint::JDirect, 4)]
            .into_iter()
            .collect(),
        parameter_resource_version: PARAMS.into(),
        validity_resource_version: VALIDITY.into(),
    }
}

pub fn d(byte: u8) -> Digest {
    [byte; 32]
}

pub fn oref(tag: &str, version: &str, digest: Digest, len: usize) -> ObjectRef {
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(digest.to_vec()),
        object_type: tag.into(),
        resource_version: version.into(),
        byte_length: DecimalU64(len as u64),
    }
}

pub fn plain(byte: u8) -> ObjectRef {
    oref("echo.fixture-object", "1", d(byte), 0)
}

pub fn scope(endpoint: Endpoint, expr: ScopeExpr) -> Scope {
    Scope::new(endpoint, expr, &catalog()).expect("fixture scope")
}

pub fn x0(lo: i64, hi: i64) -> ScopeExpr {
    ScopeExpr::Atom(Atom::NumIn {
        field: "x0".into(),
        lo,
        hi,
    })
}

pub fn kind(codes: &[u32]) -> ScopeExpr {
    ScopeExpr::Atom(Atom::EnumIn {
        field: "event_kind".into(),
        values: codes.iter().copied().collect(),
    })
}

pub fn params(entries: &[(u32, f64)]) -> EffectParameters {
    EffectParameters {
        dimension: 4,
        entries: entries.iter().copied().collect(),
    }
}

pub fn contract(
    rid: &str,
    rev: &str,
    owner: EffectOwner,
    s: &Scope,
    p: &EffectParameters,
    bound: f64,
) -> EffectContract {
    EffectContract {
        schema_version: 1,
        responsibility_id: rid.into(),
        causal_revision: rev.into(),
        owner,
        endpoint: s.endpoint.into(),
        scope: oref(
            SCOPE_TAG,
            CATALOG,
            scope_digest(s).unwrap(),
            scope_bytes(s).unwrap().len(),
        ),
        carrier: plain(0xC0),
        parameters: oref(
            PARAMETERS_TAG,
            PARAMS,
            p.digest().unwrap(),
            p.canonical_bytes().unwrap().len(),
        ),
        amplitude_bound: bound,
        qualifications: vec![plain(0xA1)],
        ancestors: vec![],
        dependencies: vec![plain(0xD1)],
    }
}

pub fn r_effect(rid: &str, s: Scope, p: EffectParameters, bound: f64) -> REffectRecord {
    REffectRecord {
        contract: contract(rid, "rev-1", EffectOwner::R_OWNER, &s, &p, bound),
        phase: CouplingPhase::ACTIVE,
        scope: s,
        parameters: p,
        q_at_anchor: 1.0,
        gamma_per_second: 0.05,
        anchor: ProcessTime(0),
        conflict_set: BTreeSet::new(),
    }
}

pub fn theta(
    module: &str,
    rid: &str,
    s: Scope,
    p: EffectParameters,
    bound: f64,
) -> ThetaModuleRecord {
    let c = contract(rid, "theta-rev-1", EffectOwner::THETA_OWNER, &s, &p, bound);
    let validity = ThetaValidity {
        scope_digest: scope_digest(&s).unwrap(),
        assumptions: BTreeSet::new(),
        valid_until: Some(ProcessTime(100 * SECOND)),
        conditions: BTreeSet::new(),
    };
    let mut rec = ThetaModuleRecord {
        module: ThetaModule {
            schema_version: 1,
            module_id: module.into(),
            revision: "m-rev-1".into(),
            source_slice_id: "slice-1".into(),
            carrier: plain(0xC0),
            parameters: c.parameters.clone(),
            composition_order: DecimalU64(1),
            enabled: true,
            scope: c.scope.clone(),
            mechanism_effect: c.clone(),
            validity_contract: plain(0xE1),
        },
        binding: ResponsibilityBinding {
            schema_version: 1,
            responsibility_id: rid.into(),
            causal_revision: c.causal_revision.clone(),
            owner: EffectOwner::THETA_OWNER,
            endpoint: c.endpoint,
            scope: c.scope.clone(),
            source_slice_id: "slice-1".into(),
            module_id: module.into(),
            disposition: plain(0xF1),
        },
        scope: s,
        parameters: p,
        validity,
    };
    bind_validity(&mut rec);
    rec
}

/// Points the module's ValidityContract reference at the record's current
/// validity object, as a legal commit of that contract would.
pub fn bind_validity(rec: &mut ThetaModuleRecord) {
    let v = &rec.validity;
    rec.module.validity_contract = oref(
        VALIDITY_TAG,
        VALIDITY,
        v.digest().unwrap(),
        v.canonical_bytes().unwrap().len(),
    );
}

/// Validity snapshot where the fixture certificate is qualified and the
/// fixture qualification, carrier and dependency are known leaves.
pub fn validity() -> ValiditySnapshot {
    ValiditySnapshot {
        certificates: [(
            d(0xA1),
            CertificateValidity {
                qualified: true,
                valid_until: None,
            },
        )]
        .into_iter()
        .collect(),
        invalidated: BTreeSet::new(),
        satisfied: BTreeSet::new(),
        dependency_graph: [d(0xA1), d(0xC0), d(0xD1)]
            .into_iter()
            .map(|n| (n, BTreeSet::new()))
            .collect(),
    }
}

pub fn compile(
    r: &[REffectRecord],
    th: &[ThetaModuleRecord],
    v: &ValiditySnapshot,
    t: u64,
) -> Result<(AuthorizedEffectSet, CompileAudit), StructuralFault> {
    compile_authorized_effects(r, th, v, ProcessTime(t), &config())
}
