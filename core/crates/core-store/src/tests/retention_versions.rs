use super::*;
use std::{collections::BTreeMap, fs, path::Path};

// Compare authoritative files, not timestamps or SQLite's transient read locks.
fn durable_bytes(path: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, path: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                let name = path.file_name().unwrap().to_string_lossy();
                if name.ends_with("-wal") {
                    assert_eq!(
                        fs::metadata(&path).unwrap().len(),
                        0,
                        "closed legacy fixture must have no uncheckpointed writes"
                    );
                    continue;
                }
                if name.ends_with("-shm") {
                    continue;
                }
                files.insert(
                    path.strip_prefix(root).unwrap().to_string_lossy().into(),
                    fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(path, path, &mut files);
    files
}

#[test]
fn p534_v2_history_readable_writer_refused_without_migration() {
    for segmented in [false, true] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "legacy");
        let (commit, r_ref, group_ref) = legacy_v2::seed(&mut store, segmented);
        let r_bytes = store.objects.read(&r_ref).unwrap();
        let group_bytes = store.objects.read(&group_ref).unwrap();
        let path = store.path().to_path_buf();
        drop(store);
        // Closing the fixture writer checkpoints its WAL before this comparison.
        let before = durable_bytes(&path);
        let reader = Reader::<Rehearsal>::open_temporary(&root, "legacy", config()).unwrap();
        assert_eq!(reader.current().unwrap().unwrap().staged_r, r_ref);
        let r: RSnapshotV2 = reader.read(&r_ref).unwrap();
        assert_eq!(r.canonical_bytes(config().encoding).unwrap(), r_bytes);
        let group: EvidenceGroupV2 = reader.read(&group_ref).unwrap();
        assert_eq!(
            group.canonical_bytes(config().encoding).unwrap(),
            group_bytes
        );
        assert_eq!(
            reader.evidence_for("legacy-t", &commit, 1).unwrap(),
            vec![EvidenceGroupRecord::V2(group)]
        );
        assert!(reader.evidence_for("legacy-t", &commit, 0).is_err());
        assert_eq!(reader.facts(10).unwrap().len(), 1);
        assert_eq!(reader.ledger_at(&commit, 1).unwrap().len(), 1);
        assert!(reader.read::<RSnapshotV3>(&r_ref).is_err());
        drop(reader);
        let opened = Store::<Rehearsal>::open_temporary(&root, "legacy", config());
        assert!(matches!(
            opened,
            Err(Error::Rejected(
                "V2 R head is read-only; P5-26 G/H replay is required before writing"
            ))
        ));
        assert_eq!(durable_bytes(&path), before);
        // Rejected startup leaves the same old history usable by the old decoder.
        let reader = Reader::<Rehearsal>::open_temporary(&root, "legacy", config()).unwrap();
        assert_eq!(reader.read::<RSnapshotV2>(&r_ref).unwrap(), r);
        assert_eq!(reader.current().unwrap().unwrap().processed_position.0, 1);
    }
}

#[test]
fn p534_v2_uncheckpointed_wal_preserved_on_rejected_writer() {
    for segmented in [false, true] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "legacy-wal");
        let (commit, r_ref, group_ref) = legacy_v2::seed(&mut store, segmented);
        let path = store.path().to_path_buf();
        let db_path = path.join("store.sqlite");
        let wal_path = path.join("store.sqlite-wal");
        let shm_path = path.join("store.sqlite-shm");
        // The sole fixture writer is idle. Capture its consistent DB/WAL pair
        // before last-close checkpointing, then restore it in the SAME owned
        // TemporaryRoot to represent a crash with committed WAL frames.
        let db_before = fs::read(&db_path).unwrap();
        let wal_before = fs::read(&wal_path).unwrap();
        assert!(!wal_before.is_empty(), "fixture must retain committed WAL");
        drop(store);
        fs::write(&db_path, &db_before).unwrap();
        fs::write(&wal_path, &wal_before).unwrap();
        if shm_path.exists() {
            fs::remove_file(&shm_path).unwrap();
        }

        let reader = Reader::<Rehearsal>::open_temporary(&root, "legacy-wal", config()).unwrap();
        let r: RSnapshotV2 = reader.read(&r_ref).unwrap();
        let group: EvidenceGroupV2 = reader.read(&group_ref).unwrap();
        assert_eq!(reader.current().unwrap().unwrap().staged_r, r_ref);
        assert_eq!(
            reader.evidence_for("legacy-t", &commit, 1).unwrap(),
            vec![EvidenceGroupRecord::V2(group.clone())]
        );
        drop(reader);
        assert!(fs::read(&db_path).unwrap() == db_before);
        assert!(fs::read(&wal_path).unwrap() == wal_before);

        let opened = Store::<Rehearsal>::open_temporary(&root, "legacy-wal", config());
        assert!(matches!(
            opened,
            Err(Error::Rejected(
                "V2 R head is read-only; P5-26 G/H replay is required before writing"
            ))
        ));
        // P5-35: physical migration must not bypass V2 read-only protection or
        // checkpoint its uncheckpointed WAL, in either physical direction.
        for target in [1, 2] {
            assert!(matches!(
                Store::<Rehearsal>::migrate_temporary(&root, "legacy-wal", config(), target),
                Err(Error::Rejected(
                    "V2 R head is read-only; P5-26 G/H replay is required before writing"
                ))
            ));
            assert!(fs::read(&db_path).unwrap() == db_before);
            assert!(fs::read(&wal_path).unwrap() == wal_before);
        }
        let db_after = fs::read(&db_path).unwrap();
        let wal_after = match fs::read(&wal_path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => panic!("read WAL after rejected writer: {error}"),
        };
        println!(
            "P534_WAL sqlite={} segmented={segmented} db_before={} db_after={} wal_before={} wal_after={} db_unchanged={} wal_unchanged={}",
            rusqlite::version(),
            db_before.len(),
            db_after.len(),
            wal_before.len(),
            wal_after.len(),
            db_before == db_after,
            wal_before == wal_after,
        );
        assert!(
            db_before == db_after && wal_before == wal_after,
            "rejected writer changed persistent DB/WAL bytes"
        );
        // Reopening the old decoder after rejection must still expose the same
        // V2 history, including both flat and segmented R representations.
        let reader = Reader::<Rehearsal>::open_temporary(&root, "legacy-wal", config()).unwrap();
        assert_eq!(reader.read::<RSnapshotV2>(&r_ref).unwrap(), r);
        assert_eq!(reader.read::<EvidenceGroupV2>(&group_ref).unwrap(), group);
        assert_eq!(reader.current().unwrap().unwrap().processed_position.0, 1);
        assert_eq!(reader.facts(10).unwrap().len(), 1);
        assert_eq!(reader.ledger_at(&commit, 1).unwrap().len(), 1);
        drop(reader);
        assert!(fs::read(&db_path).unwrap() == db_before);
        assert!(fs::read(&wal_path).unwrap() == wal_before);
    }
}

