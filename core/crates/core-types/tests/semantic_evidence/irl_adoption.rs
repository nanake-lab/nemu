use super::*;
use core_types::irl_pipe::{
    Direction, IrlAdoptionEvidence, IrlExcerpt, IrlProducerObservation, PipeCall, locate_excerpt,
};

#[test]
fn bound_excerpt_checks_utf8_repetitions_and_the_original_field() {
    let (view, mut loc) = text_view(&UserText {
        schema_version: 1,
        text: "甲🙂甲甲甲".into(),
    });
    loc.text = None;
    let find = |q, range| locate_excerpt(&view, "c", None, &loc, q, range, limits());
    let emoji = find("🙂", None).unwrap().text.unwrap();
    assert_eq!((emoji.start.0, emoji.end.0), (3, 7));
    assert!(find("甲甲", None).is_err());
    let mut chosen = Utf8Range {
        schema_version: 1,
        start: DecimalU64(7),
        end: DecimalU64(13),
    };
    assert!(find("甲甲", Some(&chosen)).is_ok());
    chosen.start.0 += 1;
    assert!(locate_excerpt(&view, "c", None, &loc, "甲甲", Some(&chosen), limits()).is_err());
    loc.source.record_id = "p0-08-0000".into();
    assert!(locate_excerpt(&view, "c", None, &loc, "🙂", None, limits()).is_err());
}

#[test]
fn only_the_observed_call_output_and_current_stage_can_be_linked() {
    for link in [
        IrlPipeLink::IRL_WORKER_BRIDGE,
        IrlPipeLink::IRL_WORKER_ADAPTER,
    ] {
        check_return(link);
    }
}

