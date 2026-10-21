//! P5-32 (ADR 0016): the successors that carry the typical-magnitude sums and the
//! replay data of allocations, and their two semantic checks.
use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::{
        EvidenceAllocation, EvidenceAllocationV2, EvidenceGroup, EvidenceGroupV2, RSnapshot,
        RSnapshotV2, Tendency, TendencyV2,
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

fn unit(c: f64, e: f64, q: f64, a: f64) -> Value {
    json!({"schema_version":1,"connection":c,"empathy":e,"curiosity":q,"achievement":a})
}

fn tendency(patch: Value) -> TendencyV2 {
    let mut v = json!({
        "schema_version":1,"tendency_id":"rel:closeness:u*",
        "concept":{"schema_version":1,"concept_id":"c","registry_version":"r"},
        "predicate":[],"direction":unit(1.0,0.0,0.0,0.0),
        "evidence_a":1.0,"evidence_b":4.0,"h_hat":0.1,"group_count":"3","flip_count":"0",
        "last_sign":2,"pi":0.875,"typical_magnitude":0.25,
        "relation":{"schema_version":1,"person_id":"u*","kind":1},"attachment_prior":20.0,
        "magnitude_sum":0.5,"magnitude_weight":2.0
    });
    for (k, x) in patch.as_object().unwrap() {
        v[k] = x.clone();
    }
    serde_json::from_value(v).unwrap()
}

fn allocation(tendency: &str, ordinal: u64) -> Value {
    json!({"schema_version":1,"tendency_id":tendency,"support":0.5,"intensity":1.25,"share":1.0,
           "ordinal":ordinal.to_string(),"mu":0.75,"projection":0.2})
}

fn group(id: &str, allocations: Vec<Value>) -> EvidenceGroupV2 {
    serde_json::from_value(json!({
        "schema_version":1,"group_id":id,"members":["e1"],"source_key":"s","credibility":1.0,
        "verifiability":1.0,"allocations":allocations
    }))
    .unwrap()
}

#[test]
fn successors_are_live_and_predecessors_superseded() {
    const {
        assert!(Tendency::SUPERSEDED && EvidenceAllocation::SUPERSEDED);
        assert!(EvidenceGroup::SUPERSEDED && RSnapshot::SUPERSEDED);
        assert!(!TendencyV2::SUPERSEDED && !EvidenceAllocationV2::SUPERSEDED);
        assert!(!EvidenceGroupV2::SUPERSEDED && !RSnapshotV2::SUPERSEDED);
    };
    let s = Schema::builtin().unwrap();
    // Each successor keeps its predecessor's fields and adds exactly the ADR 0016 fields;
    // the containers only swap the field that carries the new type.
    let names = |m: &str| -> Vec<(u32, String, String)> {
        s.message(m)
            .unwrap()
            .fields
            .iter()
            .map(|f| (f.number, f.name.clone(), f.r#type.clone()))
            .collect()
    };
    let added = |old: &str, new: &str| -> Vec<(u32, String, String)> {
        let before = names(old);
        names(new)
            .into_iter()
            .filter(|f| !before.contains(f))
            .collect()
    };
    let field = |n: u32, name: &str, t: &str| (n, name.to_owned(), t.to_owned());
    assert_eq!(
        added("Tendency", "TendencyV2"),
        [
            field(16, "magnitude_sum", "double"),
            field(17, "magnitude_weight", "double")
        ]
    );
    assert_eq!(
        added("EvidenceAllocation", "EvidenceAllocationV2"),
        [
            field(6, "ordinal", "uint64"),
            field(7, "mu", "double"),
            field(8, "projection", "double")
        ]
    );
    assert_eq!(
        added("EvidenceGroup", "EvidenceGroupV2"),
        [field(8, "allocations", "EvidenceAllocationV2")]
    );
    assert_eq!(
        added("RSnapshot", "RSnapshotV2"),
        [field(5, "tendencies", "TendencyV2")]
    );
    assert_eq!(names("Tendency").len() + 2, names("TendencyV2").len());
    assert_eq!(names("EvidenceGroup").len(), names("EvidenceGroupV2").len());
}

#[test]
fn tendency_v2_semantics_include_the_magnitude_sums() {
    assert!(semantic::check_tendency_v2(&tendency(json!({}))).is_ok());
    // An empty weight is fine with an empty sum (no matching experience group yet).
    let fresh = tendency(json!({"magnitude_sum":0.0,"magnitude_weight":0.0}));
    assert!(semantic::check_tendency_v2(&fresh).is_ok());
    for (patch, why) in [
        (
            json!({"magnitude_sum":0.25,"magnitude_weight":0.0}),
            "sum without weight",
        ),
        (
            json!({"magnitude_sum":-1e-300,"magnitude_weight":0.0}),
            "tiny sum without weight",
        ),
        (json!({"pi":1.0}), "pi at the closed bound"),
        (json!({"evidence_a":5.0}), "|A| above B"),
        (json!({"flip_count":"3"}), "more flips than adjacent pairs"),
        (
            json!({"relation":{"schema_version":1,"person_id":"u*","kind":2}}),
            "prior on trust",
        ),
    ] {
        let t = tendency(patch);
        assert!(t.canonical_bytes(limits()).is_ok(), "{why}: codec-legal");
        assert!(semantic::check_tendency_v2(&t).is_err(), "{why}");
    }
    // A negative sum is legal: p_c can be negative before the clip to [0, r_max].
    let negative = tendency(json!({"magnitude_sum":-0.5}));
    assert!(semantic::check_tendency_v2(&negative).is_ok());
}

#[test]
fn allocation_ordinals_are_exactly_zero_to_n_minus_one() {
    let t = tendency(json!({})); // group_count 3
    let other = |o| allocation("other", o);
    let ok = [
        group("g1", vec![allocation("rel:closeness:u*", 2), other(0)]),
        group("g2", vec![allocation("rel:closeness:u*", 0)]),
        group("g3", vec![allocation("rel:closeness:u*", 1), other(7)]),
    ];
    assert!(semantic::check_allocation_ordinals(&t, &ok).is_ok());
    let cases: [(&str, Vec<EvidenceGroupV2>); 4] = [
        (
            "one allocation missing",
            vec![
                group("g1", vec![allocation("rel:closeness:u*", 0)]),
                group("g2", vec![allocation("rel:closeness:u*", 1)]),
            ],
        ),
        (
            "ordinal repeated",
            vec![
                group("g1", vec![allocation("rel:closeness:u*", 0)]),
                group("g2", vec![allocation("rel:closeness:u*", 1)]),
                group("g3", vec![allocation("rel:closeness:u*", 1)]),
            ],
        ),
        (
            "ordinal past N - 1",
            vec![
                group("g1", vec![allocation("rel:closeness:u*", 0)]),
                group("g2", vec![allocation("rel:closeness:u*", 1)]),
                group("g3", vec![allocation("rel:closeness:u*", 3)]),
            ],
        ),
        (
            "one allocation too many",
            vec![
                group("g1", vec![allocation("rel:closeness:u*", 0)]),
                group("g2", vec![allocation("rel:closeness:u*", 1)]),
                group("g3", vec![allocation("rel:closeness:u*", 2)]),
                group("g4", vec![allocation("rel:closeness:u*", 3)]),
            ],
        ),
    ];
    for (why, groups) in cases {
        for g in &groups {
            assert!(g.canonical_bytes(limits()).is_ok(), "{why}: codec-legal");
        }
        assert!(
            semantic::check_allocation_ordinals(&t, &groups).is_err(),
            "{why}"
        );
    }
    // A tendency without groups has no allocations.
    let empty = tendency(
        json!({"group_count":"0","last_sign":null,"magnitude_sum":0.0,"magnitude_weight":0.0}),
    );
    assert!(semantic::check_allocation_ordinals(&empty, &[]).is_ok());
}

#[test]
fn teaching_allocation_has_no_projection_and_roundtrips() {
    let mut v = allocation("t", 0);
    v.as_object_mut().unwrap().remove("projection");
    let a: EvidenceAllocationV2 = serde_json::from_value(v).unwrap();
    assert_eq!(a.projection, None);
    let bytes = a.canonical_bytes(limits()).unwrap();
    assert_eq!(
        EvidenceAllocationV2::from_canonical(&bytes, limits()).unwrap(),
        a
    );
    let snapshot: RSnapshotV2 = serde_json::from_value(json!({
        "schema_version":1,"revision":"r1",
        "law_version":{"schema_version":1,"digest":"11".repeat(32),"object_type":"echo.law-version","resource_version":"v","byte_length":"0"},
        "tendencies":[serde_json::to_value(tendency(json!({}))).unwrap()]
    }))
    .unwrap();
    let pb = snapshot.protobuf_bytes(limits()).unwrap();
    assert_eq!(RSnapshotV2::from_protobuf(&pb, limits()).unwrap(), snapshot);
}
