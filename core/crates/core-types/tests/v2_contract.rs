//! P5-21: structural and semantic checks of the Echo Core v2.0 contract types.
use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::{
        ActionStatus, DeliveryStatus, EffectContract, EpistemicType, EvidenceRelation, FailureCode,
        GenesisManifestV2, LawParameters, RunStatus, TeachingAnnotation, Tendency, TendencyChange,
    },
    semantic,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn limits() -> Limits {
    Limits {
        max_bytes: 1048576,
        max_items: 4096,
        max_depth: 32,
    }
}

/// I28 (type part): no field reachable from the genesis manifest can carry
/// personality or style text. Strings are an explicit allowlist of identifiers,
/// bytes are fixed-size digests, and no localized text type is reachable.
#[test]
fn genesis_has_no_free_text_fields() {
    let s = Schema::builtin().unwrap();
    let allowed: BTreeSet<(&str, &str)> = [
        ("GenesisManifestV2", "nature_structure"),
        ("GenesisManifestV2", "caretaker_id"),
        ("LawVersion", "reference_form"),
        ("ObjectRef", "object_type"),
        ("ObjectRef", "resource_version"),
    ]
    .into();
    let mut seen = BTreeSet::new();
    let mut stack = vec!["GenesisManifestV2".to_string()];
    let mut strings = BTreeSet::new();
    while let Some(name) = stack.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        assert_ne!(
            name, "TrilingualText",
            "localized text reachable from genesis"
        );
        for f in &s.message(&name).unwrap().fields {
            match f.r#type.as_str() {
                "string" => {
                    assert!(!f.repeated, "{name}.{} is a list of strings", f.name);
                    assert!(f.nonempty, "{name}.{} is not an identifier", f.name);
                    strings.insert((name.clone(), f.name.clone()));
                }
                "bytes" => assert_eq!(f.size, Some(32), "{name}.{} is a free payload", f.name),
                t if s.messages.contains_key(t) => stack.push(t.to_string()),
                _ => {}
            }
        }
    }
    let strings: BTreeSet<(&str, &str)> = strings
        .iter()
        .map(|(m, f)| (m.as_str(), f.as_str()))
        .collect();
    assert_eq!(strings, allowed);
    // Genesis carries exactly the v2.0 §13 items; protocol p is referenced, not embedded.
    let top: Vec<&str> = s.message("GenesisManifestV2").unwrap().fields[1..]
        .iter()
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(
        top,
        [
            "law_version",
            "nature_structure",
            "initial_gain",
            "initial_baseline",
            "sigma0",
            "tau0_s",
            "attachment_prior",
            "caretaker_id",
            "carrier",
            "direction_set",
            "direction_validation",
            "body_readout",
            "proactivity_readout",
            "fuses",
            "boundaries",
            "protocol",
        ]
    );
}

fn unit(c: f64, e: f64, q: f64, a: f64) -> Value {
    json!({"schema_version":1,"connection":c,"empathy":e,"curiosity":q,"achievement":a})
}

fn object_ref() -> Value {
    json!({"schema_version":1,"digest":"11".repeat(32),"object_type":"t","resource_version":"v","byte_length":"0"})
}

fn attachment() -> Value {
    json!({
        "schema_version":1,"tendency_id":"caretaker-closeness",
        "concept":{"schema_version":1,"concept_id":"c","registry_version":"r"},
        "predicate":[],"direction":unit(1.0,0.0,0.0,0.0),
        "evidence_a":1.0,"evidence_b":4.0,"h_hat":0.1,"group_count":"3","flip_count":"0",
        "last_sign":2,"pi":0.875,"typical_magnitude":0.25,
        "relation":{"schema_version":1,"person_id":"u*","kind":1},"attachment_prior":20.0
    })
}

fn tendency(patch: Value) -> Tendency {
    let mut v = attachment();
    for (k, x) in patch.as_object().unwrap() {
        v[k] = x.clone();
    }
    serde_json::from_value(v).unwrap()
}

