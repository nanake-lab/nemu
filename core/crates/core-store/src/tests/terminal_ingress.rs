mod migration_checks;
use super::*;
use crate::migration::format;
use rusqlite::{Connection, OpenFlags};

pub(super) fn session() -> TerminalSession {
    TerminalSession {
        schema_version: 1,
        session_id: "server-session".into(),
        installation_id: "installation".into(),
        protocol_version: 1,
        capabilities: vec![IngressKind::INGRESS_EVENT],
        budget: TerminalBudget {
            schema_version: 1,
            version: "fixture-1".into(),
            max_frame_bytes: 8192,
            max_chunk_bytes: 1024,
            max_object_bytes: 4096,
            max_uploads: 2,
            max_upload_bytes: 8192,
        },
    }
}
pub(super) fn input(event: EventEnvelope) -> TerminalEvent {
    TerminalEvent {
        schema_version: 1,
        id: event.id,
        stable_source_key: event.stable_source_key,
        kind: event.kind,
        event_time_ns: event.event_time_ns,
        parents: event.parents,
        payload_ref: event.payload_ref,
        provenance: event.provenance,
        payload_digest: event.payload_digest,
        action_id: event.action_id,
        trial_id: event.trial_id,
        correction_of: event.correction_of,
        derivation: event.derivation,
    }
}
pub(super) fn budget() -> IngressAuditBudget {
    IngressAuditBudget {
        schema_version: 1,
        version: "fixture-audit-1".into(),
        max_records: 3,
        max_bytes: 4096,
    }
}
fn readonly(root: &TemporaryRoot, name: &str) -> Connection {
    Connection::open_with_flags(
        root.store_path(name).unwrap().join("store.sqlite"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap()
}
fn rows(db: &Connection) -> Vec<(String, Vec<Vec<rusqlite::types::Value>>)> {
    let mut tables=db.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT IN ('metadata','ingress_rejections') ORDER BY name").unwrap();
    tables
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(|name| {
            let name = name.unwrap();
            let n = db
                .prepare(&format!("SELECT * FROM {name}"))
                .unwrap()
                .column_count();
            let order = (1..=n).map(|v| v.to_string()).collect::<Vec<_>>().join(",");
            let mut s = db
                .prepare(&format!("SELECT * FROM {name} ORDER BY {order}"))
                .unwrap();
            let values = s
                .query_map([], |r| {
                    (0..n)
                        .map(|i| r.get(i))
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .unwrap()
                .map(|v| v.unwrap())
                .collect();
            (name, values)
        })
        .collect()
}

#[test]
fn terminal_positions_follow_durable_tail_after_restart_and_same_key_retry() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "ingress");
    let (e, _pin) = unpersisted_event(&mut store, 0);
    let input = input(e);
    for position in 0..3 {
        let receipt = store.ingest_terminal(&input, &session()).unwrap();
        assert_eq!(receipt.raw_log_position, Some(DecimalU64(position)));
        assert!(receipt.ingest_time_ns.unwrap().0 > 0);
        assert_eq!(receipt.stable_source_key, input.stable_source_key);
    }
    assert_eq!(store.head().unwrap(), (None, 0, 0));
    drop(store);
    let mut store = Store::<Rehearsal>::open_temporary(&root, "ingress", config()).unwrap();
    assert_eq!(
        store
            .ingest_terminal(&input, &session())
            .unwrap()
            .raw_log_position,
        Some(DecimalU64(3))
    );
    let mut stmt = store
        .db
        .prepare("SELECT event FROM raw_ingress ORDER BY position")
        .unwrap();
    for (n, row) in stmt
        .query_map([], |r| r.get::<_, Vec<u8>>(0))
        .unwrap()
        .enumerate()
    {
        let e = EventEnvelope::from_canonical(&row.unwrap(), config().encoding).unwrap();
        assert_eq!(e.raw_log_position.0, n as u64);
        assert_eq!(e.device, "installation");
        assert_eq!(e.session, "server-session");
    }
    store.verify().unwrap();
}

#[test]
fn audit_is_bounded_append_only_and_preserves_every_authority_row() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "audit");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let before = rows(&store.db);
    for n in 0..3 {
        assert_eq!(
            store
                .audit_ingress_rejection(IngressReason::AUTH_FAILED, None, &budget())
                .unwrap()
                .position
                .0,
            n
        );
    }
    assert!(matches!(
        store.audit_ingress_rejection(IngressReason::INVALID_FRAME, None, &budget()),
        Err(Error::Backpressure(IngressReason::AUDIT_FULL))
    ));
    assert_eq!(rows(&store.db), before);
    assert!(
        store
            .db
            .execute("DELETE FROM ingress_rejections", [])
            .is_err()
    );
    assert!(
        store
            .db
            .execute("UPDATE ingress_rejections SET record=x'00'", [])
            .is_err()
    );
    drop(store);
    let store = Store::<Rehearsal>::open_temporary(&root, "audit", config()).unwrap();
    assert_eq!(store.ingress_rejections().unwrap().len(), 3);
    assert_eq!(rows(&store.db), before);
    store.verify().unwrap();
    drop(store);
    assert!(Store::<Rehearsal>::migrate_temporary(&root, "audit", config(), 1).is_err());
    assert_eq!(format(&readonly(&root, "audit")).unwrap(), 2);
}

