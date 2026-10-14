use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::TouchObservation,
};
use serde_json::json;

fn limits() -> Limits {
    Limits {
        max_bytes: 1048576,
        max_items: 4096,
        max_depth: 32,
    }
}

#[test]
fn frozen_tag_is_independent_of_message_name() {
    let mut s = Schema::builtin().unwrap();
    let v = json!({"schema_version":1,"output_id":"A","reason":1});
    let bytes = s.encode_canonical("OutputEnd", &v, limits()).unwrap();
    let digest = s.digest("OutputEnd", &v, limits()).unwrap();
    let m = s.messages.remove("OutputEnd").unwrap();
    s.messages.insert("RenamedMessage".into(), m);
    assert_eq!(
        bytes,
        s.encode_canonical("RenamedMessage", &v, limits()).unwrap()
    );
    assert_eq!(digest, s.digest("RenamedMessage", &v, limits()).unwrap());
    s.messages.get_mut("RenamedMessage").unwrap().tag = "new-version-tag".into();
    assert_ne!(digest, s.digest("RenamedMessage", &v, limits()).unwrap());
}

#[test]
fn optional_nonfinite_is_rejected_before_json_can_erase_it() {
    let s = Schema::builtin().unwrap();
    // Construct from a valid DTO first; mutate only the optional floating field.
    let v = json!({"schema_version":1,"kind":1,"region":"head"});
    let mut typed: TouchObservation = serde_json::from_value(v).unwrap();
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        typed.speed = Some(bad);
        assert!(typed.canonical_bytes(limits()).is_err());
        assert!(typed.protobuf_bytes(limits()).is_err());
    }
    assert!(s.message("TouchObservation").is_ok());
}

#[test]
fn sys21_canonical_count_overflow_negative_zero_and_bad_presence() {
    let s = Schema::builtin().unwrap();
    let v = json!({"schema_version":1,"revision":"r","representation_version":"v","dimension":1,"coordinates":[0.0],"anchor_time_ns":"0"});
    let bytes = s
        .encode_canonical("FastStateSnapshot", &v, limits())
        .unwrap();
    let count = 12 + "echo.fast-state".len() + 5 + 5 + 4;
    let mut overflow = bytes.clone();
    overflow[count..count + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(
        s.decode_canonical("FastStateSnapshot", &overflow, limits(), 0)
            .is_err()
    );
    let mut negative_zero = bytes;
    negative_zero[count + 4..count + 12].copy_from_slice(&(1_u64 << 63).to_le_bytes());
    assert!(
        s.decode_canonical("FastStateSnapshot", &negative_zero, limits(), 0)
            .is_err()
    );
    let mut touch = s
        .encode_canonical(
            "TouchObservation",
            &json!({"schema_version":1,"region":"head","kind":1}),
            limits(),
        )
        .unwrap();
    *touch.last_mut().unwrap() = 2;
    assert!(
        s.decode_canonical("TouchObservation", &touch, limits(), 0)
            .is_err()
    );
}

#[test]
fn nested_encoding_repeats_header_and_vector_order_is_significant() {
    let s = Schema::builtin().unwrap();
    let nested = json!({"schema_version":1,"kind":1,"x":0.0,"y":1.0});
    let inner = s.encode_canonical("Anchor", &nested, limits()).unwrap();
    let outer = s
        .encode_canonical(
            "BodyAction",
            &json!({"schema_version":1,"kind":2,"anchor":nested}),
            limits(),
        )
        .unwrap();
    let offset = 12 + "echo.body-action".len() + 4 + 1 + 1;
    assert_eq!(
        &outer[offset..offset + 4],
        &(inner.len() as u32).to_le_bytes()
    );
    assert_eq!(&outer[offset + 4..offset + 4 + inner.len()], inner);
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../../../contracts/golden/legal.json")).unwrap();
    let mut v = corpus
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["request"]["name"] == "FastDriveProposal")
        .nth(2)
        .unwrap()["request"]["value"]
        .clone();
    let before = s.digest("FastDriveProposal", &v, limits()).unwrap();
    v["drive"].as_array_mut().unwrap().reverse();
    assert_ne!(before, s.digest("FastDriveProposal", &v, limits()).unwrap());
}
