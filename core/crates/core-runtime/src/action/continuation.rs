use super::*;

/// An action event advances Q while cognition may already have scheduled its
/// next input. Only the scheduler may compute its replacement. Never retag an
/// in-flight model proposal or leave the next request pointing at an old Q.
pub(super) fn publish_next<L: LineageKind>(
    store: &Store<L>,
    q: &mut CognitionState,
    frame: &EventCommit<L>,
    event: &EventEnvelope,
) -> Result<Option<PreparedObject<L>>> {
    if q.mode != CognitiveMode::COG_RUNNING {
        ensure(
            frame.next_input.is_none()
                && q.active_request.is_none()
                && q.runnable_successor.is_none(),
            "inactive cognition has a scheduled input",
        )?;
        q.revision = event.id.clone();
        return Ok(None);
    }
    let next = frame.next_input.as_ref().ok_or(Error::Rejected(
        "fresh scheduler input required for running Q",
    ))?;
    let b = &next.binding;
    ensure(
        b.base_commit == frame.ticket.binding().base_commit
            && b.branch_id == frame.ticket.binding().branch_id
            && b.q_revision != q.revision
            && b.environment == frame.ticket.binding().environment
            && b.source_validity_version == frame.ticket.binding().source_validity_version,
        "new scheduler input does not bind this publication",
    )?;
    let registered = q
        .active_request
        .as_ref()
        .or(q.runnable_successor.as_ref())
        .ok_or(Error::Rejected("missing previous scheduler input"))?;
    let previous: CognitiveInput = store.reader()?.read(registered)?;
    ensure(
        b.activity_id == previous.binding.activity_id
            && b.request_id != previous.binding.request_id,
        "scheduler must provide a fresh request for the active work",
    )?;
    let work = q
        .activities
        .iter_mut()
        .find(|a| a.activity_id == b.activity_id)
        .ok_or(Error::Rejected("no matching scheduled activity"))?;
    let pin = store.prepare(next, "1")?;
    let successor = Some(pin.reference().clone());
    work.successor = successor.clone();
    work.revision = b.q_revision.clone();
    q.revision = b.q_revision.clone();
    q.active_request = None;
    q.runnable_successor = successor;
    Ok(Some(pin))
}
