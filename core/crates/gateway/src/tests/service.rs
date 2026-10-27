//! Native auth/upload/dispatch/D/Gamma/commit integration with synthetic data.
mod acceptance;
mod adversarial;
mod crash;
mod permissions;
mod storage_failures;
use super::*;
use crate::{
    framing::{FrameConfig, Framed},
    gateway::{Gateway, GatewayConfig, Installation},
    hooks::{CoreHandler, Decision},
    pipe::Listener,
};
use core_store::{CommitCapability, Rehearsal, Store, TemporaryRoot};
use core_types::scalar::HexBytes;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

const CAPS: &[IngressKind] = &[
    IngressKind::INGRESS_EVENT,
    IngressKind::INGRESS_UPLOAD,
    IngressKind::INGRESS_QUERY,
    IngressKind::INGRESS_APPROVAL,
    IngressKind::INGRESS_CANCEL,
    IngressKind::INGRESS_GROWTH,
];
fn deployment() -> GatewayConfig {
    GatewayConfig {
        frames: FrameConfig {
            encoding: limits(),
            deadline: Duration::from_secs(10),
        },
        budget: TerminalBudget {
            schema_version: 1,
            version: "sys01-1".into(),
            max_frame_bytes: 16384,
            max_chunk_bytes: 2048,
            max_object_bytes: 8192,
            max_uploads: 2,
            max_upload_bytes: 32768,
        },
        audit: IngressAuditBudget {
            schema_version: 1,
            version: "sys01-audit-1".into(),
            max_records: 100,
            max_bytes: 65536,
        },
        max_connections: 4,
        max_pending_requests: 16,
        poll_interval: Duration::from_millis(5),
        upload_lifetime: Duration::from_secs(10),
        session_lifetime: Duration::from_secs(30),
    }
}
struct Core {
    cap: CommitCapability<Rehearsal>,
    enabled: Arc<AtomicBool>,
    commits: Arc<AtomicUsize>,
    reverse_pair: bool,
    held: Vec<CoreCommitResultV2>,
    calls: Vec<IngressKind>,
    controls: bool,
    titles: Arc<AtomicBool>,
    fabricate: bool,
}
impl CoreHandler<Rehearsal> for Core {
    fn authorize_event(
        &mut self,
        store: &Store<Rehearsal>,
        session: &TerminalSession,
        e: &TerminalEvent,
        _p: &SourceMetadata,
    ) -> Decision<Grants> {
        if session.installation_id != "device" || e.action_id.is_some() || e.trial_id.is_some() {
            return Err(IngressReason::CAPABILITY_DENIED);
        }
        let facts = store.reader().unwrap().facts(100).unwrap();
        if e.parents
            .iter()
            .chain(&e.correction_of)
            .any(|id| !facts.iter().any(|f| &f.id == id))
        {
            return Err(IngressReason::OBJECT_MISMATCH);
        }
        Ok(Grants {
            desktop_titles: self.titles.load(Ordering::SeqCst),
            codex_sessions: false,
        })
    }
    fn process_next(&mut self, store: &mut Store<Rehearsal>) -> Result<Vec<CoreCommitResultV2>> {
        if !self.enabled.load(Ordering::SeqCst) || store.next_raw()?.is_none() {
            return Ok(vec![]);
        }
        if self.fabricate {
            let e = store.next_raw()?.unwrap();
            let mut forged: CoreCommitResultV2 = fixture();
            forged.committed = true;
            forged.event_id = e.id;
            forged.snapshot = e.payload_ref;
            forged.commit_id = core_types::scalar::hex(&forged.snapshot.digest.0);
            return Ok(vec![forged]);
        }
        let result = super::runtime_fixture::commit_result(store, &self.cap);
        self.commits.fetch_add(1, Ordering::SeqCst);
        if self.reverse_pair {
            self.held.push(result);
            if self.held.len() < 2 {
                return Ok(vec![]);
            }
            self.reverse_pair = false;
            self.held.reverse();
            return Ok(std::mem::take(&mut self.held));
        }
        Ok(vec![result])
    }
    fn query(
        &mut self,
        _s: &mut Store<Rehearsal>,
        _c: &TerminalSession,
        q: &QueryV2,
    ) -> Result<Decision<QueryResultV2>> {
        self.calls.push(IngressKind::INGRESS_QUERY);
        Ok(Ok(QueryResultV2 {
            schema_version: 1,
            query_id: q.query_id.clone(),
            status: QueryStatus::NO_MATCH,
            items: vec![],
            growth_entries: vec![],
        }))
    }
    fn approval(
        &mut self,
        _s: &mut Store<Rehearsal>,
        _c: &TerminalSession,
        _: &ApprovalDecisionV2,
    ) -> Result<Decision<()>> {
        self.control(IngressKind::INGRESS_APPROVAL)
    }
    fn cancel(
        &mut self,
        _s: &mut Store<Rehearsal>,
        _c: &TerminalSession,
        _: &OutputCancel,
    ) -> Result<Decision<()>> {
        self.control(IngressKind::INGRESS_CANCEL)
    }
    fn growth(
        &mut self,
        _s: &mut Store<Rehearsal>,
        _c: &TerminalSession,
        _: &GrowthControl,
    ) -> Result<Decision<()>> {
        self.control(IngressKind::INGRESS_GROWTH)
    }
}
impl Core {
    fn control(&mut self, kind: IngressKind) -> Result<Decision<()>> {
        self.calls.push(kind);
        Ok(if self.controls {
            Ok(())
        } else {
            Err(IngressReason::CAPABILITY_DENIED)
        })
    }
}
fn setup(
    root: &TemporaryRoot,
    label: &str,
    cfg: GatewayConfig,
) -> (Gateway<Rehearsal, Core>, Listener, String, HexBytes) {
    setup_with_store_config(root, label, cfg, config())
}
fn setup_with_store_config(
    root: &TemporaryRoot,
    label: &str,
    cfg: GatewayConfig,
    store_config: core_store::StoreConfig,
) -> (Gateway<Rehearsal, Core>, Listener, String, HexBytes) {
    let mut store = Store::<Rehearsal>::create_temporary(
        root,
        label,
        header::<Rehearsal>(),
        super::runtime_fixture::BRANCH,
        store_config,
    )
    .unwrap();
    let cap = store.take_commit_capability().unwrap();
    let token_path = root.path().join(format!("{label}-token.dpapi"));
    let provisioned = TerminalToken::create_protected(&token_path).unwrap();
    let bytes = HexBytes(provisioned.test_bytes().to_vec());
    drop(provisioned);
    let token = TerminalToken::load_protected(&token_path).unwrap();
    let h = Core {
        cap,
        enabled: Arc::new(AtomicBool::new(false)),
        commits: Arc::new(AtomicUsize::new(0)),
        reverse_pair: false,
        held: vec![],
        calls: vec![],
        controls: true,
        titles: Arc::new(AtomicBool::new(false)),
        fabricate: false,
    };
    let suffix = format!("-service-{}-{label}", std::process::id());
    let name = format!(
        r"\\.\pipe\nemu-core-{}{suffix}",
        crate::windows::current_sid().unwrap()
    );
    let listener = Listener::for_lineage(Lineage::REHEARSAL, cfg.frames, &suffix).unwrap();
    let gateway = Gateway::new(
        store,
        h,
        vec![Installation::new("device".into(), token, CAPS.to_vec())],
        cfg,
    )
    .unwrap();
    (gateway, listener, name, bytes)
}
struct Peer {
    wire: Framed<NamedPipeClient>,
    session: TerminalSession,
}
async fn open(name: &str) -> NamedPipeClient {
    let start = std::time::Instant::now();
    loop {
        match ClientOptions::new().open(name) {
            Ok(v) => return v,
            Err(e)
                if [Some(2), Some(231)].contains(&e.raw_os_error())
                    && start.elapsed() < Duration::from_secs(10) =>
            {
                tokio::time::sleep(Duration::from_millis(10)).await
            }
            Err(e) => panic!("synthetic listener failed: {:?}", e.raw_os_error()),
        }
    }
}
impl Peer {
    async fn connect(name: &str, token: &HexBytes) -> Self {
        let mut wire = Framed::new(open(name).await, deployment().frames).unwrap();
        wire.write(&TerminalAuthenticate {
            schema_version: 1,
            request_id: "auth".into(),
            installation_id: "device".into(),
            token: token.clone(),
            protocol_versions: vec![1],
        })
        .await
        .unwrap();
        let reply = wire.read::<TerminalAuthResult>().await.unwrap();
        assert_eq!(reply.status, AuthStatus::AUTHENTICATED);
        Self {
            wire,
            session: reply.session.unwrap(),
        }
    }
    fn request(&self, id: &str, kind: IngressKind) -> TerminalRequest {
        TerminalRequest {
            schema_version: 1,
            request_id: id.into(),
            session_id: self.session.session_id.clone(),
            protocol_version: 1,
            kind,
            event: None,
            upload: None,
            query: None,
            approval: None,
            cancel: None,
            growth: None,
        }
    }
    async fn exchange(&mut self, request: &TerminalRequest) -> TerminalResponse {
        self.wire.write(request).await.unwrap();
        let reply = self.wire.read::<TerminalResponse>().await.unwrap();
        core_types::terminal_ingress::check_response(request, &reply, &self.session, limits())
            .unwrap();
        reply
    }
    async fn upload<T: WireMessage>(&mut self, id: &str, value: &T) -> ObjectRef {
        let bytes = value.canonical_bytes(limits()).unwrap();
        let reference = reference(value);
        let mut offset = 0;
        for data in bytes.chunks(128) {
            let mut r = self.request(&format!("{id}-{offset}"), IngressKind::INGRESS_UPLOAD);
            r.upload = Some(ObjectUploadChunk {
                schema_version: 1,
                upload_id: id.into(),
                object: reference.clone(),
                offset: DecimalU64(offset as u64),
                data: HexBytes(data.to_vec()),
                final_chunk: offset + data.len() == bytes.len(),
            });
            let reply = self.exchange(&r).await;
            assert_eq!(reply.kind, IngressResponseKind::UPLOAD_RECEIPT);
            let receipt = reply.upload.unwrap();
            offset += data.len();
            assert_eq!(receipt.next_offset.0 as usize, offset);
            assert_eq!(
                receipt.status == UploadStatus::UPLOAD_COMPLETE,
                offset == bytes.len()
            );
        }
        reference
    }
    async fn send_event(&mut self, request_id: &str, event: TerminalEvent) -> TerminalRequest {
        let mut r = self.request(request_id, IngressKind::INGRESS_EVENT);
        r.event = Some(event);
        self.wire.write(&r).await.unwrap();
        r
    }
    async fn reply(&mut self, request: &TerminalRequest) -> ReceiptV2 {
        let reply = self.wire.read::<TerminalResponse>().await.unwrap();
        core_types::terminal_ingress::check_response(request, &reply, &self.session, limits())
            .unwrap();
        reply.receipt.unwrap()
    }
}
fn reference<T: WireMessage>(value: &T) -> ObjectRef {
    let schema = core_types::contract::Schema::builtin().unwrap();
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(
            core_types::scalar::unhex(
                &schema
                    .digest(T::NAME, &serde_json::to_value(value).unwrap(), limits())
                    .unwrap(),
            )
            .unwrap(),
        ),
        object_type: schema.message(T::NAME).unwrap().tag.clone(),
        resource_version: "1".into(),
        byte_length: DecimalU64(value.canonical_bytes(limits()).unwrap().len() as u64),
    }
}
fn input(e: EventEnvelope) -> TerminalEvent {
    TerminalEvent {
        schema_version: 1,
        id: e.id,
        stable_source_key: e.stable_source_key,
        kind: e.kind,
        event_time_ns: e.event_time_ns,
        parents: e.parents,
        payload_ref: e.payload_ref,
        provenance: e.provenance,
        payload_digest: e.payload_digest,
        action_id: e.action_id,
        trial_id: e.trial_id,
        correction_of: e.correction_of,
        derivation: e.derivation,
    }
}
