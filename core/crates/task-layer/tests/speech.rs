use core_types::{
    generated::*,
    scalar::{DecimalU64, HexBytes},
};
use std::collections::BTreeMap;
use task_layer::task::{projection::missing_hints, speech::*};

#[test]
fn d3_d7_defaults_and_user_selected_alternatives() {
    assert_eq!(
        SpeechSettings::default(),
        SpeechSettings {
            missing: MissingHint::PerEvent,
            self_quote: SelfQuote::Loose
        }
    );
    let gaps = vec!["42".into(), "43".into()];
    assert!(missing_hints(MissingHint::None, &gaps).is_empty());
    assert_eq!(missing_hints(MissingHint::General, &gaps).len(), 1);
    assert_eq!(missing_hints(MissingHint::PerEvent, &gaps).len(), 2);
    assert!(missing_hints(MissingHint::General, &[]).is_empty());
}
#[test]
fn d6_d7_join_only_supported_points_in_order_with_visible_speaker_evidence() {
    let quotes = BTreeMap::from([
        (
            "o".into(),
            vec![VisibleQuote {
                principal: "user",
                text: "Ready?",
            }],
        ),
        (
            "t".into(),
            vec![VisibleQuote {
                principal: "nemu",
                text: "I wondered.",
            }],
        ),
    ]);
    let raw = r#"{"points":[
      {"type":"other","text":"Yes. ","cites":[]},
      {"type":"past_self","text":"Ready?","cites":["o"]},
      {"type":"past_event","text":"Invisible.","cites":["missing"]},
      {"type":"past_self","text":"I was curious. ","cites":["t"]},
      {"type":"past_self","text":"I took the train.","cites":["t"]},
      {"type":"past_self","text":"I wondered.","cites":["t"]},
      {"type":"now","text":"Yesterday I traveled.","cites":[]}
    ]}"#;
    for (strict, want) in [
        (SelfQuote::Loose, "Yes. I was curious. I wondered."),
        (SelfQuote::Strict, "Yes. I wondered."),
    ] {
        let answer = join_checked(
            raw,
            SpeechSettings {
                self_quote: strict,
                ..Default::default()
            },
            "nemu",
            &quotes,
            |p, _| !p.text.contains("train") && !p.text.contains("Yesterday"),
        )
        .unwrap();
        assert_eq!(answer, want);
    }
}
#[test]
fn quote_spans_are_checked_in_utf8_against_exact_record_and_presented_range() {
    let content = ObjectRef {
        schema_version: 1,
        digest: HexBytes(vec![1; 32]),
        object_type: "fixture".into(),
        resource_version: "1".into(),
        byte_length: DecimalU64(6),
    };
    let version = RecordVersion {
        schema_version: 1,
        branch_id: "b".into(),
        record_id: "e".into(),
        content: content.clone(),
    };
    let mut layer = QuoteLayer {
        schema_version: 1,
        source: version.clone(),
        visible: Utf8Range {
            schema_version: 1,
            start: DecimalU64(1),
            end: DecimalU64(5),
        },
        principal_id: "nemu".into(),
        identity_evidence: content,
    };
    assert_eq!(
        visible_quote(&version, &layer, "a🙂b", 1..5).unwrap().text,
        "🙂"
    );
    assert!(visible_quote(&version, &layer, "a🙂b", 2..5).is_none());
    layer.visible.end = DecimalU64(4);
    assert!(visible_quote(&version, &layer, "a🙂b", 0..6).is_none());
    layer.source.branch_id = "other".into();
    assert!(visible_quote(&version, &layer, "a🙂b", 0..6).is_none());
}
