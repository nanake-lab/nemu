use super::*;
use std::collections::BTreeSet;

fn panel(text: Option<&str>, limit: u64) -> QueryV2 {
    QueryV2 {
        schema_version: 1,
        query_id: "v276-v288".into(),
        kind: QueryKindV2::QUERY_MEMORY,
        filter: text.map(|text| QueryFilter {
            schema_version: 1,
            text: Some(text.into()),
            from: None,
            to: None,
            kinds: vec![],
            include_retracted: None,
        }),
        limit: DecimalU64(limit),
    }
}

#[test]
fn v288_all_six_accepted_reactions_are_marked_and_core_cannot_recall_them() {
    let mut w = World::new();
    w.all_reactions = true;
    w.event_time = Some(1_700_000_000_000_000_123);
    let fact = w.text("synthetic exact original 原文\nsecond line");
    let r = w.store.reader().unwrap();
    let p = w.prefix();
    let view = FactView::rebuild(&r, &p, 100, None).unwrap();
    let core = view
        .execute(&query(&p, Purpose::Core, ""), &access(), None)
        .unwrap();
    assert_eq!(core.projection.items.len(), 1);
    assert_eq!(core.projection.items[0].source_id, fact.id);
    assert!(core.memories().iter().all(|m| m.marker.is_none()));
    let task = view
        .execute(&query(&p, Purpose::Task, ""), &access(), None)
        .unwrap();
    assert_eq!(task.projection.status, QueryStatus::OK);
    assert_eq!(task.memories().len(), 7);
    let reactions = task
        .memories()
        .iter()
        .filter(|m| m.marker.is_some())
        .collect::<Vec<_>>();
    assert_eq!(reactions.len(), 6);
    for memory in reactions {
        assert_eq!(memory.event_id, fact.id);
        assert_eq!(memory.at_ns, fact.event_time_ns);
        assert_eq!(memory.marker.as_deref(), Some(PAST_REACTION_MARKER));
        assert!(
            memory
                .marked_text()
                .starts_with(&format!("【{PAST_REACTION_MARKER}】"))
        );
        assert!(!memory.original.contains("\"binding\""));
        assert!(
            serde_json::to_string(memory)
                .unwrap()
                .contains("\"at_ns\":\"1700000000000000123\"")
        );
    }
    let core_hidden = view
        .execute(
            &query(&p, Purpose::Core, "synthetic-reaction-only-marker"),
            &access(),
            None,
        )
        .unwrap();
    assert_eq!(core_hidden.projection.status, QueryStatus::NO_MATCH);
    assert!(core_hidden.trace.channels.iter().all(Vec::is_empty));
    let task_positive = view
        .execute(
            &query(&p, Purpose::Task, "synthetic-reaction-only-marker"),
            &access(),
            None,
        )
        .unwrap();
    assert_eq!(task_positive.memories().len(), 5);
    let (response, _) = view
        .user_view(&panel(None, 10), &access(), &Budgets::default(), None)
        .unwrap();
    assert_eq!(response.status, QueryStatus::OK);
    assert!(response.items.iter().all(|i| i.at == "1700000000000000123"));
    assert_eq!(
        response
            .items
            .iter()
            .find(|i| i.kind == "user_text")
            .unwrap()
            .summary,
        "synthetic exact original 原文\nsecond line"
    );
    assert_eq!(
        response
            .items
            .iter()
            .filter(|i| i.kind.starts_with("past_reaction:")
                && i.summary.contains(PAST_REACTION_MARKER))
            .count(),
        6
    );
    let audit = task.trace.audit_record(&task.projection).unwrap();
    assert!(
        String::from_utf8(audit.payload.0)
            .unwrap()
            .contains(PAST_REACTION_MARKER)
    );
}

