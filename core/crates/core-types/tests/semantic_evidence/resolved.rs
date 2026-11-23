use super::*;

fn metadata(location: &SourceLocation) -> Value {
    json!({"schema_version":1,"derivation":1,"epistemic_type":1,"resolution_state":2,"representation_version":"v1",
        "locations":[location],"roots":[location.source],"transformations":[],"producer":object("trusted-core"),"validity":object("fixture-grant")})
}

#[test]
fn accepted_graph_requires_the_bound_registry_time_and_scoped_read_permission() {
    let (view, loc) = text_view(&UserText {
        schema_version: 1,
        text: "話🙂".into(),
    });
    let mut registry = registry();
    let label = json!({"schema_version":1,"zh":"引述","en":"report","ja":"引用"});
    let mut pred = revision("p");
    pred["lifecycle"] = json!(2);
    pred["label"] = label.clone();
    pred["definition"] = label;
    pred["grounding"] = json!([loc]);
    pred["allows_reported_text"] = json!(true);
    pred["roles"] = json!([{"schema_version":1,"role":"speaker","node_kind":1,"domain":concept("a"),"mandatory":true}]);
    registry["entries"].as_array_mut().unwrap().push(pred);
    let registry: ConceptRegistry = dto(registry);
    let g = json!({"schema_version":1,"id":"g","edges":[],"actions":[],"registry":reference(&registry),
        "references":[{"schema_version":1,"id":"who","source":metadata(&loc),"referent":concept("a")}],
        "statements":[{"schema_version":1,"id":"c","predicate":concept("p"),"roles":[{"schema_version":1,"role":"speaker","node_ref":"who"}],
        "polarity":true,"source":metadata(&loc),"reported_text":loc,"mode":1,"time_kind":2}]});
    let mut graph: SemanticGraphV2 = dto(g);
    assert!(check_graph_registry(&graph, &registry, limits()).is_ok());
    assert!(check_graph_sources(&graph, &view, "c", None, &[], limits()).is_ok());
    graph.statements[0].time_kind = ClaimTimeKind::CLAIM_TIME_FACT;
    let fact: ObjectRef = dto(object("fixture-time-fact"));
    graph.statements[0].time_fact = Some(fact.clone());
    assert!(check_graph_sources(&graph, &view, "c", None, &[], limits()).is_err());
    assert!(
        check_graph_sources(
            &graph,
            &view,
            "c",
            None,
            std::slice::from_ref(&fact),
            limits()
        )
        .is_ok()
    );
    graph.statements[0].source.validity.object_type = "revoked".into();
    assert!(check_graph_sources(&graph, &view, "c", None, &[fact], limits()).is_err());
    graph.statements[0].roles[0].node_ref = "c".into();
    assert!(check_graph_registry(&graph, &registry, limits()).is_err());
}

#[test]
fn conflicting_truth_keeps_the_observed_speech_and_its_subjective_scope() {
    let (mut view, loc) = text_view(&UserText {
        schema_version: 1,
        text: "話🙂".into(),
    });
    view.event.kind = EventKind::OBSERVATION;
    let mut src = metadata(&loc);
    src["speaker"] = json!({"schema_version":1,"layers":[{"schema_version":1,"source":loc.source,"visible":loc.text,
        "principal_id":"user","identity_evidence":object("identity")}],"governance":object("review")});
    let rule = json!({"schema_version":1,"source":1,"event_kind":2,"payload_type":"echo.user-text","exogenous":true,"self_narrative":false,"growth_access":2});
    let table: AttributeRuleTable = dto(json!({"schema_version":1,"rules":[rule]}));
    let mut whole = loc.clone();
    whole.path.clear();
    whole.text = None;
    let observed = json!({"schema_version":1,"scope":whole,"value":1,"evidence":[loc.source]});
    let asserted = json!({"schema_version":1,"scope":loc,"value":3,"evidence":[],"reason":"conflicting reports"});
    let subjective = json!({"schema_version":1,"scope":loc,"value":1,"evidence":[loc.source],"experiencer":"user"});
    let a = json!({"schema_version":1,"event_id":"e","rule_table_version":"r1","exogenous":true,"self_narrative":false,
        "source_key":"same-person","event":loc.source,"rule_table":reference(&table),"applied":rule,"source":src,"governance":object("review"),
        "occurred":[observed],"truth":[asserted],"subjective":[subjective],"c_unit":loc.source,"targets":[{"schema_version":1,"target":concept("a"),
        "occurrence_scopes":[{"schema_version":1,"scope":whole,"expected":1}],"truth_scopes":[],"subjective_scopes":[],"definition":object("target-rule")}],
        "inputs":[loc.source],"person_mapping":object("accounts"),"directed_to_n":loc.source});
    let positive: EventAttributesRecordV2 = dto(a.clone());
    let origin = AttributeSource::ATTR_SOURCE_HUMAN_RESPONSE;
    assert!(
        check_applied_rule(
            &positive,
            &table,
            &view.event,
            origin,
            "same-person",
            Some(&loc.source),
            limits()
        )
        .is_ok()
    );
    assert!(
        check_applied_rule(
            &positive,
            &table,
            &view.event,
            origin,
            "another-person",
            Some(&loc.source),
            limits()
        )
        .is_err()
    );
    assert!(
        check_applied_rule(
            &positive,
            &table,
            &view.event,
            origin,
            "same-person",
            None,
            limits()
        )
        .is_err()
    );
    let mut bad = a.clone();
    bad["subjective"][0]
        .as_object_mut()
        .unwrap()
        .remove("experiencer");
    assert!(check("EventAttributesRecordV2", &bad).is_err());
    let mut bad = a;
    bad["targets"][0]["occurrence_scopes"][0]["expected"] = json!(3);
    assert!(check("EventAttributesRecordV2", &bad).is_err());
}

#[test]
fn speaker_layers_resolve_their_own_utf8_field_and_need_a_trusted_grant() {
    let (view, loc) = text_view(&UserText {
        schema_version: 1,
        text: "文🙂".into(),
    });
    let mut src = metadata(&loc);
    src["speaker"] = json!({"schema_version":1,"layers":[{"schema_version":1,"source":loc.source,"visible":loc.text,
        "principal_id":"human","identity_evidence":object("identity")}],"governance":object("review")});
    let mut g: SemanticGraphV2 = dto(
        json!({"schema_version":1,"id":"g","edges":[],"actions":[],"registry":object("echo.concept-registry"),
        "references":[{"schema_version":1,"id":"ref","source":src}],"statements":[]}),
    );
    assert!(check_graph_sources(&g, &view, "c", None, &[], limits()).is_ok());
    let layer = &mut g.references[0].source.speaker.as_mut().unwrap().layers[0];
    layer.visible.start.0 += 1;
    assert!(check_graph_sources(&g, &view, "c", None, &[], limits()).is_err());
}
