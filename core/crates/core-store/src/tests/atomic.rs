use super::*;

#[test]
fn atomic_publication_and_append_only_history() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "atomic");
    let cap = store.take_commit_capability().unwrap();
    assert!(store.take_commit_capability().is_err());
    let reader = store.reader().unwrap();
    let e = event(&mut store, 0);
    assert!(reader.facts(10).unwrap().is_empty());
    assert!(
        reader
            .read::<EventEnvelope>(&store.objects.reference(&e, "1").unwrap())
            .is_err()
    );
    let (ticket, value, _pins) = proposal(&store, &e);
    let expected = value.canonical_bytes(config().encoding).unwrap();
    let staged = store.stage(ticket, value).unwrap();
    assert!(reader.current().unwrap().is_none());
    let result = store.commit(&cap, staged, normal()).unwrap();
    assert_eq!(
        reader
            .current()
            .unwrap()
            .unwrap()
            .canonical_bytes(config().encoding)
            .unwrap(),
        expected
    );
    assert_eq!(reader.facts(10).unwrap(), vec![e]);
    assert_eq!(
        reader
            .read::<CoreTransactionV2>(&result.snapshot)
            .unwrap()
            .processed_position
            .0,
        1
    );
    assert!(
        store
            .db
            .execute("UPDATE facts SET event_id='rewrite'", [])
            .is_err()
    );
    assert!(store.db.execute("DELETE FROM raw_ingress", []).is_err());
    store.verify().unwrap();
}

#[test]
fn d101_d102_stale_and_mutated_bindings_rejected() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "bindings");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let e = event(&mut store, 1);
    let (ticket, value, _pins) = proposal(&store, &e);
    let original = serde_json::to_value(&value.binding).unwrap();
    for key in [
        "branch_id",
        "event_id",
        "base_commit",
        "x_revision",
        "r_revision",
        "theta_revision",
        "source_validity_version",
    ] {
        let mut altered = original.clone();
        altered[key] = serde_json::json!("changed");
        let mut candidate = value.clone();
        candidate.binding = serde_json::from_value(altered).unwrap();
        let fresh = store.bind(ticket.binding().clone()).unwrap();
        assert!(store.stage(fresh, candidate).is_err(), "{key}");
    }
    let mut missing = value.binding.clone();
    missing.read_set.clear();
    assert!(store.bind(missing).is_err());
    let stale = store
        .stage(store.bind(ticket.binding().clone()).unwrap(), value.clone())
        .unwrap();
    let staged = store.stage(ticket, value).unwrap();
    store.commit(&cap, staged, normal()).unwrap();
    assert!(store.commit(&cap, stale, normal()).is_err());
    assert_eq!(store.reader().unwrap().facts(10).unwrap().len(), 2);
}

#[test]
fn duplicate_and_conflict_only_advance_processed_frontier() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "dedup");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let old = store.current().unwrap().unwrap();
    let mut e = store.reader().unwrap().facts(1).unwrap().remove(0);
    e.raw_log_position = DecimalU64(1);
    e.ingest_time_ns = DecimalI64(50);
    store.ingest(&e).unwrap();
    assert_eq!(
        store.settle_duplicate().unwrap().unwrap().stage,
        ReceiptStage::FILTERED
    );
    e.raw_log_position = DecimalU64(2);
    e.event_time_ns = DecimalI64(80);
    store.ingest(&e).unwrap();
    assert_eq!(
        store.settle_duplicate().unwrap().unwrap().stage,
        ReceiptStage::REJECTED
    );
    assert_eq!(store.current().unwrap().unwrap(), old);
    assert_eq!(store.head().unwrap().1, 3);
    assert_eq!(store.head().unwrap().2, 1);
    assert_eq!(store.reader().unwrap().facts(10).unwrap().len(), 1);
    store.verify().unwrap();
}

