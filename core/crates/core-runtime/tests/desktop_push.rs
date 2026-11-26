//! Real desktop receive chain over the Windows gateway. Only stage inputs and the
//! output producer are synthetic. The producer reads back the committed event.
#![cfg(windows)]
#[path = "../../nemu-core/src/authority.rs"]
mod authority;
mod common;
#[path = "../../nemu-core/src/deployment.rs"]
mod deployment;
#[path = "handler/support.rs"]
mod support;
use common::limits;
use core_runtime::{handler::EventInputs, handler::RehearsalHandler};
use core_store::{Reader, Rehearsal, Store, TemporaryRoot};
use core_types::{
    generated::*,
    scalar::{DecimalI64, DecimalU64, HexBytes, hex},
};
use gateway::{
    Grants, TerminalToken,
    gateway::{Gateway, Installation, TerminalOutput},
    hooks::{CoreHandler, Decision},
};
use sha2::{Digest, Sha256};
use std::{path::Path, process::Command, time::Duration};
use tokio::sync::{mpsc, oneshot};

// The real local authentication record still authorizes each text event.
// This test producer owns only the output it created on the originating pipe.
struct Producer {
    local: authority::LocalAuthority,
    cancelled: Option<oneshot::Sender<()>>,
}
impl CoreHandler<Rehearsal> for Producer {
    fn authenticated(
        &mut self,
        store: &mut Store<Rehearsal>,
        s: &TerminalSession,
    ) -> gateway::Result<Decision<()>> {
        self.local.authenticated(store, s)
    }
    fn authorize_event(
        &mut self,
        store: &Store<Rehearsal>,
        s: &TerminalSession,
        e: &TerminalEvent,
        p: &SourceMetadata,
    ) -> Decision<Grants> {
        self.local.authorize_event(store, s, e, p)
    }
    fn cancel(
        &mut self,
        store: &mut Store<Rehearsal>,
        s: &TerminalSession,
        c: &OutputCancel,
    ) -> gateway::Result<Decision<()>> {
        let reader = store.verified_reader()?;
        let Some(head) = reader.current_snapshot()? else {
            return Ok(Err(IngressReason::CAPABILITY_DENIED));
        };
        let (_, event) = reader.read_published_transaction(&head)?;
        if event.session == s.session_id
            && c.output_id == "test-prefix"
            && self
                .cancelled
                .take()
                .is_some_and(|sent| sent.send(()).is_ok())
        {
            Ok(Ok(()))
        } else {
            Ok(Err(IngressReason::CAPABILITY_DENIED))
        }
    }
}

#[tokio::test]
async fn authenticated_gateway_delivers_to_the_desktop_bridge() {
    let root = TemporaryRoot::new().unwrap();
    let (store, cap) = support::initial(&root);
    let path = root.path().join("token.dpapi");
    let token = TerminalToken::create_protected(&path).unwrap();
    let name = gateway::pipe::current_pipe_name().unwrap();
    let sid = name.strip_prefix(r"\\.\pipe\nemu-core-").unwrap();
    let local =
        authority::LocalAuthority::open(&root.path().join("auth.sqlite"), sid, limits()).unwrap();
    let (signal, cancelled) = oneshot::channel();
    let (admitted, mut events) = mpsc::channel(2);
    let provider = move |store: &Store<Rehearsal>,
                         event: &EventEnvelope|
          -> core_runtime::Result<EventInputs> {
        let input = support::inputs(store, event)?;
        admitted.try_send(event.id.clone()).unwrap();
        Ok(input)
    };
    let handler = RehearsalHandler::new(
        cap,
        Producer {
            local,
            cancelled: Some(signal),
        },
        provider,
        limits(),
        SpeechMode::B_REMOTE_PROJECTED,
    );
    use IngressKind::*;
    let install = Installation::new(
        "desktop".into(),
        token,
        vec![
            INGRESS_EVENT,
            INGRESS_UPLOAD,
            INGRESS_CANCEL,
            INGRESS_APPROVAL,
        ],
    );
    let mut gateway =
        Gateway::new(store, handler, vec![install], deployment::settings().1).unwrap();
    let out = gateway.output_sender();
    let (ready, start) = oneshot::channel();
    let (done, stop) = oneshot::channel();
    let serving = gateway.serve(async move {
        let _ = ready.send(());
        let _ = stop.await;
    });
    let client = async {
        start.await.expect("test owns current SID pipe");
        let result = tokio::task::spawn_blocking(move || {
            let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(3)
                .unwrap();
            Command::new("node")
                .current_dir(repo)
                .args([
                    "--import",
                    "tsx",
                    "core/crates/core-runtime/tests/handler/desktop.mts",
                ])
                .arg(path)
                .status()
                .unwrap()
        })
        .await
        .unwrap();
        let _ = done.send(());
        assert!(result.success());
    };
    let producing = async {
        let mut cancelled = Some(cancelled);
        for turn in 0..2 {
            let id = events.recv().await.unwrap();
            let reader =
                Reader::<Rehearsal>::open_temporary(&root, "handler", support::config()).unwrap();
            let head = reader.current_snapshot().unwrap().unwrap();
            let (_, e) = reader.read_published_transaction(&head).unwrap();
            assert_eq!(e.id, id);
            let session = e.session;
            let output_id = String::from(if turn == 0 {
                "test-prefix"
            } else {
                "test-failure"
            });
            let send = |message| out.send((session.clone(), message));
            if turn == 0 {
                let mut chunk = OutputChunk {
                    schema_version: 1,
                    output_id: output_id.clone(),
                    action_id: "test-action".into(),
                    chunk_index: DecimalU64(0),
                    text: String::new(),
                    lang: Lang::EN,
                    snapshot_id: hex(&head.digest.0),
                    speech_mode: SpeechMode::B_REMOTE_PROJECTED,
                    strict: false,
                };
                for (index, text) in ["one", "two", "three", "four"].into_iter().enumerate() {
                    chunk.chunk_index = DecimalU64(index as u64);
                    chunk.text = text.into();
                    send(TerminalOutput::Chunk(chunk.clone())).await.unwrap();
                }
                cancelled.take().unwrap().await.unwrap();
                // A chunk already in flight after cancel cannot extend the visible prefix.
                chunk.chunk_index = DecimalU64(4);
                chunk.text = "late".into();
                send(TerminalOutput::Chunk(chunk)).await.unwrap();
                send(TerminalOutput::End(OutputEnd {
                    schema_version: 1,
                    output_id,
                    reason: OutputEndReason::CANCELLED,
                }))
                .await
                .unwrap();
            } else {
                send(TerminalOutput::End(OutputEnd {
                    schema_version: 1,
                    output_id,
                    reason: OutputEndReason::ERROR,
                }))
                .await
                .unwrap();
                let payload = b"synthetic approval";
                send(TerminalOutput::Approval(ApprovalRequestV2 {
                    schema_version: 1,
                    approval_id: "test-approval".into(),
                    action_id: "test-action-2".into(),
                    tier: ToolTier::T2,
                    title: "Synthetic".into(),
                    detail: "Synthetic".into(),
                    expires_at_ns: DecimalI64(i64::MAX),
                    action_payload: HexBytes(payload.to_vec()),
                    payload_digest: HexBytes(Sha256::digest(payload).to_vec()),
                }))
                .await
                .unwrap();
            }
        }
    };
    // Bound a broken producer/peer handshake so a failed test always releases the pipe.
    let (served, (), ()) = tokio::time::timeout(Duration::from_secs(45), async {
        tokio::join!(serving, client, producing)
    })
    .await
    .unwrap();
    served.unwrap();
}
