use super::*;
use action_gate::outbox::delivery::Message;
use core_store::{Fault, FaultPoint};
use core_types::scalar::DecimalU64;

mod fixtures;
mod gates;
mod recovery;
use fixtures::*;

#[test]
fn sys11_request_and_outbox_publish_together_and_only_once() {
    for point in [FaultPoint::BeforePublish, FaultPoint::SqlCommitted] {
        let mut f = Fixture::new(false, false);
        let frame = f.frame(
            EventKind::ACTION_REQUEST,
            &f.request_ref.clone(),
            "register",
        );
        f.store.set_fault(Fault::DiskFull(point));
        assert!(f.transaction(frame).register(20).is_err());
        let mut f = f.reopen();
        if point == FaultPoint::BeforePublish {
            let event = f.store.next_raw().unwrap().unwrap();
            let current = f.store.current().unwrap().unwrap();
            assert!(current.audit_refs.is_empty());
            let q: CognitionState = f.store.reader().unwrap().read(&current.staged_q).unwrap();
            assert_eq!(q.inflight[0].status, ActionStatus::PLANNED);
            let frame = f.frame_next(&event);
            f.transaction(frame).register(20).unwrap();
        }
        assert_eq!(f.status().status, ActionStatus::REQUESTED);
        assert!(f.store.next_raw().unwrap().is_none());
        let reader = f.store.reader().unwrap();
        let current = reader.current().unwrap().unwrap();
        let q: CognitionState = reader.read(&current.staged_q).unwrap();
        let h: EventEnvelope = reader.read(&current.staged_h).unwrap();
        assert_eq!(q.inflight.len(), 1);
        assert_eq!(q.inflight[0].status, ActionStatus::REQUESTED);
        assert_eq!(
            h.action_id.as_deref(),
            Some(q.inflight[0].action_id.as_str())
        );
        assert_eq!(h.kind, EventKind::ACTION_REQUEST);
        assert_eq!(h.payload_ref, q.inflight[0].request);
        let second = f.frame(
            EventKind::ACTION_REQUEST,
            &f.request_ref.clone(),
            "distinct-second-request",
        );
        assert!(f.transaction(second).register(20).is_err());
        assert_eq!(f.status().status, ActionStatus::REQUESTED);
    }
}

