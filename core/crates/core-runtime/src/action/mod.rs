//! W13 at the sole W02 writer. The caller provides the rest of the staged
//! event computation and its read ticket. This adapter never synthesizes x/R/θ.
use action_gate::{
    lifecycle,
    outbox::{self, Authority, delivery},
};
use core_store::{CommitCapability, Error, LineageKind, PreparedObject, ReadTicket, Result, Store};
use core_types::{WireMessage, contract::Limits, generated::*};

mod continuation;
mod dispatch;
mod history;
mod index;
mod planning;
pub use index::{IndexIssue, OutboxIndex};
pub use planning::stage_local_plan;

/// Prepared by the trusted event path for the next durable H record.
pub struct EventCommit<L: LineageKind> {
    pub ticket: ReadTicket<L>,
    pub transaction: CoreTransactionV2,
    pub status: DegradedStatus,
    /// Freshly computed by the scheduling owner when Q has running cognition.
    pub next_input: Option<CognitiveInput>,
}

pub struct ActionTransaction<'a, L: LineageKind> {
    store: &'a mut Store<L>,
    capability: &'a CommitCapability<L>,
    frame: Option<EventCommit<L>>,
    event: EventEnvelope,
    origin: CognitionBinding,
    reader: core_store::Reader<L>,
    q: CognitionState,
    index: usize,
    history: history::History,
    limits: Limits,
}

/// Stable deduplication key of the local issuance event. Only a digest of the
/// transient grant enters H. A different grant cannot reuse an authorization.
pub fn authorization_key(grant: &ExecutionCredential, limits: Limits) -> Result<String> {
    let digest =
        crate::binding::digest(grant, limits).map_err(|_| Error::Rejected("invalid grant"))?;
    Ok(format!(
        "w13/authorization/{}",
        core_types::scalar::hex(&digest.0)
    ))
}

/// Called by startup before offering dispatch. These IDs only admit UNKNOWN
/// maintenance or correlated late results, never another send.
pub fn unresolved_after_restart<L: LineageKind>(store: &Store<L>) -> Result<Vec<String>> {
    let Some(current) = store.current()? else {
        return Ok(vec![]);
    };
    let q: CognitionState = store.reader()?.read(&current.staged_q)?;
    Ok(q.inflight
        .into_iter()
        .filter(|a| a.status == ActionStatus::DISPATCHING)
        .map(|a| a.action_id)
        .collect())
}

fn ensure(ok: bool, message: &'static str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::Rejected(message))
    }
}

impl<'a, L: LineageKind> ActionTransaction<'a, L> {
    /// Select by the durable event's action ID. A plan must already have been
    /// admitted to Q by its local producer. Never construct work from the
    /// action-request event itself (SYS11).
    pub fn open(
        store: &'a mut Store<L>,
        capability: &'a CommitCapability<L>,
        frame: EventCommit<L>,
        limits: Limits,
        history_limit: usize,
        index_hint: &mut impl OutboxIndex,
    ) -> Result<Self> {
        let event = store
            .next_raw()?
            .ok_or(Error::Rejected("no action event"))?;
        let id = event
            .action_id
            .as_deref()
            .ok_or(Error::Rejected("missing event action ID"))?;
        let current = store.current()?.ok_or(Error::Rejected("no committed Q"))?;
        let binding = frame.ticket.binding();
        ensure(
            binding == &frame.transaction.binding
                && binding.event_id == event.id
                && binding.base_commit == store.base_commit()?
                && frame.transaction.staged_q == current.staged_q,
            "stale action transaction",
        )?;
        let reader = store.reader()?;
        let q: CognitionState = reader.read(&current.staged_q)?;
        let index = index::locate(&q, &binding.base_commit, id, index_hint)?;
        let action = &q.inflight[index];
        planning::validate_semantics(&q, limits)?;
        let local = planning::read_local(&reader, &action.origin_record, limits)?;
        ensure(
            local.action_plan.as_ref() == Some(&action.plan)
                && local.binding.step_id == action.origin_step_id
                && local.binding.round_index == action.round_index
                && local.binding.branch_id == binding.branch_id,
            "plan is not the committed local selection",
        )?;
        let history = history::History::read(&reader, &binding.base_commit, history_limit, action)?;
        Ok(Self {
            store,
            reader,
            capability,
            frame: Some(frame),
            event,
            q,
            index,
            origin: local.binding,
            history,
            limits,
        })
    }

    fn action(&self) -> &CognitiveInflightAction {
        &self.q.inflight[self.index]
    }

    fn frame(&self) -> Result<&EventCommit<L>> {
        self.frame
            .as_ref()
            .ok_or(Error::Rejected("transaction consumed"))
    }

    fn input<T: WireMessage>(&self) -> Result<T> {
        self.store
            .read_bound(&self.frame()?.ticket, &self.event.payload_ref)
    }

