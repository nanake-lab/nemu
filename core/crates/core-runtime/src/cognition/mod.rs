//! One terminal return at the trusted core boundary. The scheduler supplies a
//! registered next input, and the interpretation/F owner supplies staged effects.
//! No network or model calls are made here, including during recovery.
use core_store::{
    ActivityOutcome, ActivityTicket, CommitCapability, LineageKind, PreparedObject, Store,
};
use core_types::generated::*;

pub struct RoundEffects {
    /// Accepted m=0 interpretation records for this activity.
    pub interpretations: Vec<ObjectRef>,
    pub x: FastStateSnapshotV2,
    pub next_input: CognitiveInput,
    /// Local action gate supplies identity and the actual request, if proposed.
    pub action: Option<(String, ObjectRef)>,
    /// The caller supplies r and M. Persistent deployment must resolve them
    /// from the versioned fuse resource (ADR 0026 §6).
    pub actions_per_round: usize,
    pub inflight_capacity: usize,
}

pub fn finish_round<L: LineageKind>(
    store: &mut Store<L>,
    capability: &CommitCapability<L>,
    ticket: &ActivityTicket<L>,
    record: InternalActivityRecordV2,
    review_d8: impl FnOnce(&RemoteCognitiveReceipt, &[u8], &str) -> core_store::Result<bool>,
    effects: impl FnOnce() -> core_store::Result<RoundEffects>,
) -> crate::Result<ActivityOutcome> {
    let accepted = record.clone();
    Ok(
        store.finish_activity(capability, ticket, record, review_d8, |store, h, old| {
            let effects = effects()?;
            let binding = ticket.input().binding.clone();
            let local = accepted.local.as_ref();
            let mut q: CognitionState = store.verified_reader()?.read(&old.staged_q)?;
            let next = store.prepare(&effects.next_input, "1")?;
            let x = store.prepare(&effects.x, &old.staged_x.resource_version)?;
            let mut pins: Vec<PreparedObject<L>> = vec![];
            let work = q
                .activities
                .iter_mut()
                .find(|a| a.activity_id == binding.activity_id)
                .ok_or(core_store::Error::Rejected("activity state missing"))?;
            q.revision = effects.next_input.binding.q_revision.clone();
            work.revision = q.revision.clone();
            work.materials = effects
                .next_input
                .materials
                .iter()
                .map(|m| m.content.clone())
                .collect();
            work.latest_step = Some(h.clone());
            work.successor = Some(next.reference().clone());
            if let Some(focus) = local.and_then(|r| r.next_focus.as_ref()) {
                work.focus = Some(focus.clone());
            }
            match (local.and_then(|r| r.action_plan.as_ref()), effects.action) {
                (None, None) => {}
                (Some(plan), Some((action_id, request))) => {
                    if effects.actions_per_round == 0
                        || q.inflight.len() >= effects.inflight_capacity
                    {
                        return Err(core_store::Error::Rejected(
                            "bound action capacity exhausted",
                        ));
                    }
                    q.inflight.push(CognitiveInflightAction {
                        schema_version: 1,
                        action_id,
                        origin_record: h.clone(),
                        origin_step_id: binding.step_id.clone(),
                        round_index: binding.round_index,
                        registered_at_ns: effects.x.anchor_time_ns,
                        plan: plan.clone(),
                        status: ActionStatus::PLANNED,
                        request,
                        actual_result: None,
                        credential: None,
                    });
                }
                _ => {
                    return Err(core_store::Error::Rejected(
                        "action registration requires a local plan",
                    ));
                }
            }
            q.active_request = None;
            q.runnable_successor = Some(next.reference().clone());
            let q = store.prepare(&q, "1")?;
            let commit = InternalActivityCommitV2 {
                schema_version: 1,
                binding,
                interpretations: effects.interpretations.clone(),
                activity: accepted,
            };
            let value = CoreTransactionV4 {
                schema_version: 1,
                event_binding: None,
                activity_commit: Some(commit),
                staged_h: h.clone(),
                staged_x: x.reference().clone(),
                staged_q: q.reference().clone(),
                processed_position: core_types::scalar::DecimalU64(store.head()?.1),
                anchor_time_ns: effects.x.anchor_time_ns,
                staged_r: old.staged_r.clone(),
                staged_theta: old.staged_theta.clone(),
                staged_l: old.staged_l.clone(),
                staged_u: old.staged_u.clone(),
                accepted_results: effects.interpretations,
                staged_ledger: vec![],
                staged_concepts: vec![],
                audit_refs: vec![ticket.reference().clone()],
            };
            pins.extend([next, x, q]);
            Ok((value, pins))
        })?,
    )
}

pub use task_layer::cognition::{MaterialBudget, RecalledMaterial, RecalledPage};

/// Invoke the trusted W19 task query at the dispatched prefix. The embedding
/// and permission owners remain with the caller; task-layer receives only data.
pub fn query_task_materials<L: LineageKind>(
    store: &Store<L>,
    ticket: &ActivityTicket<L>,
    query: impl FnOnce(
        &core_store::Reader<L>,
        &CognitionBinding,
    ) -> crate::Result<(FactProjection, Vec<RecalledMaterial>)>,
    budget: MaterialBudget,
    measure: impl FnMut(&[RecalledMaterial]) -> crate::Result<(usize, usize)>,
) -> crate::Result<RecalledPage> {
    let b = &ticket.input().binding;
    crate::check(
        store.base_commit()? == b.base_commit && store.next_raw()?.is_none(),
        "stale material query",
    )?;
    let (projection, memories) = query(&store.verified_reader()?, b)?;
    let mut page = RecalledPage {
        projection,
        memories,
    };
    page.check(&b.base_commit).map_err(crate::Error::Rejected)?;
    page.bounded(budget, measure)?;
    Ok(page)
}
