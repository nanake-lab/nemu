//! Explicit synthetic V3 inputs, independent of still-unfrozen V3 golden bytes.
//! These are storage fixtures, not a replacement for the governing G engine.
use super::*;

pub(super) fn law() -> LawVersion {
    serde_json::from_value(serde_json::json!({"schema_version":1,"version":1,
        "reference_form":"LP1-storage-fixture","parameters":{
        "schema_version":1,"n0":4.0,"kappa1":0.0,"w_max":8.0,"w_teach":2.0,"v_u":0.5,
        "r_min":0.05,"r_max":1.0,"h0":0.1,"n_h":10.0,"h0_theta":0.1,"h_hat_theta_min":0.01,
        "n_b":100.0,"n_sigma":10.0,"g_min":0.2,"g_max":3.0,"recovery_min_per_s":0.001,
        "recovery_max_per_s":1.0,"w_min":0.1,"t_x_s":3600.0,"epsilon_x":0.01,"k_max":0.5}}))
    .unwrap()
}
pub(super) fn tendency(id: &str) -> TendencyV3 {
    TendencyV3 {
        schema_version: 1,
        tendency_id: id.into(),
        concept: ConceptRef {
            schema_version: 1,
            concept_id: "synthetic".into(),
            registry_version: "1".into(),
        },
        predicate: vec![],
        direction: NatureVector {
            schema_version: 1,
            connection: 1.0,
            empathy: 0.0,
            curiosity: 0.0,
            achievement: 0.0,
        },
        evidence_a: 0.0,
        evidence_b: 4.0,
        evidence_absolute: 0.0,
        h_hat: 0.0,
        group_count: DecimalU64(0),
        flip_count: DecimalU64(0),
        last_sign: None,
        pi: 0.0,
        typical_magnitude: 0.0,
        relation: None,
        attachment_prior: None,
        magnitude_sum: 0.0,
        magnitude_weight: 0.0,
    }
}
pub(super) fn snapshot(law: &ObjectRef) -> RSnapshotV3 {
    RSnapshotV3 {
        schema_version: 1,
        revision: "synthetic-r".into(),
        law_version: law.clone(),
        tendencies: vec![],
    }
}
pub(super) fn group(id: &str, tendency: &str) -> EvidenceGroupV3 {
    EvidenceGroupV3 {
        schema_version: 1,
        group_id: id.into(),
        members: vec![],
        source_key: "synthetic-source".into(),
        credibility: 1.0,
        verifiability: 1.0,
        allocations: vec![EvidenceAllocationV3 {
            schema_version: 1,
            tendency_id: tendency.into(),
            support: 0.5,
            intensity: 1.0,
            share: 1.0,
            ordinal: DecimalU64(0),
            mu: 0.75,
            projection: Some(0.25),
            actual_retention: 0.75,
        }],
    }
}
pub(super) fn genesis(reference: &ObjectRef) -> GenesisManifestV2 {
    let mut g: GenesisManifestV2 = with_refs(reference);
    g.law_version = law();
    g.initial_gain = NatureScale {
        schema_version: 1,
        connection: 1.0,
        empathy: 1.0,
        curiosity: 1.0,
        achievement: 0.5,
    };
    g.sigma0 = 1.0;
    g.tau0_s = 10.0;
    g
}
