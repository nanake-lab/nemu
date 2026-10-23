use super::*;

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