#[test]
fn v288_core_ranking_and_cost_do_not_depend_on_reaction_content_or_count() {
    let mut baseline = World::new();
    let mut extra = World::new();
    extra.all_reactions = true;
    for text in [
        "synthetic-reaction-only-marker first fact",
        "synthetic-reaction-only-marker another fact",
    ] {
        baseline.text(text);
        extra.text(text);
    }
    let r1 = baseline.store.reader().unwrap();
    let r2 = extra.store.reader().unwrap();
    let v1 = FactView::rebuild(&r1, &baseline.prefix(), 100, None).unwrap();
    let v2 = FactView::rebuild(&r2, &extra.prefix(), 100, None).unwrap();
    let a = v1
        .execute(
            &query(
                &baseline.prefix(),
                Purpose::Core,
                "synthetic-reaction-only-marker",
            ),
            &access(),
            None,
        )
        .unwrap();
    let b = v2
        .execute(
            &query(
                &extra.prefix(),
                Purpose::Core,
                "synthetic-reaction-only-marker",
            ),
            &access(),
            None,
        )
        .unwrap();
    assert_eq!(a.projection.items, b.projection.items);
    assert_eq!(a.trace.channels, b.trace.channels);
    assert_eq!(a.trace.steps_used, b.trace.steps_used);
    assert_eq!(a.memories(), b.memories());
}

#[test]
fn v276_pages_reach_beyond_old_top64_with_tied_exact_times_and_no_state() {
    let mut w = World::new();
    w.event_time = Some(1_700_000_000_000_000_123);
    let mut expected = BTreeSet::new();
    for n in 0..75 {
        expected.insert(w.text(&format!("pageable-synthetic-{n}")).id);
    }
    let reader = w.store.reader().unwrap();
    let p = w.prefix();
    let view = FactView::rebuild(&reader, &p, 100, None).unwrap();
    let mut q = query(&p, Purpose::Task, "pageable-synthetic");
    q.payload_types.insert(source::tag::<UserText>().unwrap());
    q.budget.items = 7;
    let mut seen = BTreeSet::new();
    let mut pages = 0;
    loop {
        let page = view.execute(&q, &access(), None).unwrap();
        assert!(page.memories().len() <= 7);
        for m in page.memories() {
            assert_eq!(m.at_ns.0, 1_700_000_000_000_000_123);
            assert!(seen.insert(m.event_id.clone()), "duplicate across pages");
        }
        pages += 1;
        let Some(next) = page.next_page else {
            assert_eq!(page.projection.status, QueryStatus::OK);
            break;
        };
        assert_eq!(page.projection.status, QueryStatus::BUDGET_EXHAUSTED);
        assert!(page.projection.truncated);
        if pages == 1 {
            // One independent rebuild proves continuation has no session state.
            // All 75 records and all eleven page boundaries are checked below.
            let rebuilt = FactView::rebuild(&reader, &p, 100, None).unwrap();
            let mut resumed = q.clone();
            resumed.after = Some(next.clone());
            assert_eq!(
                view.execute(&resumed, &access(), None).unwrap().projection,
                rebuilt
                    .execute(&resumed, &access(), None)
                    .unwrap()
                    .projection
            );
        }
        q.after = Some(next);
        assert!(pages < 20);
    }
    assert_eq!(seen, expected);
    assert_eq!(pages, 11);
}

#[test]
fn v276_continuations_bind_query_permissions_budget_and_prefix() {
    let mut w = World::new();
    w.text("cursor fact one");
    w.text("cursor fact two");
    let reader = w.store.reader().unwrap();
    let p = w.prefix();
    let view = FactView::rebuild(&reader, &p, 100, None).unwrap();
    let mut q = query(&p, Purpose::Task, "cursor fact");
    q.budget.items = 1;
    q.after = view.execute(&q, &access(), None).unwrap().next_page;
    assert!(q.after.is_some());
    let mut changed = q.clone();
    changed.text = "different".into();
    assert!(view.execute(&changed, &access(), None).is_err());
    changed = q.clone();
    changed.purpose = Purpose::Core;
    assert!(view.execute(&changed, &access(), None).is_err());
    changed = q.clone();
    changed.budget.items += 1;
    assert!(view.execute(&changed, &access(), None).is_err());
    let mut restricted = access();
    restricted.sources.insert("user".into());
    assert!(view.execute(&q, &restricted, None).is_err());
    assert_eq!(
        view.execute(&q, &Access::default(), None)
            .unwrap()
            .projection
            .status,
        QueryStatus::DENIED
    );
    w.text("new commit invalidates old prefix before it can leak a withdrawal");
    assert_eq!(
        view.execute(&q, &access(), None).unwrap().projection.status,
        QueryStatus::INDEX_LAGGING
    );
}

