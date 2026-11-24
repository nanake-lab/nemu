use super::*;

/// Optional advisory lookup at the supplied commit. An unreadable/missing
/// projection is never permission to create or dispatch an action.
pub trait OutboxIndex {
    fn lookup(&self, commit: &str, action_id: &str) -> Option<CognitiveInflightAction>;
    fn diagnose(&mut self, commit: &str, action_id: &str, issue: IndexIssue);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexIssue {
    Missing,
    Disagrees,
}

pub(super) fn locate(
    q: &CognitionState,
    head: &str,
    id: &str,
    index: &mut impl OutboxIndex,
) -> Result<usize> {
    let hit = index.lookup(head, id);
    let mut matching = q
        .inflight
        .iter()
        .enumerate()
        .filter(|(_, a)| a.action_id == id);
    let authoritative = matching.next();
    ensure(matching.next().is_none(), "ambiguous action id in Q")?;
    match (authoritative, hit.as_ref()) {
        (Some(_), None) => index.diagnose(head, id, IndexIssue::Missing),
        (Some((_, a)), Some(row)) if row != a => index.diagnose(head, id, IndexIssue::Disagrees),
        (None, Some(_)) => index.diagnose(head, id, IndexIssue::Disagrees),
        _ => {}
    }
    authoritative
        .map(|(i, _)| i)
        .ok_or(Error::Rejected("action is not pending in Q"))
}

/// Enumerate dispatch candidates from one committed Q. The scheduler subsequently
/// opens each action on a fresh writer ticket before consuming its credential.
pub fn pending_dispatch<L: LineageKind>(
    store: &Store<L>,
    index: &mut impl OutboxIndex,
) -> Result<Vec<String>> {
    let reader = store.reader()?;
    let Some(current) = reader.current()? else {
        return Ok(Vec::new());
    };
    let q: CognitionState = reader.read(&current.staged_q)?;
    let head = store.base_commit()?;
    q.inflight
        .iter()
        .filter(|a| a.status == ActionStatus::AUTHORIZED)
        .map(|a| {
            let i = locate(&q, &head, &a.action_id, index)?;
            Ok(q.inflight[i].action_id.clone())
        })
        .collect()
}