fn check_return(endpoint: IrlPipeLink) {
    use sha2::{Digest, Sha256};
    let (mut view, loc) = text_view(&UserText {
        schema_version: 1,
        text: "話🙂".into(),
    });
    view.staged = true;
    let registry: ConceptRegistry = dto(registry());
    let graph: SemanticGraphV2 = dto(json!({
        "schema_version":1,"id":"parsed","registry":reference(&registry),
        "statements":[],"actions":[],"edges":[],
        "references":[{"schema_version":1,"id":"word","source":{
            "schema_version":1,"derivation":1,"epistemic_type":1,"resolution_state":2,
            "representation_version":"1","locations":[loc],"roots":[loc.source],
            "transformations":[],"producer":object("parser"),"validity":object("fixture-grant")}}]
    }));
    let rows: Vec<Value> =
        serde_json::from_slice(include_bytes!("../../../../contracts/golden/legal.json")).unwrap();
    let row = rows
        .iter()
        .map(|r| &r["request"])
        .find(|q| {
            q["name"] == "IrlCandidateReturn"
                && q["value"]["producer"]["link"] == u32::from(endpoint)
        })
        .unwrap();
    let mut ret: IrlCandidateReturn = dto(row["value"].clone());
    let output = ret.frame.output.as_mut().unwrap();
    output.data = HexBytes("🙂".as_bytes().to_vec());
    output.object.byte_length = DecimalU64(output.data.0.len() as u64);
    output.object.digest = HexBytes(Sha256::digest(&output.data.0).to_vec());
    ret.producer.output = output.object.clone();
    let excerpt = std::str::from_utf8(&output.data.0).unwrap().to_owned();
    let mut opening: IrlPipeFrame = dto(sample("IrlPipeFrame"));
    let mut dispatch: IrlDispatch = dto(sample("IrlDispatch"));
    dispatch.target = loc.source.clone();
    dispatch.payload.object = loc.source.content.clone();
    dispatch.registry.object = reference(&registry);
    opening.call = ret.frame.call.clone();
    opening.dispatch = Some(dispatch.clone());
    let mut binding: ProposalBindingV2 = dto(sample("ProposalBindingV2"));
    binding.event_id = loc.source.record_id.clone();
    binding.branch_id = view.branch().into();
    binding.base_commit = view.base_commit().into();
    binding.c_core_digest = dispatch.facts.object.digest.clone();
    binding.environment.registry = dispatch.registry.object.clone();
    binding.read_set = vec![
        dispatch.payload.object,
        dispatch.facts.object.clone(),
        dispatch.registry.object,
    ];
    let p = &mut ret.producer;
    p.delegation.inputs = vec![IrlMaterial {
        schema_version: 1,
        object: loc.source.content.clone(),
        data: HexBytes(view.bytes.clone()),
    }];
    let candidate: SemanticCandidateV2 = dto(json!({
        "schema_version":1,"candidate_graph":reference(&graph),"producer":"fixture-parser",
        "producer_version":"v1","actual_output":p.output,"binding":binding
    }));
    ret.frame.graph = Some(candidate.candidate_graph.clone());
    let observed = ret.producer.clone();
    let mut stage: StageAdoption = dto(sample("StageAdoption"));
    stage.binding = binding.clone();
    stage.event = loc.source.clone();
    stage.interpretation.branch_id = binding.branch_id.clone();
    stage.transaction_id = "txn".into();
    stage.context = dispatch.facts.object;
    stage.producer_resource = Some(binding.environment.irl.clone());
    stage.registry_before = binding.environment.registry.clone();
    stage.candidate = Some(reference(&candidate));
    stage.actual_output = Some(candidate.actual_output.clone());
    stage.isolation = observed.isolation.clone().unwrap();
    let link = IrlAdoptionLink {
        schema_version: 1,
        candidate_return: reference(&ret),
        candidate: reference(&candidate),
        stage: reference(&stage),
    };
    let mut call = PipeCall::new(
        opening.clone(),
        IrlPipeLink::IRL_CORE_WORKER,
        Some(&binding),
        &dto(sample("IrlPipeBudget")),
    )
    .unwrap();
    call.receive(
        Direction::ToCallee,
        &opening.protobuf_bytes(limits()).unwrap(),
        limits(),
        |_| Ok(&[]),
    )
    .unwrap();
    let bytes = ret.protobuf_bytes(limits()).unwrap();
    let mut other = observed.clone();
    other.call.request_id.push('x');
    let observe = |record| IrlProducerObservation {
        record,
        remote_exchange: (endpoint == IrlPipeLink::IRL_WORKER_BRIDGE)
            .then_some((b"request".as_slice(), b"response".as_slice())),
    };
    let other_observation = observe(&other);
    assert!(
        call.receive_candidate(&bytes, &other_observation, limits())
            .is_err()
    );
    let observation = observe(&observed);
    if endpoint == IrlPipeLink::IRL_WORKER_BRIDGE {
        let corrupt = IrlProducerObservation {
            record: &observed,
            remote_exchange: Some((b"request", b"other")),
        };
        assert!(call.receive_candidate(&bytes, &corrupt, limits()).is_err());
    }

    let accepted = call
        .receive_candidate(&bytes, &observation, limits())
        .unwrap();
    assert!(
        call.receive_candidate(&bytes, &observation, limits())
            .is_err()
    );
    let excerpts = [IrlExcerpt {
        field: &loc,
        quote: &excerpt,
        known_range: None,
    }];
    let mut evidence = IrlAdoptionEvidence {
        execution: StageExecution {
            binding: &binding,
            transaction_id: "txn",
            event: &loc.source,
            interpretation: &stage.interpretation,
            stage: InterpretationStage::INTERPRET_STAGE_IRL,
            candidate: Some(&link.candidate),
            actual_output: Some(&observed.output),
            isolation: &stage.isolation,
            relation_manifest: None,
        },
        parsed_graph: &graph,
        excerpts: &excerpts,
        registry: &registry,
        time_facts: &[],
        producer: "fixture-parser",
        producer_version: "v1",
        verified_neutral_input: observed.neutral_input.as_ref().unwrap(),
    };
    accepted
        .check_adoption(&link, &candidate, &stage, &evidence, &view, limits())
        .unwrap();
    // A transport completion remains inspectable when EV6 evidence is absent
    // or the service returned only a prefix. Neither result can validate a stage.
    let mut incomplete = ret.clone();
    incomplete.producer.neutral_input = None;
    if let Some(remote) = &mut incomplete.producer.remote {
        remote.status = RemoteAttemptStatus::REMOTE_ATTEMPT_TRUNCATED;
    }
    let mut retry = PipeCall::new(
        opening.clone(),
        IrlPipeLink::IRL_CORE_WORKER,
        Some(&binding),
        &dto(sample("IrlPipeBudget")),
    )
    .unwrap();
    retry
        .receive(
            Direction::ToCallee,
            &opening.protobuf_bytes(limits()).unwrap(),
            limits(),
            |_| Ok(&[]),
        )
        .unwrap();
    let incomplete_observation = IrlProducerObservation {
        record: &incomplete.producer,
        remote_exchange: observation.remote_exchange,
    };
    let retained = retry
        .receive_candidate(
            &incomplete.protobuf_bytes(limits()).unwrap(),
            &incomplete_observation,
            limits(),
        )
        .unwrap();
    let incomplete_link = IrlAdoptionLink {
        candidate_return: reference(&incomplete),
        ..link.clone()
    };
    assert!(
        retained
            .check_adoption(
                &incomplete_link,
                &candidate,
                &stage,
                &evidence,
                &view,
                limits()
            )
            .is_err()
    );
    evidence.excerpts = &[];
    assert!(
        accepted
            .check_adoption(&link, &candidate, &stage, &evidence, &view, limits())
            .is_err()
    );
    evidence.excerpts = &excerpts;
    evidence.verified_neutral_input = &link.stage;
    assert!(
        accepted
            .check_adoption(&link, &candidate, &stage, &evidence, &view, limits())
            .is_err()
    );
    evidence.verified_neutral_input = observed.neutral_input.as_ref().unwrap();
    evidence.execution.transaction_id = "other";
    assert!(
        accepted
            .check_adoption(&link, &candidate, &stage, &evidence, &view, limits())
            .is_err()
    );
}
