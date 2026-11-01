use super::*;

#[tokio::test]
async fn authorized_controls_reach_the_trusted_handler_and_only_then_ack() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "control-ok", deployment());
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let mut peer = Peer::connect(&name, &token).await;
        let mut a = peer.request("a-ok", IngressKind::INGRESS_APPROVAL);
        a.approval = Some(fixture());
        let mut c = peer.request("c-ok", IngressKind::INGRESS_CANCEL);
        c.cancel = Some(fixture());
        let mut g = peer.request("g-ok", IngressKind::INGRESS_GROWTH);
        g.growth = Some(fixture());
        for r in [a, c, g] {
            assert_eq!(
                peer.exchange(&r).await.kind,
                IngressResponseKind::CONTROL_ACK
            );
        }
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, h) = gateway.into_parts();
    assert_eq!(h.calls.len(), 3);
    assert!(s.next_raw().unwrap().is_none());
    assert!(s.ingress_rejections().unwrap().is_empty());
}

#[tokio::test]
async fn malformed_frame_and_audit_witness_failure_never_receive_success() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "bad-frame", deployment());
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let mut wire = open(&name).await;
        wire.write_all(&[255; 4]).await.unwrap();
        let mut byte = [0];
        assert!(match wire.read(&mut byte).await {
            Ok(n) => n == 0,
            Err(_) => true,
        });
        drop(wire);
        // The actor must consume the already-enqueued fault before this new
        // authentication. This is a causal barrier, not a timing assumption.
        let barrier = Peer::connect(&name, &token).await;
        drop(barrier);
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
    assert!(s.next_raw().unwrap().is_none());

    let (mut gateway, listener, name, token) = setup(&root, "audit-io", deployment());
    gateway.store.set_fault(core_store::Fault::DiskFull(
        core_store::FaultPoint::AuditSqlCommitted,
    ));
    let run = gateway.serve_listener(listener, std::future::pending());
    let client = async {
        let mut wire = Framed::new(open(&name).await, deployment().frames).unwrap();
        let mut wrong = token.clone();
        wrong.0[0] ^= 1;
        wire.write(&TerminalAuthenticate {
            schema_version: 1,
            request_id: "unknown-audit-outcome".into(),
            installation_id: "device".into(),
            token: wrong,
            protocol_versions: vec![1],
        })
        .await
        .unwrap();
        assert!(wire.read::<TerminalAuthResult>().await.is_err());
    };
    let (r, ()) = tokio::join!(run, client);
    assert!(matches!(r, Err(Error::Store(core_store::Error::Io(_)))));
    let (s, _) = gateway.into_parts();
    assert!(s.next_raw().unwrap().is_none());
    assert_eq!(s.ingress_rejections().unwrap().len(), 1);
}

#[tokio::test]
async fn auth_failure_connection_replay_and_audit_full_fail_closed() {
    let root = TemporaryRoot::new().unwrap();
    let mut cfg = deployment();
    cfg.audit.max_records = 3;
    let (mut gateway, listener, name, token) = setup(&root, "auth", cfg);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let mut first = Peer::connect(&name, &token).await;
        let mut second = Peer::connect(&name, &token).await;
        let mut stolen = second.request("stolen", IngressKind::INGRESS_QUERY);
        stolen.session_id = first.session.session_id.clone();
        stolen.query = Some(fixture());
        second.wire.write(&stolen).await.unwrap();
        assert!(second.wire.read::<TerminalResponse>().await.is_err());
        // A failed second pipe cannot revoke or impersonate the first pipe.
        let mut q = first.request("still-bound", IngressKind::INGRESS_QUERY);
        q.query = Some(fixture());
        assert_eq!(
            first.exchange(&q).await.kind,
            IngressResponseKind::QUERY_REPLY
        );
        for i in 0..2 {
            let mut wire = Framed::new(open(&name).await, deployment().frames).unwrap();
            let mut wrong = token.clone();
            wrong.0[0] ^= 1;
            wire.write(&TerminalAuthenticate {
                schema_version: 1,
                request_id: format!("bad-{i}"),
                installation_id: "device".into(),
                token: wrong,
                protocol_versions: vec![1],
            })
            .await
            .unwrap();
            let reply = wire.read::<TerminalAuthResult>().await.unwrap();
            assert_eq!(reply.status, AuthStatus::AUTH_REJECTED);
            assert_eq!(reply.reason, Some(IngressReason::AUTH_FAILED));
            assert!(reply.session.is_none());
        }
        let mut wire = Framed::new(open(&name).await, deployment().frames).unwrap();
        wire.write(&TerminalAuthenticate {
            schema_version: 1,
            request_id: "audit-full".into(),
            installation_id: "device".into(),
            token: token.clone(),
            protocol_versions: vec![1],
        })
        .await
        .unwrap();
        assert_eq!(
            wire.read::<TerminalAuthResult>().await.unwrap().reason,
            Some(IngressReason::AUDIT_FULL)
        );
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, _) = gateway.into_parts();
    assert!(s.next_raw().unwrap().is_none());
    assert_eq!(s.head().unwrap(), (None, 0, 0));
    let records = s.ingress_rejections().unwrap();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].reason, IngressReason::SESSION_EXPIRED);
    for record in records {
        let bytes = record.canonical_bytes(limits()).unwrap();
        assert!(!bytes.windows(32).any(|w| w == token.0.as_slice()));
        assert!(!String::from_utf8_lossy(&bytes).contains("bad-"));
    }
    s.verify().unwrap();
}

