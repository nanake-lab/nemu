//! Cross-crate regression linking the actual workspace libraries without an application dependency.
#[path = "../../../tests/store.rs"]
mod store_fixture;
use core_runtime::cognition::finish_round;
use core_store::{Rehearsal, TemporaryRoot};
use core_types::generated::*;
use store_fixture::{
    cognition_tests::fixtures::{activity, effects, initial_q, review},
    common::with_refs,
    open, seed_with_history,
};
#[test]
fn w19_historical_reaction_after_internal_publication() {
    use core_runtime::cognition::{MaterialBudget, RecalledMaterial, query_task_materials};
    use fact_view::{Access, Budgets, FactQuery, FactView, PAST_REACTION_MARKER, Purpose};
    let root = TemporaryRoot::new().unwrap();
    let mut store = open(&root);
    let cap = store.take_commit_capability().unwrap();
    seed_with_history(
        &mut store,
        &cap,
        initial_q,
        |store, leaf, event, binding| {
            let mut reaction: NatureAppraisalRecord = with_refs(leaf);
            reaction.event_id = event.id.clone();
            reaction.binding = binding.clone();
            vec![store.prepare(&reaction, "1").unwrap().reference().clone()]
        },
    );
    let first = store.begin_activity().unwrap();
    let old = store.reader().unwrap();
    let before = first.input().binding.base_commit.clone();
    let old_view = FactView::rebuild(&old, &before, 16, None).unwrap();
    let cloud = activity(&store, &first, true);
    let e = effects(&store, &first, &cloud);
    finish_round(&mut store, &cap, &first, cloud, review, || Ok(e)).unwrap();
    let next = store.begin_activity().unwrap();
    let budget = Budgets::default();
    let access = Access {
        allowed: true,
        audit: false,
        sources: Default::default(),
    };
    assert_eq!(
        old_view
            .execute(
                &FactQuery::new(&before, Purpose::Task, "", &budget),
                &access,
                None
            )
            .unwrap()
            .projection
            .status,
        QueryStatus::INDEX_LAGGING
    );
    let read = |reader: &core_store::Reader<Rehearsal>, b: &CognitionBinding| {
        let v = FactView::rebuild(reader, &b.base_commit, 16, None).unwrap();
        let r = v
            .execute(
                &FactQuery::new(&b.base_commit, Purpose::Task, "", &budget),
                &access,
                None,
            )
            .unwrap();
        let memories = r
            .memories()
            .iter()
            .map(|m| RecalledMaterial {
                content: m.content.clone(),
                event_id: m.event_id.clone(),
                source: m.source.clone(),
                event_time_ns: m.at_ns.0,
                recorded_at_ns: m.recorded_at_ns.0,
                original: m.original.clone(),
                marker: m.marker.clone(),
                marked_text: m.marked_text(),
            })
            .collect();
        Ok((r.projection, memories))
    };
    let measure = |items: &[RecalledMaterial]| {
        let bytes = items
            .iter()
            .map(|m| m.marked_text.len() + m.source.len())
            .sum();
        Ok((bytes, bytes))
    };
    let full = query_task_materials(
        &store,
        &next,
        read,
        MaterialBudget {
            items: 8,
            bytes: 8192,
            tokens: 8192,
        },
        measure,
    )
    .unwrap();
    assert_eq!(full.projection.status, QueryStatus::OK);
    assert_eq!(full.memories.len(), 2);
    let reaction = full.memories.iter().find(|m| m.marker.is_some()).unwrap();
    assert_eq!(reaction.marker.as_deref(), Some(PAST_REACTION_MARKER));
    assert_eq!(
        reaction.marked_text,
        format!("【{PAST_REACTION_MARKER}】\n{}", reaction.original)
    );
    assert_eq!(reaction.event_id, "event-0");
    assert!(
        full.memories
            .iter()
            .all(|m| !m.source.is_empty() && m.event_id == "event-0")
    );
    let bounded = query_task_materials(
        &store,
        &next,
        read,
        MaterialBudget {
            items: 1,
            bytes: 8192,
            tokens: 8192,
        },
        measure,
    )
    .unwrap();
    assert_eq!(bounded.memories.len(), 1);
    assert_eq!(bounded.projection.items.len(), 1);
    assert!(bounded.projection.truncated);
    assert_eq!(bounded.projection.status, QueryStatus::BUDGET_EXHAUSTED);
}