#[test]
fn v276_panel_byte_pages_keep_original_text_and_resume_without_omissions() {
    let mut w = World::new();
    for n in 0..5 {
        w.text(&format!("byte-page-{n} {}", "原文".repeat(50)));
    }
    let reader = w.store.reader().unwrap();
    let view = FactView::rebuild(&reader, &w.prefix(), 100, None).unwrap();
    let request = panel(Some("byte-page"), 5);
    let budgets = Budgets {
        user_view: Budget {
            items: 5,
            bytes: 2200,
            steps: 300_000,
        },
        ..Budgets::default()
    };
    let mut after = None;
    let mut seen = BTreeSet::new();
    loop {
        let (response, execution) = view
            .user_view_page(&request, &access(), &budgets, after.as_ref(), None)
            .unwrap();
        assert!(!response.items.is_empty());
        assert!(
            response
                .canonical_bytes(crate::query::LIMITS)
                .unwrap()
                .len()
                <= 2200
        );
        for item in response.items {
            assert!(item.summary.ends_with(&"原文".repeat(50)));
            assert!(seen.insert(item.r#ref));
        }
        after = execution.next_page;
        if after.is_none() {
            break;
        }
        assert!(seen.len() <= 5);
    }
    assert_eq!(seen.len(), 5);
}

#[test]
fn v288_backfilled_reactions_keep_both_times_permissions_and_withdrawals() {
    let mut w = World::new();
    let original = w.text("original public experience");
    w.reaction_target = Some(original.id.clone());
    let backfill = w.add(
        &UserText {
            schema_version: 1,
            text: "private interpretation event".into(),
        },
        "private",
        &[],
        None,
    );
    let reader = w.store.reader().unwrap();
    let p = w.prefix();
    let view = FactView::rebuild(&reader, &p, 100, None).unwrap();
    let all = view
        .execute(&query(&p, Purpose::Task, ""), &access(), None)
        .unwrap();
    assert!(all.memories().iter().any(|m| m.marker.is_some()
        && m.at_ns == original.event_time_ns
        && m.recorded_at_ns == backfill.event_time_ns));
    let mut public = access();
    public.sources.insert("user".into());
    let limited = view
        .execute(&query(&p, Purpose::Task, ""), &public, None)
        .unwrap();
    assert!(
        limited
            .memories()
            .iter()
            .all(|m| m.recorded_at_ns == original.event_time_ns)
    );
    w.reaction_target = None;
    w.add(
        &UserRetraction {
            schema_version: 1,
            target_refs: vec![original.id.clone()],
            note: None,
        },
        "user",
        &[],
        None,
    );
    let reader = w.store.reader().unwrap();
    let p = w.prefix();
    let view = FactView::rebuild(&reader, &p, 100, None).unwrap();
    for _ in 0..100 {
        let task = view
            .execute(&query(&p, Purpose::Task, ""), &access(), None)
            .unwrap();
        assert!(task.memories().iter().all(|m| m.event_id != original.id));
    }
    assert!(
        view.execute(&query(&p, Purpose::Audit, ""), &access(), None)
            .unwrap()
            .memories()
            .iter()
            .any(|m| m.event_id == original.id && m.marker.is_some())
    );
}

#[test]
fn v288_missing_reaction_object_clears_the_entire_memory_result() {
    let mut w = World::new();
    w.text("verified original");
    let reader = w.store.reader().unwrap();
    let p = w.prefix();
    let view = FactView::rebuild(&reader, &p, 100, None).unwrap();
    let reference = &view
        .records
        .iter()
        .find(|r| r.reaction.is_some())
        .unwrap()
        .content;
    let file = w.store.path().join("objects").join(format!(
        "{}.obj",
        core_types::scalar::hex(&reference.digest.0)
    ));
    assert!(file.starts_with(w.root.path().canonicalize().unwrap()));
    std::fs::remove_file(file).unwrap();
    let task = view
        .execute(&query(&p, Purpose::Task, ""), &access(), None)
        .unwrap();
    assert_eq!(task.projection.status, QueryStatus::OBJECT_MISSING);
    assert!(
        task.projection.items.is_empty() && task.memories().is_empty() && task.next_page.is_none()
    );
    assert_eq!(
        view.execute(
            &query(&p, Purpose::Core, "verified original"),
            &access(),
            None
        )
        .unwrap()
        .projection
        .status,
        QueryStatus::OK
    );
}
