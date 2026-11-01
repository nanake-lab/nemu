use super::*;

#[tokio::test]
async fn t1_revocation_is_checked_on_the_same_authenticated_connection() {
    let root = TemporaryRoot::new().unwrap();
    let (mut gateway, listener, name, token) = setup(&root, "t1", deployment());
    let titles = Arc::clone(&gateway.handler.titles);
    titles.store(true, Ordering::SeqCst);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let run = gateway.serve_listener(listener, async {
        let _ = stopped.await;
    });
    let client = async {
        let mut peer = Peer::connect(&name, &token).await;
        let mut desktop: DesktopObservation = fixture();
        desktop.title = Some("synthetic title".into());
        let payload = TerminalPayload::Desktop(desktop.clone());
        let (first, source) = event(&payload, 0);
        let (second, _) = event(&payload, 1);
        peer.upload("desktop", &desktop).await;
        peer.upload("source", &source).await;
        let r1 = peer.send_event("allowed-title", input(first)).await;
        assert_eq!(peer.reply(&r1).await.stage, ReceiptStage::RECEIVED_DURABLE);
        titles.store(false, Ordering::SeqCst);
        let r2 = peer.send_event("revoked-title", input(second)).await;
        let reply = peer.wire.read::<TerminalResponse>().await.unwrap();
        core_types::terminal_ingress::check_response(&r2, &reply, &peer.session, limits()).unwrap();
        assert_eq!(reply.reason, Some(IngressReason::CAPABILITY_DENIED));
        assert_eq!(reply.kind, IngressResponseKind::INGRESS_REJECTED);
        let _ = stop.send(());
    };
    let (r, ()) = tokio::join!(run, client);
    r.unwrap();
    let (s, _) = gateway.into_parts();
    assert_eq!(s.next_raw().unwrap().unwrap().raw_log_position.0, 0);
    assert_eq!(
        s.ingress_rejections().unwrap()[0].reason,
        IngressReason::CAPABILITY_DENIED
    );
    assert_eq!(s.head().unwrap(), (None, 0, 0));
}

#[tokio::test]
async fn every_concurrent_pipe_instance_has_the_current_sid_dacl() {
    use std::os::windows::io::AsRawHandle;
    let suffix = format!("-owned-acl-{}", std::process::id());
    let sid = crate::windows::current_sid().unwrap();
    let name = format!(r"\\.\pipe\nemu-core-{sid}{suffix}");
    let mut listener =
        Listener::for_lineage(Lineage::REHEARSAL, deployment().frames, &suffix).unwrap();
    let mut keep = vec![];
    for _ in 0..4 {
        let client = open(&name).await;
        let server = listener.accept_owned().await.unwrap().into_inner();
        assert_eq!(
            crate::windows::dacl(server.as_raw_handle()).unwrap(),
            format!("D:P(A;;FA;;;{sid})")
        );
        assert_eq!(
            crate::windows::dacl(listener.raw_handle()).unwrap(),
            format!("D:P(A;;FA;;;{sid})")
        );
        assert!(Listener::for_lineage(Lineage::REHEARSAL, deployment().frames, &suffix).is_err());
        keep.push((client, server));
    }
    assert_eq!(keep.len(), 4);
}
