use super::*;

#[test]
fn cloud_receipt_cannot_be_relabelled_as_a_local_plan() {
    let f = Fixture::new(false, false);
    let reader = f.store.reader().unwrap();
    let q: CognitionState = reader
        .read(&f.store.current().unwrap().unwrap().staged_q)
        .unwrap();
    let old: InternalActivityRecord = reader.read(&q.inflight[0].origin_record).unwrap();
    let mut origin = InternalActivityRecordV2 {
        schema_version: 1,
        record_id: old.record_id,
        previous: old.previous,
        started_at_ns: old.started_at_ns,
        finished_at_ns: old.finished_at_ns,
        local: Some(old.result),
        remote: None,
    };
    let binding = origin.local.as_ref().unwrap().binding.clone();
    origin.remote = Some(RemoteCognitiveReceipt {
        schema_version: 1,
        binding,
        provider: "fixture".into(),
        requested_model: "synthetic".into(),
        returned_model: None,
        provider_revision: None,
        model_sha256: None,
        tokenizer_sha256: None,
        mode: RemoteTextMode::REMOTE_TEXT_INTERNAL_TEXT_ONLY_THINKING_OFF,
        parameters: f.leaf.clone(),
        format: f.leaf.clone(),
        mapping: f.leaf.clone(),
        meter: f.leaf.clone(),
        capacity: None,
        output_reserve: None,
        input_manifest: f.leaf.clone(),
        sent_request: None,
        request_sha256: None,
        raw_response: None,
        returned_text: None,
        finish_reason: None,
        usage: None,
        status: RemoteAttemptStatus::REMOTE_ATTEMPT_NOT_SENT,
        reason: Some("synthetic not sent".into()),
        adoption: TextAdoption::TEXT_ADOPTION_NO_TEXT,
        accepted_range: None,
        product: None,
        review: None,
        local_pause_reason: Some("local unavailable".into()),
    });
    assert!(lifecycle::local_plan(lifecycle::Origin::Receipt(&origin)).is_err());
    origin.local = None;
    assert!(
        stage_local_plan(
            &f.store,
            q,
            lifecycle::Origin::Receipt(&origin),
            "other".into(),
            f.payload.clone(),
            1,
            limits()
        )
        .is_err()
    );
}

#[test]
fn expired_withdrawn_wrong_session_or_changed_scope_cannot_be_consumed() {
    let mut f = Fixture::new(false, false);
    f.registered();
    f.authorized();
    let frame = f.frame(EventKind::MAINTENANCE, &f.leaf.clone(), "consume");
    let mut tx =
        ActionTransaction::open(&mut f.store, &f.cap, frame, limits(), 32, &mut f.index).unwrap();
    let revoked = outbox::Issuer::default();
    let mut auth = authority(&f.request, &f.settings, &revoked);
    let deny =
        |tx: &mut ActionTransaction<'_, core_store::Rehearsal>, auth: &Authority<'_>, now| {
            assert!(
                outbox::dispatch(
                    tx,
                    &f.request,
                    auth,
                    limits(),
                    || now,
                    |_| panic!("unauthorized dispatch")
                )
                .is_err()
            );
        };
    deny(&mut tx, &auth, Some(40));
    auth.issuer = &f.issuer;
    deny(&mut tx, &auth, Some(99));
    deny(&mut tx, &auth, None);
    auth.session = "other";
    deny(&mut tx, &auth, Some(40));
    auth.session = "executor";
    auth.revocation_version = 1;
    deny(&mut tx, &auth, Some(40));
    auth.revocation_version = 0;
    let mut different = auth.scope.clone();
    different.resource_id = "different".into();
    auth.scope = &different;
    deny(&mut tx, &auth, Some(40));
    auth.scope = &f.request.grant.scope;
    let unset = lifecycle::Settings {
        credential_lifetime_ns: None,
        ..Default::default()
    };
    auth.settings = &unset;
    deny(&mut tx, &auth, Some(40));
    let disabled = lifecycle::Settings {
        max_dispatching: Some(0),
        credential_lifetime_ns: Some(100),
        ..Default::default()
    };
    auth.settings = &disabled;
    deny(&mut tx, &auth, Some(40));
    assert_eq!(f.status().status, ActionStatus::AUTHORIZED);
}

