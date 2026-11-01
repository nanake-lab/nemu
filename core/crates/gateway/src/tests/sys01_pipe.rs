//! SYS01 structural integration only: actual pipe + generated messages + W02.
//! Payload/provenance are preloaded synthetic fixtures; auth is established by
//! the test coordinator, NOT a new authentication/upload wire protocol.
use super::runtime_fixture::{BRANCH, commit_next};
use super::*;
use crate::{
    framing::{FrameConfig, Framed},
    pipe::Listener,
};
use core_store::{Rehearsal, Store, TemporaryRoot};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::windows::named_pipe::{ClientOptions, NamedPipeClient},
};

fn frames() -> FrameConfig {
    FrameConfig {
        encoding: limits(),
        deadline: Duration::from_secs(5),
    }
}
fn listener(label: &str) -> (Listener, String) {
    let suffix = format!("-w18-sys01-{}-{label}", std::process::id());
    let name = format!(
        r"\\.\pipe\nemu-core-{}{suffix}",
        crate::windows::current_sid().unwrap()
    );
    (
        Listener::for_lineage(Lineage::REHEARSAL, frames(), &suffix).unwrap(),
        name,
    )
}
async fn receive<S: AsyncRead + AsyncWrite + Unpin>(
    client: &mut Framed<NamedPipeClient>,
    server: &mut Framed<S>,
    store: &mut Store<Rehearsal>,
    outgoing: &EventEnvelope,
    payload: TerminalPayload,
    source: SourceMetadata,
) -> Receipt {
    client.write(outgoing).await.unwrap();
    let incoming = server
        .read::<EventEnvelope>()
        .await
        .unwrap_or_else(|error| {
            if let Error::Io(io) = error {
                panic!(
                    "synthetic position {} read failed: Win32 {:?}, kind {:?}",
                    outgoing.raw_log_position.0,
                    io.raw_os_error(),
                    io.kind()
                );
            }
            panic!("synthetic frame read failed: {error}");
        });
    assert_eq!(incoming, *outgoing);
    let validated = validate_event(
        &session(),
        Grants::default(),
        incoming,
        payload,
        source,
        limits(),
    )
    .unwrap();
    let receipt = ingest_validated(store, validated).unwrap();
    assert_eq!(receipt.stage, ReceiptStage::RECEIVED_DURABLE);
    assert_eq!(receipt.stable_source_key, outgoing.stable_source_key);
    assert_eq!(receipt.event_id.as_ref(), Some(&outgoing.id));
    server.write(&receipt).await.unwrap();
    let actual = client.read::<Receipt>().await.unwrap();
    assert_eq!(actual, receipt);
    actual
}

