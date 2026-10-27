#[cfg(feature = "cpu-embedding")]
mod cpu_eval;
mod edge_cases;
mod fixture;
mod memory_pages;
mod panel_budget;
use super::*;
use core_types::{WireMessage, generated::*, scalar::DecimalU64};
use fixture::World;
fn access() -> Access {
    Access {
        allowed: true,
        audit: true,
        sources: Default::default(),
    }
}
fn query(prefix: &str, purpose: Purpose, text: &str) -> FactQuery {
    FactQuery::new(prefix, purpose, text, &Budgets::default())
}

#[test]
fn sys05_five_distinct_results_and_real_positive_control() {
    let mut world = World::new();
    let fact = world.text("合成事实 alpha-key");
    let reader = world.store.reader().unwrap();
    let prefix = world.prefix();
    let view = FactView::rebuild(&reader, &prefix, 100, None).unwrap();
    let q = query(&prefix, Purpose::UserView, "alpha-key");
    assert_eq!(
        view.execute(&q, &access(), None).unwrap().projection.status,
        QueryStatus::OK
    );
    let mut request = QueryV2 {
        schema_version: 1,
        query_id: "panel".into(),
        kind: QueryKindV2::QUERY_MEMORY,
        filter: Some(QueryFilter {
            schema_version: 1,
            text: Some("absent".into()),
            from: None,
            to: None,
            kinds: vec![],
            include_retracted: None,
        }),
        limit: DecimalU64(10),
    };
    assert_eq!(
        view.user_view(&request, &access(), &Budgets::default(), None)
            .unwrap()
            .0
            .status,
        QueryStatus::NO_MATCH
    );
    request.filter.as_mut().unwrap().text = Some("alpha-key".into());
    assert_eq!(
        view.user_view(&request, &Access::default(), &Budgets::default(), None)
            .unwrap()
            .0
            .status,
        QueryStatus::DENIED
    );
    let tiny = Budgets {
        user_view: Budget {
            items: 0,
            bytes: 1000,
            steps: 100_000,
        },
        ..Budgets::default()
    };
    assert_eq!(
        view.user_view(&request, &access(), &tiny, None)
            .unwrap()
            .0
            .status,
        QueryStatus::BUDGET_EXHAUSTED
    );
    world.text("committed after index");
    assert_eq!(
        view.user_view(&request, &access(), &Budgets::default(), None)
            .unwrap()
            .0
            .status,
        QueryStatus::INDEX_LAGGING
    );
    let reader2 = world.store.reader().unwrap();
    let prefix2 = world.prefix();
    let view2 = FactView::rebuild(&reader2, &prefix2, 100, None).unwrap();
    // Remove only a file under our owned TemporaryRoot, after initial recovery
    // verification, to exercise per-result authority revalidation.
    assert!(
        world
            .store
            .path()
            .starts_with(world.root.path().canonicalize().unwrap())
    );
    let file = world.store.path().join("objects").join(format!(
        "{}.obj",
        core_types::scalar::hex(&fact.payload_ref.digest.0)
    ));
    std::fs::remove_file(file).unwrap();
    assert_eq!(
        view2
            .user_view(&request, &access(), &Budgets::default(), None)
            .unwrap()
            .0
            .status,
        QueryStatus::OBJECT_MISSING
    );
}

#[test]
fn retraction_zero_in_one_hundred_task_queries_and_audit_preserves_original() {
    let mut w = World::new();
    let fact = w.text("蓝色合成自行车 blue bicycle 自転車");
    let before = w.prefix();
    let reader = w.store.reader().unwrap();
    let old = FactView::rebuild(&reader, &before, 100, None).unwrap();
    assert_eq!(
        old.execute(&query(&before, Purpose::Task, "bicycle"), &access(), None)
            .unwrap()
            .projection
            .items
            .len(),
        1
    );
    w.add(
        &UserRetraction {
            schema_version: 1,
            target_refs: vec![fact.id.clone()],
            note: Some("blue bicycle".into()),
        },
        "user",
        &[],
        None,
    );
    assert_eq!(
        old.execute(&query(&before, Purpose::Task, "bicycle"), &access(), None)
            .unwrap()
            .projection
            .status,
        QueryStatus::INDEX_LAGGING
    );
    let reader = w.store.reader().unwrap();
    let prefix = w.prefix();
    let view = FactView::rebuild(&reader, &prefix, 100, None).unwrap();
    for n in 0..100 {
        let text = ["bicycle", "自行车", "自転車", "蓝色", "blue"][n % 5];
        let p = view
            .execute(&query(&prefix, Purpose::Task, text), &access(), None)
            .unwrap()
            .projection;
        assert!(p.items.is_empty());
        assert_eq!(p.status, QueryStatus::NO_MATCH);
    }
    assert_eq!(
        view.execute(&query(&prefix, Purpose::Audit, "bicycle"), &access(), None)
            .unwrap()
            .projection
            .items
            .len(),
        1
    );
    assert_eq!(
        view.execute(&query(&prefix, Purpose::Core, "bicycle"), &access(), None)
            .unwrap()
            .projection
            .items
            .len(),
        1
    );
    let request = QueryV2 {
        schema_version: 1,
        query_id: "memory".into(),
        kind: QueryKindV2::QUERY_MEMORY,
        filter: Some(QueryFilter {
            schema_version: 1,
            text: Some("bicycle".into()),
            from: None,
            to: None,
            kinds: vec![],
            include_retracted: Some(true),
        }),
        limit: DecimalU64(10),
    };
    assert!(
        view.user_view(&request, &access(), &Budgets::default(), None)
            .unwrap()
            .0
            .items
            .is_empty()
    );
}

