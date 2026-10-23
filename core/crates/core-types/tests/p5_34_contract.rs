//! P5-34: ADR 0019 historical q and independent V, without a law engine or stores.
use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::{
        EvidenceAllocationV2, EvidenceAllocationV3, EvidenceGroupV2, EvidenceGroupV3,
        LawParameters, RSnapshotV2, RSnapshotV3, TendencyV2, TendencyV3,
    },
    semantic,
};
use serde_json::{Value, json};

fn limits() -> Limits {
    Limits {
        max_bytes: 1048576,
        max_items: 4096,
        max_depth: 32,
    }
}
fn tendency(patch: Value) -> TendencyV3 {
    let mut v = json!({"schema_version":1,"tendency_id":"t",
        "concept":{"schema_version":1,"concept_id":"c","registry_version":"v"},
        "predicate":[],"direction":{"schema_version":1,"connection":1.0,"empathy":0.0,"curiosity":0.0,"achievement":0.0},
        "evidence_a":-1.0,"evidence_b":5.0,"evidence_absolute":1.0,
        "h_hat":0.1,"group_count":"2","flip_count":"0","last_sign":1,
        "pi":-0.2,"typical_magnitude":0.25,"magnitude_sum":0.5,"magnitude_weight":2.0});
    for (k, x) in patch.as_object().unwrap() {
        v[k] = x.clone();
    }
    serde_json::from_value(v).unwrap()
}
fn parameters(n0: f64) -> LawParameters {
    serde_json::from_value(
        json!({"schema_version":1,"n0":n0,"kappa1":1.0,"w_max":8.0,"w_teach":2.0,"v_u":0.5,
        "r_min":0.05,"r_max":1.0,"h0":0.1,"n_h":10.0,"h0_theta":0.1,"h_hat_theta_min":0.01,
        "n_b":100.0,"n_sigma":10.0,"g_min":0.2,"g_max":3.0,"recovery_min_per_s":0.001,
        "recovery_max_per_s":1.0,"w_min":0.1,"t_x_s":3600.0,"epsilon_x":0.01,"k_max":0.5}),
    )
    .unwrap()
}
fn allocation(q: f64, ordinal: u64) -> EvidenceAllocationV3 {
    serde_json::from_value(json!({"schema_version":1,"tendency_id":"t","support":0.5,
        "intensity":1.0,"share":1.0,"ordinal":ordinal.to_string(),"mu":0.75,"projection":0.25,"actual_retention":q})).unwrap()
}
fn group(id: &str, ordinal: u64) -> EvidenceGroupV3 {
    serde_json::from_value(
        json!({"schema_version":1,"group_id":id,"members":[],"source_key":"s",
        "credibility":1.0,"verifiability":1.0,"allocations":[allocation(0.75,ordinal)]}),
    )
    .unwrap()
}
fn roundtrip<T: WireMessage + PartialEq + std::fmt::Debug>(v: &T) {
    assert_eq!(
        &T::from_canonical(&v.canonical_bytes(limits()).unwrap(), limits()).unwrap(),
        v
    );
    assert_eq!(
        &T::from_protobuf(&v.protobuf_bytes(limits()).unwrap(), limits()).unwrap(),
        v
    );
}

