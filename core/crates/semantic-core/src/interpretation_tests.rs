use super::*;
use crate::{graph::Snapshot, interpretation::*, records::Context};
use core_types::{scalar::DecimalU64, semantic::StageExecution};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn g04_each_discarded_edge_has_its_original_ordinal_and_reason() {
    let mut w = World::new();
    let edge = |from: &str, to: &str, kind: &str| Edge {
        schema_version: 1,
        from_node: from.into(),
        to_node: to.into(),
        relation: kind.into(),
    };
    w.graph.edges = vec![
        edge("who", "said", "quotes"),
        edge("who", "missing", "quotes"),
        edge("who", "said", "unregistered"),
        edge("missing", "who", "unregistered"),
    ];
    let staged = w.run(&w.candidate()).unwrap();
    let IrlProduct::Graph(graph) = staged.product else {
        panic!()
    };
    assert_eq!(graph.edges, w.graph.edges[..1]);
    assert_eq!(staged.adoption.outcome, StageOutcome::STAGE_OUTCOME_PARTIAL);
    assert_eq!(
        staged.adoption.reason.as_deref(),
        Some(
            "edge[1] unavailable target; edge[2] unregistered relation; edge[3] unavailable origin, unregistered relation"
        )
    );
}

#[test]
fn g04_judgments_record_only_observed_values_for_the_target_event() {
    use InterpretationStage::*;
    let mut w = World::new();
    // The dispatch was triggered later than the event it interprets (EV5).
    w.binding.event_id = "trigger".into();
    w.binding.environment.nature_appraiser = Some(w.mapping.clone());
    w.binding.environment.situation_matcher = Some(w.context.clone());
    let sources = BTreeMap::from([(w.turn.record_id.clone(), w.source_facts(&w.turn).unwrap())]);
    let targets = ["one", "zero", "unknown", "unassessed"]
        .map(String::from)
        .into();
    let empty = BTreeSet::new();
    let c = Context {
        snapshot: Snapshot {
            sources: &sources,
            concepts: &[],
            representation_version: "v1",
        },
        binding: &w.binding,
        producer: "worker",
        producer_version: "v1",
        target_event: &w.turn.record_id,
        known_tendencies: &targets,
        known_persons: &empty,
        prior_record_ids: &empty,
        limits: limits(),
    };
    let mut a: NatureAppraisalRecord = fixture();
    a.record_id = "judgment".into();
    a.event_id = c.target_event.into();
    a.binding = w.binding.clone();
    a.producer = c.producer.into();
    a.producer_version = c.producer_version.into();
    a.persons.clear();
    let mut endpoint = w.turn.clone();
    endpoint.record_id.clone_from(&a.record_id);
    endpoint.content = reference(&a);
    let dispatch = |stage, endpoint, candidate| JudgmentDispatch {
        seen: StageExecution {
            binding: &w.binding,
            transaction_id: "later-tx",
            event: &w.turn,
            interpretation: endpoint,
            stage,
            candidate,
            actual_output: Some(&w.mapping),
            isolation: &w.raw.provenance,
            relation_manifest: None,
        },
        context: &w.context,
        definitions: &[],
        governance: &w.mapping,
    };
    let a_ref = endpoint.content.clone();
    let a_dispatch = dispatch(INTERPRET_STAGE_A, &endpoint, Some(&a_ref));
    let accepted = stage_judgment(&a_dispatch, &c, Judgment::Appraisal(&a)).unwrap();
    assert_eq!(accepted.appraisal, Some(a.appraisal.clone()));
    a.appraisal.curiosity = if a.appraisal.curiosity == 0.0 {
        1.0
    } else {
        0.0
    };
    assert!(stage_judgment(&a_dispatch, &c, Judgment::Appraisal(&a)).is_err());

    let m = SituationMatchRecord {
        schema_version: 1,
        record_id: "judgment".into(),
        event_id: c.target_event.into(),
        binding: w.binding.clone(),
        producer: c.producer.into(),
        producer_version: c.producer_version.into(),
        matches: vec![TendencyMatch {
            schema_version: 1,
            tendency_id: "one".into(),
            mu: 1.0,
        }],
    };
    let m_ref = reference(&m);
    let m_id = RecordVersion {
        content: m_ref.clone(),
        ..endpoint.clone()
    };
    let m_dispatch = dispatch(INTERPRET_STAGE_M, &m_id, Some(&m_ref));
    let extra = vec![
        StageCoverage {
            schema_version: 1,
            target: "zero".into(),
            mu: Some(0.0),
            accepted: None,
            unknown_reason: None,
        },
        StageCoverage {
            schema_version: 1,
            target: "unknown".into(),
            mu: None,
            accepted: None,
            unknown_reason: Some("no observation".into()),
        },
    ];
    let run = |coverage| {
        stage_judgment(
            &m_dispatch,
            &c,
            Judgment::Match {
                record: &m,
                assessed_without_positive_match: coverage,
            },
        )
    };
    let partial = run(&extra).unwrap();
    assert_eq!(partial.outcome, StageOutcome::STAGE_OUTCOME_PARTIAL);
    assert_eq!(partial.coverage.len(), 3);
    assert_eq!(partial.coverage[..2], extra);
    assert_eq!(partial.coverage[2].mu, Some(1.0));
    assert!(!partial.coverage.iter().any(|r| r.target == "unassessed"));
    let conflicting = [StageCoverage {
        target: "one".into(),
        ..extra[0].clone()
    }];
    assert!(run(&conflicting).is_err());

    let status = DegradedStatus {
        schema_version: 1,
        states: vec![RunStatus::M_DEGRADED],
        reasons: Fault::Source.irl_status().reasons[1..].to_vec(),
    };
    let failure = RecordVersion {
        content: reference(&status),
        ..endpoint.clone()
    };
    let no_output = JudgmentDispatch {
        seen: StageExecution {
            actual_output: None,
            ..dispatch(INTERPRET_STAGE_M, &failure, None).seen
        },
        context: &w.context,
        definitions: &[],
        governance: &w.mapping,
    };
    let failed = stage_judgment(&no_output, &c, Judgment::Failed(&status, "unavailable")).unwrap();
    assert!(failed.coverage.is_empty());
    assert_eq!(failed.outcome, StageOutcome::STAGE_OUTCOME_FAILED);
    assert!(
        stage_judgment(
            &dispatch(INTERPRET_STAGE_A, &failure, None),
            &c,
            Judgment::Failed(&status, "unavailable")
        )
        .is_err()
    );
}

