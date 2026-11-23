use super::*;

#[tokio::test]
async fn untyped_object_store_rejection_never_becomes_a_false_protocol_mismatch() {
    let root = TemporaryRoot::new().unwrap();
    let mut cfg = config();
    cfg.max_object_bytes = 64;
    let (mut gateway, listener, name, token) =
        setup_with_store_config(&root, "object-capacity", deployment(), cfg);
    let run = gateway.serve_listener(listener, std::future::pending());
    let client = async {
        let mut peer = Peer::connect(&name, &token).await;
        let value = UserText {
            schema_version: 1,
            text: "x".repeat(100),
        };
        let mut r = peer.request("store-capacity-unknown", IngressKind::INGRESS_UPLOAD);
        r.upload = Some(ObjectUploadChunk {
            schema_version: 1,
            upload_id: "bounded".into(),
            object: reference(&value),
            offset: DecimalU64(0),
            data: HexBytes(value.canonical_bytes(limits()).unwrap()),
            final_chunk: true,
        });
        peer.wire.write(&r).await.unwrap();
        assert!(peer.wire.read::<TerminalResponse>().await.is_err());
    };
    let (r, ()) = tokio::join!(run, client);
    assert!(matches!(
        r,
        Err(Error::Store(core_store::Error::Rejected(_)))
    ));
    let (s, _) = gateway.into_parts();
    assert!(s.next_raw().unwrap().is_none());
    assert!(s.ingress_rejections().unwrap().is_empty());
    assert_eq!(s.head().unwrap(), (None, 0, 0));
}

#[tokio::test]
async fn actual_store_record_and_byte_backpressure_never_consume_a_d_position() {
    for byte_limit in [false, true] {
        let root = TemporaryRoot::new().unwrap();
        let mut store_cfg = config();
        let expected = if byte_limit {
            let (mut e, _) = event(&text(), 0);
            e.session = "0".repeat(64);
            e.ingest_time_ns = DecimalI64(
                i64::try_from(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos(),
                )
                .unwrap(),
            );
            store_cfg.max_pending_bytes = e.canonical_bytes(limits()).unwrap().len() as u64;
            IngressReason::PENDING_BYTES_FULL
        } else {
            store_cfg.max_pending_records = 1;
            IngressReason::PENDING_RECORDS_FULL
        };
        let (mut gateway, listener, name, token) = setup_with_store_config(
            &root,
            if byte_limit { "bytes" } else { "records" },
            deployment(),
            store_cfg,
        );
        let enabled = Arc::clone(&gateway.handler.enabled);
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let run = gateway.serve_listener(listener, async {
            let _ = stopped.await;
        });
        let client = async {
            let mut peer = Peer::connect(&name, &token).await;
            let p = text();
            let (e1, s) = event(&p, 0);
            let (e2, _) = event(&p, 1);
            peer.upload(
                "p",
                match &p {
                    TerminalPayload::Text(v) => v,
                    _ => unreachable!(),
                },
            )
            .await;
            peer.upload("s", &s).await;
            let a = peer.send_event("one", input(e1)).await;
            assert_eq!(peer.reply(&a).await.raw_log_position, Some(DecimalU64(0)));
            let b = peer.send_event("refused", input(e2.clone())).await;
            let no = peer.reply(&b).await;
            assert_eq!(no.stage, ReceiptStage::REJECTED);
            assert_eq!(no.reason_code, Some(expected));
            assert!(no.raw_log_position.is_none());
            enabled.store(true, Ordering::SeqCst);
            assert_eq!(peer.reply(&a).await.stage, ReceiptStage::CORE_COMMITTED);
            let retry = peer.send_event("same-key-retry", input(e2)).await;
            let yes = peer.reply(&retry).await;
            assert_eq!(yes.raw_log_position, Some(DecimalU64(1)));
            assert_eq!(yes.stable_source_key, no.stable_source_key);
            assert_eq!(peer.reply(&retry).await.stage, ReceiptStage::CORE_COMMITTED);
            let _ = stop.send(());
        };
        let (r, ()) = tokio::join!(run, client);
        r.unwrap();
        let (s, _) = gateway.into_parts();
        assert_eq!(s.head().unwrap().1, 2);
        assert_eq!(s.ingress_rejections().unwrap()[0].reason, expected);
        s.verify().unwrap();
    }
}