#[test]
fn x04_sys12_body_sent_receipt_lost_reopen_no_resend_then_late_success() {
    let start = std::time::Instant::now();
    let mut f = Fixture::new(false, false);
    f.registered();
    f.authorized();
    let frame = f.frame(EventKind::MAINTENANCE, &f.leaf.clone(), "consume");
    let request = f.request.clone();
    let settings = lifecycle::Settings {
        credential_lifetime_ns: Some(100),
        max_dispatching: Some(1),
        ..Default::default()
    };
    let mut issuer = outbox::Issuer::default();
    issuer
        .issue(
            request.grant.clone(),
            request.action_payload.0.clone(),
            limits(),
        )
        .unwrap();
    let auth = authority(&request, &settings, &issuer);
    let mut sends = 0;
    outbox::dispatch(
        &mut f.transaction(frame),
        &request,
        &auth,
        limits(),
        || Some(40),
        |msg| {
            let Message::Body(body) = msg else {
                panic!("unexpected speech")
            };
            assert_eq!(body, &request);
            recovery::deliver_then_exit(body);
            sends += 1;
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(sends, 1);
    assert_eq!(f.status().status, ActionStatus::DISPATCHING);
    let mut f = f.reopen();
    let frame = f.frame(EventKind::MAINTENANCE, &f.leaf.clone(), "recover");
    f.transaction(frame).unknown().unwrap();
    assert_eq!(f.status().status, ActionStatus::UNKNOWN_ACTION);
    let result = BodyActionResult {
        schema_version: 1,
        action_id: "a".into(),
        status: ActionStatus::SUCCEEDED,
        detail: None,
    };
    let r = f.store.prepare(&result, "1").unwrap();
    let frame = f.frame(EventKind::EXECUTION_RESULT, r.reference(), "late");
    let mut transaction = f.transaction(frame);
    assert!(
        outbox::dispatch(
            &mut transaction,
            &request,
            &auth,
            limits(),
            || Some(41),
            |_| {
                sends += 1;
                Ok(())
            }
        )
        .is_err()
    );
    assert!(
        transaction
            .history
            .check_reservation(&request.grant)
            .is_err()
    );
    let mut second = request.grant.clone();
    second.issuance_id = "different-issuance".into();
    assert!(transaction.history.check_reservation(&second).is_err());
    second.budget.version = "fresh-period".into();
    transaction.history.check_reservation(&second).unwrap();
    transaction.settle().unwrap();
    assert_eq!(sends, 1);
    let actual = f.status();
    assert_eq!(actual.status, ActionStatus::SUCCEEDED);
    assert_eq!(
        f.store
            .reader()
            .unwrap()
            .read::<BodyActionResult>(&actual.actual_result.unwrap())
            .unwrap(),
        result
    );
    println!(
        "X04 W02 temporary-store end-to-end elapsed_ms={}",
        start.elapsed().as_millis()
    );
}

#[test]
fn sys12_speech_partial_delivery_preserves_the_terminal_extent() {
    let mut f = Fixture::new(true, false);
    f.registered();
    f.authorized();
    let request = f.request.clone();
    let frame = f.frame(EventKind::MAINTENANCE, &f.leaf.clone(), "consume");
    let auth = authority(&request, &f.settings, &f.issuer);
    let mut transaction =
        ActionTransaction::open(&mut f.store, &f.cap, frame, limits(), 32, &mut f.index).unwrap();
    let mut sent = vec![];
    outbox::dispatch(
        &mut transaction,
        &request,
        &auth,
        limits(),
        || Some(41),
        |message| {
            let Message::Chunk(c) = message else {
                panic!("unexpected body")
            };
            sent.push((c.chunk_index.0, c.text.clone()));
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(sent, vec![(0, "一。".into()), (1, "二。".into())]);
    let receipt = SpeechDelivery {
        schema_version: 1,
        action_id: "a".into(),
        output_id: "o".into(),
        delivered_chunks: DecimalU64(1),
        played_chars: DecimalU64(2),
        status: DeliveryStatus::PARTIAL,
    };
    let r = f.store.prepare(&receipt, "1").unwrap();
    let frame = f.frame(EventKind::EXECUTION_RESULT, r.reference(), "partial");
    f.transaction(frame).settle().unwrap();
    let f = f.reopen();
    let actual = f.status();
    assert_eq!(actual.status, ActionStatus::CANCELLED_ACTION);
    assert_eq!(
        f.store
            .reader()
            .unwrap()
            .read::<SpeechDelivery>(&actual.actual_result.unwrap())
            .unwrap(),
        receipt
    );
}

#[test]
fn i26_denies_changed_payload_and_commits_consent_before_authorization() {
    let mut f = Fixture::new(false, true);
    f.registered();
    let request = f.request.clone();
    let consent = request.grant.consent.as_ref().unwrap();
    let reply = ApprovalDecisionV2 {
        schema_version: 1,
        approval_id: "approval".into(),
        decision: ApprovalChoice::ALLOW_ONCE,
        payload_digest: request.grant.payload_digest.clone(),
    };
    let r = f.store.prepare(&reply, "1").unwrap();
    let frame = f.frame(EventKind::OBSERVATION, r.reference(), "human-reply");
    f.transaction(frame).decision(&consent.user, 25).unwrap();
    f.authorized();
    let frame = f.frame(EventKind::MAINTENANCE, &f.leaf.clone(), "consume");
    let auth = authority(&request, &f.settings, &f.issuer);
    let mut altered = request.clone();
    altered.action_payload.0.push(1);
    let mut tx =
        ActionTransaction::open(&mut f.store, &f.cap, frame, limits(), 32, &mut f.index).unwrap();
    assert!(
        outbox::dispatch(
            &mut tx,
            &altered,
            &auth,
            limits(),
            || Some(40),
            |_| panic!("changed bytes sent")
        )
        .is_err()
    );
    outbox::dispatch(&mut tx, &request, &auth, limits(), || Some(40), |_| Ok(())).unwrap();
    assert_eq!(f.status().status, ActionStatus::DISPATCHING);
}

#[test]
fn sys12_denial_and_timeout_cancel_without_an_issued_grant() {
    for deny in [true, false] {
        let mut f = Fixture::new(false, true);
        f.registered();
        let user = f.request.grant.consent.as_ref().unwrap().user.clone();
        let frame = if deny {
            let reply = ApprovalDecisionV2 {
                schema_version: 1,
                approval_id: "approval".into(),
                decision: ApprovalChoice::DENY,
                payload_digest: f.request.grant.payload_digest.clone(),
            };
            let r = f.store.prepare(&reply, "1").unwrap();
            f.frame(EventKind::OBSERVATION, r.reference(), "deny")
        } else {
            f.frame(EventKind::MAINTENANCE, &f.leaf.clone(), "timeout")
        };
        f.transaction(frame)
            .decision(&user, if deny { 25 } else { 80 })
            .unwrap();
        assert_eq!(f.status().status, ActionStatus::CANCELLED_ACTION);
        let tx = f.store.current().unwrap().unwrap();
        let denial: PermissionResult = f.store.reader().unwrap().read(&tx.audit_refs[0]).unwrap();
        assert_eq!(
            denial.denied,
            Some(if deny {
                PermissionDenial::CONSENT_MISSING
            } else {
                PermissionDenial::GRANT_EXPIRED
            })
        );
    }
}