#[test]
fn tendency_semantics_beyond_the_codec() {
    let ok = tendency(json!({}));
    assert!(semantic::check_tendency(&ok).is_ok());
    // Each of these is accepted by the codec but rejected semantically.
    for (patch, why) in [
        (json!({"pi":1.0}), "pi at the closed bound"),
        (json!({"pi":-1.0}), "pi at the closed bound"),
        (
            json!({"direction":unit(0.6,0.6,0.0,0.0)}),
            "direction not unit",
        ),
        (json!({"evidence_a":5.0}), "|A| above B"),
        (json!({"evidence_b":0.0,"evidence_a":0.0}), "B not positive"),
        (json!({"flip_count":"3"}), "more flips than adjacent pairs"),
        (
            json!({"group_count":"0","flip_count":"0"}),
            "sign without a group",
        ),
        (
            json!({"relation":{"schema_version":1,"person_id":"u*","kind":2}}),
            "prior on trust",
        ),
    ] {
        let t = tendency(patch);
        assert!(t.canonical_bytes(limits()).is_ok(), "{why}: codec-legal");
        assert!(semantic::check_tendency(&t).is_err(), "{why}");
    }
}

#[test]
fn tendency_change_needs_an_endpoint() {
    let change = |before: Option<f64>, after: Option<f64>| TendencyChange {
        schema_version: 1,
        tendency_id: "t".into(),
        pi_before: before,
        pi_after: after,
        evidence_groups: vec![],
    };
    assert!(semantic::check_tendency_change(&change(None, Some(0.0))).is_ok());
    assert!(semantic::check_tendency_change(&change(Some(0.2), None)).is_ok());
    assert!(semantic::check_tendency_change(&change(None, None)).is_err());
    assert!(semantic::check_tendency_change(&change(Some(1.0), Some(0.5))).is_err());
}

fn parameters() -> Value {
    json!({
        "schema_version":1,"n0":3.0,"kappa1":1.0,"w_max":8.0,"w_teach":2.0,"v_u":0.5,
        "r_min":0.05,"r_max":1.0,"h0":0.1,"n_h":10.0,"h0_theta":0.1,"h_hat_theta_min":0.01,
        "n_b":100.0,"n_sigma":10.0,"g_min":0.2,"g_max":3.0,"recovery_min_per_s":0.001,
        "recovery_max_per_s":1.0,"w_min":0.1,"t_x_s":3600.0,"epsilon_x":0.01,"k_max":0.5
    })
}

#[test]
fn law_parameter_inequalities() {
    let p: LawParameters = serde_json::from_value(parameters()).unwrap();
    assert!(semantic::check_law_parameters(&p).is_ok());
    for (key, value) in [
        ("n0", 0.0),
        ("w_teach", 9.0),
        ("v_u", 1.0),
        ("r_min", 2.0),
        ("g_min", 4.0),
        ("recovery_min_per_s", 0.0),
        ("recovery_max_per_s", 0.0001),
    ] {
        let mut v = parameters();
        v[key] = json!(value);
        let p: LawParameters = serde_json::from_value(v).unwrap();
        assert!(semantic::check_law_parameters(&p).is_err(), "{key}");
    }
}

#[test]
fn genesis_semantics() {
    let fuses = json!({
        "schema_version":1,"r_capacity":64,"delta_theta_max":0.05,"jump_threshold":0.2,
        "jump_probes":object_ref(),"slow_drift_run":50,"slow_drift_margin":0.1,
        "contact_limit":5,"message_limit":100,"network_window_ns":"86400000000000"
    });
    let genesis = |gain: Value| -> GenesisManifestV2 {
        serde_json::from_value(json!({
            "schema_version":1,
            "law_version":{"schema_version":1,"version":1,"reference_form":"v2.0-5.2","parameters":parameters()},
            "nature_structure":"v2.0-3","initial_gain":gain,"initial_baseline":unit(0.0,0.0,0.0,0.0),
            "sigma0":0.3,"tau0_s":600.0,"attachment_prior":20.0,"caretaker_id":"u*",
            "carrier":object_ref(),"direction_set":object_ref(),"direction_validation":object_ref(),
            "body_readout":object_ref(),"proactivity_readout":object_ref(),"fuses":fuses,
            "boundaries":object_ref(),"protocol":object_ref()
        }))
        .unwrap()
    };
    assert!(semantic::check_genesis(&genesis(unit(1.0, 1.0, 1.0, 0.5))).is_ok());
    // v2.0 §3: achievement starts weakest; a tie is not "weakest".
    assert!(semantic::check_genesis(&genesis(unit(1.0, 1.0, 0.5, 0.5))).is_err());
    assert!(semantic::check_genesis(&genesis(unit(1.0, 1.0, 1.0, 0.1))).is_err());
    assert!(
        genesis(unit(1.0, 1.0, 1.0, 0.5))
            .canonical_bytes(limits())
            .is_ok()
    );
}