#[test]
fn capacity_is_typed_and_retry_keeps_key_without_consuming_position() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "bounds");
    let (e, _pin) = unpersisted_event(&mut store, 0);
    let e = input(e);
    store.objects.config.max_pending_records = 1;
    store.ingest_terminal(&e, &session()).unwrap();
    let error = store.ingest_terminal(&e, &session()).unwrap_err();
    assert!(matches!(
        error,
        Error::Backpressure(IngressReason::PENDING_RECORDS_FULL)
    ));
    let receipt =
        core_types::terminal_ingress::rejected_receipt(&e, error.safe_ingress_reason().unwrap());
    receipt.canonical_bytes(config().encoding).unwrap();
    assert_eq!(receipt.stable_source_key, e.stable_source_key);
    assert_eq!(
        core_types::terminal_ingress::retry_disposition(receipt.reason_code.unwrap()),
        core_types::terminal_ingress::RetryDisposition::SameSourceKeyAfterCapacity
    );
    assert!(
        Error::Io(std::io::ErrorKind::StorageFull.into())
            .safe_ingress_reason()
            .is_none()
    );
    store.objects.config.max_pending_records = 3;
    store.objects.config.max_pending_bytes = 1;
    assert!(matches!(
        store.ingest_terminal(&e, &session()),
        Err(Error::Backpressure(IngressReason::PENDING_BYTES_FULL))
    ));
    store.objects.config.max_pending_bytes = config().max_pending_bytes;
    assert_eq!(
        store
            .ingest_terminal(&e, &session())
            .unwrap()
            .raw_log_position,
        Some(DecimalU64(1))
    );
    let mut b = budget();
    b.max_bytes = 1;
    assert!(matches!(
        store.audit_ingress_rejection(IngressReason::AUTH_FAILED, None, &b),
        Err(Error::Backpressure(IngressReason::AUDIT_FULL))
    ));
    assert!(store.ingress_rejections().unwrap().is_empty());
}

#[test]
fn allocation_and_audit_io_failures_never_issue_success() {
    for point in [
        FaultPoint::IngressAllocated,
        FaultPoint::IngressBeforeSql,
        FaultPoint::IngressSqlCommitted,
    ] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "fault");
        let (e, _pin) = unpersisted_event(&mut store, 0);
        let e = input(e);
        store.set_fault(Fault::DiskFull(point));
        assert!(matches!(
            store.ingest_terminal(&e, &session()),
            Err(Error::Io(_))
        ));
        drop(store);
        let mut store = Store::<Rehearsal>::open_temporary(&root, "fault", config()).unwrap();
        let expected = u64::from(point == FaultPoint::IngressSqlCommitted);
        assert_eq!(
            store
                .ingest_terminal(&e, &session())
                .unwrap()
                .raw_log_position,
            Some(DecimalU64(expected))
        );
        store.verify().unwrap();
    }
    for point in [
        FaultPoint::AuditBeforeSql,
        FaultPoint::AuditSqlCommitted,
        FaultPoint::ReceiptFileSyncBefore,
    ] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "fault");
        let before = rows(&store.db);
        store.set_fault(Fault::DiskFull(point));
        assert!(matches!(
            store.audit_ingress_rejection(IngressReason::AUTH_FAILED, None, &budget()),
            Err(Error::Io(_))
        ));
        drop(store);
        let mut store = Store::<Rehearsal>::open_temporary(&root, "fault", config()).unwrap();
        assert_eq!(rows(&store.db), before);
        let expected = u64::from(point != FaultPoint::AuditBeforeSql);
        assert_eq!(
            store
                .audit_ingress_rejection(IngressReason::INVALID_FRAME, None, &budget())
                .unwrap()
                .position
                .0,
            expected
        );
        store.verify().unwrap();
    }
}

