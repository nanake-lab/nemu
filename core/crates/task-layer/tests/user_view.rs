use core_types::{generated::*, scalar::DecimalU64};
use fact_view::{Access, Budgets, FactView};
use task_layer::task::user_view::{Journal, UserView};
mod query {
    pub const LIMITS: core_types::contract::Limits = core_types::contract::Limits {
        max_bytes: 4 << 20,
        max_items: 100000,
        max_depth: 64,
    };
}
#[allow(dead_code)]
#[expect(
    unexpected_cfgs,
    reason = "shared W19 fixture also contains a helper gated by its owning crate's optional CPU embedding feature"
)]
#[path = "../../fact-view/src/tests/fixture.rs"]
mod fixture;

fn row(kind: &str, summary: &str) -> QueryItem {
    QueryItem {
        schema_version: 1,
        r#ref: kind.into(),
        at: "200".into(),
        source: "owner".into(),
        kind: kind.into(),
        summary: summary.into(),
        retracted: false,
    }
}
fn request(kind: QueryKindV2) -> QueryV2 {
    QueryV2 {
        schema_version: 1,
        query_id: "query-user".into(),
        filter: None,
        limit: DecimalU64(5),
        kind,
    }
}
#[test]
fn three_user_views_keep_provenance_and_audit_before_return() {
    let mut world = fixture::World::new();
    world.text("a recorded observation");
    let reader = world.store.verified_reader().unwrap();
    let view = FactView::rebuild(&reader, &world.prefix(), 20, None).unwrap();
    let actions = [row("denied", "declined action")];
    let history = [row("speech_mode", "mode B")];
    let access = Access {
        allowed: true,
        ..Access::default()
    };
    let budgets = Budgets::default();
    let handler = UserView {
        facts: &view,
        access: &access,
        budgets: &budgets,
        journal: Journal {
            actions: Ok(&actions),
            run_history: Ok(&history),
        },
        limits: query::LIMITS,
    };
    for (kind, expected) in [
        (QueryKindV2::QUERY_MEMORY, "a recorded observation"),
        (QueryKindV2::QUERY_ACTIONS, "declined action"),
        (QueryKindV2::QUERY_RUN_HISTORY, "mode B"),
    ] {
        let mut audited = false;
        let result = handler
            .handle(&request(kind), |q, r| {
                assert_eq!(q.query_id, r.query_id);
                audited = true;
                Ok(())
            })
            .unwrap();
        assert!(audited);
        assert_eq!(result.status, QueryStatus::OK);
        assert!(result.growth_entries.is_empty());
        assert!(result.items.iter().any(|i| i.summary.contains(expected)));
        assert!(
            handler
                .handle(&request(kind), |_, _| Err("audit unavailable".into()))
                .is_err()
        );
    }
}
#[test]
fn empty_and_failed_queries_keep_five_distinct_statuses() {
    use task_layer::task::user_view::query as run;
    let allowed = Access {
        allowed: true,
        ..Access::default()
    };
    let q = request(QueryKindV2::QUERY_ACTIONS);
    let rows = [row("denied", "first"), row("denied", "second")];
    for status in [
        QueryStatus::NO_MATCH,
        QueryStatus::INDEX_LAGGING,
        QueryStatus::DENIED,
        QueryStatus::OBJECT_MISSING,
        QueryStatus::BUDGET_EXHAUSTED,
    ] {
        let none = &[][..];
        let actions = match status {
            QueryStatus::NO_MATCH => Ok(none),
            QueryStatus::DENIED | QueryStatus::BUDGET_EXHAUSTED => Ok(&rows[..]),
            other => Err(other),
        };
        let denied = Access::default();
        let access = if status == QueryStatus::DENIED {
            &denied
        } else {
            &allowed
        };
        let mut budget = Budgets::default().user_view;
        if status == QueryStatus::BUDGET_EXHAUSTED {
            budget.steps = 0
        }
        let result = run(
            &q,
            Journal {
                actions,
                run_history: Ok(none),
            },
            access,
            budget,
            query::LIMITS,
            |_| panic!("wrong domain"),
            |_, r| {
                assert_eq!(r.status, status);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(result.status, status);
        assert!(result.items.is_empty());
    }
}
