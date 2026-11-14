use super::*;
#[test]
fn ledger_order_survives_normal_commit_and_reopen() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "ledger-order");
    let cap = store.take_commit_capability().unwrap();
    let e = event(&mut store, 0);
    let (ticket, mut value, _pins) = proposal(&store, &e);
    let mut first = value.staged_ledger[0].clone();
    first.entry_id = "z-first".into();
    let mut second = first.clone();
    second.entry_id = "a-second".into();
    second.tendency_change.as_mut().unwrap().tendency_id = "another-tendency".into();
    value.staged_ledger = vec![first, second];
    let canonical = CoreTransactionV2::from_canonical(
        &value.canonical_bytes(config().encoding).unwrap(),
        config().encoding,
    )
    .unwrap();
    assert_eq!(canonical.staged_ledger[0].entry_id, "z-first");
    let staged = store.stage(ticket, value).unwrap();
    let result = store.commit(&cap, staged, normal()).unwrap();
    store.verify().unwrap();
    drop(store);
    let reopened = Store::<Rehearsal>::open_temporary(&root, "ledger-order", config()).unwrap();
    let reader = reopened.reader().unwrap();
    let history = reader.transactions_at(&result.commit_id, 10).unwrap();
    let expected: Vec<_> = history[0]
        .staged_ledger
        .iter()
        .map(|e| e.entry_id.clone())
        .collect();
    let actual: Vec<_> = reader
        .ledger_at(&result.commit_id, 10)
        .unwrap()
        .iter()
        .map(|e| e.entry_id.clone())
        .collect();
    println!("canonical ledger order={expected:?}; ledger_at order={actual:?}; verify/reopen=OK");
    assert_eq!(
        actual, expected,
        "ledger query must retain canonical append order"
    );
}

#[test]
fn missing_reference_rows_are_rejected_before_gc() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "missing-ref");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let e = store.reader().unwrap().facts(1).unwrap().remove(0);
    let digest = core_types::scalar::hex(&e.payload_ref.digest.0);
    let path = store.path().to_path_buf();
    drop(store);
    // Deliberate logical index corruption, NOT a legal Store write or crash simulation.
    let db = rusqlite::Connection::open(path.join("store.sqlite")).unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON; DROP TRIGGER immutable_visible_objects_DELETE; DROP TRIGGER immutable_object_refs_DELETE;").unwrap();
    db.execute("DELETE FROM visible_objects WHERE digest=?1", [&digest])
        .unwrap();
    db.execute("DELETE FROM object_refs WHERE digest=?1", [&digest])
        .unwrap();
    assert_eq!(
        db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(
        db.prepare("PRAGMA foreign_key_check")
            .unwrap()
            .query([])
            .unwrap()
            .next()
            .unwrap()
            .is_none()
    );
    drop(db);
    assert!(Store::<Rehearsal>::open_temporary(&root, "missing-ref", config()).is_err());
    assert!(path.join("objects").join(format!("{digest}.obj")).exists());
}

#[test]
fn missing_latest_group_is_rejected_instead_of_resurrecting_association() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "known-evidence");
    let cap = store.take_commit_capability().unwrap();
    let mut commits = vec![];
    for n in 0..2 {
        let e = event(&mut store, n);
        let (ticket, mut value, _pins) = proposal(&store, &e);
        let mut group: EvidenceGroupV3 = fixture();
        group.group_id = "group".into();
        let mut allocation: EvidenceAllocationV3 = fixture();
        allocation.tendency_id = "tendency".into();
        group.allocations = if n == 0 { vec![allocation] } else { vec![] };
        let pin = store.prepare(&group, "1").unwrap();
        value.accepted_results.push(pin.reference().clone());
        let tx = store.stage(ticket, value).unwrap();
        commits.push(store.commit(&cap, tx, normal()).unwrap().commit_id);
    }
    assert!(
        store
            .reader()
            .unwrap()
            .evidence_for("tendency", &commits[1], 10)
            .unwrap()
            .is_empty()
    );
    let path = store.path().to_path_buf();
    drop(store);
    let db = rusqlite::Connection::open(path.join("store.sqlite")).unwrap();
    db.execute_batch("DROP TRIGGER immutable_evidence_groups_DELETE;")
        .unwrap();
    db.execute(
        "DELETE FROM evidence_groups WHERE commit_id=?1",
        [&commits[1]],
    )
    .unwrap();
    drop(db);
    assert!(Store::<Rehearsal>::open_temporary(&root, "known-evidence", config()).is_err());
}

