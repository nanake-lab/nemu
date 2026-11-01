//! Synthetic, TemporaryRoot-only authority fixture. No production open API.
use core_store::{CommitCapability, Experiment, PreparedObject, Store, StoreConfig, TemporaryRoot};
use core_types::{
    WireMessage,
    generated::*,
    scalar::{DecimalI64, DecimalU64},
};
use std::sync::OnceLock;

pub fn fixture<T: WireMessage>() -> T {
    static VALUES: OnceLock<serde_json::Value> = OnceLock::new();
    let values = VALUES.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../contracts/golden/legal.json")).unwrap()
    });
    let v = values
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["request"]["name"] == T::NAME)
        .unwrap();
    serde_json::from_value(v["request"]["value"].clone()).unwrap()
}
pub fn with_refs<T: WireMessage>(r: &ObjectRef) -> T {
    fn replace(v: &mut serde_json::Value, r: &serde_json::Value) {
        if v.get("object_type").is_some() && v.get("digest").is_some() {
            *v = r.clone();
        } else if let Some(m) = v.as_object_mut() {
            for v in m.values_mut() {
                replace(v, r);
            }
        } else if let Some(a) = v.as_array_mut() {
            for v in a {
                replace(v, r);
            }
        }
    }
    let mut value = serde_json::to_value(fixture::<T>()).unwrap();
    replace(&mut value, &serde_json::to_value(r).unwrap());
    serde_json::from_value(value).unwrap()
}
pub struct World {
    pub store: Store<Experiment>,
    cap: CommitCapability<Experiment>,
    pub root: TemporaryRoot,
    pub all_reactions: bool,
    pub reaction_target: Option<String>,
    pub event_time: Option<i64>,
}
impl World {
    pub fn new() -> Self {
        let root = TemporaryRoot::new().unwrap();
        let cfg = StoreConfig {
            encoding: crate::query::LIMITS,
            busy_timeout: std::time::Duration::from_millis(30),
            max_pending_records: 32,
            max_pending_bytes: 1024 * 1024,
            max_object_bytes: 4 * 1024 * 1024,
            max_store_bytes: 128 * 1024 * 1024,
            max_sql_pages: 32768,
        };
        let header = LineageHeaderV2 {
            schema_version: 1,
            lineage: Lineage::EXPERIMENT,
            created_at_ns: DecimalI64(1),
            genesis_manifest_digest: None,
            world_id: Some("w19-synthetic".into()),
            world_seed: Some(DecimalU64(42)),
        };
        let mut store =
            Store::<Experiment>::create_temporary(&root, "facts", header, "w19-synthetic", cfg)
                .unwrap();
        let cap = store.take_commit_capability().unwrap();
        Self {
            root,
            store,
            cap,
            all_reactions: false,
            reaction_target: None,
            event_time: None,
        }
    }
    pub fn text(&mut self, text: &str) -> EventEnvelope {
        self.add(
            &UserText {
                schema_version: 1,
                text: text.into(),
            },
            "user",
            &[],
            None,
        )
    }
    pub fn add<T: WireMessage>(
        &mut self,
        payload: &T,
        device: &str,
        corrections: &[String],
        entity: Option<&str>,
    ) -> EventEnvelope {
        let n = self.store.head().unwrap().1;
        let mut pins: Vec<PreparedObject<Experiment>> =
            vec![self.store.prepare(payload, "1").unwrap()];
        let leaf = self
            .store
            .prepare(
                &UserText {
                    schema_version: 1,
                    text: "fixture-resource".into(),
                },
                "1",
            )
            .unwrap();
        let mut e: EventEnvelope = with_refs(pins[0].reference());
        e.id = format!("event-{n}");
        e.stable_source_key = format!("key-{n}");
        e.device = device.into();
        e.kind = EventKind::OBSERVATION;
        e.raw_log_position = DecimalU64(n);
        e.event_time_ns = DecimalI64(self.event_time.unwrap_or(n as i64 + 1));
        e.ingest_time_ns = DecimalI64(n as i64 + 1);
        e.parents = vec![];
        e.correction_of = corrections.to_vec();
        e.payload_digest = pins[0].reference().digest.clone();
        self.store.ingest(&e).unwrap();
        let mut b: ProposalBindingV2 = with_refs(leaf.reference());
        b.branch_id = "w19-synthetic".into();
        b.event_id = e.id.clone();
        b.raw_log_position = e.raw_log_position;
        b.base_commit = self.store.base_commit().unwrap();
        b.anchor_time_ns = DecimalU64(self.store.head().unwrap().2);
        b.process_time_ns = DecimalU64(n + 1);
        b.read_set = vec![leaf.reference().clone()];
        if let Some(old) = self.store.current().unwrap() {
            // This fixture itself assigns these revisions below. Reopening
            // and fully recovering the growing store at every synthetic
            // insertion adds cubic setup work, unrelated to retrieval.
            b.x_revision = format!("x-{}", n - 1);
            b.r_revision = format!("r-{}", n - 1);
            b.theta_revision = format!("theta-{}", n - 1);
            b.read_set.extend([
                old.staged_h,
                old.staged_x,
                old.staged_r,
                old.staged_theta,
                old.staged_l,
                old.staged_q,
                old.staged_u,
            ]);
        }
        b.read_set
            .sort_by_key(|r| core_types::scalar::hex(&r.digest.0));
        b.read_set.dedup();
        let ticket = self.store.bind(b.clone()).unwrap();
        let mut theta: ThetaSnapshotV2 = fixture();
        let law: LawVersion = with_refs(leaf.reference());
        let law = self.store.prepare(&law, &theta.parameters.law_id).unwrap();
        let mut r: RSnapshotV3 = with_refs(leaf.reference());
        r.revision = format!("r-{n}");
        r.law_version = law.reference().clone();
        let r = self.store.prepare_r(&r).unwrap();
        theta.revision = format!("theta-{n}");
        theta.law_version = law.reference().clone();
        let theta = self.store.prepare(&theta, "1").unwrap();
        let mut x: FastStateSnapshotV2 = fixture();
        x.revision = format!("x-{n}");
        x.anchor_time_ns = DecimalU64(n + 1);
        let x = self.store.prepare(&x, "1").unwrap();
        let mut ledger: LedgerEntry = with_refs(leaf.reference());
        ledger.entry_id = format!("ledger-{n}");
        ledger.event_id = e.id.clone();
        let l = self.store.prepare(&ledger, "1").unwrap();
        let mut accepted = Vec::new();
        if let Some(entity) = entity {
            let mut node: Ref = with_refs(leaf.reference());
            node.id = format!("node-{n}");
            node.metadata.source_refs = vec![e.id.clone()];
            node.referent = Some(ConceptRef {
                schema_version: 1,
                concept_id: entity.into(),
                registry_version: "fixture-1".into(),
            });
            let graph = SemanticGraph {
                schema_version: 1,
                id: format!("graph-{n}"),
                refs: vec![node],
                claims: vec![],
                acts: vec![],
                edges: vec![],
            };
            let g = self.store.prepare(&graph, "1").unwrap();
            accepted.push(g.reference().clone());
            pins.push(g);
        }
        // Accepted interpretations are available only as marked task memories.
        let mut a: NatureAppraisalRecord = with_refs(leaf.reference());
        a.binding = b.clone();
        a.event_id = self.reaction_target.clone().unwrap_or_else(|| e.id.clone());
        let a = self.store.prepare(&a, "1").unwrap();
        accepted.push(a.reference().clone());
        let mut extra = Vec::new();
        if self.all_reactions {
            macro_rules! add {($($ty:ty),*) => {$(
                let mut record: $ty = with_refs(leaf.reference());
                record.event_id = self.reaction_target.clone().unwrap_or_else(|| e.id.clone());
                record.binding = b.clone();
                record.record_id = format!("{}-{n}", <$ty as WireMessage>::NAME);
                record.producer = "synthetic-reaction-only-marker".into();
                let prepared = self.store.prepare(&record, "1").unwrap();
                accepted.push(prepared.reference().clone());
                extra.push(prepared);
            )*};}
            add!(
                SituationMatchRecord,
                TendencyCandidateRecord,
                TeachingAnnotation,
                LieAnnotation,
                SameMatterAnnotation
            );
        }
        pins.extend(extra);
        accepted.sort_by_key(|r| core_types::scalar::hex(&r.digest.0));
        // Event reference is computed via its exact public encoding.
        let schema = core_types::contract::Schema::builtin().unwrap();
        let h = ObjectRef {
            schema_version: 1,
            digest: core_types::scalar::HexBytes(
                core_types::scalar::unhex(
                    &schema
                        .digest(
                            EventEnvelope::NAME,
                            &serde_json::to_value(&e).unwrap(),
                            crate::query::LIMITS,
                        )
                        .unwrap(),
                )
                .unwrap(),
            ),
            object_type: schema.message(EventEnvelope::NAME).unwrap().tag.clone(),
            resource_version: "1".into(),
            byte_length: DecimalU64(e.canonical_bytes(crate::query::LIMITS).unwrap().len() as u64),
        };
        let tx = CoreTransactionV2 {
            schema_version: 1,
            binding: b,
            staged_h: h,
            staged_x: x.reference().clone(),
            staged_r: r.reference().clone(),
            staged_theta: theta.reference().clone(),
            staged_l: l.reference().clone(),
            staged_q: leaf.reference().clone(),
            staged_u: leaf.reference().clone(),
            processed_position: DecimalU64(n + 1),
            anchor_time_ns: DecimalU64(n + 1),
            staged_concepts: vec![],
            accepted_results: accepted,
            audit_refs: vec![],
            staged_ledger: vec![ledger],
        };
        let staged = self.store.stage(ticket, tx).unwrap();
        self.store
            .commit(
                &self.cap,
                staged,
                DegradedStatus {
                    schema_version: 1,
                    states: vec![RunStatus::NORMAL],
                    reasons: vec![],
                },
            )
            .unwrap();
        e
    }
    pub fn prefix(&self) -> String {
        self.store.base_commit().unwrap()
    }
    #[cfg(feature = "cpu-embedding")]
    pub fn reopen(self) -> Self {
        let Self {
            root,
            store,
            cap,
            all_reactions,
            reaction_target,
            event_time,
        } = self;
        drop(cap);
        drop(store);
        let cfg = StoreConfig {
            encoding: crate::query::LIMITS,
            busy_timeout: std::time::Duration::from_millis(30),
            max_pending_records: 32,
            max_pending_bytes: 1024 * 1024,
            max_object_bytes: 4 * 1024 * 1024,
            max_store_bytes: 128 * 1024 * 1024,
            max_sql_pages: 32768,
        };
        let mut store = Store::<Experiment>::open_temporary(&root, "facts", cfg).unwrap();
        let cap = store.take_commit_capability().unwrap();
        Self {
            root,
            store,
            cap,
            all_reactions,
            reaction_target,
            event_time,
        }
    }
}
