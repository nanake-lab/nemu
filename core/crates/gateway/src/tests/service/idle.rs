use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn idle_deployment() -> GatewayConfig {
    let mut cfg = deployment();
    cfg.frames.deadline = Duration::from_secs(2);
    cfg.audit.max_records = 2; // the review's small budget; never enlarge it
    cfg
}
async fn closed(wire: &mut NamedPipeClient) {
    let mut byte = [0];
    let result = tokio::time::timeout(Duration::from_secs(8), wire.read(&mut byte))
        .await
        .expect("connection did not close within its bound");
    assert!(matches!(result, Ok(0) | Err(_)));
}
fn query(peer: &Peer, id: &str) -> TerminalRequest {
    let mut r = peer.request(id, IngressKind::INGRESS_QUERY);
    let mut q: QueryV2 = fixture();
    q.query_id = id.into();
    r.query = Some(q);
    r
}

#[tokio::test]
async fn authenticated_zero_request_idle_reconnects_do_not_consume_audit() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "idle-reconnect", idle_deployment());
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        // More legitimate cycles than the entire audit quota must remain usable.
        for cycle in 1..=3 {
            let peer = Peer::connect(&name, &token).await;
            let wire = peer.wire.into_inner();
            tokio::time::sleep(Duration::from_millis(2200)).await;
            let mut byte = [0];
            let result = wire.try_read(&mut byte);
            assert!(
                matches!(result, Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock),
                "zero-request authenticated idle was disconnected: {result:?}"
            );
            drop(wire);
            println!(
                "authenticated_idle_cycle={cycle} idle_ms=2200 request_bytes_sent=0 connection_still_open=true"
            );
        }
        let mut peer = Peer::connect(&name, &token).await;
        let request = query(&peer, "still-authorized");
        assert_eq!(
            peer.exchange(&request).await.query.unwrap().status,
            QueryStatus::NO_MATCH
        );
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (store, _) = gateway.into_parts();
    assert!(store.ingress_rejections().unwrap().is_empty());
    assert!(store.next_raw().unwrap().is_none());
    assert_eq!(store.head().unwrap(), (None, 0, 0));
    store.verify().unwrap();
    println!(
        "idle_reconnects: persistent_rejection_records=0 fourth_valid_auth=AUTHENTICATED query=NO_MATCH D_empty=true H_anchor_unchanged=true"
    );
}

#[tokio::test]
async fn waiting_for_delayed_commit_and_next_request_is_legal_idle() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "idle-receipt", idle_deployment());
    let enabled = Arc::clone(&gateway.handler.enabled);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
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
        let r = peer.send_event("delayed", input(e)).await;
        assert_eq!(peer.reply(&r).await.stage, ReceiptStage::RECEIVED_DURABLE);
        tokio::time::sleep(Duration::from_millis(2200)).await;
        enabled.store(true, Ordering::SeqCst);
        assert_eq!(peer.reply(&r).await.stage, ReceiptStage::CORE_COMMITTED);
        tokio::time::sleep(Duration::from_millis(2200)).await;
        let q = query(&peer, "after-delayed-receipt");
        assert_eq!(
            peer.exchange(&q).await.query.unwrap().status,
            QueryStatus::NO_MATCH
        );
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, h) = gateway.into_parts();
    assert_eq!(h.commits.load(Ordering::SeqCst), 1);
    assert!(s.ingress_rejections().unwrap().is_empty());
    assert_eq!((s.head().unwrap().1, s.head().unwrap().2), (1, 1));
    s.verify().unwrap();
    println!(
        "delayed_receipt: each_idle_ms=2200 frame_deadline_ms=2000 CORE_COMMITTED_then_same_connection_query=true audit=0 commits=1"
    );
}

#[tokio::test]
async fn authenticated_partial_header_and_body_timeout_are_still_audited() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "partial-frame", idle_deployment());
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        for bytes in [vec![3, 0], vec![20, 0, 0, 0, 8, 1]] {
            let peer = Peer::connect(&name, &token).await;
            let mut wire = peer.wire.into_inner();
            wire.write_all(&bytes).await.unwrap();
            closed(&mut wire).await;
        }
        // Real malformed traffic can still exhaust the SAME small audit quota.
        let mut wire = Framed::new(open(&name).await, deployment().frames).unwrap();
        wire.write(&TerminalAuthenticate {
            schema_version: 1,
            request_id: "audit-bound".into(),
            installation_id: "device".into(),
            token: token.clone(),
            protocol_versions: vec![1],
        })
        .await
        .unwrap();
        let denied = wire.read::<TerminalAuthResult>().await.unwrap();
        assert_eq!(denied.status, AuthStatus::AUTH_REJECTED);
        assert_eq!(denied.reason, Some(IngressReason::AUDIT_FULL));
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, _) = gateway.into_parts();
    let audit = s.ingress_rejections().unwrap();
    assert_eq!(audit.len(), 2);
    assert!(
        audit
            .iter()
            .all(|r| r.reason == IngressReason::INVALID_FRAME)
    );
    assert!(s.next_raw().unwrap().is_none());
    assert_eq!(s.head().unwrap(), (None, 0, 0));
    println!(
        "partial_header_body: timed_out=true INVALID_FRAME=2 audit_max_records=2 subsequent_auth=AUDIT_FULL D_empty=true"
    );
}

#[tokio::test]
async fn session_expiry_closes_idle_connection_without_rejection_audit() {
    let root = TemporaryRoot::new().unwrap();
    let mut cfg = idle_deployment();
    cfg.frames.deadline = Duration::from_millis(250);
    cfg.session_lifetime = Duration::from_millis(900);
    let (mut gateway, listener, name, token) = setup(&root, "idle-expiry", cfg);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let peer = Peer::connect(&name, &token).await;
        let mut wire = peer.wire.into_inner();
        let mut byte = [0];
        assert!(
            tokio::time::timeout(Duration::from_millis(500), wire.read(&mut byte))
                .await
                .is_err()
        );
        closed(&mut wire).await;
        let _fresh = Peer::connect(&name, &token).await;
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, _) = gateway.into_parts();
    assert!(s.ingress_rejections().unwrap().is_empty());
    assert_eq!(s.head().unwrap(), (None, 0, 0));
    println!(
        "session_expiry: idle_survives_frame_deadline=true session_lifetime_ms=900 closed=true audit=0 reauthentication=true"
    );
}

#[tokio::test]
async fn unauthenticated_zero_byte_handshake_remains_time_bounded() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "auth-deadline", idle_deployment());
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let mut wire = open(&name).await;
        closed(&mut wire).await;
        let _barrier = Peer::connect(&name, &token).await;
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, _) = gateway.into_parts();
    assert_eq!(s.ingress_rejections().unwrap().len(), 1);
    assert_eq!(
        s.ingress_rejections().unwrap()[0].reason,
        IngressReason::INVALID_FRAME
    );
    assert_eq!(s.head().unwrap(), (None, 0, 0));
    println!(
        "preauthentication_zero_bytes: deadline_ms=2000 timed_out=true invalid_auth_frame_audit=1 D_empty=true"
    );
}
