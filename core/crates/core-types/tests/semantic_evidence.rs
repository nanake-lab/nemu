//! Contract-layer examples for ADR 0048. No authoritative store is opened.
use core_types::{
    WireMessage,
    contract::{Limits, Result, Schema},
    generated::*,
    scalar::{DecimalU64, HexBytes, unhex},
    semantic::*,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[path = "semantic_evidence/irl_adoption.rs"]
mod irl_adoption;
#[path = "semantic_evidence/resolved.rs"]
mod resolved;

fn limits() -> Limits {
    Limits {
        max_bytes: 1 << 20,
        max_depth: 32,
        max_items: 4096,
    }
}
fn dto<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}
fn value(x: &impl serde::Serialize) -> Value {
    serde_json::to_value(x).unwrap()
}
fn check(name: &str, x: &Value) -> Result<()> {
    check_semantic_evidence_message(name, x, limits())
}
fn object(name: &str) -> Value {
    json!({"schema_version":1,"digest":"12".repeat(32),"resource_version":"r1","object_type":name,"byte_length":"128"})
}
fn record(id: &str, content: Value) -> Value {
    json!({"schema_version":1,"branch_id":"b","record_id":id,"content":content})
}
fn concept(id: &str) -> Value {
    json!({"schema_version":1,"concept_id":id,"registry_version":"r1"})
}
fn reference<T: WireMessage>(obj: &T) -> ObjectRef {
    let s = Schema::builtin().unwrap();
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(unhex(&s.digest(T::NAME, &value(obj), limits()).unwrap()).unwrap()),
        object_type: s.message(T::NAME).unwrap().tag.clone(),
        resource_version: "r1".into(),
        byte_length: DecimalU64(obj.canonical_bytes(limits()).unwrap().len() as u64),
    }
}
fn sample(name: &str) -> Value {
    // Reuse a complete legal legacy binding; each test supplies its own synthetic observations.
    let rows: Vec<Value> =
        serde_json::from_slice(include_bytes!("../../../contracts/golden/legal.json")).unwrap();
    rows.iter().find(|r| r["request"]["name"] == name).unwrap()["request"]["value"].clone()
}

struct View {
    event: EventEnvelope,
    bytes: Vec<u8>,
    staged: bool,
}
impl SemanticReadView for View {
    fn branch(&self) -> &str {
        "b"
    }
    fn base_commit(&self) -> &str {
        "c"
    }
    fn transaction(&self) -> Option<&str> {
        self.staged.then_some("txn")
    }
    fn authorize_provenance(&self, meta: &SourceMetadataV2) -> Result<()> {
        if meta.validity.object_type == "fixture-grant" {
            Ok(())
        } else {
            Err("untrusted fixture".into())
        }
    }
    fn content(&self, _: &RecordVersion) -> Result<VisibleContent<'_>> {
        Ok(VisibleContent::Event(&self.event, &self.bytes))
    }
}
fn text_view<T: WireMessage>(payload: &T) -> (View, SourceLocation) {
    let r = reference(payload);
    let mut event: EventEnvelope = dto(sample("EventEnvelope"));
    event.id = "e".into();
    event.payload_ref = r.clone();
    event.payload_digest = r.digest.clone();
    let loc = dto(
        json!({"schema_version":1,"source":record("e",value(&r)),"path":[{"schema_version":1,"field_number":2}],"text":{"schema_version":1,"start":"3","end":"7"}}),
    );
    (
        View {
            event,
            bytes: payload.canonical_bytes(limits()).unwrap(),
            staged: false,
        },
        loc,
    )
}

#[test]
fn exact_utf8_range_needs_the_right_payload_field_prefix_and_transaction() {
    let (mut view, mut loc) = text_view(&UserText {
        schema_version: 1,
        text: "語🙂e\u{301}".into(),
    });
    assert_eq!(
        resolve_location(&view, "c", None, &loc, limits()).unwrap(),
        "🙂"
    );
    view.staged = true;
    assert!(resolve_location(&view, "c", None, &loc, limits()).is_err());
    assert!(resolve_location(&view, "future", Some("txn"), &loc, limits()).is_err());
    assert_eq!(
        resolve_location(&view, "c", Some("txn"), &loc, limits()).unwrap(),
        "🙂"
    );
    loc.text.as_mut().unwrap().end.0 = 4;
    assert!(resolve_location(&view, "c", Some("txn"), &loc, limits()).is_err());
    loc.text = None;
    loc.path[0].element = Some(DecimalU64(0));
    assert!(resolve_location(&view, "c", Some("txn"), &loc, limits()).is_err());
    loc.path[0].element = None;
    loc.source.content.resource_version = "other".into();
    assert!(resolve_location(&view, "c", Some("txn"), &loc, limits()).is_err());
    loc.source.content.resource_version = "r1".into();
    *view.bytes.last_mut().unwrap() ^= 1;
    assert!(resolve_location(&view, "c", Some("txn"), &loc, limits()).is_err());
}

