//! P5-53 1/4. Public runtime entry, real isolated W02 writer and independent readers.
mod common;
use common::{fixture, limits, with_refs};
use core_runtime::{
    stages::{Components, Snapshot, StageSet},
    store::{self, Emission},
};
use core_store::{Fault, FaultPoint, Reader, Rehearsal, Store, StoreConfig, TemporaryRoot};
use core_types::{
    generated::*,
    scalar::{DecimalU64, hex},
};
use std::time::Duration;

fn config() -> StoreConfig {
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

#[test]
fn bootstrap_exposes_one_complete_head_and_restarts_idempotently() {
    let temp = TemporaryRoot::new().unwrap();
    let mut header: LineageHeaderV2 = fixture();
    header.lineage = Lineage::REHEARSAL;
    header.genesis_manifest_digest = None;
    header.world_id = None;
    header.world_seed = None;
    let mut writer =
        Store::<Rehearsal>::create_temporary(&temp, "rehearsal", header.clone(), "test", config())
            .unwrap();
    let cap = writer.take_commit_capability().unwrap();
    let resource = writer.prepare_raw(b"synthetic rehearsal resource").unwrap();
    let mut state = Snapshot {
        x: fixture(),
        r: fixture(),
        theta: fixture(),
    };
    let law = writer
        .prepare(
            &with_refs::<LawVersion>(resource.reference()),
            &state.theta.parameters.law_id,
        )
        .unwrap();
    state.r.tendencies.clear();
    state.x.learned.clear();
    state.r.law_version = law.reference().clone();
    state.theta.law_version = law.reference().clone();
    let mut env: VersionedEnvironmentV2 = with_refs(resource.reference());
    env.implementations.clear();
    env.law_version = None;
    let reader = Reader::<Rehearsal>::open_temporary(&temp, "rehearsal", config()).unwrap();
    // A declared law different from the snapshots, or a coordinate outside R+,
    // cannot produce a visible initial head.
    let mut other_env = env.clone();
    other_env.law_version = Some(resource.reference().clone());
    assert!(store::bootstrap_rehearsal(&mut writer, &cap, &other_env, &state).is_err());
    let mut bad_state = state.clone();
    bad_state.x.learned.push(LearnedComponent {
        schema_version: 1,
        tendency_id: "absent".into(),
        value: 0.0,
    });
    assert!(store::bootstrap_rehearsal(&mut writer, &cap, &env, &bad_state).is_err());
    assert!(reader.current_snapshot().unwrap().is_none());
    writer.set_fault(Fault::DiskFull(FaultPoint::BeforePublish));
    assert!(store::bootstrap_rehearsal(&mut writer, &cap, &env, &state).is_err());
    assert_eq!(reader.current_snapshot().unwrap(), None);
    assert!(reader.read::<LawVersion>(law.reference()).is_err());
    assert!(reader.read_raw(resource.reference()).is_err());
    writer.set_fault(Fault::None);
    let reference = store::bootstrap_rehearsal(&mut writer, &cap, &env, &state).unwrap();
    let id = hex(&reference.digest.0);
    assert_eq!(
        writer.head().unwrap(),
        (Some(id.clone()), 0, state.x.anchor_time_ns.0)
    );
    assert_eq!(reader.snapshot_at(&id).unwrap(), Some(reference.clone()));
    let p: InitialPublication = reader.read(&reference).unwrap();
    assert_eq!(p.header, header);
    assert_eq!(p.branch_id, "test");
    // Pin the persisted ID for LineageHeaderV2.hex (REHEARSAL at 0 ns) and branch "test".
    // A change to its encoding would prevent existing initial publications from reopening.
    assert_eq!(
        p.store_id,
        "83a1391955d22b4763eba84332927191abf154c651c5416cb05d6d75bd7cb559"
    );
    assert_eq!(
        p.empty_history.unwrap().kind,
        EmptyPrefixKind::EMPTY_HISTORY
    );
    assert_eq!(p.q.environment, env);
    assert!(
        [
            &p.q.fast,
            &p.q.compiler,
            &p.q.growth,
            &p.q.dynamics,
            &p.q.irl,
            &p.q.appraiser,
            &p.q.matcher
        ]
        .into_iter()
        .all(Option::is_none)
    );
    assert_eq!(reader.read::<FastStateSnapshotV2>(&p.x).unwrap(), state.x);
    assert_eq!(reader.read::<RSnapshotV3>(&p.r).unwrap(), state.r);
    assert_eq!(
        reader.read::<ThetaSnapshotV2>(&p.theta).unwrap(),
        state.theta
    );
    for kind in [
        EmptyPrefixKind::EMPTY_HISTORY,
        EmptyPrefixKind::EMPTY_CHANGE_LEDGER,
        EmptyPrefixKind::EMPTY_INGRESS_DECISIONS,
    ] {
        let empty = EmptyPrefixRoot {
            schema_version: 1,
            kind,
        };
        let object = writer.prepare(&empty, "1").unwrap();
        assert_eq!(
            reader.read::<EmptyPrefixRoot>(object.reference()).unwrap(),
            empty
        );
    }
    let q = writer.prepare(&p.q, "1").unwrap();
    assert_eq!(
        reader.read::<InitialRuntimeRoot>(q.reference()).unwrap(),
        p.q
    );
    assert!(reader.transactions_at(&id, 4).unwrap().is_empty());
    assert!(reader.facts(4).unwrap().is_empty());
    assert!(reader.ledger_at(&id, 4).unwrap().is_empty());
    drop(reader);
    drop(writer);
    assert!(Store::<Rehearsal>::migrate_temporary(&temp, "rehearsal", config(), 2).is_err());
    assert!(Reader::<core_store::Life>::open_temporary(&temp, "rehearsal", config()).is_err());
    let mut reopened = Store::<Rehearsal>::open_temporary(&temp, "rehearsal", config()).unwrap();
    reopened.collect_garbage().unwrap();
    let cap = reopened.take_commit_capability().unwrap();
    assert_eq!(
        store::bootstrap_rehearsal(&mut reopened, &cap, &env, &state).unwrap(),
        reference
    );
    let stages = StageSet::rehearsal(Components::default(), env.clone(), limits()).unwrap();
    let Emission::Unmodulated(status) = stages
        .compile_committed(&reopened, state.x.anchor_time_ns.0)
        .unwrap();
    assert!(!status.autonomy());
    for s in [
        RunStatus::CORE_UNMODULATED,
        RunStatus::NO_GROWTH,
        RunStatus::IRL_DEGRADED,
        RunStatus::A_DEGRADED,
        RunStatus::M_DEGRADED,
    ] {
        assert!(status.value().states.contains(&s));
    }
    state.x.anchor_time_ns = DecimalU64(state.x.anchor_time_ns.0 + 1);
    assert!(store::bootstrap_rehearsal(&mut reopened, &cap, &env, &state).is_err());
    reopened.verify().unwrap();
    assert_eq!(reopened.head().unwrap().0, Some(id));
}
