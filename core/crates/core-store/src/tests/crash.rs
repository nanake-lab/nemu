use super::crash_harness::run;
use super::*;
use FaultPoint::*;

const OBJECT: &[FaultPoint] = &[ObjectWritten, FileSynced, Renamed, DirectorySynced];
const RECEIPT: &[FaultPoint] = &[
    ReceiptWritten,
    ReceiptFileSyncBefore,
    ReceiptFileSynced,
    ReceiptRenameBefore,
    ReceiptRenamed,
    ReceiptTargetSyncBefore,
    ReceiptTargetSynced,
    ReceiptDirectorySyncBefore,
    ReceiptDirectorySynced,
];
const REUSE: &[FaultPoint] = &[
    ReuseFileSyncBefore,
    ReuseFileSynced,
    ReuseDirectorySyncBefore,
    ReuseDirectorySynced,
];
const COMMIT: &[FaultPoint] = &[
    GStaged,
    CStaged,
    LedgerWritten,
    BeforePublish,
    SqlCommitted,
    AfterPublish,
];
const INGRESS: &[FaultPoint] = &[IngressBeforeSql, IngressSqlCommitted, IngressAcknowledged];
const INIT: &[FaultPoint] = &[
    InitRootCreated,
    InitRootSynced,
    InitObjectsCreated,
    InitObjectsSynced,
    InitSqlCommitted,
    InitDbDirectorySynced,
    ReceiptRootCreated,
    ReceiptKindCreated,
    ReceiptKindSynced,
    ReceiptRootSynced,
    ReceiptParentSynced,
];
const GC: &[FaultPoint] = &[GcDeleted, GcDirectorySyncBefore, GcDirectorySynced];
pub(super) fn points() -> Vec<FaultPoint> {
    [
        OBJECT,
        RECEIPT,
        REUSE,
        COMMIT,
        INGRESS,
        INIT,
        GC,
        &[
            IngressAllocated,
            AuditBeforeSql,
            AuditSqlCommitted,
            MigrationBeforeSql,
            MigrationSqlCommitted,
        ],
    ]
    .concat()
}
fn tag<T: WireMessage>() -> String {
    core_types::contract::Schema::builtin()
        .unwrap()
        .message(T::NAME)
        .unwrap()
        .tag
        .clone()
}