#[test]
fn all_derived_tables_match_authoritative_rebuild() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "rebuild");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    advance(&mut store, &cap, 1);
    event(&mut store, 2); // D pending objects retained but not visible.
    let rebuilt = crate::derived::rebuild(&store.db, &store.objects).unwrap();
    for table in crate::derived::TABLES {
        assert_eq!(
            crate::derived::rows(&store.db, table).unwrap(),
            crate::derived::rows(&rebuilt, table).unwrap(),
            "{table}"
        );
    }
    store.verify().unwrap();
    // The generic comparison also detects missing rows in every populated table.
    for table in crate::derived::TABLES {
        if crate::derived::rows(&store.db, table).unwrap().is_empty() {
            continue;
        }
        store
            .db
            .execute_batch("SAVEPOINT corrupt_projection")
            .unwrap();
        store
            .db
            .execute_batch(&format!("DROP TRIGGER IF EXISTS immutable_{table}_DELETE"))
            .unwrap();
        // Deleting visible_objects is independent of its reference parent;
        // object_refs is covered by the dedicated paired-deletion regression.
        if *table != "object_refs" && *table != "facts" {
            store
                .db
                .execute(&format!("DELETE FROM {table}"), [])
                .unwrap();
            assert!(
                crate::derived::verify(&store.db, &store.objects).is_err(),
                "{table}"
            );
        }
        store
            .db
            .execute_batch("ROLLBACK TO corrupt_projection; RELEASE corrupt_projection")
            .unwrap();
    }
    store.collect_garbage().unwrap();
    store.verify().unwrap();
}

#[test]
fn initialization_crash_repairs_empty_reject_namespace() {
    let root = TemporaryRoot::new().unwrap();
    super::crash_harness::run(&root, "init", FaultPoint::ReceiptKindSynced, 2, None);
    let mut store = Store::<Rehearsal>::open_temporary(&root, "crash", config()).unwrap();
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    store
        .audit_ingress_rejection(
            IngressReason::AUTH_FAILED,
            None,
            &IngressAuditBudget {
                schema_version: 1,
                version: "budget".into(),
                max_records: 3,
                max_bytes: 4096,
            },
        )
        .unwrap();
    store.verify().unwrap();
    drop(store);
    Store::<Rehearsal>::open_temporary(&root, "crash", config())
        .unwrap()
        .verify()
        .unwrap();
}

#[test]
fn missing_reject_namespace_on_nonempty_store_refuses_writer_and_reader() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "nonempty");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    drop(store);
    std::fs::remove_dir(root.path().join(".nonempty.w02-receipts-v1/reject")).unwrap();
    assert!(Store::<Rehearsal>::open_temporary(&root, "nonempty", config()).is_err());
    assert!(Reader::<Rehearsal>::open_temporary(&root, "nonempty", config()).is_err());
}

#[test]
fn gc_refuses_live_reference_corruption_without_removing_history() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "gc-corrupt");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let e = store.reader().unwrap().facts(1).unwrap().remove(0);
    let digest = core_types::scalar::hex(&e.payload_ref.digest.0);
    store.db.execute_batch("DROP TRIGGER immutable_visible_objects_DELETE; DROP TRIGGER immutable_object_refs_DELETE;").unwrap();
    store
        .db
        .execute("DELETE FROM visible_objects WHERE digest=?1", [&digest])
        .unwrap();
    store
        .db
        .execute("DELETE FROM object_refs WHERE digest=?1", [&digest])
        .unwrap();
    assert!(store.collect_garbage().is_err());
    assert!(
        store
            .path()
            .join("objects")
            .join(format!("{digest}.obj"))
            .exists()
    );
}