fn files(path: &std::path::Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut values = Vec::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            values.extend(files(&p))
        } else {
            values.push((p.clone(), std::fs::read(p).unwrap()))
        }
    }
    values.sort_by(|a, b| a.0.cmp(&b.0));
    values
}
#[test]
fn authentication_token_is_redacted_and_cannot_be_stored() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "secret");
    let auth = TerminalAuthenticate {
        schema_version: 1,
        request_id: "auth".into(),
        installation_id: "install".into(),
        token: HexBytes((193..225).collect()),
        protocol_versions: vec![1],
    };
    let token = auth.token.0.clone();
    let text = core_types::scalar::hex(&token);
    let log = format!("{auth:?}");
    assert!(!log.contains(&text));
    assert!(log.contains("REDACTED"));
    assert!(store.prepare(&auth, "1").is_err());
    store
        .audit_ingress_rejection(IngressReason::AUTH_FAILED, None, &budget())
        .unwrap();
    drop(store);
    for (_, bytes) in files(root.path()) {
        assert!(!bytes.windows(token.len()).any(|v| v == token));
        assert!(!bytes.windows(text.len()).any(|v| v == text.as_bytes()));
    }
}

#[test]
fn process_crashes_recover_allocation_audit_and_both_migration_directions() {
    for point in [
        FaultPoint::IngressAllocated,
        FaultPoint::IngressBeforeSql,
        FaultPoint::IngressSqlCommitted,
    ] {
        let root = TemporaryRoot::new().unwrap();
        drop(rehearsal(&root, "crash"));
        super::crash_harness::run(&root, "terminal", point, 1, None);
        let mut store = Store::<Rehearsal>::open_temporary(&root, "crash", config()).unwrap();
        let (e, _pin) = unpersisted_event(&mut store, 0);
        let r = store.ingest_terminal(&input(e), &session()).unwrap();
        assert_eq!(
            r.raw_log_position.unwrap().0,
            u64::from(point == FaultPoint::IngressSqlCommitted)
        );
        store.verify().unwrap();
    }
    for point in [
        FaultPoint::AuditBeforeSql,
        FaultPoint::AuditSqlCommitted,
        FaultPoint::ReceiptRenamed,
    ] {
        let root = TemporaryRoot::new().unwrap();
        drop(rehearsal(&root, "crash"));
        super::crash_harness::run(&root, "audit", point, 1, None);
        let store = Store::<Rehearsal>::open_temporary(&root, "crash", config()).unwrap();
        assert_eq!(
            store.ingress_rejections().unwrap().len(),
            usize::from(point != FaultPoint::AuditBeforeSql)
        );
        assert_eq!(store.head().unwrap(), (None, 0, 0));
        store.verify().unwrap();
    }
    for target in [1, 2] {
        for point in [
            FaultPoint::MigrationBeforeSql,
            FaultPoint::MigrationSqlCommitted,
        ] {
            let root = TemporaryRoot::new().unwrap();
            let mut store = rehearsal(&root, "crash");
            let cap = store.take_commit_capability().unwrap();
            advance(&mut store, &cap, 0);
            let before = rows(&store.db);
            drop(store);
            if target == 2 {
                Store::<Rehearsal>::migrate_temporary(&root, "crash", config(), 1).unwrap();
            }
            super::crash_harness::run(
                &root,
                if target == 2 {
                    "migrate-up"
                } else {
                    "migrate-down"
                },
                point,
                1,
                None,
            );
            let db = readonly(&root, "crash");
            let current = format(&db).unwrap();
            assert_eq!(
                current,
                if point == FaultPoint::MigrationSqlCommitted {
                    target
                } else {
                    3 - target
                }
            );
            assert_eq!(rows(&db), before);
            drop(db);
            if current == 1 {
                Store::<Rehearsal>::migrate_temporary(&root, "crash", config(), 2).unwrap();
            }
            Store::<Rehearsal>::open_temporary(&root, "crash", config())
                .unwrap()
                .verify()
                .unwrap();
        }
    }
}
