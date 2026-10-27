//! P5-36: the existing C trajectory crosses the real publication/reopen boundary.
use super::*;
use core_types::scalar::hex;
use serde_json::{Value, json};
use std::fs;

fn states() -> Vec<Value> {
    let bytes = match std::env::var_os("P536_STORE_INPUT") {
        Some(path) => fs::read(path).unwrap(),
        None => include_bytes!("../../../../contracts/fixtures/theta-states.json").to_vec(),
    };
    serde_json::from_slice(&bytes).unwrap()
}

#[test]
fn actual_states_commit_close_reopen() {
    let rows = states();
    let mut returned = Vec::new();
    for row in rows {
        let prefix = row["prefix"].as_u64().unwrap();
        let mut snapshot: ThetaSnapshotV2 = if let Some(encoded) = row["canonical"].as_str() {
            ThetaSnapshotV2::from_canonical(
                &core_types::scalar::unhex(encoded).unwrap(),
                config().encoding,
            )
            .unwrap()
        } else {
            serde_json::from_value(row["value"].clone()).unwrap()
        };
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "theta");
        let cap = store.take_commit_capability().unwrap();
        let e = event(&mut store, 0);
        let (ticket, mut tx, pins) = proposal(&store, &e);
        // Bind the unchanged estimator to an actual persisted synthetic law.
        snapshot.law_version = pins[1].reference().clone();
        let expected = snapshot.canonical_bytes(config().encoding).unwrap();
        let object = store.prepare(&snapshot, &snapshot.revision).unwrap();
        let reference = object.reference().clone();
        tx.staged_theta = reference.clone();
        let staged = store.stage(ticket, tx).unwrap();
        let reader = store.reader().unwrap();
        assert!(reader.read::<ThetaSnapshotV2>(&reference).is_err());
        store.commit(&cap, staged, normal()).unwrap();
        drop((reader, object, pins, cap, store));

        let reader = Reader::<Rehearsal>::open_temporary(&root, "theta", config()).unwrap();
        assert_eq!(reader.current().unwrap().unwrap().staged_theta, reference);
        let decoded: ThetaSnapshotV2 = reader.read(&reference).unwrap();
        let bytes = decoded.canonical_bytes(config().encoding).unwrap();
        assert_eq!(bytes, expected);
        assert!(reader.read::<ThetaSnapshot>(&reference).is_err());
        returned.push(json!({"prefix":prefix, "hex":hex(&bytes),
            "law_version":snapshot.law_version, "reference":reference}));
        drop(reader);
        // The current head's V2 revision also binds the next real transaction.
        let mut store = Store::<Rehearsal>::open_temporary(&root, "theta", config()).unwrap();
        let cap = store.take_commit_capability().unwrap();
        advance(&mut store, &cap, 1);
        store.verify().unwrap();
        assert_eq!(store.head().unwrap().1, 2);
    }
    if let Some(path) = std::env::var_os("P536_STORE_OUTPUT") {
        fs::write(path, serde_json::to_vec_pretty(&returned).unwrap()).unwrap();
    }
    println!(
        "P536_STORE committed, closed, reopened and read {} actual C prefixes",
        returned.len()
    );
}

#[test]
fn legacy_theta_readable_writer_refused_without_touching_history() {
    for uncheckpointed in [false, true] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "legacy-theta");
        let e = event(&mut store, 0);
        let (_ticket, mut tx, _pins) = proposal(&store, &e);
        let legacy: ThetaSnapshot = with_refs(&tx.staged_q);
        let reference = legacy_v2::raw(&store, &legacy, "legacy");
        tx.staged_theta = reference.clone();
        let commit = legacy_v2::publish(&mut store, &e, &tx, None);
        let path = store.path().to_path_buf();
        let db_path = path.join("store.sqlite");
        let wal_path = path.join("store.sqlite-wal");
        let db = fs::read(&db_path).unwrap();
        let wal = fs::read(&wal_path).unwrap();
        assert!(!wal.is_empty());
        drop((_ticket, _pins, store));
        if uncheckpointed {
            // Same owned TemporaryRoot, same consistent pair: existing W02 harness.
            fs::write(&db_path, &db).unwrap();
            fs::write(&wal_path, &wal).unwrap();
            let shm = path.join("store.sqlite-shm");
            if shm.exists() {
                fs::remove_file(shm).unwrap();
            }
        }
        let before = (!uncheckpointed).then(|| retention_versions::durable_bytes(&path));
        for _ in 0..2 {
            let reader =
                Reader::<Rehearsal>::open_temporary(&root, "legacy-theta", config()).unwrap();
            assert_eq!(reader.current().unwrap().unwrap(), tx);
            let decoded: ThetaSnapshot = reader.read(&reference).unwrap();
            assert_eq!(
                decoded.canonical_bytes(config().encoding).unwrap(),
                legacy.canonical_bytes(config().encoding).unwrap()
            );
            assert!(reader.read::<ThetaSnapshotV2>(&reference).is_err());
            assert_eq!(reader.facts(10).unwrap(), vec![e.clone()]);
            assert_eq!(reader.ledger_at(&commit, 1).unwrap().len(), 1);
            drop(reader);
            assert!(matches!(
                Store::<Rehearsal>::open_temporary(&root, "legacy-theta", config()),
                Err(Error::Rejected(crate::theta::LEGACY_READ_ONLY))
            ));
            // Existing physical-format maintenance cannot bypass the same guard.
            // No theta migration or replay is performed by either direction.
            for target in [1, 2] {
                assert!(matches!(
                    Store::<Rehearsal>::migrate_temporary(&root, "legacy-theta", config(), target),
                    Err(Error::Rejected(crate::theta::LEGACY_READ_ONLY))
                ));
            }
            if let Some(before) = &before {
                assert_eq!(&retention_versions::durable_bytes(&path), before);
            } else {
                assert_eq!(fs::read(&db_path).unwrap(), db);
                assert_eq!(fs::read(&wal_path).unwrap(), wal);
            }
        }
        println!(
            "P536_LEGACY uncheckpointed={uncheckpointed} old_bytes_readable=true rejected_writer_unchanged=true"
        );
    }
}

