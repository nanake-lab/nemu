use super::*;

#[tokio::test]
async fn sys01_queued_duplicate_backlog_never_reaches_a_second_core_step() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "gamma-backlog", deployment());
    let enabled = Arc::clone(&gateway.handler.enabled);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = async {
        let result = gateway
            .serve_listener(listener, async {
                let _ = stopped.await;
            })
            .await;
        assert!(
            result.is_ok(),
            "synthetic backlog service failed: {result:?}"
        );
        result
    };
    let client = async {
        let mut peer = Peer::connect(&name, &token).await;
        let p = text();
        let (e, source) = event(&p, 0);
        peer.upload(
            "p",
            match &p {
                TerminalPayload::Text(v) => v,
                _ => unreachable!(),
            },
        )
        .await;
        peer.upload("s", &source).await;
        let mut requests = vec![];
        for n in 0..4 {
            let r = peer
                .send_event(&format!("duplicate-{n}"), input(e.clone()))
                .await;
            assert_eq!(peer.reply(&r).await.raw_log_position, Some(DecimalU64(n)));
            requests.push(r);
        }
        enabled.store(true, Ordering::SeqCst);
        assert_eq!(
            peer.reply(&requests[0]).await.stage,
            ReceiptStage::CORE_COMMITTED
        );
        for r in &requests[1..] {
            assert_eq!(peer.reply(r).await.stage, ReceiptStage::FILTERED);
        }
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, h) = gateway.into_parts();
    assert_eq!(h.commits.load(Ordering::SeqCst), 1);
    assert_eq!(s.head().unwrap().1, 4);
    assert_eq!(s.head().unwrap().2, 1);
    assert_eq!(s.reader().unwrap().facts(10).unwrap().len(), 1);
    s.verify().unwrap();
}

