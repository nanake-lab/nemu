use super::*;

#[test]
fn acknowledged_audit_detects_old_database_restore() {
    let root = TemporaryRoot::new().unwrap();
    let store = rehearsal(&root, "restore");
    let database = store.path().join("store.sqlite");
    drop(store);
    let old = std::fs::read(&database).unwrap();
    let mut store = Store::<Rehearsal>::open_temporary(&root, "restore", config()).unwrap();
    store
        .audit_ingress_rejection(IngressReason::AUTH_FAILED, None, &budget())
        .unwrap();
    drop(store);
    // Restore only an old synthetic database, retaining the acknowledged witness.
    std::fs::write(&database, old).unwrap();
    assert!(Store::<Rehearsal>::open_temporary(&root, "restore", config()).is_err());
    assert!(Reader::<Rehearsal>::open_temporary(&root, "restore", config()).is_err());
    assert!(Store::<Rehearsal>::migrate_temporary(&root, "restore", config(), 1).is_err());
}

#[test]
fn sys17_physical_format_roundtrip_preserves_v3_and_pending_data() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "migrate");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    event(&mut store, 1);
    let before = rows(&store.db);
    let head = store.current().unwrap();
    drop(store);
    Store::<Rehearsal>::migrate_temporary(&root, "migrate", config(), 1).unwrap();
    assert_eq!(format(&readonly(&root, "migrate")).unwrap(), 1);
    assert!(Store::<Rehearsal>::open_temporary(&root, "migrate", config()).is_err());
    assert_eq!(rows(&readonly(&root, "migrate")), before);
    Store::<Rehearsal>::migrate_temporary(&root, "migrate", config(), 2).unwrap();
    let store = Store::<Rehearsal>::open_temporary(&root, "migrate", config()).unwrap();
    assert_eq!(rows(&store.db), before);
    assert_eq!(store.current().unwrap(), head);
    assert!(store.ingress_rejections().unwrap().is_empty());
    store.verify().unwrap();
}

#[test]
fn sys17_precommit_failure_retains_complete_previous_physical_format() {
    for target in [1, 2] {
        let root = TemporaryRoot::new().unwrap();
        let store = rehearsal(&root, "migrate");
        let before = rows(&store.db);
        drop(store);
        if target == 2 {
            Store::<Rehearsal>::migrate_temporary(&root, "migrate", config(), 1).unwrap();
        }
        assert!(
            Store::<Rehearsal>::migrate_format(
                root.store_path("migrate").unwrap(),
                config(),
                true,
                target,
                Fault::DiskFull(FaultPoint::MigrationBeforeSql)
            )
            .is_err()
        );
        let db = readonly(&root, "migrate");
        assert_eq!(format(&db).unwrap(), 3 - target);
        assert_eq!(rows(&db), before);
        drop(db);
        Store::<Rehearsal>::migrate_temporary(&root, "migrate", config(), target).unwrap();
        assert_eq!(format(&readonly(&root, "migrate")).unwrap(), target);
    }
}

#[test]
fn v2_head_remains_byte_preserving_and_cannot_use_structure_migration() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "legacy");
    super::super::legacy_v2::seed(&mut store, true);
    drop(store);
    // Existing historical reader works; neither target may turn it writable.
    for target in [1, 2] {
        let persistent = |values: Vec<(std::path::PathBuf, Vec<u8>)>| {
            values
                .into_iter()
                .filter(|(p, b)| {
                    !p.to_string_lossy().ends_with("-shm")
                        && !(p.to_string_lossy().ends_with("-wal") && b.is_empty())
                })
                .collect::<Vec<_>>()
        };
        // WAL indexes are rebuildable; DB, nonempty WAL and object bytes are authority.
        let before = persistent(files(root.path()));
        assert!(Store::<Rehearsal>::migrate_temporary(&root, "legacy", config(), target).is_err());
        assert!(
            persistent(files(root.path())) == before,
            "migration modified durable V2 bytes"
        );
        assert!(Store::<Rehearsal>::open_temporary(&root, "legacy", config()).is_err());
    }
}
