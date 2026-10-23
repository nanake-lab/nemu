use super::*;

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
        .execute_batch(
            "DROP TRIGGER immutable_facts_DELETE; DELETE FROM facts WHERE event_id='event-0';",
        )
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