#[tokio::test]
async fn sys01_pipe_retransmission_conflict_reconnect_and_interleaved_receipts() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = Store::<Rehearsal>::create_temporary(
        &root,
        "sys01",
        header::<Rehearsal>(),
        BRANCH,
        config(),
    )
    .unwrap();
    let cap = store.take_commit_capability().unwrap();
    let (mut listener, name) = listener("dedup");
    let p = text();
    let (e1, s1) = event(&p, 0);
    let (e2, s2) = event(&p, 1);
    let mut observed = vec![];
    let (first, second) = {
        let client = ClientOptions::new().open(&name).unwrap();
        let mut client = Framed::new(client, frames()).unwrap();
        let mut server = listener.accept().await.unwrap();
        observed.push(receive(&mut client, &mut server, &mut store, &e1, p.clone(), s1).await);
        observed.push(receive(&mut client, &mut server, &mut store, &e2, p.clone(), s2).await);
        assert!(store.reader().unwrap().facts(10).unwrap().is_empty());
        let first = commit_next(&mut store, &cap);
        assert_eq!(first.event_id.as_ref(), Some(&e1.id));
        let second = commit_next(&mut store, &cap);
        assert_eq!(second.event_id.as_ref(), Some(&e2.id));
        // Drop before publishing CORE_COMMITTED receipts. Retry must recover
        // its outcome instead of applying a second authoritative transition.
        (first, second)
    };
    listener.disconnect().unwrap();
    let snapshot = store.current().unwrap().unwrap();
    let (server, client) = tokio::join!(listener.accept(), async {
        ClientOptions::new().open(&name)
    });
    {
        let mut server = server.unwrap();
        let mut client = Framed::new(client.unwrap(), frames()).unwrap();
        let (mut replay, replay_source) = event(&p, 2);
        replay.id = e2.id.clone();
        replay.stable_source_key = e2.stable_source_key.clone();
        replay.ingest_time_ns = DecimalI64(999);
        observed.push(
            receive(
                &mut client,
                &mut server,
                &mut store,
                &replay,
                p.clone(),
                replay_source,
            )
            .await,
        );
        let filtered = store.settle_duplicate().unwrap().unwrap();
        assert_eq!(filtered.stage, ReceiptStage::FILTERED);
        server.write(&filtered).await.unwrap();
        assert_eq!(client.read::<Receipt>().await.unwrap(), filtered);
        // Delayed receipts arrive across reconnect, in reverse event order.
        // Their generated event/key bindings, not the last submitted event,
        // determine which input is acknowledged.
        for delayed in [&second, &first] {
            server.write(delayed).await.unwrap();
            let actual = client.read::<Receipt>().await.unwrap();
            assert_eq!(&actual, delayed);
            observed.push(actual);
        }
        assert_eq!(observed[3].event_id.as_ref(), Some(&e2.id));
        assert_eq!(observed[4].event_id.as_ref(), Some(&e1.id));
        assert_eq!(store.current().unwrap().unwrap(), snapshot);
        assert_eq!(store.head().unwrap().2, 2);
        assert_eq!(store.head().unwrap().1, 3);
        assert_eq!(
            store.reader().unwrap().facts(10).unwrap(),
            vec![e1.clone(), e2.clone()]
        );
        let conflict = TerminalPayload::Text(UserText {
            schema_version: 1,
            text: "synthetic conflicting bytes".into(),
        });
        let (mut bad, source) = event(&conflict, 3);
        bad.id = e2.id.clone();
        bad.stable_source_key = e2.stable_source_key.clone();
        receive(&mut client, &mut server, &mut store, &bad, conflict, source).await;
        let rejected = store.settle_duplicate().unwrap().unwrap();
        assert_eq!(rejected.stage, ReceiptStage::REJECTED);
        assert_eq!(rejected.reason.as_deref(), Some("STABLE_KEY_CONFLICT"));
        server.write(&rejected).await.unwrap();
        assert_eq!(client.read::<Receipt>().await.unwrap(), rejected);
    }
    let reader = store.reader().unwrap();
    assert_eq!(reader.facts(10).unwrap(), vec![e1, e2]);
    assert_eq!(store.current().unwrap().unwrap(), snapshot);
    let (head, processed, anchor) = store.head().unwrap();
    assert_eq!((processed, anchor), (4, 2));
    assert_eq!(reader.ledger_at(&head.unwrap(), 10).unwrap().len(), 2);
    assert!(store.next_raw().unwrap().is_none());
    store.verify().unwrap();
    assert_eq!(
        observed.iter().map(|r| r.stage).collect::<Vec<_>>(),
        vec![
            ReceiptStage::RECEIVED_DURABLE,
            ReceiptStage::RECEIVED_DURABLE,
            ReceiptStage::RECEIVED_DURABLE,
            ReceiptStage::CORE_COMMITTED,
            ReceiptStage::CORE_COMMITTED
        ]
    );
    println!(
        "SYS01 structural pipe: D=e1,e2,e2,conflict; H=e1,e2; processed=4; anchor=2; ledger=2; delayed commits delivered e2 then e1 across reconnect and bound by event/key"
    );
}

#[tokio::test]
async fn sys01_pipe_disconnect_before_commit_reopens_durable_input() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = Store::<Rehearsal>::create_temporary(
        &root,
        "restart",
        header::<Rehearsal>(),
        BRANCH,
        config(),
    )
    .unwrap();
    let (mut listener, name) = listener("restart");
    let payload = text();
    let (e, source) = event(&payload, 0);
    {
        let client = ClientOptions::new().open(&name).unwrap();
        let mut client = Framed::new(client, frames()).unwrap();
        let mut server = listener.accept().await.unwrap();
        receive(&mut client, &mut server, &mut store, &e, payload, source).await;
    }
    listener.disconnect().unwrap();
    assert!(store.current().unwrap().is_none());
    drop(store);
    let mut store = Store::<Rehearsal>::open_temporary(&root, "restart", config()).unwrap();
    assert_eq!(store.next_raw().unwrap().unwrap(), e);
    assert_eq!(store.head().unwrap().1, 0);
    assert_eq!(store.head().unwrap().2, 0);
    let cap = store.take_commit_capability().unwrap();
    let receipt = commit_next(&mut store, &cap);
    let (server, client) = tokio::join!(listener.accept(), async {
        ClientOptions::new().open(&name)
    });
    let mut server = server.unwrap();
    let mut client = Framed::new(client.unwrap(), frames()).unwrap();
    server.write(&receipt).await.unwrap();
    assert_eq!(client.read::<Receipt>().await.unwrap(), receipt);
    assert_eq!(receipt.stage, ReceiptStage::CORE_COMMITTED);
    assert_eq!(store.reader().unwrap().facts(10).unwrap(), vec![e]);
    store.verify().unwrap();
    println!(
        "SYS01 structural pipe: RECEIVED_DURABLE survives uncommitted close/reopen; CORE_COMMITTED only after actual store.commit; not a physical power-cut test"
    );
}