#[test]
fn superseded_registry() {
    const { assert!(EffectContract::SUPERSEDED) };
    // P5-32 (ADR 0016) superseded Tendency by TendencyV2; see tests/p5_32_contract.rs.
    const { assert!(Tendency::SUPERSEDED) };
    let marked: BTreeSet<&str> = core_types::generated::SUPERSEDED_MESSAGES
        .iter()
        .copied()
        .collect();
    // 32 at P5-21; P5-32 adds Tendency, EvidenceAllocation, EvidenceGroup, RSnapshot.
    // P5-22 adds six terminal predecessors, all kept readable with frozen bytes.
    // P5-34 adds four V2 predecessors, retained for decoding.
    assert_eq!(marked.len(), 46);
    for name in [
        "Readout",
        "RunState",
        "Query",
        "QueryResult",
        "ApprovalRequest",
        "ApprovalDecision",
        "EffectContract",
        "ThetaModule",
        "SingleRootLedger",
        "FastDriveProposal",
        "ModulationEnvelope",
        "VersionedEnvironment",
    ] {
        assert!(marked.contains(name), "{name}");
    }
    assert!(!marked.contains("ModulationEnvelopeV2"));
    assert!(RunStatus::AX_DEGRADED.is_superseded());
    assert!(RunStatus::CORE_QUARANTINED.is_superseded());
    assert!(!RunStatus::A_DEGRADED.is_superseded());
    assert!(FailureCode::GROWTH_NOT_ENABLED_V1.is_superseded());
    assert!(!FailureCode::GROWTH_OFF_REHEARSAL.is_superseded());
}

/// ADR 0014 (with its 2026-10-21 correction) relies on these values, so they must stay
/// live: no superseded mark, and the message is not in the superseded registry.
/// `EVIDENCE_UNKNOWN` belongs to the superseded `EvidenceRelation`; per the correction it
/// is not used, and this test pins that it stays superseded rather than revived.
#[test]
fn adr_0014_values_are_live() {
    assert!(!EpistemicType::USER_SELF_REPORT.is_superseded());
    assert!(!ActionStatus::UNKNOWN_ACTION.is_superseded());
    assert!(!DeliveryStatus::UNKNOWN_DELIVERY.is_superseded());
    const { assert!(!TeachingAnnotation::SUPERSEDED) };
    assert!(!core_types::generated::SUPERSEDED_MESSAGES.contains(&"TeachingAnnotation"));
    // The same facts in the schema every language loads.
    let s: Value =
        serde_json::from_str(include_str!("../../../contracts/generated/schema.json")).unwrap();
    for (e, v) in [
        ("EpistemicType", "USER_SELF_REPORT"),
        ("ActionStatus", "UNKNOWN_ACTION"),
        ("DeliveryStatus", "UNKNOWN_DELIVERY"),
    ] {
        assert!(s["enums"][e][v].is_u64(), "{e}.{v} exists");
        assert!(s["superseded_enums"].get(e).is_none(), "{e} is live");
        assert!(
            s["superseded_values"][e].get(v).is_none(),
            "{e}.{v} is live"
        );
    }
    assert!(
        s["messages"]["TeachingAnnotation"]
            .get("superseded")
            .is_none()
    );
    assert!(EvidenceRelation::EVIDENCE_UNKNOWN.is_superseded());
}

/// The codec's set order and `semantic::tendency_order` agree, so P5-10 can order
/// learned components exactly as RSnapshot and FastStateSnapshotV2 store them.
#[test]
fn tendency_order_matches_codec_set_order() {
    let s = Schema::builtin().unwrap();
    let ids = ["zz", "a", "中文", "b", "aaa", "t-10", "t-2"];
    let learned: Vec<Value> = ids
        .iter()
        .map(|id| json!({"schema_version":1,"tendency_id":id,"value":0.0}))
        .collect();
    let v = json!({
        "schema_version":1,"revision":"r","representation_version":"v","anchor_time_ns":"0",
        "nature":unit(0.0,0.0,0.0,0.0),"learned":learned
    });
    let normalized = s.normalize("FastStateSnapshotV2", &v, limits(), 0).unwrap();
    let got: Vec<&str> = normalized["learned"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["tendency_id"].as_str().unwrap())
        .collect();
    let mut want = ids.to_vec();
    want.sort_by(|a, b| semantic::tendency_order(a, b));
    assert_eq!(got, want);
}
