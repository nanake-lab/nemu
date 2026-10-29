//! W02 integration fixtures, not a production bootstrap or terminal smoke test.
mod common;
use common::*;
use core_runtime::{binding::digest, stages::*, store as runtime};
use core_store::{
    CommitCapability, Fault, FaultPoint, PreparedObject, Rehearsal, Store, StoreConfig,
    TemporaryRoot,
};
use core_types::{WireMessage, generated::*, scalar::DecimalU64};
use std::time::Duration;

fn config() -> StoreConfig {
    StoreConfig {
        encoding: limits(),
        busy_timeout: Duration::from_millis(30),
        max_pending_records: 32,
        max_pending_bytes: 1 << 20,
        max_object_bytes: 4 << 20,
        max_store_bytes: 64 << 20,
        max_sql_pages: 8192,
    }
}
fn open(root: &TemporaryRoot) -> Store<Rehearsal> {
    let mut h: LineageHeaderV2 = fixture();
    h.lineage = Lineage::REHEARSAL;
    h.genesis_manifest_digest = None;
    h.world_id = None;
    h.world_seed = None;
    Store::<Rehearsal>::create_temporary(root, "runtime", h, "fixture-branch", config()).unwrap()
}
fn environment(r: &ObjectRef) -> VersionedEnvironmentV2 {
    with_refs(r)
}
fn ingress(store: &mut Store<Rehearsal>, n: u64) -> EventEnvelope {
    let text = store
        .prepare(
            &UserText {
                schema_version: 1,
                text: format!("synthetic-runtime-{n}"),
            },
            "1",
        )
        .unwrap();
    let mut e: EventEnvelope = with_refs(text.reference());
    e.id = format!("event-{n}");
    e.stable_source_key = format!("key-{n}");
    e.raw_log_position = DecimalU64(n);
    e.payload_digest = e.payload_ref.digest.clone();
    store.ingest(&e).unwrap();
    e
}
fn seed(
    store: &mut Store<Rehearsal>,
    cap: &CommitCapability<Rehearsal>,
) -> PreparedObject<Rehearsal> {
    let e = ingress(store, 0);
    let leaf = store
        .prepare(
            &UserText {
                schema_version: 1,
                text: "SYNTHETIC ROOTS - not runtime genesis".into(),
            },
            "1",
        )
        .unwrap();
    let mut b: ProposalBindingV2 = with_refs(leaf.reference());
    b.branch_id = "fixture-branch".into();
    b.base_commit = store.base_commit().unwrap();
    b.event_id = e.id.clone();
    b.raw_log_position = e.raw_log_position;
    b.anchor_time_ns = DecimalU64(0);
    b.process_time_ns = DecimalU64(10);
    b.read_set = vec![leaf.reference().clone()];
    let law = store
        .prepare(&with_refs::<LawVersion>(leaf.reference()), "1")
        .unwrap();
    let mut r: RSnapshotV3 = with_refs(leaf.reference());
    r.tendencies.clear();
    r.law_version = law.reference().clone();
    let r = store.prepare_r(&r).unwrap();
    let mut theta: ThetaSnapshot = with_refs(leaf.reference());
    theta.law_version = law.reference().clone();
    let theta = store.prepare(&theta, "1").unwrap();
    let mut x: FastStateSnapshotV2 = fixture();
    x.anchor_time_ns = DecimalU64(10);
    let x = store.prepare(&x, "1").unwrap();
    let h = store.prepare(&e, "1").unwrap();
    let tx = CoreTransactionV2 {
        schema_version: 1,
        binding: b.clone(),
        staged_h: h.reference().clone(),
        staged_x: x.reference().clone(),
        staged_r: r.reference().clone(),
        staged_theta: theta.reference().clone(),
        staged_l: leaf.reference().clone(),
        staged_q: leaf.reference().clone(),
        staged_u: leaf.reference().clone(),
        staged_concepts: vec![],
        accepted_results: vec![],
        audit_refs: vec![],
        staged_ledger: vec![],
        processed_position: DecimalU64(1),
        anchor_time_ns: DecimalU64(10),
    };
    let tx = store.stage(store.bind(b).unwrap(), tx).unwrap();
    store
        .commit(cap, tx, core_runtime::status::Status::default().value())
        .unwrap();
    leaf
}
fn case(store: &Store<Rehearsal>, e: EventEnvelope, leaf: &ObjectRef) -> Case {
    let mut c = Case::new();
    c.base = runtime::read_state(store).unwrap();
    c.event = e;
    c.attributes.event_id = c.event.id.clone();
    c.binding = with_refs(leaf);
    c.binding.environment = environment(leaf);
    c.binding.branch_id = "fixture-branch".into();
    c.binding.base_commit = store.base_commit().unwrap();
    c.binding.event_id = c.event.id.clone();
    c.binding.raw_log_position = c.event.raw_log_position;
    c.binding.anchor_time_ns = c.base.x.anchor_time_ns;
    c.binding.process_time_ns = DecimalU64(30);
    c.binding.x_revision = c.base.x.revision.clone();
    c.binding.r_revision = c.base.r.revision.clone();
    c.binding.theta_revision = c.base.theta.revision.clone();
    let old = store.current().unwrap().unwrap();
    c.binding.read_set = vec![
        leaf.clone(),
        old.staged_h,
        old.staged_x,
        old.staged_r,
        old.staged_theta,
        old.staged_l,
        old.staged_q,
        old.staged_u,
    ];
    c.binding
        .read_set
        .sort_by_key(|r| core_types::scalar::hex(&r.digest.0));
    c.binding.read_set.dedup();
    c.facts.items.clear();
    c.facts.history_prefix = c.binding.base_commit.clone();
    c.binding.c_core_digest = digest(&c.facts, limits()).unwrap();
    let mut x = c.base.x.clone();
    x.anchor_time_ns = DecimalU64(30);
    c.binding.projected_state_digest = digest(&x, limits()).unwrap();
    c
}
fn staged(c: &Case) -> core_runtime::pipeline::StagedEvent<Rehearsal> {
    StageSet::<Rehearsal>::rehearsal(
        Components::default(),
        c.binding.environment.clone(),
        limits(),
    )
    .unwrap()
    .process_head(c.input(false))
    .unwrap()
}
#[test]
fn uninitialized_store_is_refused_without_fabricating_runtime_roots() {
    let root = TemporaryRoot::new().unwrap();
    let store = open(&root);
    assert!(runtime::read_state(&store).is_err());
}
#[test]
fn staged_visibility_rollback_commit_and_duplicate_filter_use_real_w02() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = open(&root);
    let cap = store.take_commit_capability().unwrap();
    let leaf = seed(&mut store, &cap);
    let old = store.current().unwrap().unwrap();
    let e = ingress(&mut store, 1);
    assert!(matches!(
        runtime::next(&mut store).unwrap(),
        runtime::Next::Event(_)
    ));
    let c = case(&store, e.clone(), leaf.reference());
    let expected = staged(&c).snapshot().x.clone();
    let prepared = runtime::prepare_existing_head(
        &store,
        store.bind(c.binding.clone()).unwrap(),
        staged(&c),
        limits(),
    )
    .unwrap();
    let xref = prepared
        .pins
        .iter()
        .find(|p| p.reference().digest == digest(&expected, limits()).unwrap())
        .unwrap()
        .reference()
        .clone();
    let reader = store.reader().unwrap();
    assert!(reader.read::<FastStateSnapshotV2>(&xref).is_err());
    assert_eq!(reader.facts(10).unwrap().len(), 1);
    store.set_fault(Fault::DiskFull(FaultPoint::BeforePublish));
    assert!(
        store
            .commit(&cap, prepared.transaction, prepared.status)
            .is_err()
    );
    assert_eq!(store.current().unwrap().unwrap(), old);
    assert!(reader.read::<FastStateSnapshotV2>(&xref).is_err());
    store.set_fault(Fault::None);
    let retry = runtime::prepare_existing_head(
        &store,
        store.bind(c.binding.clone()).unwrap(),
        staged(&c),
        limits(),
    )
    .unwrap();
    let result = store.commit(&cap, retry.transaction, retry.status).unwrap();
    assert!(result.committed);
    assert!(result.status.states.contains(&RunStatus::IRL_DEGRADED));
    assert!(result.status.states.contains(&RunStatus::NO_GROWTH));
    assert_eq!(reader.read::<FastStateSnapshotV2>(&xref).unwrap(), expected);
    assert_eq!(reader.facts(10).unwrap().len(), 2);
    let current = store.current().unwrap().unwrap();
    assert_eq!(current.staged_r, old.staged_r);
    assert_eq!(current.staged_theta, old.staged_theta);
    let stages = StageSet::<Rehearsal>::rehearsal(
        Components::default(),
        c.binding.environment.clone(),
        limits(),
    )
    .unwrap();
    let runtime::Emission::Unmodulated(status) = stages.compile_committed(&store, 30).unwrap();
    assert!(!status.autonomy());
    let mut duplicate = e;
    duplicate.raw_log_position = DecimalU64(2);
    duplicate.ingest_time_ns.0 += 500;
    store.ingest(&duplicate).unwrap();
    assert!(stages.compile_committed(&store, 30).is_err());
    assert!(matches!(
        runtime::next(&mut store).unwrap(),
        runtime::Next::Settled(Receipt {
            stage: ReceiptStage::FILTERED,
            ..
        })
    ));
    assert_eq!(
        store
            .current()
            .unwrap()
            .unwrap()
            .canonical_bytes(limits())
            .unwrap(),
        current.canonical_bytes(limits()).unwrap()
    );
    assert_eq!(store.head().unwrap().2, 30);
    assert!(matches!(
        runtime::next(&mut store).unwrap(),
        runtime::Next::Idle
    ));
    store.verify().unwrap();
    drop(reader);
    drop(store);
    let reopened = Store::<Rehearsal>::open_temporary(&root, "runtime", config()).unwrap();
    assert_eq!(reopened.head().unwrap().2, 30);
    assert_eq!(reopened.reader().unwrap().facts(10).unwrap().len(), 2);
}
#[test]
fn same_revision_forged_base_and_old_ticket_are_rejected() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = open(&root);
    let cap = store.take_commit_capability().unwrap();
    let leaf = seed(&mut store, &cap);
    let e = ingress(&mut store, 1);
    let mut c = case(&store, e, leaf.reference());
    c.base.x.nature.connection += 0.01;
    let mut x = c.base.x.clone();
    x.anchor_time_ns = DecimalU64(30);
    c.binding.projected_state_digest = digest(&x, limits()).unwrap();
    assert!(
        runtime::prepare_existing_head(
            &store,
            store.bind(c.binding.clone()).unwrap(),
            staged(&c),
            limits()
        )
        .is_err()
    );
    let c = case(&store, store.next_raw().unwrap().unwrap(), leaf.reference());
    let stale = runtime::prepare_existing_head(
        &store,
        store.bind(c.binding.clone()).unwrap(),
        staged(&c),
        limits(),
    )
    .unwrap();
    let first = runtime::prepare_existing_head(
        &store,
        store.bind(c.binding.clone()).unwrap(),
        staged(&c),
        limits(),
    )
    .unwrap();
    store.commit(&cap, first.transaction, first.status).unwrap();
    assert!(store.commit(&cap, stale.transaction, stale.status).is_err());
}