fn seed(root: &TemporaryRoot) -> CoreTransactionV2 {
    let mut store = rehearsal(root, "crash");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    store.current().unwrap().unwrap()
}
fn expected_new() -> CoreTransactionV2 {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "oracle");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    advance(&mut store, &cap, 1);
    store.current().unwrap().unwrap()
}
fn recover(root: &TemporaryRoot, expected: &CoreTransactionV2, raw: i64, label: &str) {
    let mut store = Store::<Rehearsal>::open_temporary(root, "crash", config()).unwrap();
    let n = expected.processed_position.0 as i64;
    // Equality checks every state pointer, every L entry, all binding fields,
    // accepted results, concepts, processed position and anchor, not one counter.
    assert_eq!(&store.current().unwrap().unwrap(), expected, "{label}");
    assert_eq!(store.head().unwrap().1, n as u64);
    assert_eq!(store.head().unwrap().2, expected.anchor_time_ns.0);
    for (table, count) in [
        ("raw_ingress", raw),
        ("facts", n),
        ("ledger_index", n),
        ("accepted_results", n),
        ("snapshots", n),
    ] {
        assert_eq!(
            store
                .db
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            count,
            "{label}/{table}"
        );
    }
    assert_eq!(
        store
            .db
            .query_row(
                "SELECT count(*) FROM snapshot_objects WHERE role='concept'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        n
    );
    let reader = Reader::<Rehearsal>::open_temporary(root, "crash", config()).unwrap();
    assert_eq!(reader.facts(10).unwrap().len(), n as usize);
    assert_eq!(
        reader
            .ledger_at(&store.base_commit().unwrap(), 10)
            .unwrap()
            .len(),
        n as usize
    );
    drop(reader);
    let captured = crate::witness::capture(store.path(), &store.objects).unwrap();
    for kind in ["d", "commits"] {
        assert_eq!(
            captured[kind],
            crate::witness::expected(&store.db, &store.objects, kind).unwrap(),
            "{label}/{kind}"
        );
    }
    store.verify().unwrap(); // includes complete object closure + index manifests
    store.collect_garbage().unwrap();
    store.verify().unwrap();
    drop(store);
    let store = Store::<Rehearsal>::open_temporary(root, "crash", config()).unwrap();
    assert_eq!(&store.current().unwrap().unwrap(), expected);
    println!(
        "X03 {label}: exit=86; state={n}; D={raw}; closure/L/H/concepts/accepted/receipts/reopen/GC verified"
    );
}

#[test]
fn x03_ingress_and_transaction_publication_boundaries() {
    let new = expected_new();
    for point in COMMIT {
        let root = TemporaryRoot::new().unwrap();
        let old = seed(&root);
        run(&root, "commit", *point, 1, None);
        let expected = if matches!(point, SqlCommitted | AfterPublish) {
            &new
        } else {
            &old
        };
        recover(&root, expected, 2, &format!("commit/{point:?}"));
    }
    let event_tag = tag::<EventEnvelope>();
    for point in [OBJECT, INGRESS].concat() {
        let root = TemporaryRoot::new().unwrap();
        let old = seed(&root);
        run(
            &root,
            "ingress",
            point,
            1,
            if OBJECT.contains(&point) {
                Some(&event_tag)
            } else {
                None
            },
        );
        let raw = if matches!(point, IngressSqlCommitted | IngressAcknowledged) {
            2
        } else {
            1
        };
        recover(&root, &old, raw, &format!("ingress/{point:?}"));
    }
}

#[test]
fn x03_receipt_publish_and_repair_can_crash_again() {
    let new = expected_new();
    for operation in ["ingress", "commit"] {
        for repair in [false, true] {
            for point in RECEIPT {
                let root = TemporaryRoot::new().unwrap();
                let old = seed(&root);
                if repair {
                    run(
                        &root,
                        operation,
                        if operation == "ingress" {
                            IngressSqlCommitted
                        } else {
                            SqlCommitted
                        },
                        1,
                        None,
                    );
                    // Do not open in the parent: it would repair the tail before
                    // the second process can exercise interrupted reconciliation.
                    run(&root, "repair", *point, 1, None);
                } else {
                    run(&root, operation, *point, 1, None);
                }
                recover(
                    &root,
                    if operation == "commit" { &new } else { &old },
                    2,
                    &format!("{operation}/repair={repair}/{point:?}"),
                );
            }
        }
    }
}

#[test]
fn x03_targeted_r_segments_manifest_and_reused_objects() {
    for point in OBJECT {
        for (operation, object_tag, nth) in [
            ("r", tag::<TendencyV3>(), 2),
            ("r", tag::<RSnapshotV3>(), 1),
            ("manifest-second", tag::<CoreTransactionV2>(), 2),
        ] {
            let root = TemporaryRoot::new().unwrap();
            let old = seed(&root);
            run(&root, operation, *point, nth, Some(&object_tag));
            recover(
                &root,
                &old,
                2,
                &format!("{operation}/{object_tag}/{nth}/{point:?}"),
            );
        }
    }
    for point in REUSE {
        let root = TemporaryRoot::new().unwrap();
        let old = seed(&root);
        run(
            &root,
            "reuse",
            *point,
            2,
            Some(&tag::<FastStateSnapshotV2>()),
        );
        recover(
            &root,
            &old,
            1,
            &format!("reuse/FastStateSnapshotV2/2/{point:?}"),
        );
    }
}

#[test]
fn initialization_and_gc_crash_boundaries_have_separate_expectations() {
    for point in INIT {
        let occurrences: &[usize] = if matches!(point, ReceiptKindCreated | ReceiptKindSynced) {
            &[1, 2]
        } else {
            &[1]
        };
        for nth in occurrences {
            let root = TemporaryRoot::new().unwrap();
            run(&root, "init", *point, *nth, None);
            let opened = Store::<Rehearsal>::open_temporary(&root, "crash", config());
            // Before both receipt namespaces exist initialization must fail closed.
            let complete = matches!(point, ReceiptRootSynced | ReceiptParentSynced)
                || (matches!(point, ReceiptKindCreated | ReceiptKindSynced) && *nth == 2);
            assert_eq!(opened.is_ok(), complete, "init/{point:?}/{nth}");
            if let Ok(store) = opened {
                store.verify().unwrap();
                assert!(store.current().unwrap().is_none());
            }
            assert!(
                Store::<Rehearsal>::create_temporary(
                    &root,
                    "crash",
                    header::<Rehearsal>(),
                    "test-branch",
                    config()
                )
                .is_err()
            );
            println!(
                "INIT {point:?}/{nth}: exit=86; {}",
                if complete {
                    "verified empty store"
                } else {
                    "partial initialization refused"
                }
            );
        }
    }
    for (point, nth) in [
        (GcDeleted, 1),
        (GcDeleted, 2),
        (GcDirectorySyncBefore, 1),
        (GcDirectorySynced, 1),
    ] {
        let root = TemporaryRoot::new().unwrap();
        let old = seed(&root);
        let store = Store::<Rehearsal>::open_temporary(&root, "crash", config()).unwrap();
        for text in ["unpublished-one", "unpublished-two"] {
            drop(
                store
                    .prepare(
                        &UserText {
                            schema_version: 1,
                            text: text.into(),
                        },
                        "1",
                    )
                    .unwrap(),
            );
        }
        drop(store);
        run(&root, "gc", point, nth, None);
        recover(&root, &old, 1, &format!("gc/{point:?}/{nth}"));
    }
}
