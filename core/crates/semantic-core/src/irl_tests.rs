use super::{
    Fault,
    graph::SourceFacts,
    irl::*,
    tests::{fixture, limits, reference},
};
use core_types::{
    WireMessage, contract,
    generated::*,
    semantic::{SemanticReadView, VisibleContent},
};
use serde_json::json;

struct World {
    raw: EventEnvelope,
    bytes: Vec<u8>,
    turn: RecordVersion,
    registry: ConceptRegistry,
    adapter: TextAdapterTable,
    mapping: ObjectRef,
    facts: FactProjection,
    context: ObjectRef,
    binding: ProposalBindingV2,
    graph: SemanticGraphV2,
    transaction: Option<&'static str>,
    initial: Option<RecordVersion>,
    stage_bytes: Vec<u8>,
    committed: Option<(StageAdoption, Vec<u8>)>,
}

impl SemanticReadView for World {
    fn branch(&self) -> &str {
        &self.turn.branch_id
    }
    fn base_commit(&self) -> &str {
        if self.transaction.is_some() {
            &self.binding.base_commit
        } else {
            "published"
        }
    }
    fn transaction(&self) -> Option<&str> {
        self.transaction
    }
    fn authorize_provenance(&self, meta: &SourceMetadataV2) -> contract::Result<()> {
        if meta.validity != self.raw.provenance
            || meta.speaker.is_some()
            || !meta.transformations.is_empty()
        {
            return Err("unverified provenance".into());
        }
        Ok(())
    }
    fn content(&self, id: &RecordVersion) -> contract::Result<VisibleContent<'_>> {
        if id == &self.turn {
            return Ok(VisibleContent::Event(&self.raw, &self.bytes));
        }
        if let Some(at) = &self.initial
            && at == id
            && !self.stage_bytes.is_empty()
        {
            return Ok(VisibleContent::Adopted(at, &self.stage_bytes));
        }
        if let Some((stage, bytes)) = &self.committed
            && id == &stage.interpretation
        {
            return Ok(VisibleContent::Adopted(&stage.interpretation, bytes));
        }
        Err("missing".into())
    }
}
impl IrlReadView for World {
    fn source_facts(&self, id: &RecordVersion) -> contract::Result<SourceFacts> {
        if id != &self.turn {
            return Err("not in dispatch".into());
        }
        Ok(SourceFacts {
            epistemic: EpistemicType::USER_ASSERTION,
            self_narrative: false,
            speaker: None,
            verifiable: false,
        })
    }
}
impl CommittedIrlPrefix for World {
    fn initial_irl(&self, _: &RecordVersion) -> contract::Result<Option<&RecordVersion>> {
        Ok(self.initial.as_ref())
    }
}
impl World {
    fn new() -> Self {
        let payload = UserText {
            schema_version: 1,
            text: "字🙂".into(),
        };
        let mut raw: EventEnvelope = fixture();
        raw.payload_ref = reference(&payload);
        raw.payload_digest = raw.payload_ref.digest.clone();
        let mut binding: ProposalBindingV2 = fixture();
        binding.event_id = raw.id.clone();
        binding.raw_log_position = raw.raw_log_position;
        let turn = RecordVersion {
            schema_version: 1,
            record_id: raw.id.clone(),
            branch_id: binding.branch_id.clone(),
            content: raw.payload_ref.clone(),
        };
        let path = json!([{"schema_version":1,"field_number":2}]);
        let loc = json!({"schema_version":1,"source":turn,"path":path,"text":{"schema_version":1,"start":"0","end":"7"}});
        let label = json!({"schema_version":1,"en":"reported","ja":"引用","zh":"引述"});
        let concept = ConceptRef {
            schema_version: 1,
            concept_id: "report".into(),
            registry_version: "synthetic-v1".into(),
        };
        let registry: ConceptRegistry = serde_json::from_value(json!({"schema_version":1,"version":"synthetic-v1",
            "entries":[{"schema_version":1,"concept_id":"report","lifecycle":2,"governance":raw.provenance,
                "grounding":[loc],"label":label,"definition":label,"positive_examples":[],"negative_examples":[],
                "roles":[{"schema_version":1,"role":"subject","node_kind":1,"mandatory":true}],"allows_reported_text":true,"constraints":[]}],
            "aliases":[],"links":[],"relation_kinds":[],"interaction_kinds":[],"edge_kinds":["quotes"]})).unwrap();
        binding.environment.registry = reference(&registry);
        let entry = |tag| json!({"schema_version":1,"payload_tag":tag,"payload_schema":1,"path":path,"predicate":concept,"role":"subject"});
        let adapter = serde_json::from_value(
            json!({"schema_version":1,"registry":binding.environment.registry,
            "entries":[entry("echo.user-text"),entry("echo.user-utterance")]}),
        )
        .unwrap();
        let mapping = reference(&adapter);
        let facts = FactProjection {
            schema_version: 1,
            history_prefix: binding.base_commit.clone(),
            query_version: "q".into(),
            status: QueryStatus::OK,
            items: vec![],
            index_version: "index".into(),
            truncated: false,
        };
        let context = reference(&facts);
        binding.c_core_digest = context.digest.clone();
        binding.read_set = vec![context.clone(), mapping.clone()];
        let source = json!({"schema_version":1,"locations":[loc],"roots":[turn],"transformations":[],"producer":binding.environment.irl,
            "validity":raw.provenance,"representation_version":"synthetic-v1","derivation":1,"epistemic_type":1,"resolution_state":1});
        let graph = serde_json::from_value(json!({"schema_version":1,"id":"g","registry":binding.environment.registry,
            "references":[{"schema_version":1,"id":"who","source":source}],
            "statements":[{"schema_version":1,"id":"said","source":source,"predicate":concept,
               "roles":[{"schema_version":1,"role":"subject","node_ref":"who"}],"polarity":true,"reported_text":loc,"mode":1,"time_kind":2}],
            "actions":[],"edges":[]})).unwrap();
        Self {
            raw,
            bytes: payload.canonical_bytes(limits()).unwrap(),
            turn,
            registry,
            adapter,
            mapping,
            facts,
            context,
            binding,
            graph,
            transaction: Some("tx"),
            initial: None,
            stage_bytes: vec![],
            committed: None,
        }
    }
    fn candidate(&self) -> SemanticCandidateV2 {
        SemanticCandidateV2 {
            schema_version: 1,
            candidate_graph: reference(&self.graph),
            binding: self.binding.clone(),
            producer: "irl".into(),
            producer_version: "v1".into(),
            actual_output: self.raw.provenance.clone(),
        }
    }
    fn run(&self, c: &SemanticCandidateV2) -> super::Result<StagedIrl> {
        stage_irl(
            &IrlDispatch {
                binding: &self.binding,
                event: &self.turn,
                transaction: "tx",
                interpretation_id: "first-irl",
                producer: "irl",
                producer_version: "v1",
                representation_version: "synthetic-v1",
                candidate: &reference(c),
                actual_output: &self.raw.provenance,
                context: &self.context,
                facts: &self.facts,
                adapter: &self.adapter,
                adapter_ref: &self.mapping,
                isolation: &self.binding.validation_snapshot,
                governance: &self.binding.environment.config,
                time_facts: &[],
            },
            self,
            c,
            &self.graph,
            &self.registry,
            limits(),
        )
    }
    fn publish(&mut self, stage: StageAdoption, product: &impl WireMessage) {
        self.transaction = None;
        self.initial = Some(RecordVersion {
            schema_version: 1,
            branch_id: self.turn.branch_id.clone(),
            record_id: "adoption".into(),
            content: reference(&stage),
        });
        self.stage_bytes = stage.canonical_bytes(limits()).unwrap();
        self.committed = Some((stage, product.canonical_bytes(limits()).unwrap()));
    }
}

