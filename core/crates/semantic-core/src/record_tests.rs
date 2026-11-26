use super::tests::*;
use super::*;
use core_types::generated::*;
use std::collections::BTreeSet;

#[test]
fn annotation_and_reinterpretation_references_do_not_fill_missing_events() {
    let sources = sources();
    let binding: ProposalBindingV2 = fixture();
    let empty = BTreeSet::new();
    let prior = BTreeSet::from(["old".into(), "new".into()]);
    let c = records::Context {
        snapshot: graph::Snapshot {
            sources: &sources,
            concepts: &[],
            representation_version: "v1",
        },
        binding: &binding,
        producer: "worker",
        producer_version: "v1",
        target_event: "human",
        known_tendencies: &empty,
        known_persons: &empty,
        prior_record_ids: &prior,
        limits: limits(),
    };
    macro_rules! prepare {
        ($r:expr) => {{
            $r.record_id = "r".into();
            $r.event_id = "human".into();
            $r.binding = binding.clone();
            $r.producer = "worker".into();
            $r.producer_version = "v1".into();
        }};
    }
    let mut same: SameMatterAnnotation = fixture();
    prepare!(same);
    same.same_as = vec!["absent".into()];
    assert_eq!(
        records::preflight(records::Record::SameMatter(&same), &c),
        Err(Fault::Source)
    );
    same.same_as = vec!["human".into()];
    records::preflight(records::Record::SameMatter(&same), &c).unwrap();
    let mut lie: LieAnnotation = fixture();
    prepare!(lie);
    lie.statement = "human".into();
    lie.contradicts = "model".into();
    assert_eq!(
        records::preflight(records::Record::Lie(&lie), &c),
        Err(Fault::Source)
    );
    let mut link: ReinterpretationLink = fixture();
    prepare!(link);
    link.interpreter_version = "v1".into();
    link.new_record = "new".into();
    link.prior_records = vec!["old".into()];
    records::preflight(records::Record::Reinterpretation(&link), &c).unwrap();
    link.prior_records = vec!["missing".into()];
    assert_eq!(
        records::preflight(records::Record::Reinterpretation(&link), &c),
        Err(Fault::Reference)
    );
    link.prior_records = vec!["new".into()];
    assert_eq!(
        records::preflight(records::Record::Reinterpretation(&link), &c),
        Err(Fault::Reference)
    );
}

#[test]
fn candidate_direction_is_rejected_without_normalizing_or_filling_languages() {
    let sources = sources();
    let binding: ProposalBindingV2 = fixture();
    let empty = BTreeSet::new();
    let c = records::Context {
        snapshot: graph::Snapshot {
            sources: &sources,
            concepts: &[],
            representation_version: "v1",
        },
        binding: &binding,
        producer: "worker",
        producer_version: "v1",
        target_event: "human",
        known_tendencies: &empty,
        known_persons: &empty,
        prior_record_ids: &empty,
        limits: limits(),
    };
    let mut r: TendencyCandidateRecord = fixture();
    r.record_id = "candidate".into();
    r.event_id = "human".into();
    r.binding = binding.clone();
    r.producer = "worker".into();
    r.producer_version = "v1".into();
    r.direction = NatureVector {
        schema_version: 1,
        connection: 1.0,
        empathy: 0.0,
        curiosity: 0.0,
        achievement: 0.0,
    };
    r.relation = None;
    records::preflight(records::Record::Candidate(&r), &c).unwrap();
    r.direction.connection = 0.5;
    assert_eq!(
        records::preflight(records::Record::Candidate(&r), &c),
        Err(Fault::Numeric)
    );
    assert_eq!(r.direction.connection, 0.5);
    r.direction.connection = 1.0;
    r.concept.name.ja.clear();
    assert!(records::preflight(records::Record::Candidate(&r), &c).is_err());
}
#[test]
fn interpretation_numeric_source_and_binding_counterexamples() {
    let sources = sources();
    let binding: ProposalBindingV2 = fixture();
    let tendencies = BTreeSet::from(["t".into()]);
    let persons = BTreeSet::from(["person".into()]);
    let ids = BTreeSet::new();
    let c = records::Context {
        snapshot: graph::Snapshot {
            sources: &sources,
            concepts: &[],
            representation_version: "v1",
        },
        binding: &binding,
        producer: "worker",
        producer_version: "v1",
        target_event: "human",
        known_tendencies: &tendencies,
        known_persons: &persons,
        prior_record_ids: &ids,
        limits: limits(),
    };
    let mut r: NatureAppraisalRecord = fixture();
    r.record_id = "record".into();
    r.event_id = "human".into();
    r.binding = binding.clone();
    r.producer = "worker".into();
    r.producer_version = "v1".into();
    r.persons.clear();
    records::preflight(records::Record::Appraisal(&r), &c).unwrap();
    for value in [1.01, -1.01, f64::NAN, f64::INFINITY] {
        let mut bad = r.clone();
        bad.appraisal.connection = value;
        assert!(records::preflight(records::Record::Appraisal(&bad), &c).is_err());
    }
    let mut m: SituationMatchRecord = fixture();
    m.record_id = r.record_id.clone();
    m.event_id = r.event_id.clone();
    m.binding = binding.clone();
    m.producer = r.producer.clone();
    m.producer_version = r.producer_version.clone();
    for mu in [0.0, -0.1, 1.1, f64::NAN] {
        m.matches = vec![TendencyMatch {
            schema_version: 1,
            tendency_id: "t".into(),
            mu,
        }];
        assert!(records::preflight(records::Record::Match(&m), &c).is_err());
    }
    m.matches[0].mu = 0.5;
    records::preflight(records::Record::Match(&m), &c).unwrap();
    let mut teaching: TeachingAnnotation = fixture();
    teaching.record_id = r.record_id;
    teaching.event_id = r.event_id;
    teaching.binding = binding.clone();
    teaching.producer = r.producer;
    teaching.producer_version = r.producer_version;
    teaching.tendency_id = "t".into();
    teaching.speaker = "impostor".into();
    assert_eq!(
        records::preflight(records::Record::Teaching(&teaching), &c),
        Err(Fault::Source)
    );
    teaching.speaker = "person".into();
    records::preflight(records::Record::Teaching(&teaching), &c).unwrap();
    teaching.event_id = "model".into();
    assert!(records::preflight(records::Record::Teaching(&teaching), &c).is_err());
}
#[test]
fn event_attribute_forgery_cannot_make_own_story_external() {
    use source_rules::Origin::*;
    let aliases = aliases::Aliases::build(
        &["account".into(), "person".into()],
        &[("account".into(), "person".into())],
        limits(),
    )
    .unwrap();
    for (origin, external, narrative) in [
        (DirectedUserInteraction, true, false),
        (OtherPersonsReply, true, false),
        (DesktopAboutUser, false, false),
        (OwnNarrative, false, true),
        (OwnAction, false, false),
        (TriggeredProgramResult, false, false),
        (SystemMaintenance, false, false),
        (ImportedRehearsalDiary, false, true),
    ] {
        let mut r = EventAttributesRecord {
            schema_version: 1,
            event_id: "e".into(),
            rule_table_version: source_rules::PARTITION_VERSION.into(),
            exogenous: external,
            self_narrative: narrative,
            verifiable: false,
            source_key: "person".into(),
        };
        source_rules::preflight(&r, "e", origin, "account", &aliases, limits()).unwrap();
        r.exogenous = !external;
        assert_eq!(
            source_rules::preflight(&r, "e", origin, "account", &aliases, limits()),
            Err(Fault::Source)
        );
    }
}