#[test]
fn p534_v3_q_v_survive_commit_reopen_gc_and_continued_writes() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "v3");
    let cap = store.take_commit_capability().unwrap();
    let e = event(&mut store, 0);
    let (ticket, mut value, pins) = proposal(&store, &e);
    let mut r: RSnapshotV3 = with_refs(&value.staged_q);
    let mut tendency: TendencyV3 = fixture();
    tendency.tendency_id = "t".into();
    tendency.evidence_absolute = f64::from_bits(1);
    r.tendencies = vec![tendency];
    let r_object = store.prepare_r(&r).unwrap();
    value.staged_r = r_object.reference().clone();
    let mut group: EvidenceGroupV3 = fixture();
    let mut allocation: EvidenceAllocationV3 = fixture();
    allocation.tendency_id = "t".into();
    allocation.actual_retention = f64::from_bits(1.0_f64.to_bits() - 1);
    group.allocations = vec![allocation];
    let group_object = store.prepare(&group, "1").unwrap();
    value
        .accepted_results
        .push(group_object.reference().clone());
    let staged = store.stage(ticket, value).unwrap();
    let committed = store.commit(&cap, staged, normal()).unwrap();
    let r_ref = r_object.reference().clone();
    let group_ref = group_object.reference().clone();
    drop((r_object, group_object, pins, store));
    let mut store = Store::<Rehearsal>::open_temporary(&root, "v3", config()).unwrap();
    store.collect_garbage().unwrap();
    let reader = store.reader().unwrap();
    let decoded: RSnapshotV3 = reader.read(&r_ref).unwrap();
    assert_eq!(
        decoded.canonical_bytes(config().encoding).unwrap(),
        r.canonical_bytes(config().encoding).unwrap()
    );
    assert_eq!(decoded.tendencies[0].evidence_absolute.to_bits(), 1);
    let decoded: EvidenceGroupV3 = reader.read(&group_ref).unwrap();
    assert_eq!(
        decoded.allocations[0].actual_retention.to_bits(),
        1.0_f64.to_bits() - 1
    );
    assert_eq!(
        reader.evidence_for("t", &committed.commit_id, 1).unwrap(),
        vec![EvidenceGroupRecord::V3(group)]
    );
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 1);
    store.verify().unwrap();
    assert_eq!(store.head().unwrap().1, 2);
}

#[test]
fn p534_old_messages_and_flat_r_cannot_enter_new_writes() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "reject");
    assert!(store.prepare(&fixture::<RSnapshotV2>(), "1").is_err());
    assert!(store.prepare(&fixture::<RSnapshotV3>(), "1").is_err());
    assert!(store.prepare(&fixture::<TendencyV2>(), "1").is_err());
    assert!(
        store
            .prepare(&fixture::<EvidenceAllocationV2>(), "1")
            .is_err()
    );
    assert!(store.prepare(&fixture::<EvidenceGroupV2>(), "1").is_err());
    let e = event(&mut store, 0);
    let (ticket, mut value, _pins) = proposal(&store, &e);
    let r: RSnapshotV2 = with_refs(&value.staged_q);
    value.staged_r = legacy_v2::raw(&store, &r, "1");
    assert!(store.stage(ticket, value).is_err());
    assert_eq!(store.head().unwrap(), (None, 0, 0));
    store.verify().unwrap();
}
