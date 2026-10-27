use super::{objects::*, tests::*, *};
use core_types::generated::*;

#[test]
fn alias_byte_budget_rejects_large_ids_before_cloning_the_index() {
    let ids = ["x".repeat(100)];
    let mut budget = limits();
    budget.max_bytes = 10;
    assert!(matches!(
        aliases::Aliases::build(&ids, &[], budget),
        Err(Fault::Budget)
    ));
}

#[test]
fn semantic_failure_emits_only_existing_degraded_status_without_a_graph() {
    use core_types::WireMessage;
    for fault in [
        Fault::Schema,
        Fault::Budget,
        Fault::Binding,
        Fault::Object,
        Fault::Duplicate,
        Fault::Reference,
        Fault::Source,
        Fault::Epistemic,
        Fault::Concept,
        Fault::AliasCycle,
        Fault::Numeric,
        Fault::MissingContract,
    ] {
        let status = fault.irl_status();
        status.canonical_bytes(limits()).unwrap();
        assert_eq!(status.states, [RunStatus::IRL_DEGRADED]);
        assert_eq!(status.reasons[0].code, FailureCode::IRL_FAILED);
    }
}

#[test]
fn typed_adapters_preserve_actual_structured_payloads_and_reject_substitution() {
    macro_rules! check {
        ($ty:ty, $variant:ident) => {{
            let payload: $ty = fixture();
            let mut event: EventEnvelope = fixture();
            event.payload_ref = reference(&payload);
            event.payload_digest = event.payload_ref.digest.clone();
            check_structured(&event, StructuredPayload::$variant(&payload), limits()).unwrap();
            event.payload_digest.0[0] ^= 1;
            assert_eq!(
                check_structured(&event, StructuredPayload::$variant(&payload), limits()),
                Err(Fault::Object)
            );
        }};
    }
    check!(TouchObservation, Touch);
    check!(DesktopObservation, Desktop);
    check!(SpeechDelivery, Delivery);
    check!(CodexSessionObservation, Codex);
    check!(TerminalMaintenance, Maintenance);
}

#[test]
fn candidate_graph_must_be_the_actual_bound_object_with_the_expected_producer() {
    let graph: SemanticGraph = fixture();
    let mut candidate: SemanticCandidateV2 = fixture();
    candidate.candidate_graph = reference(&graph);
    let binding = candidate.binding.clone();
    let producer = (
        candidate.producer.clone(),
        candidate.producer_version.clone(),
    );
    check_candidate(
        &candidate,
        &graph,
        &binding,
        (&producer.0, &producer.1),
        limits(),
    )
    .unwrap();
    candidate.producer_version.push_str("-forged");
    assert_eq!(
        check_candidate(
            &candidate,
            &graph,
            &binding,
            (&producer.0, &producer.1),
            limits()
        ),
        Err(Fault::Binding)
    );
    candidate.producer_version = producer.1.clone();
    candidate.candidate_graph.digest.0[0] ^= 1;
    assert_eq!(
        check_candidate(
            &candidate,
            &graph,
            &binding,
            (&producer.0, &producer.1),
            limits()
        ),
        Err(Fault::Object)
    );
}

#[test]
fn binding_compares_nested_environment_and_read_set_not_only_event_id() {
    let b: ProposalBindingV2 = fixture();
    let mut changed = b.clone();
    changed
        .environment
        .registry
        .resource_version
        .push_str("-changed");
    assert_eq!(check_binding(&changed, &b, limits()), Err(Fault::Binding));
    changed = b.clone();
    changed.read_set.push(fixture());
    assert!(check_binding(&changed, &b, limits()).is_err());
}