#[test]
fn disk_full_rolls_back_ledger_and_all_state() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "diskfull");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let old = store.current().unwrap();
    let e = event(&mut store, 1);
    let (ticket, value, _pins) = proposal(&store, &e);
    let staged = store.stage(ticket, value).unwrap();
    store.set_fault(Fault::DiskFull(FaultPoint::LedgerWritten));
    assert!(store.commit(&cap, staged, normal()).is_err());
    assert_eq!(store.current().unwrap(), old);
    assert_eq!(
        store
            .db
            .query_row("SELECT count(*) FROM ledger_index", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    store.verify().unwrap();
}

#[test]
fn crash_worker() {
    let Ok(path) = std::env::var("NEMU_W02_CHILD_TEMP") else {
        return;
    };
    // This private test entry receives only the parent's freshly owned temp root.
    let mut store =
        Store::<Rehearsal>::open(std::path::PathBuf::from(path), config(), true).unwrap();
    let cap = store.take_commit_capability().unwrap();
    let point = match std::env::var("NEMU_W02_CRASH").unwrap().as_str() {
        "G" => FaultPoint::GStaged,
        "C" => FaultPoint::CStaged,
        "L" => FaultPoint::LedgerWritten,
        "before" => FaultPoint::BeforePublish,
        "sql" => FaultPoint::SqlCommitted,
        "after" => FaultPoint::AfterPublish,
        "write" => FaultPoint::ObjectWritten,
        "file" => FaultPoint::FileSynced,
        "rename" => FaultPoint::Renamed,
        "directory" => FaultPoint::DirectorySynced,
        _ => panic!("unknown fault"),
    };
    if matches!(
        point,
        FaultPoint::ObjectWritten
            | FaultPoint::FileSynced
            | FaultPoint::Renamed
            | FaultPoint::DirectorySynced
    ) {
        store.set_fault(Fault::Crash(point));
    }
    let e = event(&mut store, 1);
    let (ticket, value, _pins) = proposal(&store, &e);
    store.set_fault(Fault::Crash(point));
    let staged = store.stage(ticket, value).unwrap();
    store.commit(&cap, staged, normal()).unwrap();
    panic!("fault did not fire");
}

#[test]
fn x03_real_process_exit_at_every_durability_boundary() {
    for point in [
        "G",
        "C",
        "L",
        "before",
        "sql",
        "after",
        "write",
        "file",
        "rename",
        "directory",
    ] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "crash");
        let cap = store.take_commit_capability().unwrap();
        advance(&mut store, &cap, 0);
        let old = store.current().unwrap();
        let path = store.path().to_path_buf();
        drop(store);
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "tests::atomic::crash_worker", "--nocapture"])
            .env("NEMU_W02_CHILD_TEMP", &path)
            .env("NEMU_W02_CRASH", point)
            .output()
            .unwrap();
        assert_eq!(
            child.status.code(),
            Some(86),
            "{point}: {} {}",
            String::from_utf8_lossy(&child.stdout),
            String::from_utf8_lossy(&child.stderr)
        );
        let mut recovered = Store::<Rehearsal>::open_temporary(&root, "crash", config()).unwrap();
        if point == "after" || point == "sql" {
            assert_eq!(
                recovered.current().unwrap().unwrap().processed_position.0,
                2
            );
        } else {
            assert_eq!(recovered.current().unwrap(), old, "{point}");
        }
        recovered.verify().unwrap();
        recovered.collect_garbage().unwrap();
        recovered.verify().unwrap();
        let count = if point == "after" || point == "sql" {
            2
        } else {
            1
        };
        for table in ["facts", "ledger_index", "accepted_results"] {
            assert_eq!(
                recovered
                    .db
                    .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                        .get::<_, i64>(0))
                    .unwrap(),
                count,
                "{point}/{table}"
            );
        }
        assert_eq!(
            recovered
                .db
                .query_row(
                    "SELECT count(*) FROM snapshot_objects WHERE role='concept'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            count
        );
        println!(
            "X03 {point}: exit=86, recovered all-{}",
            if point == "after" || point == "sql" {
                "new"
            } else {
                "old"
            }
        );
    }
}

