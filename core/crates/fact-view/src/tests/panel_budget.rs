use super::*;
use core_store::Experiment;

fn request(id: &str, text: Option<&str>) -> QueryV2 {
    QueryV2 {
        schema_version: 1,
        query_id: id.into(),
        kind: QueryKindV2::QUERY_MEMORY,
        filter: text.map(|text| QueryFilter {
            schema_version: 1,
            text: Some(text.into()),
            from: None,
            to: None,
            kinds: vec![],
            include_retracted: None,
        }),
        limit: DecimalU64(10),
    }
}

fn empty_response(request: &QueryV2, status: QueryStatus) -> QueryResultV2 {
    QueryResultV2 {
        schema_version: 1,
        query_id: request.query_id.clone(),
        status,
        items: vec![],
        growth_entries: vec![],
    }
}

fn encoded_len(response: &QueryResultV2) -> usize {
    response
        .canonical_bytes(crate::query::LIMITS)
        .unwrap()
        .len()
}

fn check_empty_status(view: &FactView<'_, Experiment>, grant: &Access, status: QueryStatus) {
    for id in ["short".into(), "q".repeat(70_000), "查询".repeat(12_000)] {
        let request = request(&id, Some("absent-wire-budget-query"));
        // All of these IDs are legal under the frozen request contract.
        request.canonical_bytes(crate::query::LIMITS).unwrap();
        let minimum = encoded_len(&empty_response(&request, status));
        for bytes in [0, minimum - 1, minimum, minimum + 1, 65_536] {
            let mut budgets = Budgets::default();
            budgets.user_view.bytes = bytes;
            let result = view.user_view(&request, grant, &budgets, None);
            if bytes < minimum {
                assert!(
                    matches!(result, Err(Error::Invalid(_))),
                    "must reject an envelope requiring {minimum} bytes under a {bytes}-byte budget"
                );
            } else {
                let (response, execution) = result.unwrap();
                assert_eq!(response.status, status);
                assert_eq!(response.query_id, id);
                assert!(response.items.is_empty() && response.growth_entries.is_empty());
                assert!(encoded_len(&response) <= bytes);
                assert_eq!(execution.projection.status, status);
                assert!(execution.next_page.is_none());
            }
        }
    }
}

#[test]
fn sys05_empty_panel_statuses_enforce_the_entire_response_envelope() {
    let mut world = World::new();
    world.text("wire budget positive fact");
    let reader = world.store.reader().unwrap();
    let view = FactView::rebuild(&reader, &world.prefix(), 100, None).unwrap();
    check_empty_status(&view, &access(), QueryStatus::NO_MATCH);
    check_empty_status(&view, &Access::default(), QueryStatus::DENIED);
    world.text("a new commit invalidates the old memory index");
    check_empty_status(&view, &access(), QueryStatus::INDEX_LAGGING);
}

#[test]
fn sys05_wire_trimming_can_remove_every_item_at_the_exact_envelope_limit() {
    let mut world = World::new();
    world.text("wire budget positive fact");
    let reader = world.store.reader().unwrap();
    let prefix = world.prefix();
    let view = FactView::rebuild(&reader, &prefix, 100, None).unwrap();
    let budgets = Budgets::default();
    let mut request = request("q", None);
    let overhead = encoded_len(&empty_response(&request, QueryStatus::BUDGET_EXHAUSTED)) - 1;
    request.query_id = "q".repeat(budgets.user_view.bytes - overhead);
    assert_eq!(
        encoded_len(&empty_response(&request, QueryStatus::BUDGET_EXHAUSTED)),
        budgets.user_view.bytes
    );
    let positive = view
        .execute(&query(&prefix, Purpose::UserView, ""), &access(), None)
        .unwrap();
    assert!(!positive.projection.items.is_empty());
    let (response, execution) = view.user_view(&request, &access(), &budgets, None).unwrap();
    assert_eq!(response.status, QueryStatus::BUDGET_EXHAUSTED);
    assert_eq!(response.query_id, request.query_id);
    assert_eq!(encoded_len(&response), budgets.user_view.bytes);
    assert!(response.items.is_empty() && execution.projection.items.is_empty());
    assert!(execution.memories().is_empty() && execution.trace.returned.is_empty());
    assert!(execution.projection.truncated && execution.next_page.is_none());
}

#[test]
fn sys05_wire_trimmed_pages_preserve_the_cursor_and_all_original_items() {
    let mut world = World::new();
    for n in 0..3 {
        world.text(&format!("wire-page-{n} original exact text"));
    }
    let reader = world.store.reader().unwrap();
    let view = FactView::rebuild(&reader, &world.prefix(), 100, None).unwrap();
    let budgets = Budgets::default();
    let mut request = request("q", Some("wire-page"));
    let (all, _) = view.user_view(&request, &access(), &budgets, None).unwrap();
    assert_eq!(all.items.len(), 3);
    let expected = all.items.clone();
    let mut one = all;
    one.items.truncate(1);
    // Retrieval can afford all three; the ID leaves room for one wire item.
    request.query_id = "q".repeat(budgets.user_view.bytes - encoded_len(&one) + 1);
    one.query_id = request.query_id.clone();
    assert_eq!(encoded_len(&one), budgets.user_view.bytes);
    let mut after = None;
    let mut found = Vec::new();
    for page in 0..3 {
        let (response, execution) = view
            .user_view_page(&request, &access(), &budgets, after.as_ref(), None)
            .unwrap();
        assert_eq!(response.query_id, request.query_id);
        assert_eq!(response.items.len(), 1);
        assert!(encoded_len(&response) <= budgets.user_view.bytes);
        assert_eq!(execution.projection.items.len(), 1);
        assert_eq!(execution.memories().len(), 1);
        assert_eq!(execution.trace.returned.len(), 1);
        found.extend(response.items);
        after = execution.next_page;
        if page < 2 {
            assert_eq!(response.status, QueryStatus::BUDGET_EXHAUSTED);
            assert!(after.is_some());
        } else {
            assert_eq!(response.status, QueryStatus::OK);
            assert!(after.is_none());
        }
    }
    assert_eq!(found, expected);
}
