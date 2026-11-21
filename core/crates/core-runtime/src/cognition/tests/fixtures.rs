//! Synthetic receipts and a registered first request on a fresh temporary store.
use super::*;
pub(crate) fn initial_q(
    store: &Store<Rehearsal>,
    leaf: &ObjectRef,
    event: &EventEnvelope,
) -> ObjectRef {
    let binding = CognitionBinding {
        schema_version: 1,
        activity_id: "activity".into(),
        step_id: "step-0".into(),
        request_id: "request-0".into(),
        round_index: DecimalU64(0),
        branch_id: "fixture-branch".into(),
        base_commit: store.base_commit().unwrap(),
        q_revision: "q-0".into(),
        input_frontier: vec![event.id.clone()],
        snapshot_id: "snap".into(),
        environment: environment(leaf),
        protocol: leaf.clone(),
        permission_snapshot: leaf.clone(),
        source_validity_version: "sources".into(),
        read_set: vec![leaf.clone()],
    };
    let input = CognitiveInput {
        schema_version: 1,
        binding,
        materials: vec![],
        focus: Some(leaf.clone()),
        new_inputs: vec![],
        results: vec![],
        retrievals: vec![],
        budget: leaf.clone(),
    };
    let input = store.prepare(&input, "1").unwrap();
    let transition = store
        .prepare(
            &SystemMaintenanceRecord {
                schema_version: 1,
                record_id: "start".into(),
                kind: CognitiveMaintenanceKind::COG_MAINTENANCE_RESUME,
                reason: "synthetic start".into(),
                details: leaf.clone(),
                ledger_refs: vec![],
                related_commit: store.base_commit().unwrap(),
                recorded_at_ns: DecimalU64(10),
            },
            "1",
        )
        .unwrap();
    store
        .prepare(
            &CognitionState {
                schema_version: 1,
                revision: "q-0".into(),
                mode: CognitiveMode::COG_RUNNING,
                reason: None,
                activities: vec![ActivityState {
                    schema_version: 1,
                    activity_id: "activity".into(),
                    revision: "q-0".into(),
                    materials: vec![],
                    focus: Some(leaf.clone()),
                    latest_step: None,
                    waiting: vec![],
                    successor: None,
                    status: CognitiveActivityStatus::COG_ACTIVITY_RUNNING,
                }],
                inflight: vec![],
                active_request: Some(input.reference().clone()),
                runnable_successor: None,
                transition: transition.reference().clone(),
            },
            "1",
        )
        .unwrap()
        .reference()
        .clone()
}
pub(super) const TEXT: &str = "也许明天会下雨。";
pub(crate) fn activity(
    store: &Store<Rehearsal>,
    ticket: &ActivityTicket<Rehearsal>,
    cloud: bool,
) -> InternalActivityRecordV2 {
    let b = &ticket.input().binding;
    let leaf = &b.protocol;
    let text = store.prepare_raw(TEXT.as_bytes()).unwrap();
    let product = CognitiveMaterial {
        schema_version: 1,
        content: text.reference().clone(),
        past_reaction: false,
        metadata: SourceMetadata {
            schema_version: 1,
            source_refs: b.input_frontier.clone(),
            source_span: leaf.clone(),
            derivation: Derivation::NORMALIZED,
            epistemic_type: EpistemicType::MODEL_INFERENCE,
            resolution_state: ResolutionState::UNRESOLVED,
            representation_version: "fixture".into(),
        },
    };
    let mut record = InternalActivityRecordV2 {
        schema_version: 1,
        record_id: format!("thought-{}", b.round_index.0),
        previous: ticket.previous().cloned(),
        started_at_ns: DecimalU64(10),
        finished_at_ns: DecimalU64(11),
        local: None,
        remote: None,
    };
    if cloud {
        let body = store.prepare_request_body(br#"{"messages":[]}"#).unwrap();
        let review = store
            .prepare_raw(b"fixture: explicitly speculative")
            .unwrap();
        record.remote = Some(RemoteCognitiveReceipt {
            schema_version: 1,
            binding: b.clone(),
            provider: "fixture".into(),
            requested_model: "synthetic".into(),
            returned_model: Some("synthetic".into()),
            provider_revision: None,
            model_sha256: None,
            tokenizer_sha256: None,
            mode: RemoteTextMode::REMOTE_TEXT_INTERNAL_TEXT_ONLY_THINKING_OFF,
            parameters: leaf.clone(),
            format: leaf.clone(),
            mapping: leaf.clone(),
            meter: leaf.clone(),
            capacity: Some(DecimalU64(64)),
            output_reserve: Some(DecimalU64(16)),
            input_manifest: ticket.reference().clone(),
            sent_request: Some(body.reference().clone()),
            request_sha256: Some(body.reference().digest.clone()),
            raw_response: Some(text.reference().clone()),
            returned_text: Some(text.reference().clone()),
            finish_reason: Some("stop".into()),
            usage: None,
            status: RemoteAttemptStatus::REMOTE_ATTEMPT_COMPLETED,
            reason: None,
            adoption: TextAdoption::TEXT_ADOPTION_FULL,
            accepted_range: Some(Utf8Range {
                schema_version: 1,
                start: DecimalU64(0),
                end: text.reference().byte_length,
            }),
            product: Some(product),
            review: Some(review.reference().clone()),
            local_pause_reason: None,
        });
    } else {
        let modulation = store
            .prepare(&with_refs::<ModulationEnvelopeV2>(leaf), "1")
            .unwrap();
        record.local = Some(CognitiveResultAndReceipt {
            schema_version: 1,
            binding: b.clone(),
            input: ticket.reference().clone(),
            products: vec![product],
            next_focus: None,
            queries: vec![],
            action_plan: Some(ActionPlan {
                schema_version: 1,
                plan_id: "local-plan".into(),
                snapshot_id: b.snapshot_id.clone(),
                task_version: b.q_revision.clone(),
                protocol_version: b.protocol.resource_version.clone(),
                output_or_tool_parameters: leaf.clone(),
                expected_scope: leaf.clone(),
                dependencies: vec![],
                deadline_ns: core_types::scalar::DecimalI64(100),
            }),
            completion: CognitiveCompletion::COG_COMPLETED,
            reason: None,
            model_hash: b.environment.model.digest.clone(),
            tokenizer_hash: b.environment.tokenizer.digest.clone(),
            modulation: modulation.reference().clone(),
            carrier_receipt: Some(leaf.clone()),
            usage: leaf.clone(),
        });
    }
    record
}
pub(crate) fn effects(
    store: &Store<Rehearsal>,
    ticket: &ActivityTicket<Rehearsal>,
    record: &InternalActivityRecordV2,
) -> RoundEffects {
    let mut input = ticket.input().clone();
    let n = input.binding.round_index.0 + 1;
    input.binding.step_id = format!("step-{n}");
    input.binding.request_id = format!("request-{n}");
    input.binding.round_index = DecimalU64(n);
    input.binding.q_revision = format!("q-{n}");
    let (products, action) = if let Some(l) = &record.local {
        (
            l.products.clone(),
            Some((format!("action-{n}"), input.binding.protocol.clone())),
        )
    } else {
        (
            vec![record.remote.as_ref().unwrap().product.clone().unwrap()],
            None,
        )
    };
    input.materials = products;
    let head = store.current_manifest().unwrap().unwrap();
    let mut x: FastStateSnapshotV2 = store.reader().unwrap().read(&head.staged_x).unwrap();
    x.anchor_time_ns.0 += 1;
    let mut appraisal: NatureAppraisalRecord = with_refs(&input.binding.protocol);
    appraisal.event_id = record.record_id.clone();
    appraisal.binding.event_id = record.record_id.clone();
    appraisal.binding.branch_id = input.binding.branch_id.clone();
    appraisal.binding.base_commit = input.binding.base_commit.clone();
    appraisal.binding.environment = input.binding.environment.clone();
    appraisal.binding.source_validity_version = input.binding.source_validity_version.clone();
    let decision = store.prepare(&appraisal, "1").unwrap();
    RoundEffects {
        interpretations: vec![decision.reference().clone()],
        x,
        next_input: input,
        action,
        actions_per_round: 1,
        inflight_capacity: 2,
    }
}
pub(crate) fn review(
    _: &RemoteCognitiveReceipt,
    decision: &[u8],
    text: &str,
) -> core_store::Result<bool> {
    Ok(decision == b"fixture: explicitly speculative" && text == TEXT)
}
pub(super) fn rejected_effects() -> core_store::Result<RoundEffects> {
    panic!("audit-only return reached state computation")
}
