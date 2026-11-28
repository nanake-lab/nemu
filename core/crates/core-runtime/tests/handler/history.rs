//! P5-53 3/4. Only credentials, clock/stage inputs and events are synthetic.
use super::*;
use core_store::CommitCapability;
use core_types::{WireMessage, scalar::DecimalI64};
use fact_view::{Access, Budgets, FactQuery, FactView, Purpose};

fn session(id: &str) -> TerminalSession {
    TerminalSession {
        schema_version: 1,
        session_id: id.into(),
        installation_id: "shared-installation".into(),
        protocol_version: 1,
        capabilities: vec![IngressKind::INGRESS_EVENT],
        budget: TerminalBudget {
            schema_version: 1,
            version: "test".into(),
            max_frame_bytes: 65536,
            max_chunk_bytes: 4096,
            max_object_bytes: 8192,
            max_uploads: 2,
            max_upload_bytes: 16384,
        },
    }
}

// Synthetic service registry is independent of payload, session spelling and device claims.
struct Issuer;
impl CoreHandler<Rehearsal> for Issuer {
    fn authorize_event(
        &mut self,
        _: &Store<Rehearsal>,
        _: &TerminalSession,
        _: &TerminalEvent,
        _: &SourceMetadata,
    ) -> Decision<Grants> {
        Err(IngressReason::CAPABILITY_DENIED)
    }
    fn local_text_author(
        &self,
        event: &EventEnvelope,
    ) -> gateway::Result<Option<LocalUserBinding>> {
        let (issued, principal) = match event.session.as_str() {
            "first" | "wrong" => ("first", "person-a"),
            "second" => ("second", "person-b"),
            _ => return Ok(None),
        };
        Ok(Some(LocalUserBinding {
            schema_version: 1,
            principal_id: principal.into(),
            authentication_record_id: format!("auth-{issued}"),
            session_id: issued.into(),
            installation_id: "shared-installation".into(),
        }))
    }
}

fn handler(cap: CommitCapability<Rehearsal>) -> impl CoreHandler<Rehearsal> {
    RehearsalHandler::new(
        cap,
        Issuer,
        inputs,
        limits(),
        SpeechMode::B_REMOTE_PROJECTED,
    )
}

fn ingest<T: WireMessage>(
    writer: &mut Store<Rehearsal>,
    session: &TerminalSession,
    id: &str,
    key: &str,
    payload: &T,
) -> RecordVersion {
    let payload = writer.prepare(payload, "1").unwrap();
    let origin = writer
        .prepare(
            &SourceMetadata {
                schema_version: 1,
                source_refs: vec![],
                source_span: payload.reference().clone(),
                derivation: Derivation::VERBATIM,
                epistemic_type: EpistemicType::USER_ASSERTION,
                resolution_state: ResolutionState::UNRESOLVED,
                representation_version: "synthetic".into(),
            },
            "1",
        )
        .unwrap();
    let event = TerminalEvent {
        schema_version: 1,
        id: id.into(),
        stable_source_key: key.into(),
        kind: EventKind::OBSERVATION,
        event_time_ns: DecimalI64(17),
        parents: vec![],
        payload_ref: payload.reference().clone(),
        provenance: origin.reference().clone(),
        payload_digest: payload.reference().digest.clone(),
        action_id: None,
        trial_id: None,
        correction_of: vec![],
        derivation: Derivation::VERBATIM,
    };
    let receipt = writer.ingest_terminal(&event, session).unwrap();
    assert_eq!(receipt.stage, ReceiptStage::RECEIVED_DURABLE);
    RecordVersion {
        schema_version: 1,
        branch_id: "test".into(),
        record_id: id.into(),
        content: event.payload_ref,
    }
}

fn text(s: &str) -> UserText {
    UserText {
        schema_version: 1,
        text: s.into(),
    }
}

fn evidence(name: &str, value: &impl WireMessage) {
    if let Some(dir) = std::env::var_os("NEMU_P5_53_EVIDENCE_DIR") {
        std::fs::write(
            std::path::Path::new(&dir).join(name),
            value.protobuf_bytes(limits()).unwrap(),
        )
        .unwrap();
    }
}

fn access() -> Access {
    Access {
        allowed: true,
        ..Access::default()
    }
}

