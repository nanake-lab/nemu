use super::*;

#[test]
fn at01_canonical_identity_and_superseded_writes() {
    let root = TemporaryRoot::new().unwrap();
    let store = rehearsal(&root, "canonical");
    let value = UserText {
        schema_version: 1,
        text: "test".into(),
    };
    let mut protobuf = value.protobuf_bytes(config().encoding).unwrap();
    let first = protobuf.drain(..2).collect::<Vec<_>>();
    protobuf.extend(first);
    let decoded = UserText::from_protobuf(&protobuf, config().encoding).unwrap();
    let a = store.prepare(&value, "1").unwrap();
    let b = store.prepare(&decoded, "1").unwrap();
    assert_eq!(a.reference(), b.reference());
    assert_eq!(
        store.objects.read(a.reference()).unwrap(),
        value.canonical_bytes(config().encoding).unwrap()
    );
    assert!(store.prepare(&fixture::<RSnapshot>(), "1").is_err());
    assert!(store.prepare(&fixture::<Tendency>(), "1").is_err());
}

#[test]
fn at02_corrupt_or_missing_object_never_publishes() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "corrupt");
    let cap = store.take_commit_capability().unwrap();
    let e = event(&mut store, 0);
    let (ticket, value, _pins) = proposal(&store, &e);
    let bad = value.staged_theta.clone();
    let staged = store.stage(ticket, value).unwrap();
    std::fs::write(store.objects.path(&bad).unwrap(), b"corrupt").unwrap();
    assert!(store.commit(&cap, staged, normal()).is_err());
    assert_eq!(store.head().unwrap(), (None, 0, 0));
    assert!(store.reader().unwrap().facts(10).unwrap().is_empty());
}

#[test]
fn at03_pins_close_prepare_gc_publish_race() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "gc");
    let cap = store.take_commit_capability().unwrap();
    let e = event(&mut store, 0);
    let (ticket, value, pins) = proposal(&store, &e);
    let staged = store.stage(ticket, value).unwrap();
    drop(pins);
    store.collect_garbage().unwrap();
    store.commit(&cap, staged, normal()).unwrap();
    let orphan = store
        .prepare(
            &UserText {
                schema_version: 1,
                text: "orphan".into(),
            },
            "1",
        )
        .unwrap();
    let path = store.objects.path(orphan.reference()).unwrap();
    store.collect_garbage().unwrap();
    assert!(path.exists());
    drop(orphan);
    assert!(store.collect_garbage().unwrap() > 0);
    assert!(!path.exists());
    store.verify().unwrap();
}

#[test]
fn r_index_reassembles_canonical_and_reuses_unchanged_tendencies() {
    let root = TemporaryRoot::new().unwrap();
    let store = rehearsal(&root, "chunks");
    let law = store.prepare(&r_fixtures::law(), "1").unwrap();
    let mut first = r_fixtures::tendency("a");
    first.tendency_id = "a".into();
    let mut second = first.clone();
    second.tendency_id = "b".into();
    let mut r = r_fixtures::snapshot(law.reference());
    r.tendencies = vec![first.clone(), second];
    let old = store.prepare_r(&r).unwrap();
    let first_ref = store.objects.reference(&first, "1").unwrap();
    let unchanged = store.objects.path(&first_ref).unwrap();
    let modified = std::fs::metadata(&unchanged).unwrap().modified().unwrap();
    r.revision = "next".into();
    r.tendencies[1].evidence_a += 1.0;
    r.tendencies[1].evidence_absolute = 1.0;
    r.tendencies[1].evidence_b = 5.0;
    r.tendencies[1].pi = 0.2;
    let new = store.prepare_r(&r).unwrap();
    assert_ne!(old.reference(), new.reference());
    assert_eq!(
        store.objects.read(new.reference()).unwrap(),
        r.canonical_bytes(config().encoding).unwrap()
    );
    assert_eq!(
        std::fs::metadata(&unchanged).unwrap().modified().unwrap(),
        modified
    );
    assert!(new.closure.contains(&first_ref));
}

