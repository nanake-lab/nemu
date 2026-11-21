use super::*;

/// Stage P in the owner's new Q, while preserving the activity that selected
/// it. The caller publishes this pin with the originating outer transaction.
pub fn stage_local_plan<L: LineageKind>(
    store: &Store<L>,
    mut q: CognitionState,
    origin: &InternalActivityRecordV2,
    action_id: String,
    request: ObjectRef,
    registered_at_ns: u64,
) -> Result<PreparedObject<L>> {
    let local = lifecycle::local_plan(origin)?;
    let source = store.prepare(origin, "1")?;
    ensure(
        q.activities.iter().any(|a| {
            a.activity_id == local.binding.activity_id
                && a.latest_step.as_ref() == Some(source.reference())
        }),
        "plan origin is not this activity's accepted step",
    )?;
    ensure(
        !q.inflight
            .iter()
            .any(|a| a.action_id == action_id || &a.origin_record == source.reference()),
        "action ID or planned step already registered",
    )?;
    q.inflight.push(lifecycle::planned(
        origin,
        source.reference().clone(),
        action_id,
        request,
        registered_at_ns,
    )?);
    store.prepare(&q, "1")
}