#[test]
fn mode_b_reads_only_published_text_and_authentication_at_one_frontier_after_restart() {
    let root = TemporaryRoot::new().unwrap();
    let (mut writer, cap) = initial(&root);
    let start = writer.base_commit().unwrap();
    let mut runtime = handler(cap);
    assert!(writer.take_commit_capability().is_err());
    let shared = "她引用了「🙂」";
    let first = ingest(&mut writer, &session("first"), "r1", "k1", &text(shared));
    let reader = Reader::<Rehearsal>::open_temporary(&root, "handler", config()).unwrap();
    assert!(reader.transactions_at(&start, 8).unwrap().is_empty());
    assert!(reader.published_event_at(&start, "r1").is_err());
    let one = runtime.process_next(&mut writer).unwrap().remove(0);
    let reader = Reader::<Rehearsal>::open_temporary(&root, "handler", config()).unwrap();
    let before = FactView::rebuild(&reader, &one.commit_id, 8, None).unwrap();
    let original = before.mode_b_text("r1", &access()).unwrap();
    assert_eq!(original.version, first);
    assert_eq!(original.h_position, 1); // Initial publication precedes D[0].
    assert_eq!(original.speaker.governance, one.snapshot);
    let second = ingest(&mut writer, &session("second"), "r2", "k2", &text(shared));
    assert_eq!(first.content, second.content);
    assert!(before.mode_b_text("r2", &access()).is_err());
    writer.set_fault(Fault::DiskFull(FaultPoint::BeforePublish));
    assert!(matches!(
        runtime.process_next(&mut writer),
        Err(gateway::Error::Store(core_store::Error::Io(_)))
    ));
    assert_eq!(writer.base_commit().unwrap(), one.commit_id);
    assert_eq!(before.mode_b_text("r1", &access()).unwrap(), original);
    let after_failure = Reader::<Rehearsal>::open_temporary(&root, "handler", config()).unwrap();
    assert!(
        after_failure
            .published_event_at(&one.commit_id, "r2")
            .is_err()
    );
    drop((after_failure, runtime, writer));

    let mut writer = Store::<Rehearsal>::open_temporary(&root, "handler", config()).unwrap();
    assert_eq!(writer.next_raw().unwrap().unwrap().id, "r2");
    let mut runtime = handler(writer.take_commit_capability().unwrap());
    let two = runtime.process_next(&mut writer).unwrap().remove(0);
    assert!(before.mode_b_text("r1", &access()).is_err());
    let at_two = Reader::<Rehearsal>::open_temporary(&root, "handler", config()).unwrap();
    let second_view = FactView::rebuild(&at_two, &two.commit_id, 8, None).unwrap();
    let conflict = ingest(
        &mut writer,
        &session("second"),
        "rejected",
        "k2",
        &text("not history"),
    );
    assert_eq!(
        writer.settle_duplicate().unwrap().unwrap().stage,
        ReceiptStage::REJECTED
    );
    let prefix = writer.base_commit().unwrap();
    assert_eq!(at_two.current_snapshot().unwrap().unwrap(), two.snapshot);
    // An unchanged state snapshot after rejection still has a different commit frontier.
    assert!(second_view.mode_b_text("r2", &access()).is_err());
    drop((runtime, writer));
    let independent = Reader::<Rehearsal>::open_temporary(&root, "handler", config()).unwrap();
    let view = FactView::rebuild(&independent, &prefix, 8, None).unwrap();
    assert!(view.mode_b_text(&conflict.record_id, &access()).is_err());
    let query = FactQuery::new(&prefix, Purpose::Task, shared, &Budgets::default());
    let material = view.mode_b_input(&second, &query, &access(), None).unwrap();
    let facts = &material.task_facts.projection;
    evidence("mode-b-facts.pb", facts);
    evidence(
        "mode-b-query-audit.pb",
        &material.task_facts.trace.audit_record(facts).unwrap(),
    );
    assert_eq!(material.latest, second);
    assert_eq!(facts.history_prefix, prefix);
    assert_eq!(facts.items.len(), 2);
    assert_eq!(material.records.len(), 2);
    for (version, pos, principal, snapshot) in [
        (&first, 1, "person-a", &one.snapshot),
        (&second, 2, "person-b", &two.snapshot),
    ] {
        let record = &material.records[&version.record_id];
        assert_eq!(record.h_position, pos);
        assert_eq!(record.version, *version);
        assert_eq!(record.text, shared);
        assert_eq!(record.speaker.governance, *snapshot);
        assert_eq!(record.speaker.layers.len(), 1);
        let layer = &record.speaker.layers[0];
        assert_eq!(layer.source, *version);
        assert_eq!(layer.principal_id, principal);
        assert_eq!(
            (layer.visible.start.0, layer.visible.end.0),
            (0, shared.len() as u64)
        );
        let auth: LocalUserBinding = independent.read(&layer.identity_evidence).unwrap();
        assert_eq!(auth.principal_id, principal);
        let (tx, event) = independent.read_published_transaction(snapshot).unwrap();
        assert_eq!(event.payload_ref, version.content);
        assert_eq!(auth.session_id, event.session);
        evidence(&format!("{}-event.pb", event.id), &event);
        evidence(&format!("{}-version.pb", event.id), &record.version);
        evidence(&format!("{}-speaker.pb", event.id), &record.speaker);
        evidence(&format!("{}-issuer.pb", event.id), &auth);
        evidence(
            &format!("{}-publication.pb", event.id),
            &independent.read::<CoreTransactionV4>(snapshot).unwrap(),
        );
        let q: InitialRuntimeRoot = independent.read(&tx.staged_q).unwrap();
        assert!(q.fast.is_none() && q.environment.law_version.is_none());
        eprintln!(
            "frontier={prefix} event={} h={pos} principal={principal} content={} bytes={}",
            version.record_id,
            core_types::scalar::hex(&version.content.digest.0),
            shared.len()
        );
    }
    let miss = FactQuery::new(&prefix, Purpose::Task, "absent term", &Budgets::default());
    let base = view.mode_b_input(&second, &miss, &access(), None).unwrap();
    assert_eq!(base.task_facts.projection.status, QueryStatus::NO_MATCH);
    assert_eq!(base.records.len(), 1);
    let mut wrong = second.clone();
    wrong.content.resource_version = "other".into();
    assert!(view.mode_b_input(&wrong, &query, &access(), None).is_err());
    wrong = second.clone();
    wrong.branch_id = "other".into();
    assert!(view.mode_b_input(&wrong, &query, &access(), None).is_err());
    let old_query = FactQuery::new(&one.commit_id, Purpose::Task, shared, &Budgets::default());
    assert!(
        view.mode_b_input(&second, &old_query, &access(), None)
            .is_err()
    );
    let mut denied = access();
    denied.sources.insert("another-installation".into());
    assert!(view.mode_b_text("r1", &denied).is_err());
    let mut short = query;
    short.budget.items = 1;
    assert!(view.mode_b_input(&second, &short, &access(), None).is_err());
}

