use super::*;

#[test]
fn receipts_extend_the_verified_prefix_and_recheck_after_reopen() {
    use crate::reader::FULL_CHECKS;
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "receipt-prefix");
    let cap = store.take_commit_capability().unwrap();
    let first = advance(&mut store, &cap, 0);
    let mut slot = None;
    let before = FULL_CHECKS.get();
    let reader = store.receipt_reader(&mut slot).unwrap();
    assert_eq!(
        reader
            .read_published_transaction(&first.snapshot)
            .unwrap()
            .1
            .id,
        first.event_id
    );
    assert_eq!(FULL_CHECKS.get() - before, 1);

    let second = advance(&mut store, &cap, 1);
    assert!(
        slot.as_ref()
            .unwrap()
            .read_published_transaction(&second.snapshot)
            .is_err()
    );
    for outcome in [&second, &first] {
        let reader = store.receipt_reader(&mut slot).unwrap();
        assert_eq!(
            reader
                .read_published_transaction(&outcome.snapshot)
                .unwrap()
                .1
                .id,
            outcome.event_id
        );
    }
    assert_eq!(FULL_CHECKS.get() - before, 1);
    event(&mut store, 2);
    store.receipt_reader(&mut slot).unwrap();
    assert_eq!(FULL_CHECKS.get() - before, 1);

    drop(store);
    let reopened = Store::<Rehearsal>::open_temporary(&root, "receipt-prefix", config()).unwrap();
    reopened.receipt_reader(&mut slot).unwrap();
    assert_eq!(FULL_CHECKS.get() - before, 2);
}

#[test]
fn cached_receipt_checks_transaction_objects_and_live_publication() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "receipt-damage");
    let cap = store.take_commit_capability().unwrap();
    let result = advance(&mut store, &cap, 0);
    let mut slot = None;
    let reader = store.receipt_reader(&mut slot).unwrap();
    let (_, event) = reader.read_published_transaction(&result.snapshot).unwrap();
    let path = store.objects.path(&event.payload_ref).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let mut damaged = bytes.clone();
    *damaged.last_mut().unwrap() ^= 1;
    std::fs::write(&path, damaged).unwrap();
    assert!(reader.read_published_transaction(&result.snapshot).is_err());
    std::fs::write(&path, bytes).unwrap();
    assert!(reader.read_published_transaction(&result.snapshot).is_ok());

    // Remove the synthetic commit while leaving its head, indexes and objects intact.
    let db = rusqlite::Connection::open(store.path().join("store.sqlite")).unwrap();
    db.execute_batch(
        "PRAGMA foreign_keys=OFF; DROP TRIGGER immutable_commits_DELETE; DELETE FROM commits;",
    )
    .unwrap();
    assert!(reader.read::<CoreTransactionV2>(&result.snapshot).is_ok());
    assert!(reader.read_published_transaction(&result.snapshot).is_err());
    assert!(store.receipt_reader(&mut slot).is_err());
    assert!(slot.is_none());
}

#[test]
fn visible_transaction_dependency_is_not_a_published_commit() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "receipt-membership");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let mut other = store.current().unwrap().unwrap();
    other.audit_refs.push(other.staged_q.clone());
    let nested = store.prepare(&other, "1").unwrap();
    let next = event(&mut store, 1);
    let (ticket, mut tx, _pins) = proposal(&store, &next);
    tx.audit_refs.push(nested.reference().clone());
    let staged = store.stage(ticket, tx).unwrap();
    store.commit(&cap, staged, normal()).unwrap();
    let mut slot = None;
    let reader = store.receipt_reader(&mut slot).unwrap();
    assert_eq!(
        reader
            .read::<CoreTransactionV2>(nested.reference())
            .unwrap(),
        other
    );
    assert!(
        reader
            .read_published_transaction(nested.reference())
            .is_err()
    );
}

#[test]
fn complete_transaction_prefix_is_verified_and_bounded() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "prefix");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let first = store.base_commit().unwrap();
    advance(&mut store, &cap, 1);
    let head = store.base_commit().unwrap();
    let reader = store.reader().unwrap();
    let prefix = reader.transactions_at(&first, 1).unwrap();
    assert_eq!(prefix.len(), 1);
    assert_eq!(prefix[0].binding.event_id, "event-0");
    assert_eq!(reader.transactions_at(&head, 2).unwrap().len(), 2);
    assert!(reader.transactions_at(&head, 1).is_err());
    assert!(reader.transactions_at("unknown", 10).is_err());
    // Even damage introduced after Reader::open must fail recovery in the
    // fixed query snapshot, rather than presenting an incomplete history.
    store
        .db
        .execute_batch("DROP TRIGGER immutable_snapshots_DELETE;")
        .unwrap();
    store
        .db
        .execute("DELETE FROM snapshots WHERE commit_id=?1", [&first])
        .unwrap();
    assert!(reader.transactions_at(&head, 10).is_err());
    assert!(Reader::<Life>::open_temporary(&root, "prefix", config()).is_err());
}