#[test]
fn clock_expiring_after_consumption_leaves_no_side_effect() {
    let mut f = Fixture::new(false, false);
    f.registered();
    f.authorized();
    let frame = f.frame(EventKind::MAINTENANCE, &f.leaf.clone(), "consume");
    let mut tx =
        ActionTransaction::open(&mut f.store, &f.cap, frame, limits(), 32, &mut f.index).unwrap();
    let auth = authority(&f.request, &f.settings, &f.issuer);
    let mut times = [40, 99].into_iter();
    assert!(
        outbox::dispatch(
            &mut tx,
            &f.request,
            &auth,
            limits(),
            || times.next(),
            |_| panic!("expired at actual send")
        )
        .is_err()
    );
    assert_eq!(f.status().status, ActionStatus::DISPATCHING);
}

#[test]
fn desktop_three_tiers_and_approval_deadline_are_configurable() {
    let f = Fixture::new(false, false);
    let reader = f.store.reader().unwrap();
    let q: CognitionState = reader
        .read(&f.store.current().unwrap().unwrap().staged_q)
        .unwrap();
    let mut action = q.inflight[0].clone();
    action.status = ActionStatus::REQUESTED;
    let mut settings = lifecycle::Settings::default();
    assert_eq!(settings.irreversible, DesktopTier::IRREVERSIBLE_ASK);
    assert_eq!(settings.approval_timeout_ns, 60_000_000_000);
    assert_eq!(settings.credential_lifetime_ns, Some(600_000_000_000));
    assert_eq!(settings.max_dispatching, Some(4));
    settings.approval_timeout_ns = 7;
    let display = lifecycle::ask(
        &action,
        "selected".into(),
        ToolTier::T0,
        &f.request.action_payload.0,
        10,
        &settings,
        limits(),
    )
    .unwrap();
    assert_eq!(display.expires_at_ns.0, 17);
    // A user may tighten the lifetime below the fixture grant's 69 ns.
    settings.credential_lifetime_ns = Some(68);
    let mut issuer = outbox::Issuer::default();
    assert!(
        issuer
            .issue(
                f.request.grant.clone(),
                f.request.action_payload.0.clone(),
                &settings,
                limits()
            )
            .is_err()
    );
    settings.credential_lifetime_ns = Some(100);
    settings.max_dispatching = Some(1);
    for reversible in [true, false] {
        for tier in [
            DesktopTier::IRREVERSIBLE_NEVER,
            DesktopTier::IRREVERSIBLE_ASK,
            DesktopTier::IRREVERSIBLE_DIRECT,
        ] {
            let mut grant = f.request.grant.clone();
            grant.scope.operation_id = "close-game".into();
            grant.scope.reversible = Some(reversible);
            grant.scope.desktop.as_mut().unwrap().irreversible = tier;
            let mut issuer = outbox::Issuer::default();
            settings.irreversible = tier;
            let allowed = issuer
                .issue(
                    grant,
                    f.request.action_payload.0.clone(),
                    &settings,
                    limits(),
                )
                .is_ok();
            assert_eq!(
                allowed,
                reversible || tier == DesktopTier::IRREVERSIBLE_DIRECT
            );
        }
    }
}

#[test]
fn changed_scheduler_frontier_cannot_consume_the_authorized_action() {
    let mut f = Fixture::new(false, false);
    f.registered();
    f.authorized();
    let mut frame = f.frame(EventKind::MAINTENANCE, &f.leaf.clone(), "changed-frontier");
    frame
        .next_input
        .as_mut()
        .unwrap()
        .binding
        .input_frontier
        .push("new-input".into());
    let mut tx =
        ActionTransaction::open(&mut f.store, &f.cap, frame, limits(), 32, &mut f.index).unwrap();
    let auth = authority(&f.request, &f.settings, &f.issuer);
    assert!(
        outbox::dispatch(
            &mut tx,
            &f.request,
            &auth,
            limits(),
            || Some(40),
            |_| panic!("sent despite changed frontier")
        )
        .is_err()
    );
    assert_eq!(f.status().status, ActionStatus::AUTHORIZED);
}