#[test]
fn sys22_capacity_and_busy_wait_are_bounded() {
    let root = TemporaryRoot::new().unwrap();
    let mut cfg = config();
    cfg.max_pending_records = 1;
    let mut store = Store::<Rehearsal>::create_temporary(
        &root,
        "capacity",
        header::<Rehearsal>(),
        "branch",
        cfg,
    )
    .unwrap();
    let first = event(&mut store, 0);
    let mut second = first.clone();
    second.raw_log_position = DecimalU64(1);
    assert!(store.ingest(&second).is_err());
    assert_eq!(
        store
            .db
            .query_row("SELECT count(*) FROM raw_ingress", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    let other = rusqlite::Connection::open(store.path().join("store.sqlite")).unwrap();
    other.execute_batch("BEGIN IMMEDIATE").unwrap();
    let start = std::time::Instant::now();
    assert!(store.ingest(&second).is_err());
    assert!(start.elapsed() < Duration::from_secs(2));
    other.execute_batch("ROLLBACK").unwrap();
    let pages: i64 = store
        .db
        .pragma_query_value(None, "page_count", |r| r.get(0))
        .unwrap();
    store
        .db
        .pragma_update(None, "max_page_count", pages)
        .unwrap();
    // Exercise real SQLITE_FULL, not just a simulated I/O error.
    let result = store.db.execute(
        "INSERT INTO raw_ingress VALUES(99,zeroblob(1000000),1000000)",
        [],
    );
    assert!(
        matches!(result, Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == rusqlite::ErrorCode::DiskFull)
    );
    store.verify().unwrap();
}

#[test]
fn p532_evidence_versions_and_ledger_prefix_are_complete() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "evidence");
    let cap = store.take_commit_capability().unwrap();
    let mut commits = Vec::new();
    for n in 0..2 {
        let e = event(&mut store, n);
        let (ticket, mut value, _pins) = proposal(&store, &e);
        let mut group = r_fixtures::group("group", "tendency");
        if n != 0 {
            group.allocations.clear();
        }
        let object = store.prepare(&group, "1").unwrap();
        value.accepted_results.push(object.reference().clone());
        let staged = store.stage(ticket, value).unwrap();
        commits.push(store.commit(&cap, staged, normal()).unwrap().commit_id);
    }
    let reader = store.reader().unwrap();
    assert_eq!(
        reader
            .evidence_for("tendency", &commits[0], 10)
            .unwrap()
            .len(),
        1
    );
    assert!(
        reader
            .evidence_for("tendency", &commits[1], 10)
            .unwrap()
            .is_empty()
    );
    assert!(
        reader
            .evidence_for("tendency", "missing-commit", 10)
            .is_err()
    );
    assert!(reader.evidence_for("tendency", &commits[0], 0).is_err());
    assert_eq!(reader.ledger_at(&commits[1], 10).unwrap().len(), 2);
    assert!(reader.ledger_at(&commits[1], 1).is_err());
}

#[test]
fn published_closure_damage_refuses_recovery() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "closure");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let snapshot = store.current().unwrap().unwrap();
    let path = store.objects.path(&snapshot.staged_theta).unwrap();
    drop(store);
    std::fs::remove_file(path).unwrap();
    assert!(Store::<Rehearsal>::open_temporary(&root, "closure", config()).is_err());
}

#[test]
fn object_capacity_and_io_failures_never_acknowledge_d() {
    for point in [
        FaultPoint::ObjectWritten,
        FaultPoint::FileSynced,
        FaultPoint::Renamed,
        FaultPoint::DirectorySynced,
    ] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "io");
        store.set_fault(Fault::DiskFull(point));
        assert!(
            store
                .prepare(
                    &UserText {
                        schema_version: 1,
                        text: "synthetic".into()
                    },
                    "1"
                )
                .is_err()
        );
        assert_eq!(store.head().unwrap(), (None, 0, 0));
        store.verify().unwrap();
    }
    let root = TemporaryRoot::new().unwrap();
    let mut cfg = config();
    cfg.max_object_bytes = 1;
    cfg.max_store_bytes = 1;
    let store =
        Store::<Rehearsal>::create_temporary(&root, "tiny", header::<Rehearsal>(), "tiny", cfg)
            .unwrap();
    assert!(
        store
            .prepare(
                &UserText {
                    schema_version: 1,
                    text: "synthetic".into()
                },
                "1"
            )
            .is_err()
    );
}

#[test]
fn publication_primitive_never_replaces_existing_target() {
    use std::io::Write;
    let root = TemporaryRoot::new().unwrap();
    let target = root.path().join("published");
    let mut first = tempfile::NamedTempFile::new_in(root.path()).unwrap();
    first.write_all(b"original").unwrap();
    first.as_file().sync_all().unwrap();
    crate::platform::persist_noclobber(first, &target).unwrap();
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        assert_eq!(
            std::fs::metadata(&target).unwrap().file_attributes() & 0x100,
            0,
            "published file must not retain FILE_ATTRIBUTE_TEMPORARY"
        );
    }
    let mut second = tempfile::NamedTempFile::new_in(root.path()).unwrap();
    second.write_all(b"replacement").unwrap();
    second.as_file().sync_all().unwrap();
    assert!(crate::platform::persist_noclobber(second, &target).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), b"original");
    crate::platform::sync_dir(root.path()).unwrap();
}

#[test]
fn segmented_r_commit_retains_both_tendencies_through_reopen_and_gc() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "segmented-commit");
    let cap = store.take_commit_capability().unwrap();
    let e = event(&mut store, 0);
    let (ticket, mut value, _pins) = proposal(&store, &e);
    let mut r = RSnapshotV3::from_canonical(
        &store.objects.read(&value.staged_r).unwrap(),
        config().encoding,
    )
    .unwrap();
    let mut tendency = r_fixtures::tendency("a");
    tendency.tendency_id = "a".into();
    r.tendencies = vec![tendency.clone()];
    tendency.tendency_id = "b".into();
    r.tendencies.push(tendency);
    let object = store.prepare_r(&r).unwrap();
    value.staged_r = object.reference().clone();
    let staged = store.stage(ticket, value).unwrap();
    store.commit(&cap, staged, normal()).unwrap();
    let reference = object.reference().clone();
    drop(object);
    drop(store);
    let mut store =
        Store::<Rehearsal>::open_temporary(&root, "segmented-commit", config()).unwrap();
    store.collect_garbage().unwrap();
    assert_eq!(
        store
            .reader()
            .unwrap()
            .read::<RSnapshotV3>(&reference)
            .unwrap(),
        r
    );
    store.verify().unwrap();
}