struct History {
    world: World,
    edge: RecordRelation,
    targets: Vec<String>,
    manifest: ObjectRef,
}
impl SemanticReadView for History {
    fn base_commit(&self) -> &str {
        self.world.base_commit()
    }
    fn branch(&self) -> &str {
        self.world.branch()
    }
    fn transaction(&self) -> Option<&str> {
        self.world.transaction()
    }
    fn content(&self, id: &RecordVersion) -> contract::Result<VisibleContent<'_>> {
        self.world.content(id)
    }
    fn authorize_provenance(&self, m: &SourceMetadataV2) -> contract::Result<()> {
        self.world.authorize_provenance(m)
    }
}
impl RevisionPrefix for History {
    fn approved_revision(&self, _: &StageAdoption) -> contract::Result<RevisionApproval<'_>> {
        Ok(RevisionApproval {
            prior_adoption: self.world.initial.as_ref().ok_or("no prior")?,
            edge: &self.edge,
            targets: &self.targets,
            manifest: &self.manifest,
        })
    }
}
fn receipt(set: RelationSet) -> RelationQueryReceipt {
    RelationQueryReceipt {
        schema_version: 1,
        expected_count: DecimalU64(set.relations.len() as u64),
        manifest: reference(&set),
        result: set,
    }
}
#[test]
fn g04_revision_resolves_prior_adoption_and_independently_scoped_manifest() {
    let mut w = World::new();
    let old = w.run(&w.candidate()).unwrap();
    let IrlProduct::Graph(graph) = old.product else {
        panic!()
    };
    let mut new = old.adoption.clone();
    new.interpretation.record_id = "second".into();
    new.binding.base_commit = "published".into();
    new.transaction_id = "backfill".into();
    let edge = RecordRelation {
        schema_version: 1,
        kind: RelationFact::REL_FACT_REINTERPRETATION,
        origin: new.interpretation.clone(),
        target: old.adoption.interpretation.clone(),
        purpose: "interpretation".into(),
        effective_commit: "next".into(),
        governance: w.mapping.clone(),
    };
    w.publish(old.adoption, &graph);
    let set = RelationSet {
        schema_version: 1,
        endpoint: edge.target.clone(),
        purpose: edge.purpose.clone(),
        frontier: "published".into(),
        relations: vec![],
    };
    let mut h = History {
        world: w,
        edge,
        targets: vec!["said".into()],
        manifest: reference(&set),
    };
    let empty = receipt(set.clone());
    let revised = stage_revision(new.clone(), &h, empty.clone(), limits()).unwrap();
    assert_eq!(revised.replaced_targets, ["said"]);
    assert_eq!(revised.relations, Some(empty.clone()));
    let previous = RecordRelation {
        origin: h.edge.target.clone(),
        target: new.event.clone(),
        kind: RelationFact::REL_FACT_SOURCE_LOCATOR,
        effective_commit: "published".into(),
        ..h.edge.clone()
    };
    let set = RelationSet {
        relations: vec![
            previous.clone(),
            RecordRelation {
                origin: previous.target.clone(),
                target: previous.origin.clone(),
                kind: RelationFact::REL_FACT_CORRECTION,
                ..previous
            },
        ],
        ..set
    };
    h.manifest = reference(&set);
    assert!(stage_revision(new.clone(), &h, empty, limits()).is_err()); // self-consistent but incomplete
    assert!(stage_revision(new.clone(), &h, receipt(set.clone()), limits()).is_ok());
    for change in [
        |q: &mut RelationSet| q.frontier.push('x'),
        |q: &mut RelationSet| q.purpose.push('x'),
        |q: &mut RelationSet| q.endpoint.record_id.push('x'),
    ] {
        let mut wrong = set.clone();
        change(&mut wrong);
        wrong.relations.clear();
        h.manifest = reference(&wrong); // internally consistent, independently wrong scope
        assert!(stage_revision(new.clone(), &h, receipt(wrong), limits()).is_err());
    }
    h.manifest = reference(&set);
    let mut other = new.clone();
    other.event.record_id.push('x');
    assert!(stage_revision(other, &h, receipt(set.clone()), limits()).is_err());
    let mut other = new.clone();
    other.stage = InterpretationStage::INTERPRET_STAGE_M;
    other.binding.environment.situation_matcher = other.producer_resource.clone();
    other.coverage[0].accepted = None;
    other.coverage[0].mu = Some(0.0);
    other.coverage.truncate(1);
    other.coverage[0].target = "said".into();
    assert!(stage_revision(other, &h, receipt(set.clone()), limits()).is_err());
    h.targets = vec!["not assessed".into()];
    assert!(stage_revision(new.clone(), &h, receipt(set.clone()), limits()).is_err());
    h.targets = vec!["said".into()];
    h.world.stage_bytes.clear();
    assert!(stage_revision(new, &h, receipt(set), limits()).is_err());
}