#[test]
fn u6_reads_the_same_turn_graph_only_from_a_committed_prefix() {
    let mut w = World::new();
    let staged = w.run(&w.candidate()).unwrap();
    assert_eq!(staged.adoption.outcome, StageOutcome::STAGE_OUTCOME_VALID);
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Err(Fault::Binding)
    );
    w.transaction = None;
    assert_eq!(committed_irl(&w, &w.turn, "tx", limits()), Ok(None)); // rollback/receipt only
    let IrlProduct::Graph(graph) = staged.product else {
        panic!("expected a governed graph")
    };
    w.publish(staged.adoption, &graph);
    let stage_bytes = std::mem::take(&mut w.stage_bytes);
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Err(Fault::Source)
    ); // graph alone
    w.stage_bytes = stage_bytes;
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Ok(Some(CommittedIrl::Graph(graph.clone())))
    );
    assert_eq!(
        committed_irl(&w, &w.turn, "retry", limits()),
        Err(Fault::Binding)
    );
    let mut other = w.turn.clone();
    other.record_id.push('2');
    assert_eq!(
        committed_irl(&w, &other, "tx", limits()),
        Err(Fault::Binding)
    );
    w.committed.as_mut().unwrap().1.push(0);
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Err(Fault::Object)
    );
    // A failed adoption that still points to a real graph has the wrong product type.
    let mut stage = w.committed.take().unwrap().0;
    stage.outcome = StageOutcome::STAGE_OUTCOME_FAILED;
    stage.reason = Some("failed".into());
    stage.coverage.clear();
    w.publish(stage, &graph);
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Err(Fault::Object)
    );
}

