use super::super::*;
use core_types::{
    WireMessage,
    contract::Limits,
    generated::*,
    scalar::{DecimalI64, DecimalU64},
};
use std::{collections::BTreeMap, sync::OnceLock};

/// Existing contract golden values supply schema-valid irrelevant binding fields.
/// All identities and reactions below are synthetic; no H/store/model is opened.
pub fn golden<T: WireMessage>() -> T {
    static FIXTURES: OnceLock<Vec<serde_json::Value>> = OnceLock::new();
    let rows = FIXTURES.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../../contracts/golden/legal.json")).unwrap()
    });
    let row = rows
        .iter()
        .find(|r| r["request"]["name"] == T::NAME)
        .unwrap();
    serde_json::from_value(row["request"]["value"].clone()).unwrap()
}

pub fn vector(v: f64) -> NatureVector {
    NatureVector {
        schema_version: 1,
        connection: v,
        empathy: 0.0,
        curiosity: 0.0,
        achievement: 0.0,
    }
}
pub fn binding(f: &Frame) -> ProposalBindingV2 {
    let mut b: ProposalBindingV2 = golden();
    b.event_id = f.event.id.clone();
    b.raw_log_position = f.event.raw_log_position;
    b.branch_id = "synthetic-p5-24".into();
    b
}
pub fn frame(i: usize, source: &str, a: Option<f64>) -> Frame {
    let mut e: EventEnvelope = golden();
    e.id = format!("event-{i}");
    e.raw_log_position = DecimalU64(i as u64 + 1);
    e.event_time_ns = DecimalI64(i as i64);
    e.ingest_time_ns = e.event_time_ns;
    e.parents.clear();
    e.correction_of.clear();
    e.kind = EventKind::OBSERVATION;
    e.action_id = None;
    e.trial_id = None;
    e.session = "shared-app-session-not-conversation".into();
    e.stable_source_key = format!("delivery-{i}");
    let mut f = Frame {
        attributes: Some(EventAttributesRecord {
            schema_version: 1,
            event_id: e.id.clone(),
            rule_table_version: "synthetic-attributes".into(),
            exogenous: true,
            self_narrative: false,
            verifiable: true,
            source_key: source.into(),
        }),
        semantics: Some(Semantics {
            version: "synthetic-semantics".into(),
            conversation: None,
            kind: EventUse::Experience,
            utterance: Utterance::Other,
            epistemic: EpistemicType::USER_ASSERTION,
            speaker: None,
            participants: vec![],
            relied_on: vec![],
            outcome: Outcome {
                observed: true,
                ..Outcome::default()
            },
            match_basis: BTreeMap::new(),
        }),
        event: e,
        appraisals: vec![],
        situations: vec![],
        teachings: vec![],
        same_matters: vec![],
        lies: vec![],
        candidates: vec![],
        simultaneous_proposals: None,
        structural_admissions: None,
    };
    if let Some(a) = a {
        f.appraisals.push(NatureAppraisalRecord {
            schema_version: 1,
            record_id: format!("a-{i}"),
            event_id: f.event.id.clone(),
            appraisal: vector(a),
            persons: vec![],
            binding: binding(&f),
            producer: "synthetic-A".into(),
            producer_version: "1".into(),
        });
    }
    f
}
pub fn propose(f: &mut Frame, id: &str, concept: &str) {
    let mut r: TendencyCandidateRecord = golden();
    r.record_id = id.into();
    r.event_id = f.event.id.clone();
    r.binding = binding(f);
    r.direction = vector(1.0);
    r.relation = None;
    r.predicate = vec![TrilingualText {
        schema_version: 1,
        zh: "条件".into(),
        en: "condition".into(),
        ja: "条件".into(),
    }];
    f.candidates.push(RegisteredCandidate {
        record: r,
        concept: ConceptRef {
            schema_version: 1,
            concept_id: concept.into(),
            registry_version: "synthetic-registry".into(),
        },
    });
}
pub fn matched(f: &mut Frame, target: &str, mu: f64, basis: MatchBasis) {
    let r = SituationMatchRecord {
        schema_version: 1,
        record_id: format!("m-{}-{target}", f.event.id),
        event_id: f.event.id.clone(),
        matches: vec![TendencyMatch {
            schema_version: 1,
            tendency_id: target.into(),
            mu,
        }],
        binding: binding(f),
        producer: "synthetic-M".into(),
        producer_version: "1".into(),
    };
    f.situations.push(r);
    f.semantics
        .as_mut()
        .unwrap()
        .match_basis
        .insert(target.into(), basis);
}
pub fn teach(f: &mut Frame, target: &str, speaker: &str, supports: bool) {
    f.teachings.push(TeachingAnnotation {
        schema_version: 1,
        record_id: format!("teach-{}-{target}", f.event.id),
        event_id: f.event.id.clone(),
        speaker: speaker.into(),
        tendency_id: target.into(),
        supports,
        binding: binding(f),
        producer: "synthetic-M".into(),
        producer_version: "1".into(),
    });
    let sem = f.semantics.as_mut().unwrap();
    sem.speaker = Some(speaker.into());
    sem.participants.push(speaker.into());
    sem.utterance = Utterance::TeachingAboutHer;
}
pub fn prefix<'a>(
    frames: &'a [Frame],
    aliases: &'a BTreeMap<String, String>,
    persons: &'a BTreeMap<String, String>,
) -> Prefix<'a> {
    Prefix {
        frames,
        attribute_version: "synthetic-attributes",
        semantic_version: "synthetic-semantics",
        concept_aliases: aliases,
        person_aliases: persons,
        genesis_order: &[],
        limits: Limits {
            max_bytes: 1 << 20,
            max_items: 4096,
            max_depth: 32,
        },
    }
}
pub fn run(frames: &[Frame]) -> Result<Assembly> {
    assemble(prefix(frames, &BTreeMap::new(), &BTreeMap::new()))
}
pub fn base() -> Vec<Frame> {
    let mut f = frame(0, "one", Some(0.8));
    propose(&mut f, "k", "concept");
    matched(&mut f, "k", 1.0, MatchBasis::Reaction);
    vec![f]
}