fn revision(id: &str) -> Value {
    json!({"schema_version":1,"concept_id":id,"lifecycle":1,"governance":object("review"),
    "grounding":[],"positive_examples":[],"negative_examples":[],"roles":[],"allows_reported_text":false,"constraints":[]})
}
fn registry() -> Value {
    json!({"schema_version":1,"version":"r1","entries":[revision("a"),revision("b"),revision("c")],
    "links":[],"aliases":[],"relation_kinds":[],"interaction_kinds":[],"edge_kinds":[]})
}
fn alias(from: &str, to: &str) -> Value {
    json!({"schema_version":1,"origin":concept(from),"target":concept(to),"kind":"canonical","basis":object("basis")})
}

#[test]
fn canonical_aliases_preserve_ids_and_require_a_unique_reachable_representative() {
    let mut r = registry();
    r["aliases"] = json!([alias("a", "b"), alias("b", "c")]);
    assert!(check("ConceptRegistry", &r).is_ok());
    for bad in [alias("c", "a"), alias("a", "c"), alias("c", "missing")] {
        let mut broken = r.clone();
        broken["aliases"].as_array_mut().unwrap().push(bad);
        assert!(check("ConceptRegistry", &broken).is_err());
    }
    let old: ConceptRegistry = dto(registry());
    let mut next = registry();
    next["version"] = json!("r2");
    next["previous"] = value(&reference(&old));
    next["entries"][0]["lifecycle"] = json!(3);
    assert!(check_registry_successor(Some(&old), &dto(next.clone()), limits()).is_err());
    next["entries"][0]["previous"] = value(&reference(&old.entries[0]));
    assert!(check_registry_successor(Some(&old), &dto(next.clone()), limits()).is_ok());
    next["entries"].as_array_mut().unwrap().pop();
    assert!(check_registry_successor(Some(&old), &dto(next), limits()).is_err());
}

#[test]
fn only_registered_text_and_transcript_fields_produce_a_reported_operand() {
    let text = UserText {
        schema_version: 1,
        text: "我在".into(),
    };
    let (view, loc) = text_view(&text);
    let mut r = registry();
    r["entries"][0]["lifecycle"] = json!(2);
    let label = json!({"schema_version":1,"zh":"原话","en":"reported words","ja":"発話"});
    r["entries"][0]["label"] = label.clone();
    r["entries"][0]["definition"] = label;
    r["entries"][0]["grounding"] = json!([{"schema_version":1,"source":loc.source,"path":[]}]);
    r["entries"][0]["roles"] =
        json!([{"schema_version":1,"role":"speaker","node_kind":1,"mandatory":true}]);
    r["entries"][0]["allows_reported_text"] = json!(true);
    let r: ConceptRegistry = dto(r);
    let make_entry = |tag: &str| {
        json!({"schema_version":1,"payload_tag":tag,"payload_schema":1,"path":loc.path,
        "predicate":concept("a"),"role":"speaker"})
    };
    let mut table: TextAdapterTable = dto(
        json!({"schema_version":1,"registry":reference(&r),"entries":[make_entry("echo.user-text"),make_entry("echo.user-utterance")]}),
    );
    let (_, span) = text_adapter(&table, &r, &view, &loc.source, limits())
        .unwrap()
        .unwrap();
    assert_eq!(
        resolve_location(&view, "c", None, &span, limits()).unwrap(),
        "我在"
    );
    let speech: UserUtterance = dto(
        json!({"schema_version":1,"text":"聞いた","lang":3,"asr_confidence":0.8,"duration_ms":"340"}),
    );
    let (voice, transcript) = text_view(&speech);
    let (_, span) = text_adapter(&table, &r, &voice, &transcript.source, limits())
        .unwrap()
        .unwrap();
    assert_eq!(
        resolve_location(&voice, "c", None, &span, limits()).unwrap(),
        "聞いた"
    );
    for tag in ["button", "echo.touch-v2", "echo.body-action-result"] {
        let mut other = loc.source.clone();
        other.content.object_type = tag.into();
        assert!(
            text_adapter(&table, &r, &view, &other, limits())
                .unwrap()
                .is_none()
        );
    }
    table.entries[0].role = "invented".into();
    assert!(text_adapter(&table, &r, &view, &loc.source, limits()).is_err());
}
fn adoption() -> Value {
    let mut binding = sample("ProposalBindingV2");
    binding["branch_id"] = json!("b");
    let context = object("echo.fact-projection");
    binding["read_set"] = json!([context]);
    binding["c_core_digest"] = context["digest"].clone();
    let env = &mut binding["environment"];
    env["situation_matcher"] = object("matcher");
    json!({"schema_version":1,"event":record("event",object("echo.user-text")),"interpretation":record("interpret",object("recorded-output")),
        "stage":3,"outcome":1,"binding":binding,"producer_resource":object("matcher"),"candidate":object("candidate"),
        "actual_output":object("worker-output"),"context":context,"definitions":[],"isolation":object("isolated-execution"),
        "governance":object("governance"),"coverage":[{"schema_version":1,"target":"a","mu":0.0},{"schema_version":1,"target":"b","unknown_reason":"missing observation"}],
        "transaction_id":"txn","replaced_targets":[],"registry_before":binding["environment"]["registry"],"staged_revisions":[]})
}