#[test]
fn u6_returns_the_recorded_failure_and_detects_missing_or_substituted_status() {
    let mut w = World::new();
    w.graph.references[0].source.epistemic_type = EpistemicType::USER_SELF_REPORT;
    let mut staged = w.run(&w.candidate()).unwrap();
    assert_eq!(staged.adoption.outcome, StageOutcome::STAGE_OUTCOME_FAILED);
    let IrlProduct::Degraded(mut status) = staged.product else {
        panic!("expected a degraded interpretation")
    };
    // A recorded detail cannot be reconstructed from the stage outcome or reason.
    status.reasons[0].detail_ref = Some(w.context.clone());
    staged.adoption.interpretation.content = reference(&status);
    w.publish(staged.adoption, &status);
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Ok(Some(CommittedIrl::Failed(status.clone())))
    );
    let (stage, _) = w.committed.take().unwrap();
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Err(Fault::Source)
    );
    status.reasons[0].detail_ref.as_mut().unwrap().digest.0[0] ^= 1;
    w.committed = Some((stage, status.canonical_bytes(limits()).unwrap()));
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Err(Fault::Object)
    );
}
#[test]
fn partial_result_quarantines_broken_spans_and_dependent_claims() {
    let mut w = World::new();
    let mut legal = w.graph.references[0].clone();
    legal.id = "independent".into();
    w.graph.references.push(legal);
    w.graph.references[0].source.locations[0]
        .text
        .as_mut()
        .unwrap()
        .end
        .0 = 1;
    let stage = w.run(&w.candidate()).unwrap();
    assert_eq!(stage.adoption.outcome, StageOutcome::STAGE_OUTCOME_PARTIAL);
    let IrlProduct::Graph(graph) = stage.product else {
        panic!("expected partial graph")
    };
    assert_eq!(graph.references.len(), 1);
    assert_eq!(graph.references[0].id, "independent");
    assert!(graph.statements.is_empty());
    assert_eq!(
        stage
            .adoption
            .coverage
            .iter()
            .filter(|t| t.unknown_reason.is_some())
            .count(),
        2
    );
    w.publish(stage.adoption, &graph);
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Ok(Some(CommittedIrl::Graph(graph)))
    );
}
#[test]
fn dispatch_and_authenticated_source_cannot_be_replaced_by_candidate_metadata() {
    let mut w = World::new();
    for mutate in [
        |c: &mut SemanticCandidateV2| c.binding.base_commit.push('x'),
        |c: &mut SemanticCandidateV2| c.producer_version.push('x'),
        |c: &mut SemanticCandidateV2| c.actual_output.digest.0[0] ^= 1,
        |c: &mut SemanticCandidateV2| c.candidate_graph.digest.0[0] ^= 1,
    ] {
        let mut c = w.candidate();
        mutate(&mut c);
        assert!(w.run(&c).is_err());
    }
    w.graph.references[0].source.epistemic_type = EpistemicType::USER_SELF_REPORT;
    let failed = w.run(&w.candidate()).unwrap();
    assert_eq!(failed.adoption.outcome, StageOutcome::STAGE_OUTCOME_FAILED);
    let IrlProduct::Degraded(status) = failed.product else {
        panic!("invalid roots cannot yield z")
    };
    assert_eq!(status.states, [RunStatus::IRL_DEGRADED]);
    assert!(failed.adoption.coverage.is_empty());
    w.graph.references[0].source.epistemic_type = EpistemicType::USER_ASSERTION;
    w.turn.record_id.push('x');
    assert!(w.run(&w.candidate()).is_err());
}
#[test]
fn depth_budget_counts_relations_between_flat_wire_nodes() {
    let mut w = World::new();
    w.graph.statements.clear();
    let template = w.graph.references[0].clone();
    for i in 0..limits().max_depth {
        let mut next = template.clone();
        next.id = format!("chain-{i}");
        w.graph.edges.push(Edge {
            schema_version: 1,
            from_node: w.graph.references.last().unwrap().id.clone(),
            to_node: next.id.clone(),
            relation: "quotes".into(),
        });
        w.graph.references.push(next);
    }
    let stage = w.run(&w.candidate()).unwrap();
    assert_eq!(stage.adoption.outcome, StageOutcome::STAGE_OUTCOME_FAILED);
    assert_eq!(stage.adoption.reason.as_deref(), Some("Budget"));
    let IrlProduct::Degraded(status) = stage.product else {
        panic!("expected a degraded interpretation")
    };
    w.publish(stage.adoption, &status);
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Ok(Some(CommittedIrl::Failed(status)))
    );
}