#[tokio::test]
async fn sys01_authenticated_upload_dispatch_reconnect_gamma_and_reverse_receipts() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "sys01", deployment());
    gateway.handler_mut().reverse_pair = true;
    let enabled = Arc::clone(&gateway.handler.enabled);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let mut peer = Peer::connect(&name, &token).await;
        let old_session = peer.session.session_id.clone();
        let p = text();
        let (e1, source) = event(&p, 0);
        let (e2, _) = event(&p, 1);
        peer.upload(
            "text",
            match &p {
                TerminalPayload::Text(v) => v,
                _ => unreachable!(),
            },
        )
        .await;
        peer.upload("source", &source).await;
        let r1 = peer.send_event("r1", input(e1.clone())).await;
        let d1 = peer.reply(&r1).await;
        assert_eq!(d1.stage, ReceiptStage::RECEIVED_DURABLE);
        assert_eq!(d1.raw_log_position, Some(DecimalU64(0)));
        let r2 = peer.send_event("r2", input(e2.clone())).await;
        let d2 = peer.reply(&r2).await;
        assert_eq!(d2.stage, ReceiptStage::RECEIVED_DURABLE);
        assert_eq!(d2.raw_log_position, Some(DecimalU64(1)));
        // Another authenticated pipe can query while the first holds uncommitted D.
        let mut other = Peer::connect(&name, &token).await;
        assert_ne!(peer.session.session_id, other.session.session_id);
        let mut q = other.request("other-query", IngressKind::INGRESS_QUERY);
        let mut query: QueryV2 = fixture();
        query.query_id = "other-query".into();
        q.query = Some(query);
        assert_eq!(
            other.exchange(&q).await.query.unwrap().status,
            QueryStatus::NO_MATCH
        );
        drop(other);
        enabled.store(true, Ordering::SeqCst);
        // Actual store.commit results are delivered e2 BEFORE e1, with original
        // request IDs, despite another connection and multiple outstanding inputs.
        assert_eq!(peer.reply(&r2).await.stage, ReceiptStage::CORE_COMMITTED);
        assert_eq!(peer.reply(&r1).await.stage, ReceiptStage::CORE_COMMITTED);
        drop(peer);
        let mut peer = Peer::connect(&name, &token).await;
        assert_ne!(peer.session.session_id, old_session);
        let retry = peer.send_event("retry-e2", input(e2.clone())).await;
        assert_eq!(
            peer.reply(&retry).await.raw_log_position,
            Some(DecimalU64(2))
        );
        assert_eq!(peer.reply(&retry).await.stage, ReceiptStage::FILTERED);
        let changed = TerminalPayload::Text(UserText {
            schema_version: 1,
            text: "synthetic conflicting text".into(),
        });
        let (mut bad, bad_source) = event(&changed, 3);
        bad.id = e2.id.clone();
        bad.stable_source_key = e2.stable_source_key.clone();
        peer.upload(
            "conflict",
            match &changed {
                TerminalPayload::Text(v) => v,
                _ => unreachable!(),
            },
        )
        .await;
        peer.upload("conflict-source", &bad_source).await;
        let conflict = peer.send_event("conflict", input(bad)).await;
        assert_eq!(
            peer.reply(&conflict).await.raw_log_position,
            Some(DecimalU64(3))
        );
        let rejected = peer.reply(&conflict).await;
        assert_eq!(rejected.stage, ReceiptStage::REJECTED);
        assert_eq!(
            rejected.reason_code,
            Some(IngressReason::SOURCE_KEY_CONFLICT)
        );
        let _ = stop.send(());
        (e1, e2)
    };
    let (server, (e1, e2)) = tokio::join!(run, client);
    server.unwrap();
    let (store, core) = gateway.into_parts();
    assert_eq!(core.commits.load(Ordering::SeqCst), 2);
    assert_eq!((store.head().unwrap().1, store.head().unwrap().2), (4, 2));
    let facts = store.reader().unwrap().facts(10).unwrap();
    assert_eq!(
        facts.iter().map(|e| &e.id).collect::<Vec<_>>(),
        vec![&e1.id, &e2.id]
    );
    let current = store.current().unwrap().unwrap();
    assert_eq!(current.anchor_time_ns.0, 2);
    assert_eq!(
        store
            .reader()
            .unwrap()
            .ledger_at(&store.head().unwrap().0.unwrap(), 10)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        store
            .ingress_rejections()
            .unwrap()
            .iter()
            .filter(|r| r.reason == IngressReason::SOURCE_KEY_CONFLICT)
            .count(),
        1
    );
    store.verify().unwrap();
    println!(
        "SYS01 native: DPAPI/auth/upload/dispatch, concurrent pipes, reverse e2/e1 CORE receipts, D=e1,e2,e2,conflict; H=e1,e2; processed=4 anchor=2 ledger=2"
    );
}

#[tokio::test]
async fn control_dispatch_never_acknowledges_denial_and_does_not_enter_d() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "controls", deployment());
    gateway.handler_mut().controls = false;
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let mut peer = Peer::connect(&name, &token).await;
        let mut a = peer.request("approval", IngressKind::INGRESS_APPROVAL);
        a.approval = Some(fixture());
        let mut c = peer.request("cancel", IngressKind::INGRESS_CANCEL);
        c.cancel = Some(fixture());
        let mut g = peer.request("growth", IngressKind::INGRESS_GROWTH);
        g.growth = Some(fixture());
        for r in [a, c, g] {
            let reply = peer.exchange(&r).await;
            assert_eq!(reply.kind, IngressResponseKind::INGRESS_REJECTED);
            assert_eq!(reply.reason, Some(IngressReason::CAPABILITY_DENIED));
        }
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, h) = gateway.into_parts();
    assert_eq!(
        h.calls,
        vec![
            IngressKind::INGRESS_APPROVAL,
            IngressKind::INGRESS_CANCEL,
            IngressKind::INGRESS_GROWTH
        ]
    );
    assert!(s.next_raw().unwrap().is_none());
    assert_eq!(s.head().unwrap(), (None, 0, 0));
    assert_eq!(s.ingress_rejections().unwrap().len(), 3);
}