#[test]
fn all_binding_fields_are_immutable_after_dispatch() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "all-fields");
    let e = event(&mut store, 0);
    let (ticket, value, _pins) = proposal(&store, &e);
    let encoded = serde_json::to_value(&value.binding).unwrap();
    for key in encoded.as_object().unwrap().keys() {
        let mut changed = encoded.clone();
        let field = &mut changed[key];
        if field.is_string() {
            let s = field.as_str().unwrap();
            *field = serde_json::json!(if s.len() == 64 {
                "11".repeat(32)
            } else if s.parse::<u64>().is_ok() {
                "999".into()
            } else {
                format!("{s}-changed")
            });
        } else if field.is_number() {
            *field = serde_json::json!(99);
        } else if field.is_array() {
            *field = serde_json::json!([]);
        } else {
            field["schema_version"] = serde_json::json!(99);
        }
        let mut proposed = value.clone();
        proposed.binding = serde_json::from_value(changed).unwrap();
        let fresh = store.bind(ticket.binding().clone()).unwrap();
        assert!(
            store.stage(fresh, proposed).is_err(),
            "binding field {key} was mutable"
        );
    }
}

#[test]
fn later_ingress_cannot_overtake_uncommitted_input() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "fifo");
    let first = event(&mut store, 0);
    let second = event(&mut store, 1);
    let (ticket, _, _) = proposal(&store, &first);
    let mut binding = ticket.binding().clone();
    binding.event_id = second.id;
    binding.raw_log_position = second.raw_log_position;
    assert!(store.bind(binding).is_err());
    assert_eq!(store.next_raw().unwrap().unwrap().id, first.id);
}

#[test]
fn f161_exact_e1_e2_e2_survives_restart_without_second_ledger_charge() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "e1-e2-e2");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    advance(&mut store, &cap, 1);
    let old = store.current().unwrap();
    let mut e2 = store.reader().unwrap().facts(10).unwrap().pop().unwrap();
    e2.raw_log_position = DecimalU64(2);
    store.ingest(&e2).unwrap();
    assert_eq!(
        store.settle_duplicate().unwrap().unwrap().stage,
        ReceiptStage::FILTERED
    );
    drop(store);
    let store = Store::<Rehearsal>::open_temporary(&root, "e1-e2-e2", config()).unwrap();
    assert_eq!(store.head().unwrap().1, 3);
    assert_eq!(store.head().unwrap().2, 2);
    assert_eq!(store.current().unwrap(), old);
    let reader = store.reader().unwrap();
    assert_eq!(reader.facts(10).unwrap().len(), 2);
    assert_eq!(
        reader
            .ledger_at(&store.base_commit().unwrap(), 10)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn recovery_rejects_valid_but_mixed_head_and_missing_ledger() {
    for damage in ["head", "ledger"] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "mixed");
        let cap = store.take_commit_capability().unwrap();
        let first = advance(&mut store, &cap, 0);
        advance(&mut store, &cap, 1);
        if damage == "head" {
            store
                .db
                .execute(
                    "UPDATE branch_head SET snapshot=?1",
                    [first.snapshot.canonical_bytes(config().encoding).unwrap()],
                )
                .unwrap();
        } else {
            // Deliberate valid-SQL corruption, stronger than an accidental partial
            // page: the verifier must compare against the immutable manifest.
            store.db.execute_batch("DROP TRIGGER immutable_ledger_index_DELETE; DELETE FROM ledger_index WHERE entry_id='entry-1';").unwrap();
        }
        assert!(store.verify().is_err(), "{damage}");
        drop(store);
        assert!(
            Store::<Rehearsal>::open_temporary(&root, "mixed", config()).is_err(),
            "{damage}"
        );
    }
}