#[test]
fn registered_transcript_and_unsupported_touch_follow_different_stage_paths() {
    let mut w = World::new();
    let voice = UserUtterance {
        text: "字🙂".into(),
        ..fixture()
    };
    let voice_ref = reference(&voice);
    w.bytes = voice.canonical_bytes(limits()).unwrap();
    w.raw.payload_ref = voice_ref.clone();
    w.raw.payload_digest = voice_ref.digest.clone();
    w.turn.content = voice_ref;
    for source in [
        &mut w.graph.references[0].source,
        &mut w.graph.statements[0].source,
    ] {
        source.roots[0] = w.turn.clone();
        source.locations[0].source = w.turn.clone();
    }
    w.graph.statements[0].reported_text.source = w.turn.clone();
    let stage = w.run(&w.candidate()).unwrap();
    assert_eq!(stage.adoption.outcome, StageOutcome::STAGE_OUTCOME_VALID);
    let touch = fixture::<TouchObservationV2>();
    w.bytes = touch.canonical_bytes(limits()).unwrap();
    w.raw.payload_ref = reference(&touch);
    w.raw.payload_digest = w.raw.payload_ref.digest.clone();
    w.turn.content = w.raw.payload_ref.clone();
    assert!(matches!(
        w.run(&w.candidate()),
        Err(Fault::UnsupportedSource)
    ));
}

#[test]
fn a_provisional_ref_needs_an_id_already_in_the_pinned_registry() {
    let mut w = World::new();
    let mut existing = w.registry.entries[0].clone();
    existing.concept_id = "existing-provisional".into();
    existing.lifecycle = ConceptLifecycle::CONCEPT_LIFE_PROVISIONAL;
    w.registry.entries.push(existing);
    let registry_ref = reference(&w.registry);
    w.graph.registry = registry_ref.clone();
    w.binding.environment.registry = registry_ref.clone();
    w.adapter.registry = registry_ref;
    w.mapping = reference(&w.adapter);
    w.binding.read_set = vec![w.mapping.clone(), w.context.clone()];
    w.graph.references[0].source.resolution_state = ResolutionState::PROVISIONAL;
    w.graph.references[0].referent = Some(ConceptRef {
        schema_version: 1,
        concept_id: "existing-provisional".into(),
        registry_version: w.registry.version.clone(),
    });
    let stage = w.run(&w.candidate()).unwrap();
    assert_eq!(stage.adoption.outcome, StageOutcome::STAGE_OUTCOME_VALID);
    assert!(stage.adoption.registry_after.is_none());
    w.graph.references[0].referent.as_mut().unwrap().concept_id = "invented".into();
    assert_eq!(
        w.run(&w.candidate()).unwrap().adoption.outcome,
        StageOutcome::STAGE_OUTCOME_FAILED
    );
}