#[test]
fn successor_fields_reserved_numbers_and_lifecycle() {
    const {
        assert!(TendencyV2::SUPERSEDED && EvidenceAllocationV2::SUPERSEDED);
        assert!(EvidenceGroupV2::SUPERSEDED && RSnapshotV2::SUPERSEDED);
        assert!(!TendencyV3::SUPERSEDED && !EvidenceAllocationV3::SUPERSEDED);
        assert!(!EvidenceGroupV3::SUPERSEDED && !RSnapshotV3::SUPERSEDED);
    }
    let s = Schema::builtin().unwrap();
    for (name, number, field, typ) in [
        ("TendencyV3", 18, "evidence_absolute", "double"),
        ("EvidenceAllocationV3", 9, "actual_retention", "double"),
        ("EvidenceGroupV3", 9, "allocations", "EvidenceAllocationV3"),
        ("RSnapshotV3", 6, "tendencies", "TendencyV3"),
    ] {
        let m = s.message(name).unwrap();
        let f = m.fields.iter().find(|f| f.number == number).unwrap();
        assert_eq!((&*f.name, &*f.r#type), (field, typ));
        assert!(f.required);
    }
    let raw: Value =
        serde_json::from_str(include_str!("../../../contracts/generated/schema.json")).unwrap();
    assert_eq!(
        raw["messages"]["EvidenceGroupV3"]["reserved"],
        json!([7, 8])
    );
    assert_eq!(raw["messages"]["RSnapshotV3"]["reserved"], json!([4, 5]));
    assert_eq!(s.message("TendencyV3").unwrap().fields.len(), 18);
    assert_eq!(s.message("EvidenceAllocationV3").unwrap().fields.len(), 9);
}

#[test]
fn q_boundaries_and_current_support_do_not_redefine_historical_q() {
    for q in [f64::from_bits(1), 0.25, 0.75, 1.0] {
        let mut a = allocation(q, 0);
        assert!(semantic::check_allocation_v3(&a).is_ok());
        roundtrip(&a);
        a.support = 0.0; // A later revision to zero support keeps the original q.
        assert!(semantic::check_allocation_v3(&a).is_ok());
        roundtrip(&a);
    }
    for q in [
        0.0,
        -0.0,
        -0.25,
        f64::from_bits(1.0f64.to_bits() + 1),
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let mut a = allocation(0.75, 0);
        a.actual_retention = q;
        assert!(semantic::check_allocation_v3(&a).is_err(), "q={q}");
        assert!(a.canonical_bytes(limits()).is_err(), "q={q}");
        assert!(a.protobuf_bytes(limits()).is_err(), "q={q}");
    }
}

#[test]
fn v_semantics_reject_both_signs_and_nonfinite_native_values() {
    let p = parameters(4.0);
    let t = tendency(json!({}));
    assert!(semantic::check_tendency_v3_with_parameters(&t, &p).is_ok());
    roundtrip(&t);
    for signed in [-1.0, 1.0] {
        let t = tendency(
            json!({"evidence_a":signed,"evidence_absolute":f64::from_bits(1.0f64.to_bits()-1)}),
        );
        assert!(semantic::check_tendency_v3(&t).is_err());
        assert!(t.canonical_bytes(limits()).is_err());
    }
    for v in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut t = tendency(json!({}));
        t.evidence_absolute = v;
        assert!(semantic::check_tendency_v3(&t).is_err());
        assert!(t.protobuf_bytes(limits()).is_err());
    }
    for patch in [
        json!({"h_hat":1.0}),
        json!({"h_hat":-0.1}),
        json!({"pi":1.0}),
        json!({"flip_count":"2"}),
        json!({"group_count":"9007199254740992"}),
        json!({"magnitude_weight":-1.0}),
        json!({"typical_magnitude":-0.1}),
        json!({"magnitude_weight":0.0}),
        json!({"group_count":"0","last_sign":null}),
        json!({"attachment_prior":1.0}),
    ] {
        assert!(semantic::check_tendency_v3(&tendency(patch)).is_err());
    }
    for patch in [
        json!({"evidence_b":6.0}),
        json!({"pi":-0.25}),
        json!({"typical_magnitude":0.5}),
    ] {
        let t = tendency(patch);
        assert!(semantic::check_tendency_v3(&t).is_ok());
        assert!(semantic::check_tendency_v3_with_parameters(&t, &p).is_err());
    }
}

#[test]
fn rounded_b_does_not_erase_v_or_the_subnormal_m_u_edge() {
    let v = 2.0f64.powi(-53);
    assert_eq!((4.0 + v) - 4.0, 0.0);
    let t = tendency(
        json!({"evidence_a":v,"evidence_absolute":v,"evidence_b":4.0,"pi":v/4.0,
        "magnitude_sum":f64::from_bits(1),"magnitude_weight":0.0,"typical_magnitude":0.0}),
    );
    assert!(semantic::check_tendency_v3_with_parameters(&t, &parameters(4.0)).is_ok());
    roundtrip(&t);
    let back = TendencyV3::from_protobuf(&t.protobuf_bytes(limits()).unwrap(), limits()).unwrap();
    assert_eq!(back.evidence_absolute.to_bits(), v.to_bits());
    assert_eq!(back.magnitude_sum.to_bits(), 1);
    let fresh = tendency(
        json!({"evidence_a":0.0,"evidence_absolute":0.0,"evidence_b":4.0,"pi":0.0,
        "group_count":"0","last_sign":null,"magnitude_sum":0.0,"magnitude_weight":0.0,"typical_magnitude":0.0}),
    );
    assert!(semantic::check_tendency_v3_with_parameters(&fresh, &parameters(4.0)).is_ok());
}

#[test]
fn attachment_denominator_uses_the_normative_grouping() {
    let (prior, c, v, a) = (1e16_f64, 1.0, 5.0, -5.0);
    let denominator = (prior + c) + v;
    let b = c + v;
    assert_ne!(denominator.to_bits(), (prior + b).to_bits());
    let pi = (prior + a) / denominator;
    let mut t = tendency(json!({"attachment_prior":prior,
        "relation":{"schema_version":1,"person_id":"u*","kind":1},
        "evidence_a":a,"evidence_absolute":v,"evidence_b":b,"pi":pi}));
    assert!(semantic::check_tendency_v3_with_parameters(&t, &parameters(c)).is_ok());
    roundtrip(&t);
    t.pi = (prior + a) / (prior + b);
    assert!(semantic::check_tendency_v3_with_parameters(&t, &parameters(c)).is_err());
}

#[test]
fn ordinals_and_nested_containers_are_checked() {
    let t = tendency(json!({}));
    let ok = [group("g1", 1), group("g2", 0)];
    assert!(semantic::check_allocation_ordinals_v3(&t, &ok).is_ok());
    for groups in [
        vec![group("g", 0)],
        vec![group("a", 0), group("b", 0)],
        vec![group("a", 0), group("b", 2)],
    ] {
        assert!(semantic::check_allocation_ordinals_v3(&t, &groups).is_err());
    }
    let mut bad = ok.clone();
    bad[0].allocations[0].actual_retention = 0.0;
    assert!(semantic::check_allocation_ordinals_v3(&t, &bad).is_err());
    for g in ok {
        roundtrip(&g);
    }
    let r: RSnapshotV3 = serde_json::from_value(json!({"schema_version":1,"revision":"r",
        "law_version":{"schema_version":1,"digest":"11".repeat(32),"object_type":"echo.law-version","resource_version":"v","byte_length":"0"},
        "tendencies":[t]})).unwrap();
    roundtrip(&r);
}
