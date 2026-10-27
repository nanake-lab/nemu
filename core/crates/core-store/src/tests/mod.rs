use super::*;
use core_types::{
    WireMessage,
    generated::*,
    scalar::{DecimalI64, DecimalU64, HexBytes},
};
use std::{sync::OnceLock, time::Duration};
mod atomic;
mod crash;
mod crash_harness;
mod isolation;
mod legacy_v2;
mod objects;
mod reader;
mod retention_versions;
fn normal() -> DegradedStatus {
    DegradedStatus {
        schema_version: 1,
        states: vec![RunStatus::NORMAL],
        reasons: vec![],
    }
}

fn config() -> StoreConfig {
    StoreConfig {
        encoding: core_types::contract::Limits {
            max_bytes: 4 * 1024 * 1024,
            max_items: 4096,
            max_depth: 64,
        },
        busy_timeout: Duration::from_millis(30),
        max_pending_records: 32,
        max_pending_bytes: 1024 * 1024,
        max_object_bytes: 4 * 1024 * 1024,
        max_store_bytes: 64 * 1024 * 1024,
        max_sql_pages: 8192,
    }
}
fn fixture<T: WireMessage>() -> T {
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
// Structural fixtures intentionally use a small synthetic versioned resource.
// They never claim valid appraisal/consolidation policy or access real stores.
fn with_refs<T: WireMessage>(r: &ObjectRef) -> T {
    fn replace(value: &mut serde_json::Value, r: &serde_json::Value) {
        if value.get("object_type").is_some() && value.get("digest").is_some() {
            *value = r.clone();
        } else if let Some(m) = value.as_object_mut() {
            for v in m.values_mut() {
                replace(v, r);
            }
        } else if let Some(a) = value.as_array_mut() {
            for v in a {
                replace(v, r);
            }
        }
    }
    let mut value = serde_json::to_value(fixture::<T>()).unwrap();
    replace(&mut value, &serde_json::to_value(r).unwrap());
    serde_json::from_value(value).unwrap()
}
fn header<L: LineageKind>() -> LineageHeaderV2 {
    LineageHeaderV2 {
        schema_version: 1,
        lineage: L::KIND,
        created_at_ns: DecimalI64(1),
        genesis_manifest_digest: (L::KIND == Lineage::LIFE).then(|| HexBytes(vec![1; 32])),
        world_id: (L::KIND == Lineage::EXPERIMENT).then(|| "synthetic-world".into()),
        world_seed: (L::KIND == Lineage::EXPERIMENT).then_some(DecimalU64(42)),
    }
}
fn rehearsal(root: &TemporaryRoot, name: &str) -> Store<Rehearsal> {
    Store::<Rehearsal>::create_temporary(root, name, header::<Rehearsal>(), "test-branch", config())
        .unwrap()
}
fn unpersisted_event<L: LineageKind>(
    store: &mut Store<L>,
    n: u64,
) -> (EventEnvelope, PreparedObject<L>) {
    let payload = store
        .prepare(
            &UserText {
                schema_version: 1,
                text: format!("synthetic-{n}"),
            },
            "1",
        )
        .unwrap();
    let mut event: EventEnvelope = with_refs(payload.reference());
    event.id = format!("event-{n}");
    event.stable_source_key = format!("key-{n}");
    event.raw_log_position = DecimalU64(n);
    event.payload_digest = payload.reference().digest.clone();
    (event, payload)
}
fn event<L: LineageKind>(store: &mut Store<L>, n: u64) -> EventEnvelope {
    let (event, _payload) = unpersisted_event(store, n);
    store.ingest(&event).unwrap();
    event
}
fn proposal<L: LineageKind>(
    store: &Store<L>,
    event: &EventEnvelope,
) -> (ReadTicket<L>, CoreTransactionV2, Vec<PreparedObject<L>>) {
    let n = event.raw_log_position.0;
    let leaf = store
        .prepare(
            &UserText {
                schema_version: 1,
                text: "fixture-resource".into(),
            },
            "1",
        )
        .unwrap();
    let mut binding: ProposalBindingV2 = with_refs(leaf.reference());
    binding.branch_id = store.branch.clone();
    binding.event_id = event.id.clone();
    binding.raw_log_position = event.raw_log_position;
    binding.base_commit = store.base_commit().unwrap();
    binding.anchor_time_ns = DecimalU64(store.head().unwrap().2);
    binding.process_time_ns = DecimalU64(n + 1);
    binding.read_set = vec![leaf.reference().clone()];
    if let Some(old) = store.current().unwrap() {
        binding.x_revision = FastStateSnapshotV2::from_canonical(
            &store.objects.read(&old.staged_x).unwrap(),
            config().encoding,
        )
        .unwrap()
        .revision;
        binding.r_revision = RSnapshotV3::from_canonical(
            &store.objects.read(&old.staged_r).unwrap(),
            config().encoding,
        )
        .unwrap()
        .revision;
        binding.theta_revision = ThetaSnapshot::from_canonical(
            &store.objects.read(&old.staged_theta).unwrap(),
            config().encoding,
        )
        .unwrap()
        .revision;
        binding.read_set.extend([
            old.staged_h,
            old.staged_x,
            old.staged_r,
            old.staged_theta,
            old.staged_l,
            old.staged_q,
            old.staged_u,
        ]);
    }
    binding
        .read_set
        .sort_by_key(|r| core_types::scalar::hex(&r.digest.0));
    binding.read_set.dedup();
    let ticket = store.bind(binding.clone()).unwrap();
    let mut pins = vec![leaf];
    let law: LawVersion = with_refs(pins[0].reference());
    pins.push(store.prepare(&law, "1").unwrap());
    let mut r: RSnapshotV3 = with_refs(pins[0].reference());
    r.revision = format!("r-{n}");
    r.law_version = pins[1].reference().clone();
    pins.push(store.prepare_r(&r).unwrap());
    let mut theta: ThetaSnapshot = with_refs(pins[0].reference());
    theta.revision = format!("theta-{n}");
    theta.law_version = pins[1].reference().clone();
    pins.push(store.prepare(&theta, "1").unwrap());
    let mut x: FastStateSnapshotV2 = fixture();
    x.revision = format!("x-{n}");
    x.anchor_time_ns = DecimalU64(n + 1);
    pins.push(store.prepare(&x, "1").unwrap());
    let mut ledger: LedgerEntry = with_refs(pins[0].reference());
    ledger.entry_id = format!("entry-{n}");
    ledger.event_id = event.id.clone();
    pins.push(store.prepare(&ledger, "1").unwrap());
    let semantic: SemanticGraph = fixture();
    pins.push(store.prepare(&semantic, "1").unwrap());
    let mut concept: ConceptProposalV2 = with_refs(pins[0].reference());
    concept.binding = binding.clone();
    pins.push(store.prepare(&concept, "1").unwrap());
    let value = CoreTransactionV2 {
        schema_version: 1,
        staged_h: store.objects.reference(event, "1").unwrap(),
        staged_x: pins[4].reference().clone(),
        staged_r: pins[2].reference().clone(),
        staged_theta: pins[3].reference().clone(),
        staged_l: pins[5].reference().clone(),
        staged_q: pins[0].reference().clone(),
        staged_u: pins[0].reference().clone(),
        processed_position: DecimalU64(n + 1),
        anchor_time_ns: DecimalU64(n + 1),
        staged_concepts: vec![pins[7].reference().clone()],
        accepted_results: vec![pins[6].reference().clone()],
        audit_refs: vec![],
        binding,
        staged_ledger: vec![ledger],
    };
    (ticket, value, pins)
}
fn advance<L: LineageKind>(
    store: &mut Store<L>,
    cap: &CommitCapability<L>,
    n: u64,
) -> CoreCommitResultV2 {
    let e = event(store, n);
    let (ticket, value, _pins) = proposal(store, &e);
    let staged = store.stage(ticket, value).unwrap();
    store.commit(cap, staged, normal()).unwrap()
}