#[test]
fn mode_b_withheld_withdrawn_or_unattributed_text_cannot_supply_a_speaker() {
    let root = TemporaryRoot::new().unwrap();
    let (mut writer, cap) = initial(&root);
    let mut runtime = handler(cap);
    let known = ingest(
        &mut writer,
        &session("first"),
        "known",
        "one",
        &text("synthetic wording"),
    );
    runtime.process_next(&mut writer).unwrap();
    ingest(
        &mut writer,
        &session("unproven"),
        "unknown",
        "two",
        &text("synthetic wording"),
    );
    runtime.process_next(&mut writer).unwrap();
    let read = writer.verified_reader().unwrap();
    let prefix = writer.base_commit().unwrap();
    let view = FactView::rebuild(&read, &prefix, 8, None).unwrap();
    assert!(view.mode_b_text("unknown", &access()).is_err());
    let q = FactQuery::new(&prefix, Purpose::Task, "synthetic", &Budgets::default());
    assert!(view.mode_b_input(&known, &q, &access(), None).is_err());
    ingest(
        &mut writer,
        &session("unproven"),
        "withdrawal",
        "three",
        &UserRetraction {
            schema_version: 1,
            target_refs: vec![known.record_id.clone()],
            note: None,
        },
    );
    runtime.process_next(&mut writer).unwrap();
    let read = writer.verified_reader().unwrap();
    let current = writer.base_commit().unwrap();
    let view = FactView::rebuild(&read, &current, 8, None).unwrap();
    assert!(view.mode_b_text("known", &access()).is_err());
    ingest(
        &mut writer,
        &session("wrong"),
        "other-session",
        "four",
        &text("still pending"),
    );
    assert!(runtime.process_next(&mut writer).is_err());
    assert_eq!(writer.base_commit().unwrap(), current);
    assert_eq!(writer.next_raw().unwrap().unwrap().id, "other-session");
    assert!(view.mode_b_text("other-session", &access()).is_err());
}