#[tokio::test]
async fn upload_reservations_binding_expiry_and_disconnect_cleanup() {
    let root = TemporaryRoot::new().unwrap();
    let mut cfg = deployment();
    cfg.budget.max_uploads = 1;
    cfg.budget.max_upload_bytes = 8192;
    cfg.upload_lifetime = Duration::from_millis(100);
    let (mut gateway, listener, name, token) = setup(&root, "upload", cfg);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let mut peer = Peer::connect(&name, &token).await;
        let value = UserText {
            schema_version: 1,
            text: "synthetic chunked text".repeat(3),
        };
        let bytes = value.canonical_bytes(limits()).unwrap();
        let reference = reference(&value);
        let mut r = peer.request("first", IngressKind::INGRESS_UPLOAD);
        r.upload = Some(ObjectUploadChunk {
            schema_version: 1,
            upload_id: "partial".into(),
            object: reference.clone(),
            offset: DecimalU64(0),
            data: HexBytes(bytes[..10].to_vec()),
            final_chunk: false,
        });
        assert_eq!(
            peer.exchange(&r).await.upload.unwrap().status,
            UploadStatus::UPLOAD_CONTINUE
        );
        let mut second = r.clone();
        second.request_id = "full".into();
        second.upload.as_mut().unwrap().upload_id = "another".into();
        assert_eq!(
            peer.exchange(&second).await.reason,
            Some(IngressReason::UPLOAD_BUDGET_FULL)
        );
        let mut changed = r.clone();
        changed.request_id = "changed".into();
        changed.upload.as_mut().unwrap().object.resource_version = "changed".into();
        assert_eq!(
            peer.exchange(&changed).await.reason,
            Some(IngressReason::OBJECT_MISMATCH)
        );
        // Expiry releases the FULL declared reservation, not just bytes received.
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(
            peer.exchange(&second).await.upload.unwrap().status,
            UploadStatus::UPLOAD_CONTINUE
        );
        let old_session = peer.session.session_id.clone();
        drop(peer);
        let mut peer = Peer::connect(&name, &token).await;
        assert_ne!(peer.session.session_id, old_session);
        let mut continuation = peer.request("old-offset", IngressKind::INGRESS_UPLOAD);
        continuation.upload = Some(ObjectUploadChunk {
            schema_version: 1,
            upload_id: "partial".into(),
            object: reference.clone(),
            offset: DecimalU64(10),
            data: HexBytes(bytes[10..].to_vec()),
            final_chunk: true,
        });
        assert_eq!(
            peer.exchange(&continuation).await.reason,
            Some(IngressReason::OBJECT_MISMATCH)
        );
        assert_eq!(peer.upload("fresh", &value).await, reference);
        // Even a structurally valid credential cannot be uploaded as an object.
        let auth = TerminalAuthenticate {
            schema_version: 1,
            request_id: "secret".into(),
            installation_id: "device".into(),
            token: token.clone(),
            protocol_versions: vec![1],
        };
        let mut forbidden = peer.request("forbidden", IngressKind::INGRESS_UPLOAD);
        let data = auth.canonical_bytes(limits()).unwrap();
        forbidden.upload = Some(ObjectUploadChunk {
            schema_version: 1,
            upload_id: "auth-object".into(),
            object: super::reference(&auth),
            offset: DecimalU64(0),
            data: HexBytes(data),
            final_chunk: true,
        });
        assert_eq!(
            peer.exchange(&forbidden).await.reason,
            Some(IngressReason::OBJECT_MISMATCH)
        );
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, _) = gateway.into_parts();
    assert!(s.next_raw().unwrap().is_none());
    assert!(s.reader().unwrap().facts(10).unwrap().is_empty());
    s.verify().unwrap();
}

#[tokio::test]
async fn pending_backpressure_is_typed_and_capacity_retry_keeps_source_key() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "capacity", deployment());
    gateway.config.max_pending_requests = 1;
    let enabled = Arc::clone(&gateway.handler.enabled);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let mut peer = Peer::connect(&name, &token).await;
        let p = text();
        let (e1, source) = event(&p, 0);
        let (e2, _) = event(&p, 1);
        peer.upload(
            "p",
            match &p {
                TerminalPayload::Text(v) => v,
                _ => unreachable!(),
            },
        )
        .await;
        peer.upload("s", &source).await;
        let r1 = peer.send_event("first", input(e1)).await;
        assert_eq!(peer.reply(&r1).await.raw_log_position, Some(DecimalU64(0)));
        let r2 = peer.send_event("pressure", input(e2.clone())).await;
        let failed = peer.reply(&r2).await;
        assert_eq!(failed.stage, ReceiptStage::REJECTED);
        assert_eq!(
            failed.reason_code,
            Some(IngressReason::PENDING_RECORDS_FULL)
        );
        assert!(failed.raw_log_position.is_none());
        enabled.store(true, Ordering::SeqCst);
        assert_eq!(peer.reply(&r1).await.stage, ReceiptStage::CORE_COMMITTED);
        let retry = peer.send_event("capacity-retry", input(e2)).await;
        let durable = peer.reply(&retry).await;
        assert_eq!(durable.raw_log_position, Some(DecimalU64(1)));
        assert_eq!(durable.stable_source_key, failed.stable_source_key);
        assert_eq!(peer.reply(&retry).await.stage, ReceiptStage::CORE_COMMITTED);
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, h) = gateway.into_parts();
    assert_eq!(h.commits.load(Ordering::SeqCst), 2);
    assert_eq!(s.head().unwrap().1, 2);
    assert_eq!(
        s.ingress_rejections().unwrap()[0].reason,
        IngressReason::PENDING_RECORDS_FULL
    );
    s.verify().unwrap();
}
