use super::*;
use core_types::scalar::hex;

#[test]
fn sys17_v2_v4_event_history_and_rollback_preserve_bytes() {
    let root = TemporaryRoot::new().unwrap();
    let mut s = format_two(&root, "mixed");
    let e = event(&mut s, 0);
    let (_, old, _pins) = proposal(&s, &e);
    let id = super::legacy_v2::publish(&mut s, &e, &old, None);
    let original = s.reader().unwrap().current_snapshot().unwrap().unwrap();
    assert_eq!(original.object_type, "echo.transaction-v2");
    drop(s);
    let migrate = |n| Store::<Rehearsal>::migrate_temporary(&root, "mixed", config(), n);
    for target in [3, 2, 3] {
        let failed = Store::<Rehearsal>::migrate_format(
            root.store_path("mixed").unwrap(),
            config(),
            true,
            target,
            Fault::DiskFull(FaultPoint::MigrationBeforeSql),
        );
        assert!(failed.is_err());
        let committed = Store::<Rehearsal>::migrate_format(
            root.store_path("mixed").unwrap(),
            config(),
            true,
            target,
            Fault::DiskFull(FaultPoint::MigrationSqlCommitted),
        );
        assert!(committed.is_err());
        // SQL publication is authoritative even when no success reached the caller.
        assert!(migrate(target).is_err());
        let db = rusqlite::Connection::open_with_flags(
            root.store_path("mixed").unwrap().join("store.sqlite"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        assert_eq!(crate::migration::format(&db).unwrap(), target);
        drop(db);
        let r = Reader::<Rehearsal>::open_temporary(&root, "mixed", config()).unwrap();
        assert_eq!(r.current_snapshot().unwrap().unwrap(), original);
        assert_eq!(r.event_snapshot(&original).unwrap(), old);
        assert!(r.read::<CoreTransactionV4>(&original).is_err());
    }
    let mut s = Store::<Rehearsal>::open_temporary(&root, "mixed", config()).unwrap();
    let cap = s.take_commit_capability().unwrap();
    let second = advance(&mut s, &cap, 1);
    assert_eq!(second.snapshot.object_type, "echo.transaction-v4");
    let r = s.reader().unwrap();
    let event = r.event_snapshot(&second.snapshot).unwrap();
    assert_eq!(event.binding.event_id, "event-1");
    assert_eq!(
        r.transactions_at(&second.commit_id, 2).unwrap(),
        [old.clone(), event]
    );
    assert_eq!(r.transactions_at(&id, 1).unwrap(), [old]);
    assert!(r.read::<CoreTransactionV2>(&second.snapshot).is_err());
    let mut unknown = original.clone();
    unknown.object_type = "unknown".into();
    assert!(r.event_snapshot(&unknown).is_err());
    drop(r);
    drop(s);
    assert!(migrate(2).is_err());
    let reopened = Store::<Rehearsal>::open_temporary(&root, "mixed", config()).unwrap();
    assert_eq!(reopened.base_commit().unwrap(), second.commit_id);
    assert_eq!(hex(&original.digest.0), id);
}

#[test]
fn bytes_are_type_scoped_and_verified_before_use() {
    let root = TemporaryRoot::new().unwrap();
    let mut s = rehearsal(&root, "bytes");
    let normal = s
        .prepare(
            &UserText {
                schema_version: 1,
                text: "fixture".into(),
            },
            "1",
        )
        .unwrap();
    let body = s.objects.read(normal.reference()).unwrap();
    let raw = s.prepare_raw(&body).unwrap();
    let mut same_digest = normal.reference().clone();
    same_digest.digest = raw.reference().digest.clone();
    assert_ne!(
        s.objects.path(&same_digest).unwrap(),
        s.objects.path(raw.reference()).unwrap()
    );
    assert!(
        s.reader()
            .unwrap()
            .read::<UserText>(raw.reference())
            .is_err()
    );
    assert_ne!(
        s.objects.path(raw.reference()).unwrap(),
        s.objects.path(normal.reference()).unwrap()
    );
    assert!(s.objects.closure(raw.reference()).unwrap() == [raw.reference().clone()]);
    // Raw leaves may have arbitrary binary prefixes and invalid UTF-8; only a text
    // consumer decodes UTF-8. Neither NCV1 nor the R segment header dispatches these.
    assert_eq!(
        s.objects
            .read(s.prepare_raw(b"NSI1\xff").unwrap().reference())
            .unwrap(),
        b"NSI1\xff"
    );
    let bad = s.prepare_raw(b"valid").unwrap();
    std::fs::write(s.objects.path(bad.reference()).unwrap(), b"false").unwrap();
    assert!(s.objects.read(bad.reference()).is_err());
    std::fs::write(s.objects.path(bad.reference()).unwrap(), b"longer").unwrap();
    assert!(s.objects.read(bad.reference()).is_err());
    s.objects.config.max_object_bytes = 2;
    assert!(s.prepare_raw(b"123").is_err());
}

#[test]
fn request_body_checks_keys_and_preserves_text_and_response_bytes() {
    let root = TemporaryRoot::new().unwrap();
    let s = rehearsal(&root, "request");
    let text = r#"password: cookie: Bearer xyz sk-abcdefgh12 and "headers": {}"#;
    let request =
        serde_json::to_vec(&serde_json::json!({"messages": [{"content": text}]})).unwrap();
    let sent = s.prepare_request_body(&request).unwrap();
    assert_eq!(s.objects.read(sent.reference()).unwrap(), request);
    for response in [
        text.as_bytes(),
        br#"{"headers":{},"Authorization":"fixture"}"#,
        b"\xff\nAuthorization: fixture",
        br#"{"response":0,"X-Auth-Token":"fixture"#,
    ] {
        let raw = s.prepare_raw(response).unwrap();
        assert_eq!(s.objects.read(raw.reference()).unwrap(), response);
    }

    let files = || std::fs::read_dir(s.path().join("objects")).unwrap().count();
    let before = files();
    for body in [
        br#"{"Authorization":"fixture"}"#.as_slice(),
        br#"{"messages":[],"X-Goog-Api-Key":"fixture"}"#,
        br#"{"X-Auth-Token":"fixture"}"#,
        br#"{"x":{"API_Key":"fixture"},"x":0}"#,
        br#"{"messages":[],"headers":{"Accept":"text/plain"}}"#,
        br#"{"messages":[{"nested":{"Headers":null}}]}"#,
        br#"{"messages":[{"\u0041uthorization":null}]}"#,
        br#"["headers"]"#,
        b"Authorization: fixture\r\n\r\n{}",
    ] {
        assert!(s.prepare_request_body(body).is_err());
    }
    assert_eq!(files(), before);
}