fn published(store: &Store<Rehearsal>) -> Vec<i64> {
    [
        "commits",
        "snapshots",
        "facts",
        "ledger_index",
        "visible_objects",
        "snapshot_objects",
        "accepted_results",
        "runtime_records",
        "dependency_index",
        "ingress_decisions",
        "outbox",
    ]
    .map(|table| {
        store
            .db
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    })
    .to_vec()
}

#[test]
fn invalid_new_and_legacy_staging_never_publish() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "invalid-theta");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let e = event(&mut store, 1);
    let (ticket, tx, _pins) = proposal(&store, &e);
    let valid = ThetaSnapshotV2::from_canonical(
        &store.objects.read(&tx.staged_theta).unwrap(),
        config().encoding,
    )
    .unwrap();
    let old_head = store.current().unwrap();
    let before = published(&store);
    // Structural violations cannot even obtain a canonical object/digest.
    for mass in [-1.0, f64::NAN, f64::INFINITY] {
        let mut bad = valid.clone();
        bad.estimator.duration_mass[0] = mass;
        let count = fs::read_dir(&store.objects.root).unwrap().count();
        assert!(bad.canonical_bytes(config().encoding).is_err());
        assert!(store.prepare(&bad, "bad").is_err());
        assert_eq!(fs::read_dir(&store.objects.root).unwrap().count(), count);
        assert_eq!(store.current().unwrap(), old_head);
        assert_eq!(published(&store), before);
    }
    let mutations: Vec<fn(&mut ThetaSnapshotV2)> = vec![
        |s| s.parameters_digest.0[0] ^= 1,
        |s| s.parameters.numerics_id = "unknown".into(),
        |s| s.law_version.resource_version = "wrong-law".into(),
        |s| s.estimator.detector_changes = DecimalU64(s.estimator.event_count.0 + 1),
        |s| s.estimator.reaction_sum.pop().map(drop).unwrap(),
    ];
    for mutate in mutations {
        let mut bad = valid.clone();
        mutate(&mut bad);
        let count = fs::read_dir(&store.objects.root).unwrap().count();
        assert!(store.prepare(&bad, "bad").is_err());
        assert_eq!(fs::read_dir(&store.objects.root).unwrap().count(), count);
        // Bypass only the new-write entry in this synthetic corruption fixture:
        // correct canonical bytes/digest do not make an invalid estimator valid.
        let reference = legacy_v2::raw(&store, &bad, "bad");
        assert!(store.objects.read(&reference).is_err());
        let mut candidate = tx.clone();
        candidate.staged_theta = reference;
        assert!(
            store
                .stage(store.bind(ticket.binding().clone()).unwrap(), candidate)
                .is_err()
        );
        assert_eq!(store.current().unwrap(), old_head);
        assert_eq!(published(&store), before);
    }
    let legacy: ThetaSnapshot = with_refs(&tx.staged_q);
    assert!(matches!(
        store.prepare(&legacy, "1"),
        Err(Error::Rejected(
            "new writes must use the active successor message"
        ))
    ));
    let mut candidate = tx.clone();
    candidate.staged_theta = legacy_v2::raw(&store, &legacy, "1");
    assert!(matches!(
        store.stage(store.bind(ticket.binding().clone()).unwrap(), candidate),
        Err(Error::Rejected(crate::theta::LEGACY_READ_ONLY))
    ));
    let mut wrong_type = tx.clone();
    wrong_type.staged_theta = tx.staged_q.clone();
    assert!(matches!(
        store.stage(store.bind(ticket.binding().clone()).unwrap(), wrong_type),
        Err(Error::Rejected("unsupported theta snapshot object type"))
    ));

    // Existing transaction fault point: all seven state pointers/indexes roll back.
    let reference = tx.staged_theta.clone();
    let staged = store.stage(ticket, tx).unwrap();
    store.set_fault(Fault::DiskFull(FaultPoint::BeforePublish));
    assert!(store.commit(&cap, staged, normal()).is_err());
    assert_eq!(store.current().unwrap(), old_head);
    assert_eq!(published(&store), before);
    assert!(
        store
            .reader()
            .unwrap()
            .read::<ThetaSnapshotV2>(&reference)
            .is_err()
    );
    store.verify().unwrap();
}
