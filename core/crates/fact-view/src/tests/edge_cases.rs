use super::*;

#[test]
fn p522_active_memory_messages_never_emit_growth_or_serve_other_services() {
    let mut w = World::new();
    w.text("synthetic memory control");
    let reader = w.store.reader().unwrap();
    let view = FactView::rebuild(&reader, &w.prefix(), 100, None).unwrap();
    let mut request = QueryV2 {
        schema_version: 1,
        query_id: "p522-memory".into(),
        kind: QueryKindV2::QUERY_MEMORY,
        filter: None,
        limit: DecimalU64(10),
    };
    let (response, _) = view
        .user_view(&request, &access(), &Budgets::default(), None)
        .unwrap();
    assert_eq!(response.status, QueryStatus::OK);
    assert_eq!(response.items.len(), 1);
    assert!(response.growth_entries.is_empty());
    assert_eq!(
        QueryResultV2::from_canonical(
            &response.canonical_bytes(crate::query::LIMITS).unwrap(),
            crate::query::LIMITS
        )
        .unwrap(),
        response
    );
    for kind in [
        QueryKindV2::QUERY_ACTIONS,
        QueryKindV2::QUERY_RUN_HISTORY,
        QueryKindV2::QUERY_GROWTH,
    ] {
        request.kind = kind;
        let (denied, _) = view
            .user_view(&request, &access(), &Budgets::default(), None)
            .unwrap();
        assert_eq!(denied.status, QueryStatus::DENIED);
        assert!(denied.items.is_empty() && denied.growth_entries.is_empty());
    }
}

#[test]
fn panel_time_and_payload_filters_are_compatible_and_invalid_dates_fail() {
    let mut q = query("prefix", Purpose::UserView, "");
    let mut filter = QueryFilter {
        schema_version: 1,
        text: None,
        from: Some("1970-01-01".into()),
        to: Some("1970-01-01".into()),
        kinds: vec!["user_text".into()],
        include_retracted: None,
    };
    q.filter(&filter).unwrap();
    assert_eq!(q.from_ns, Some(0));
    assert_eq!(q.to_ns, Some(86_400_000_000_000 - 1));
    assert_eq!(q.payload_types.len(), 1);
    filter.from = Some("1970-01-01T08:00:00+08:00".into());
    q.filter(&filter).unwrap();
    assert_eq!(q.from_ns, Some(0));
    filter.to = Some("invalid-date".into());
    assert!(q.filter(&filter).is_err());
}

#[test]
fn withdrawing_object_reference_also_suppresses_correction_replacements() {
    let mut w = World::new();
    let old = w.text("synthetic-item old detail");
    let replacement = w.add(
        &UserText {
            schema_version: 1,
            text: "synthetic-item corrected detail".into(),
        },
        "user",
        std::slice::from_ref(&old.id),
        None,
    );
    let control = w.text("synthetic-item unrelated positive control");
    w.add(
        &UserRetraction {
            schema_version: 1,
            target_refs: vec![core_types::scalar::hex(&old.payload_ref.digest.0)],
            note: Some("synthetic-item old detail".into()),
        },
        "user",
        &[],
        None,
    );
    let reader = w.store.reader().unwrap();
    let p = w.prefix();
    let view = FactView::rebuild(&reader, &p, 100, None).unwrap();
    let task = view
        .execute(&query(&p, Purpose::Task, "synthetic-item"), &access(), None)
        .unwrap();
    assert_eq!(
        task.projection
            .items
            .iter()
            .map(|i| &i.source_id)
            .collect::<Vec<_>>(),
        vec![&control.id]
    );
    let core = view
        .execute(&query(&p, Purpose::Core, "synthetic-item"), &access(), None)
        .unwrap();
    assert!(
        core.projection
            .items
            .iter()
            .any(|i| i.source_id == replacement.id)
    );
    let audit = view
        .execute(
            &query(&p, Purpose::Audit, "synthetic-item"),
            &access(),
            None,
        )
        .unwrap();
    assert_eq!(audit.projection.items.len(), 3);
    assert_eq!(
        task.trace
            .audit_record(&task.projection)
            .unwrap()
            .audit_schema_tag,
        "echo.fact-query-trace"
    );
}