#[test]
fn stage_coverage_keeps_known_zero_unknown_unassessed_and_failed_distinct() {
    let good = adoption();
    assert!(check("StageAdoption", &good).is_ok());
    let mut bad = good.clone();
    bad["coverage"][0]["unknown_reason"] = json!("missing");
    assert!(check("StageAdoption", &bad).is_err());
    let mut absent = good.clone();
    absent["coverage"] = json!([]);
    assert!(check("StageAdoption", &absent).is_ok());
    absent["outcome"] = json!(3);
    absent["reason"] = json!("worker failure");
    assert!(check("StageAdoption", &absent).is_ok());
    absent["coverage"] = good["coverage"].clone();
    assert!(check("StageAdoption", &absent).is_err());
    let mut a = good.clone();
    a["stage"] = json!(2);
    a["coverage"] = json!([]);
    a["binding"]["environment"]["nature_appraiser"] = a["producer_resource"].clone();
    assert!(check("StageAdoption", &a).is_err());
    a["appraisal"] = json!({"schema_version":1,"connection":0.0,"empathy":0.1,"curiosity":0.0,"achievement":0.0});
    assert!(check("StageAdoption", &a).is_ok());
    let s: StageAdoption = dto(good);
    let actual = StageExecution {
        binding: &s.binding,
        transaction_id: "txn",
        event: &s.event,
        interpretation: &s.interpretation,
        stage: s.stage,
        candidate: s.candidate.as_ref(),
        actual_output: s.actual_output.as_ref(),
        isolation: &s.isolation,
        relation_manifest: None,
    };
    assert!(check_stage_execution(&s, &actual, limits()).is_ok());
    let mut changed = s.clone();
    changed.actual_output = Some(dto(object("different-output")));
    assert!(check_stage_execution(&changed, &actual, limits()).is_err());
}
#[test]
fn reinterpretation_only_replaces_the_named_stage_event_and_targets() {
    let prior = adoption();
    let mut next = prior.clone();
    next["interpretation"]["record_id"] = json!("second");
    next["replaced_targets"] = json!(["a"]);
    next["revision"] = json!({"schema_version":1,"kind":14,"origin":next["interpretation"],"target":prior["interpretation"],
        "purpose":"reinterpret","effective_commit":"publication","governance":object("review")});
    let prev = dto(prior);
    assert!(check_reinterpretation(&dto(next.clone()), &prev, limits()).is_ok());
    for (key, val) in [
        ("replaced_targets", json!(["other"])),
        ("event", record("other", object("echo.user-text"))),
    ] {
        let mut bad = next.clone();
        bad[key] = val;
        assert!(check_reinterpretation(&dto(bad), &prev, limits()).is_err());
    }
}
#[test]
fn unverifiable_content_is_independent_of_occurrence_and_source_class() {
    let loc =
        json!({"schema_version":1,"source":record("heard",object("echo.user-text")),"path":[]});
    let known = json!({"schema_version":1,"scope":loc,"value":1,"evidence":[loc["source"]]});
    let unknown = json!({"schema_version":1,"scope":loc,"value":3,"reason":"content unverified","evidence":[]});
    for v in [&known, &unknown] {
        assert!(check("ScopedVerification", v).is_ok());
    }
    let mut wrong = known;
    wrong["evidence"] = json!([]);
    assert!(check("ScopedVerification", &wrong).is_err());
    let mut rule = json!({"schema_version":1,"source":5,"event_kind":4,"payload_type":"echo.body-action-result",
        "exogenous":false,"self_narrative":false,"growth_access":3});
    assert!(check("AttributeRule", &rule).is_ok());
    rule["source"] = json!(1);
    rule["exogenous"] = json!(true);
    rule["growth_access"] = json!(2);
    assert!(check("AttributeRule", &rule).is_err());
}
#[test]
fn terminal_rejects_every_draft_tag() {
    let s = Schema::builtin().unwrap();
    let all: Vec<_> = s
        .messages
        .values()
        .filter(|m| m.file == "semantic_evidence.proto")
        .collect();
    assert!(!all.is_empty());
    for m in all {
        assert!(
            core_types::terminal_ingress::upload_type(&m.tag).is_err(),
            "{}",
            m.tag
        );
    }
    assert!(core_types::terminal_ingress::upload_type("echo.user-text").is_ok());
}