#[test]
fn structural_allowlist_excludes_unaccepted_wrapped_explanations_and_core_reactions() {
    let mut w = World::new();
    w.text("visible-original");
    let leaf = w
        .store
        .prepare(
            &UserText {
                schema_version: 1,
                text: "hidden-appraisal-marker".into(),
            },
            "1",
        )
        .unwrap();
    macro_rules! inject {($($t:ty),*)=>{$(w.add(&fixture::with_refs::<$t>(leaf.reference()),"user",&[],None);)*};}
    inject!(
        NatureAppraisalRecord,
        SituationMatchRecord,
        TendencyCandidateRecord,
        TeachingAnnotation,
        LieAnnotation,
        SameMatterAnnotation
    );
    let reader = w.store.reader().unwrap();
    let prefix = w.prefix();
    let view = FactView::rebuild(&reader, &prefix, 100, None).unwrap();
    assert_eq!(
        view.records.iter().filter(|r| r.reaction.is_none()).count(),
        1
    );
    for purpose in [Purpose::Core, Purpose::Task] {
        let execution = view
            .execute(&query(&prefix, purpose, ""), &access(), None)
            .unwrap();
        let facts = execution
            .projection
            .items
            .iter()
            .filter(|i| i.content.object_type == source::tag::<UserText>().unwrap())
            .count();
        assert_eq!(facts, 1);
        if purpose == Purpose::Core {
            assert_eq!(execution.projection.items.len(), 1);
        } else {
            assert_eq!(execution.projection.items.len(), 2);
            assert_eq!(
                execution
                    .trace
                    .memories
                    .iter()
                    .filter(|m| m.marker.as_deref() == Some(PAST_REACTION_MARKER))
                    .count(),
                1
            );
        }
        let encoded = execution
            .projection
            .canonical_bytes(crate::query::LIMITS)
            .unwrap();
        assert!(
            !encoded
                .windows(b"hidden-appraisal-marker".len())
                .any(|w| w == b"hidden-appraisal-marker")
        );
    }
}

#[test]
fn deterministic_rrf_filters_corrections_and_budgets() {
    let mut w = World::new();
    let a = w.add(
        &UserText {
            schema_version: 1,
            text: "old-corrected".into(),
        },
        "user",
        &[],
        Some("person-A"),
    );
    let replacement = w.add(
        &UserText {
            schema_version: 1,
            text: "new-corrected".into(),
        },
        "user",
        std::slice::from_ref(&a.id),
        Some("person-A"),
    );
    w.add(
        &UserText {
            schema_version: 1,
            text: "private person-A".into(),
        },
        "private",
        &[],
        Some("person-A"),
    );
    let reader = w.store.reader().unwrap();
    let p = w.prefix();
    let view = FactView::rebuild(&reader, &p, 100, None).unwrap();
    let mut q = query(&p, Purpose::Task, "person-A");
    q.payload_types.insert(source::tag::<UserText>().unwrap());
    q.entities.insert("person-A".into());
    let mut grant = access();
    grant.sources.insert("user".into());
    let first = view.execute(&q, &grant, None).unwrap();
    assert_eq!(first.projection.items.len(), 1);
    assert_eq!(first.projection.items[0].source_id, replacement.id);
    assert_eq!(first.trace.channels[0], vec![replacement.id.clone()]);
    assert_eq!(
        first.projection,
        view.execute(&q, &grant, None).unwrap().projection
    );
    assert!(
        !first
            .trace
            .audit_record(&first.projection)
            .unwrap()
            .payload
            .0
            .is_empty()
    );
    let mut tiny = q.clone();
    tiny.budget.steps = 1;
    assert_eq!(
        view.execute(&tiny, &grant, None).unwrap().projection.status,
        QueryStatus::BUDGET_EXHAUSTED
    );
    tiny = q.clone();
    tiny.budget.bytes = 1;
    assert_eq!(
        view.execute(&tiny, &grant, None).unwrap().projection.status,
        QueryStatus::BUDGET_EXHAUSTED
    );
    q.from_ns = Some(3);
    assert_eq!(
        view.execute(&q, &grant, None).unwrap().projection.status,
        QueryStatus::NO_MATCH
    );
    q.sources.insert("private".into());
    assert_eq!(
        view.execute(&q, &grant, None).unwrap().projection.status,
        QueryStatus::DENIED
    );
    assert!(FactView::rebuild(&reader, "not-a-prefix", 100, None).is_err());
    assert!(FactView::rebuild(&reader, &p, 1, None).is_err());
}

#[test]
fn trigram_and_short_unicode_literal_queries_are_not_fts_syntax() {
    let mut w = World::new();
    w.text("合成记忆 coffee 猫 blue \"quoted\" OR *");
    let r = w.store.reader().unwrap();
    let p = w.prefix();
    let view = FactView::rebuild(&r, &p, 100, None).unwrap();
    for text in ["合成记忆", "coffee", "猫", "\"quoted\"", "OR *"] {
        assert_eq!(
            view.execute(&query(&p, Purpose::Task, text), &access(), None)
                .unwrap()
                .projection
                .status,
            QueryStatus::OK,
            "{text}"
        );
    }
    let mut limited = query(&p, Purpose::Task, "coffee");
    limited.budget.steps = 8;
    assert_eq!(
        view.execute(&limited, &access(), None)
            .unwrap()
            .projection
            .status,
        QueryStatus::BUDGET_EXHAUSTED
    );
    assert_eq!(
        view.execute(&query(&p, Purpose::Task, "coffee"), &access(), None)
            .unwrap()
            .projection
            .status,
        QueryStatus::OK
    );
}