#[tokio::test]
async fn bad_object_digest_has_no_complete_receipt_and_orphan_pins_are_released() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "bad-digest", deployment());
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let mut peer = Peer::connect(&name, &token).await;
        let value = UserText {
            schema_version: 1,
            text: "synthetic digest fixture".into(),
        };
        let mut bytes = value.canonical_bytes(limits()).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        let mut r = peer.request("digest-mismatch", IngressKind::INGRESS_UPLOAD);
        r.upload = Some(ObjectUploadChunk {
            schema_version: 1,
            upload_id: "u".into(),
            object: reference(&value),
            offset: DecimalU64(0),
            data: HexBytes(bytes),
            final_chunk: true,
        });
        let reply = peer.exchange(&r).await;
        assert_eq!(reply.kind, IngressResponseKind::INGRESS_REJECTED);
        assert_eq!(reply.reason, Some(IngressReason::OBJECT_MISMATCH));
        assert!(reply.upload.is_none());
        peer.upload("u", &value).await;
        // A final-reply retry is idempotent while its completion is pinned.
        let bytes = value.canonical_bytes(limits()).unwrap();
        let mut replay = peer.request("complete-again", IngressKind::INGRESS_UPLOAD);
        replay.upload = Some(ObjectUploadChunk {
            schema_version: 1,
            upload_id: "u".into(),
            object: reference(&value),
            offset: DecimalU64(0),
            data: HexBytes(bytes),
            final_chunk: true,
        });
        assert_eq!(
            peer.exchange(&replay).await.upload.unwrap().status,
            UploadStatus::UPLOAD_COMPLETE
        );
        let other = UserText {
            schema_version: 1,
            text: "different declaration".into(),
        };
        replay.upload.as_mut().unwrap().object = reference(&other);
        replay.upload.as_mut().unwrap().data = HexBytes(other.canonical_bytes(limits()).unwrap());
        assert_eq!(
            peer.exchange(&replay).await.reason,
            Some(IngressReason::OBJECT_MISMATCH)
        );
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (mut s, _) = gateway.into_parts();
    assert!(s.next_raw().unwrap().is_none());
    assert_eq!(s.collect_garbage().unwrap(), 1);
    s.verify().unwrap();
    drop(s); // close SQLite/its byte-range writer lock before scanning files
    fn no_secret(path: &std::path::Path, token: &[u8], hex: &[u8]) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                no_secret(&entry.path(), token, hex)
            } else {
                let bytes = std::fs::read(entry.path()).unwrap();
                assert!(
                    !bytes.windows(token.len()).any(|v| v == token),
                    "credential in synthetic storage"
                );
                assert!(
                    !bytes.windows(hex.len()).any(|v| v == hex),
                    "credential hex in synthetic storage"
                );
            }
        }
    }
    no_secret(
        root.path(),
        &token.0,
        core_types::scalar::hex(&token.0).as_bytes(),
    );
}

#[tokio::test]
async fn a_fabricated_core_result_cannot_be_forwarded_as_committed() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "fake-core", deployment());
    gateway.handler_mut().fabricate = true;
    let enabled = Arc::clone(&gateway.handler.enabled);
    let run = gateway.serve_listener(listener, std::future::pending());
    let client = async {
        let mut peer = Peer::connect(&name, &token).await;
        let p = text();
        let (e, s) = event(&p, 0);
        peer.upload(
            "p",
            match &p {
                TerminalPayload::Text(v) => v,
                _ => unreachable!(),
            },
        )
        .await;
        peer.upload("s", &s).await;
        let r = peer.send_event("not-committed", input(e)).await;
        assert_eq!(peer.reply(&r).await.stage, ReceiptStage::RECEIVED_DURABLE);
        enabled.store(true, Ordering::SeqCst);
        assert!(peer.wire.read::<TerminalResponse>().await.is_err());
    };
    let (r, ()) = tokio::join!(run, client);
    assert!(r.is_err());
    let (s, _) = gateway.into_parts();
    assert_eq!(s.head().unwrap(), (None, 0, 0));
    assert!(s.next_raw().unwrap().is_some());
    assert!(s.reader().unwrap().facts(10).unwrap().is_empty());
}