#[test]
fn reader_receipts_precede_sql_snapshot_under_controlled_appends() {
    for after_snapshot in [false, true] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "concurrent");
        let cap = store.take_commit_capability().unwrap();
        advance(&mut store, &cap, 0);
        let path = store.path().to_path_buf();
        let append = || {
            advance(&mut store, &cap, 1);
        };
        let reader = if after_snapshot {
            Reader::<Rehearsal>::open_checked(&path, config(), true, || {}, append)
        } else {
            Reader::<Rehearsal>::open_checked(&path, config(), true, append, || {})
        }
        .unwrap();
        assert_eq!(reader.facts(10).unwrap().len(), 2);
        assert_eq!(
            reader
                .ledger_at(&store.base_commit().unwrap(), 10)
                .unwrap()
                .len(),
            2
        );
        store.verify().unwrap();
        // Same ordering with only unprocessed D publication, as in review B2.
        let reader = Reader::<Rehearsal>::open_checked(
            &path,
            config(),
            true,
            || {
                event(&mut store, 2);
            },
            || {},
        )
        .unwrap();
        assert_eq!(reader.facts(10).unwrap().len(), 2);
        assert_eq!(reader.current().unwrap().unwrap().processed_position.0, 2);
    }
}

#[test]
fn live_readers_open_during_ingress_without_false_rollback() {
    let root = TemporaryRoot::new().unwrap();
    let mut cfg = config();
    cfg.max_pending_records = 64;
    let mut store = Store::<Rehearsal>::create_temporary(
        &root,
        "concurrent",
        header::<Rehearsal>(),
        "test-branch",
        cfg,
    )
    .unwrap();
    event(&mut store, 0);
    std::thread::scope(|scope| {
        let writer = scope.spawn(|| {
            for n in 1..12 {
                event(&mut store, n);
            }
        });
        for _ in 0..6 {
            Reader::<Rehearsal>::open_temporary(&root, "concurrent", cfg).unwrap();
        }
        writer.join().unwrap();
    });
    store.verify().unwrap();
}

#[test]
fn background_publication_proof_requires_the_same_writer_epoch_on_delivery() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "delivery-epoch");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let opened = store.receipt_check(None, || {}).unwrap();
    let reader = std::thread::spawn(opened).join().unwrap().unwrap();
    assert!(store.publication_matches(&reader).unwrap());

    // Another committed SQL write, even off the publication path, invalidates
    // this delivery. Leave the authoritative tables intact for the reopen check.
    let other = rusqlite::Connection::open(store.path().join("store.sqlite")).unwrap();
    other
        .execute_batch("CREATE TABLE synthetic_epoch_probe(value INTEGER)")
        .unwrap();
    drop(other);
    assert!(!store.publication_matches(&reader).unwrap());
    let refreshed = store.receipt_check(Some(reader), || {}).unwrap()().unwrap();
    assert!(store.publication_matches(&refreshed).unwrap());
    advance(&mut store, &cap, 1);
    assert!(!store.publication_matches(&refreshed).unwrap());

    let old = store.receipt_check(None, || {}).unwrap()().unwrap();
    drop(store);
    let reopened = Store::<Rehearsal>::open_temporary(&root, "delivery-epoch", config()).unwrap();
    assert!(!reopened.publication_matches(&old).unwrap());
}

#[test]
fn incremental_verification_hashes_new_references_and_full_checks_cold_history() {
    for cold in [true, false] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "damaged-publication");
        let cap = store.take_commit_capability().unwrap();
        let first = advance(&mut store, &cap, 0);
        let mut slot = None;
        store.receipt_reader(&mut slot).unwrap();
        advance(&mut store, &cap, 1);
        let current = store.current().unwrap().unwrap();
        // Damage a referenced object, separately from the transaction manifest.
        let reference = if cold {
            first.snapshot
        } else {
            current.staged_x
        };
        let path = store.objects.path(&reference).unwrap();
        let mut bytes = std::fs::read(&path).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        std::fs::write(path, bytes).unwrap();
        if cold {
            let reader = store.receipt_reader(&mut slot).unwrap();
            assert!(reader.read::<CoreTransactionV2>(&reference).is_err());
            assert!(reader.read_published_transaction(&reference).is_err());
        } else {
            assert!(store.receipt_reader(&mut slot).is_err());
        }
        assert!(store.full_publication_check().unwrap()().is_err());
        assert!(store.verify().is_err());
    }
}

#[test]
fn external_sql_write_forces_full_check_for_all_store_readers() {
    use crate::reader::FULL_CHECKS;
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "external-epoch");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    store.verified_reader().unwrap();
    let before = FULL_CHECKS.get();
    advance(&mut store, &cap, 1);
    store.reader().unwrap();
    assert_eq!(FULL_CHECKS.get(), before);
    let other = rusqlite::Connection::open(store.path().join("store.sqlite")).unwrap();
    other
        .execute_batch("CREATE TABLE synthetic_epoch_probe(value INTEGER)")
        .unwrap();
    store.verified_reader().unwrap();
    assert_eq!(FULL_CHECKS.get(), before + 1);
}