    fn publish(
        &mut self,
        status: ActionStatus,
        result: Option<ObjectRef>,
        mut audit: Vec<PreparedObject<L>>,
    ) -> Result<()> {
        let mut frame = self
            .frame
            .take()
            .ok_or(Error::Rejected("transaction consumed"))?;
        let _next_pin = continuation::publish_next(self.store, &mut self.q, &frame, &self.event)?;
        let action = &mut self.q.inflight[self.index];
        action.status = status;
        if result.is_some() {
            action.actual_result = result;
        }
        let record = ActionLifecycle {
            schema_version: 1,
            action_id: action.action_id.clone(),
            plan: action.plan.clone(),
            status,
            request: Some(action.request.clone()),
            actual_result: action.actual_result.clone(),
        };
        audit.push(self.store.prepare(&record, "1")?);
        if matches!(
            status,
            ActionStatus::SUCCEEDED | ActionStatus::FAILED | ActionStatus::CANCELLED_ACTION
        ) {
            self.q.inflight.remove(self.index);
        }
        planning::validate_semantics(&self.q, self.limits)?;
        let q = self.store.prepare(&self.q, "1")?;
        frame.transaction.staged_q = q.reference().clone();
        frame
            .transaction
            .audit_refs
            .extend(audit.iter().map(|p| p.reference().clone()));
        let staged = self.store.stage(frame.ticket, frame.transaction)?;
        self.store.commit(self.capability, staged, frame.status)?;
        Ok(())
    }

    pub fn register(mut self, now_ns: i64) -> Result<()> {
        let status = lifecycle::request(self.action(), &self.event, now_ns)?;
        ensure(!self.history.registered, "duplicate action registration")?;
        self.publish(status, None, vec![])
    }

    /// ApprovalRequestV2 is the immutable request displayed to the user. Its
    /// bytes and ID were registered with the action. The reply is read from H's
    /// actual input, and the service supplies the authenticated local identity.
    pub fn decision(mut self, user: &LocalUserBinding, now_ns: i64) -> Result<()> {
        let request: ApprovalRequestV2 = self.reader.read(&self.action().request)?;
        let actual = self.payload_bytes()?;
        let reply = match self.event.kind {
            EventKind::OBSERVATION => Some(self.input::<ApprovalDecisionV2>()?),
            EventKind::MAINTENANCE => None,
            _ => return Err(Error::Rejected("not an approval reply or timeout")),
        };
        ensure(self.history.consent.is_none(), "approval already settled")?;
        match lifecycle::decide(
            &request,
            reply.as_ref(),
            user,
            self.action(),
            &actual,
            now_ns,
            self.limits,
        )? {
            lifecycle::Decision::Consented(c) => {
                let pin = self.store.prepare(&c, "1")?;
                self.publish(ActionStatus::REQUESTED, None, vec![pin])
            }
            lifecycle::Decision::Cancelled(reason) => {
                let denial = PermissionResult {
                    schema_version: 1,
                    request_id: self.action().action_id.clone(),
                    audit_record_id: self.event.id.clone(),
                    denied: Some(reason),
                    management_commit: None,
                    dispatch_record_id: None,
                };
                let pin = self.store.prepare(&denial, "1")?;
                self.publish(ActionStatus::CANCELLED_ACTION, None, vec![pin])
            }
        }
    }

    fn payload_bytes(&self) -> Result<Vec<u8>> {
        let r = &self.action().plan.output_or_tool_parameters;
        let reader = &self.reader;
        match r.object_type.as_str() {
            "echo.body-action" => Ok(reader.read::<BodyAction>(r)?.canonical_bytes(self.limits)?),
            "echo.plan-carrier-receipt" => Ok(reader
                .read::<PlanAndCarrierReceipt>(r)?
                .canonical_bytes(self.limits)?),
            _ => Err(Error::Rejected("unsupported builtin payload")),
        }
    }

    pub fn cancel(mut self) -> Result<()> {
        ensure(
            self.event.kind == EventKind::CANCEL,
            "not an action cancellation",
        )?;
        self.publish(lifecycle::cancel(self.action().status)?, None, vec![])
    }

    pub fn unknown(mut self) -> Result<()> {
        ensure(
            self.event.kind == EventKind::MAINTENANCE
                && self.action().status == ActionStatus::DISPATCHING,
            "only unresolved dispatch can recover to UNKNOWN",
        )?;
        self.publish(ActionStatus::UNKNOWN_ACTION, None, vec![])
    }

    pub fn settle(mut self) -> Result<()> {
        let action = self.action();
        ensure(
            self.event.kind == EventKind::EXECUTION_RESULT
                && matches!(
                    action.status,
                    ActionStatus::DISPATCHING | ActionStatus::UNKNOWN_ACTION
                ),
            "no matching unresolved send",
        )?;
        let status = match action.plan.output_or_tool_parameters.object_type.as_str() {
            "echo.body-action" => {
                delivery::body_result(&action.action_id, &self.input::<BodyActionResult>()?)?
            }
            "echo.plan-carrier-receipt" => {
                let speech = self.reader.read(&action.plan.output_or_tool_parameters)?;
                let receipt: SpeechDelivery = self.input()?;
                if let Some(previous) = &action.actual_result {
                    let earlier: SpeechDelivery = self.reader.read(previous)?;
                    ensure(
                        receipt.delivered_chunks.0 >= earlier.delivered_chunks.0
                            && receipt.played_chars.0 >= earlier.played_chars.0,
                        "delivery would erase confirmed output",
                    )?;
                }
                delivery::speech_result(&action.action_id, &speech, &receipt)?
            }
            _ => return Err(Error::Rejected("unsupported execution result")),
        };
        self.publish(status, Some(self.event.payload_ref.clone()), vec![])
    }
}

#[cfg(test)]
mod tests;
