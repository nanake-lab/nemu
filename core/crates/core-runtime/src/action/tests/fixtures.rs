use super::*;
use core_store::{Rehearsal, StoreConfig, TemporaryRoot};
use core_types::contract::{Limits, Schema};
use core_types::scalar::{DecimalI64, DecimalU64, HexBytes, unhex};
use std::time::Duration;

pub(super) fn limits() -> Limits {
    Limits {
        max_bytes: 4 << 20,
        max_items: 4096,
        max_depth: 64,
    }
}
pub(super) fn config() -> StoreConfig {
    StoreConfig {
        encoding: limits(),
        busy_timeout: Duration::from_secs(1),
        max_pending_records: 32,
        max_pending_bytes: 1 << 20,
        max_object_bytes: 4 << 20,
        max_store_bytes: 64 << 20,
        max_sql_pages: 8192,
    }
}
fn fixture<T: WireMessage>() -> T {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{}.hex", T::NAME));
    T::from_canonical(
        &unhex(std::fs::read_to_string(path).unwrap().trim()).unwrap(),
        limits(),
    )
    .unwrap()
}
fn with_refs<T: WireMessage>(r: &ObjectRef) -> T {
    let s = Schema::builtin().unwrap();
    let mut v = s
        .decode_canonical(
            T::NAME,
            &fixture::<T>().canonical_bytes(limits()).unwrap(),
            limits(),
            0,
        )
        .unwrap();
    let leaf = s
        .decode_canonical(
            ObjectRef::NAME,
            &r.canonical_bytes(limits()).unwrap(),
            limits(),
            0,
        )
        .unwrap();
    let mut todo = vec![&mut v];
    while let Some(v) = todo.pop() {
        if v.get("object_type").is_some() && v.get("digest").is_some() {
            *v = leaf.clone();
        } else if v.is_object() {
            todo.extend(v.as_object_mut().unwrap().values_mut());
        } else if v.is_array() {
            todo.extend(v.as_array_mut().unwrap());
        }
    }
    T::from_canonical(
        &s.encode_canonical(T::NAME, &v, limits()).unwrap(),
        limits(),
    )
    .unwrap()
}
#[derive(Default)]
pub(super) struct TestIndex {
    pub hint: Option<CognitiveInflightAction>,
    pub diagnostics: Vec<IndexIssue>,
}
impl OutboxIndex for TestIndex {
    fn lookup(&self, _: &str, _: &str) -> Option<CognitiveInflightAction> {
        self.hint.clone()
    }
    fn diagnose(&mut self, _: &str, _: &str, issue: IndexIssue) {
        self.diagnostics.push(issue);
    }
}
pub(super) struct Fixture {
    pub root: TemporaryRoot,
    pub store: Store<Rehearsal>,
    pub cap: CommitCapability<Rehearsal>,
    pub leaf: ObjectRef,
    pub payload: ObjectRef,
    pub request_ref: ObjectRef,
    pub index: TestIndex,
    pub issuer: outbox::Issuer,
    pub request: ActionRequestV2,
    pub settings: lifecycle::Settings,
}
impl Fixture {
    pub fn new(speech: bool, ask: bool) -> Self {
        Self::with_approval_deadline(speech, ask, 80)
    }
    pub fn with_approval_deadline(speech: bool, ask: bool, approval_deadline: i64) -> Self {
        let root = TemporaryRoot::new().unwrap();
        let header = LineageHeaderV2 {
            schema_version: 1,
            lineage: Lineage::REHEARSAL,
            created_at_ns: DecimalI64(0),
            genesis_manifest_digest: None,
            world_id: None,
            world_seed: None,
        };
        let mut store =
            Store::<Rehearsal>::create_temporary(&root, "action", header, "branch", config())
                .unwrap();
        let cap = store.take_commit_capability().unwrap();
        let leaf_pin = store
            .prepare(
                &UserText {
                    schema_version: 1,
                    text: "synthetic".into(),
                },
                "1",
            )
            .unwrap();
        let leaf = leaf_pin.reference().clone();
        let scope = ConnectorScope {
            schema_version: 1,
            kind: ScopeKind::LOCAL_DESKTOP,
            connector_id: "fixture".into(),
            operation_id: if speech { "speech" } else { "body" }.into(),
            resource_id: "stage".into(),
            desktop: Some(DesktopPermissionPolicy {
                schema_version: 1,
                version: "1".into(),
                irreversible: DesktopTier::IRREVERSIBLE_ASK,
                reversibility_rule: leaf.clone(),
            }),
            reversible: Some(!ask),
            wallet_minor_units: None,
            currency: None,
            depositor: None,
        };
        let scope_pin = store.prepare(&scope, "1").unwrap();
        let (payload_pin, bytes) = if speech {
            let chunks = ["一。", "二。"]
                .into_iter()
                .enumerate()
                .map(|(i, t)| OutputChunk {
                    schema_version: 1,
                    action_id: "a".into(),
                    output_id: "o".into(),
                    chunk_index: DecimalU64(i as u64),
                    text: t.into(),
                    lang: Lang::ZH,
                    snapshot_id: "snap".into(),
                    speech_mode: SpeechMode::A_LOCAL_STRICT,
                    strict: true,
                })
                .collect();
            let p = PlanAndCarrierReceipt {
                schema_version: 1,
                output_chunks: chunks,
                action_plan: None,
                speech_mode: SpeechMode::A_LOCAL_STRICT,
                strict: true,
                model_hash: leaf.digest.clone(),
                tokenizer_hash: leaf.digest.clone(),
                carrier_receipt: Some(leaf.clone()),
                projection_version: None,
                intermediate_results: vec![],
                usage: leaf.clone(),
            };
            (
                store.prepare(&p, "1").unwrap(),
                p.canonical_bytes(limits()).unwrap(),
            )
        } else {
            let p = BodyAction {
                schema_version: 1,
                kind: BodyActionKind::MOTION,
                motion_id: Some("wave".into()),
                anchor: None,
                platform_id: None,
                side: None,
                target: None,
                hold_ms: None,
            };
            (
                store.prepare(&p, "1").unwrap(),
                p.canonical_bytes(limits()).unwrap(),
            )
        };
        let payload = payload_pin.reference().clone();
        let plan = ActionPlan {
            schema_version: 1,
            plan_id: "p".into(),
            snapshot_id: "snap".into(),
            task_version: "q-0".into(),
            protocol_version: "1".into(),
            output_or_tool_parameters: payload.clone(),
            expected_scope: scope_pin.reference().clone(),
            dependencies: vec![leaf.clone()],
            deadline_ns: DecimalI64(1_000_000_000),
        };
        let budget = ExecutionBudget {
            schema_version: 1,
            budget_id: "fixture".into(),
            version: "1".into(),
            unit: "dispatch".into(),
            limit: DecimalU64(1),
            charge: DecimalU64(1),
        };
        let digest = HexBytes(action_gate::permissions::approval::payload_digest(&bytes).to_vec());
        let user = LocalUserBinding {
            schema_version: 1,
            principal_id: "u".into(),
            session_id: "local".into(),
            installation_id: "i".into(),
            authentication_record_id: "auth".into(),
        };
        let approval = action_gate::permissions::approval::build_request(
            "approval".into(),
            "a".into(),
            ToolTier::T0,
            &bytes,
            approval_deadline,
            limits(),
        )
        .unwrap();
        let approval_pin = store.prepare(&approval, "1").unwrap();
        let request_ref = if ask {
            approval_pin.reference().clone()
        } else {
            payload.clone()
        };
        let grant = ExecutionCredential {
            schema_version: 1,
            issuance_id: "single-use".into(),
            executor_session_id: "executor".into(),
            action_id: "a".into(),
            payload_digest: digest.clone(),
            protocol: leaf.clone(),
            permission_version: "1".into(),
            revocation_version: DecimalU64(0),
            scope,
            budget,
            issued_at_ns: DecimalI64(30),
            expires_at_ns: DecimalI64(99),
            tier: ToolTier::T0,
            consent: ask.then_some(ActionConsent {
                schema_version: 1,
                user,
                confirmed_at_ns: DecimalI64(25),
                approval_id: "approval".into(),
                action_id: "a".into(),
                payload_digest: digest,
            }),
            trust: None,
        };
        let mut issuer = outbox::Issuer::default();
        let request = issuer.issue(grant, bytes, limits()).unwrap();
        let environment: VersionedEnvironmentV2 = with_refs(&leaf);
        let binding = CognitionBinding {
            schema_version: 1,
            activity_id: "activity".into(),
            step_id: "step".into(),
            request_id: "generation".into(),
            round_index: DecimalU64(0),
            branch_id: "branch".into(),
            base_commit: store.base_commit().unwrap(),
            q_revision: "q-0".into(),
            input_frontier: vec!["initial".into()],
            snapshot_id: "snap".into(),
            environment,
            protocol: leaf.clone(),
            permission_snapshot: leaf.clone(),
            source_validity_version: "valid".into(),
            read_set: vec![leaf.clone()],
        };
        let current_input = CognitiveInput {
            schema_version: 1,
            binding: binding.clone(),
            materials: vec![],
            focus: Some(leaf.clone()),
            new_inputs: vec![],
            results: vec![],
            retrievals: vec![],
            budget: leaf.clone(),
        };
        let input = store.prepare(&current_input, "1").unwrap();
        let modulation = store
            .prepare(&with_refs::<ModulationEnvelopeV2>(&leaf), "1")
            .unwrap();
        let local = CognitiveResultAndReceipt {
            schema_version: 1,
            binding,
            input: input.reference().clone(),
            products: vec![],
            next_focus: None,
            queries: vec![],
            action_plan: Some(plan.clone()),
            completion: CognitiveCompletion::COG_COMPLETED,
            reason: None,
            model_hash: leaf.digest.clone(),
            tokenizer_hash: leaf.digest.clone(),
            modulation: modulation.reference().clone(),
            carrier_receipt: Some(leaf.clone()),
            usage: leaf.clone(),
        };
        let record = InternalActivityRecord {
            schema_version: 1,
            record_id: "step".into(),
            previous: None,
            started_at_ns: DecimalU64(0),
            finished_at_ns: DecimalU64(1),
            result: local,
        };
        let source = store.prepare(&record, "1").unwrap();
        let transition = store
            .prepare(
                &SystemMaintenanceRecord {
                    schema_version: 1,
                    record_id: "start".into(),
                    kind: CognitiveMaintenanceKind::COG_MAINTENANCE_RESUME,
                    reason: "fixture".into(),
                    details: leaf.clone(),
                    ledger_refs: vec![],
                    related_commit: store.base_commit().unwrap(),
                    recorded_at_ns: DecimalU64(0),
                },
                "1",
            )
            .unwrap();
        let q = CognitionState {
            schema_version: 1,
            revision: "q-0".into(),
            mode: CognitiveMode::COG_RUNNING,
            reason: None,
            activities: vec![ActivityState {
                schema_version: 1,
                activity_id: "activity".into(),
                revision: "q-0".into(),
                materials: vec![],
                focus: None,
                latest_step: Some(source.reference().clone()),
                waiting: vec![],
                successor: Some(input.reference().clone()),
                status: CognitiveActivityStatus::COG_ACTIVITY_RUNNING,
            }],
            inflight: vec![],
            active_request: None,
            runnable_successor: Some(input.reference().clone()),
            transition: transition.reference().clone(),
        };
        let q = stage_local_plan(
            &store,
            q,
            lifecycle::Origin::Local(&record),
            "a".into(),
            request_ref.clone(),
            1,
            limits(),
        )
        .unwrap();
        let mut theta: ThetaSnapshotV2 = fixture();
        let law = store
            .prepare(&with_refs::<LawVersion>(&leaf), &theta.parameters.law_id)
            .unwrap();
        let mut r: RSnapshotV3 = with_refs(&leaf);
        r.tendencies.clear();
        r.law_version = law.reference().clone();
        theta.law_version = law.reference().clone();
        let r = store.prepare_r(&r).unwrap();
        let theta = store.prepare(&theta, "1").unwrap();
        let mut x: FastStateSnapshotV2 = fixture();
        x.anchor_time_ns = DecimalU64(0);
        let x = store.prepare(&x, "1").unwrap();
        let e = event(&mut store, &leaf, EventKind::OBSERVATION, "initial");
        let h = store.prepare(&e, "1").unwrap();
        let mut b: ProposalBindingV2 = with_refs(&leaf);
        b.branch_id = "branch".into();
        b.base_commit = store.base_commit().unwrap();
        b.event_id = e.id.clone();
        b.raw_log_position = e.raw_log_position;
        b.anchor_time_ns = DecimalU64(0);
        b.process_time_ns = DecimalU64(0);
        b.read_set = vec![leaf.clone()];
        b.source_validity_version = "valid".into();
        let tx = CoreTransactionV2 {
            schema_version: 1,
            staged_h: h.reference().clone(),
            staged_x: x.reference().clone(),
            staged_r: r.reference().clone(),
            staged_theta: theta.reference().clone(),
            staged_l: leaf.clone(),
            staged_q: q.reference().clone(),
            staged_u: leaf.clone(),
            processed_position: DecimalU64(1),
            anchor_time_ns: DecimalU64(0),
            staged_concepts: vec![],
            accepted_results: vec![],
            audit_refs: vec![],
            binding: b.clone(),
            staged_ledger: vec![],
        };
        let staged = store.stage(store.bind(b).unwrap(), tx).unwrap();
        store
            .commit(&cap, staged, crate::status::Status::default().value())
            .unwrap();
        let settings = lifecycle::Settings {
            credential_lifetime_ns: Some(100),
            max_dispatching: Some(1),
            ..Default::default()
        };
        Self {
            root,
            store,
            cap,
            leaf,
            payload,
            request_ref,
            request,
            index: TestIndex::default(),
            issuer,
            settings,
        }
    }
    pub fn reopen(self) -> Self {
        let Self {
            root,
            store,
            cap,
            leaf,
            payload,
            request_ref,
            request,
            index,
            issuer,
            settings,
        } = self;
        drop((cap, store));
        let mut store = Store::open_temporary(&root, "action", config()).unwrap();
        let cap = store.take_commit_capability().unwrap();
        Self {
            root,
            store,
            cap,
            leaf,
            payload,
            request_ref,
            request,
            index,
            issuer,
            settings,
        }
    }
    pub fn frame(
        &mut self,
        kind: EventKind,
        payload: &ObjectRef,
        key: &str,
    ) -> EventCommit<Rehearsal> {
        let e = event(&mut self.store, payload, kind, key);
        self.frame_next(&e)
    }
    pub fn frame_next(&self, e: &EventEnvelope) -> EventCommit<Rehearsal> {
        let mut tx = self.store.current().unwrap().unwrap();
        let reader = self.store.reader().unwrap();
        let mut b = tx.binding.clone();
        b.event_id = e.id.clone();
        b.base_commit = self.store.base_commit().unwrap();
        b.raw_log_position = e.raw_log_position;
        b.x_revision = reader
            .read::<FastStateSnapshotV2>(&tx.staged_x)
            .unwrap()
            .revision;
        b.r_revision = reader.read::<RSnapshotV3>(&tx.staged_r).unwrap().revision;
        b.theta_revision = reader
            .read::<ThetaSnapshotV2>(&tx.staged_theta)
            .unwrap()
            .revision;
        b.read_set = vec![
            self.leaf.clone(),
            tx.staged_h.clone(),
            tx.staged_x.clone(),
            tx.staged_r.clone(),
            tx.staged_theta.clone(),
            tx.staged_l.clone(),
            tx.staged_q.clone(),
            tx.staged_u.clone(),
            e.payload_ref.clone(),
        ];
        b.read_set.sort_by(|a, b| a.digest.0.cmp(&b.digest.0));
        b.read_set.dedup();
        let h = self.store.prepare(e, "1").unwrap();
        tx.staged_h = h.reference().clone();
        tx.processed_position = DecimalU64(e.raw_log_position.0 + 1);
        tx.audit_refs.clear();
        tx.binding = b.clone();
        let q: CognitionState = reader.read(&tx.staged_q).unwrap();
        let mut next: CognitiveInput = reader.read(q.runnable_successor.as_ref().unwrap()).unwrap();
        next.binding.q_revision = format!("q-{}", e.id);
        next.binding.request_id = format!("next-{}", e.id);
        next.binding.base_commit = b.base_commit.clone();
        EventCommit {
            next_input: Some(next),
            ticket: self.store.bind(b).unwrap(),
            transaction: tx,
            status: crate::status::Status::default().value(),
        }
    }
    pub fn transaction(
        &mut self,
        frame: EventCommit<Rehearsal>,
    ) -> ActionTransaction<'_, Rehearsal> {
        ActionTransaction::open(
            &mut self.store,
            &self.cap,
            frame,
            limits(),
            32,
            &mut self.index,
        )
        .unwrap()
    }
    pub fn registered(&mut self) {
        let frame = self.frame(
            EventKind::ACTION_REQUEST,
            &self.request_ref.clone(),
            "register",
        );
        self.transaction(frame).register(20).unwrap();
    }
    pub fn authorized(&mut self) {
        let key = authorization_key(&self.request.grant, limits()).unwrap();
        let frame = self.frame(EventKind::MAINTENANCE, &self.leaf.clone(), &key);
        let auth = authority(&self.request, &self.settings, &self.issuer);
        ActionTransaction::open(
            &mut self.store,
            &self.cap,
            frame,
            limits(),
            32,
            &mut self.index,
        )
        .unwrap()
        .authorize(&self.request, &auth, 30)
        .unwrap();
    }
    pub fn status(&self) -> ActionLifecycle {
        let tx = self.store.current().unwrap().unwrap();
        let reader = self.store.reader().unwrap();
        reader.read(tx.audit_refs.last().unwrap()).unwrap()
    }
}
fn event(store: &mut Store<Rehearsal>, r: &ObjectRef, kind: EventKind, key: &str) -> EventEnvelope {
    let pos = store.head().unwrap().1;
    let mut e: EventEnvelope = with_refs(r);
    e.id = format!("event-{pos}");
    e.kind = kind;
    e.action_id = Some("a".into());
    e.stable_source_key = key.into();
    e.payload_digest = r.digest.clone();
    e.raw_log_position = DecimalU64(pos);
    store.ingest(&e).unwrap();
    e
}
pub(super) fn authority<'a>(
    request: &'a ActionRequestV2,
    settings: &'a lifecycle::Settings,
    issuer: &'a outbox::Issuer,
) -> Authority<'a> {
    let g = &request.grant;
    Authority {
        issuer,
        session: "executor",
        protocol: &g.protocol,
        permission: &g.protocol,
        revocation_version: 0,
        scope: &g.scope,
        budget: &g.budget,
        consent: g.consent.as_ref(),
        trust: None,
        tier: g.tier,
        settings,
    }
}
