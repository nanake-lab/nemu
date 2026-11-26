//! Gateway entry point for the disconnected ADR 0049 rehearsal. The caller
//! supplies trusted, version-bound stage inputs and its authenticated authority.
use crate::{
    clock::ProcessTime,
    pipeline::Input,
    stages::{Components, StageSet},
    status::Status,
    store,
};
use core_store::{CommitCapability, Reader, Rehearsal, Store};
use core_types::{contract::Limits, generated::*};
use gateway::{
    Grants,
    hooks::{CoreHandler, Decision},
};

/// The fact projection and clock owners supply these for the exact durable
/// event. A missing input must fail preparation, never synthesize a binding.
pub struct EventInputs {
    pub binding: ProposalBindingV2,
    pub time: ProcessTime,
    pub facts: FactProjection,
    pub attributes: EventAttributesRecord,
}

pub struct RehearsalHandler<A, P> {
    capability: CommitCapability<Rehearsal>,
    authority: A,
    inputs: P,
    limits: Limits,
    mode: SpeechMode,
}

impl<A, P> RehearsalHandler<A, P> {
    /// Move the same capability used for bootstrap (or newly taken on reopen).
    pub fn new(
        capability: CommitCapability<Rehearsal>,
        authority: A,
        inputs: P,
        limits: Limits,
        mode: SpeechMode,
    ) -> Self {
        Self {
            capability,
            authority,
            inputs,
            limits,
            mode,
        }
    }
}

fn disconnected(q: &InitialRuntimeRoot) -> gateway::Result<()> {
    if [
        &q.irl,
        &q.appraiser,
        &q.matcher,
        &q.fast,
        &q.growth,
        &q.dynamics,
        &q.compiler,
    ]
    .into_iter()
    .any(Option::is_some)
    {
        return Err(gateway::Error::Rejected(
            "connected rehearsal stages require their adapters",
        ));
    }
    Ok(())
}

impl<A, P> CoreHandler<Rehearsal> for RehearsalHandler<A, P>
where
    A: CoreHandler<Rehearsal>,
    P: FnMut(&Store<Rehearsal>, &EventEnvelope) -> crate::Result<EventInputs>,
{
    fn authenticated(
        &mut self,
        store: &mut Store<Rehearsal>,
        session: &TerminalSession,
    ) -> gateway::Result<Decision<()>> {
        self.authority.authenticated(store, session)
    }
    fn authorize_event(
        &mut self,
        store: &Store<Rehearsal>,
        session: &TerminalSession,
        event: &TerminalEvent,
        source: &SourceMetadata,
    ) -> Decision<Grants> {
        self.authority
            .authorize_event(store, session, event, source)
    }
    fn cancel(
        &mut self,
        store: &mut Store<Rehearsal>,
        session: &TerminalSession,
        cancel: &OutputCancel,
    ) -> gateway::Result<Decision<()>> {
        self.authority.cancel(store, session, cancel)
    }
    fn approval(
        &mut self,
        store: &mut Store<Rehearsal>,
        session: &TerminalSession,
        decision: &ApprovalDecisionV2,
    ) -> gateway::Result<Decision<()>> {
        self.authority.approval(store, session, decision)
    }
    fn process_next(
        &mut self,
        store: &mut Store<Rehearsal>,
    ) -> gateway::Result<Vec<CoreCommitResultV2>> {
        // The gateway settles Gamma before this call; Store::bind rechecks it.
        let Some(event) = store.next_raw()? else {
            return Ok(vec![]);
        };
        let roots = store
            .published_roots()?
            .ok_or(gateway::Error::Rejected("missing rehearsal root"))?;
        let q: InitialRuntimeRoot = store.verified_reader()?.read(&roots.q)?;
        disconnected(&q)?;
        let input = (self.inputs)(store, &event).map_err(runtime_error)?;
        let ticket = store.bind(input.binding.clone())?;
        let base = store::read_state(store).map_err(runtime_error)?;
        let mut stages = StageSet::rehearsal(Components::default(), q.environment, self.limits)
            .map_err(runtime_error)?;
        let staged = stages
            .process_head(Input {
                event,
                attributes: &input.attributes,
                base,
                facts: &input.facts,
                binding: input.binding,
                time: input.time,
                growth_paused: false,
            })
            .map_err(runtime_error)?;
        let prepared = store::prepare_existing_head(store, ticket, staged, self.limits)
            .map_err(runtime_error)?;
        Ok(vec![store.commit(
            &self.capability,
            prepared.transaction,
            prepared.status,
        )?])
    }
    fn committed_run_state(
        &self,
        reader: &Reader<Rehearsal>,
        head: &ObjectRef,
    ) -> gateway::Result<Option<RunStateV2>> {
        let q = if head.object_type == "echo.initial-publication" {
            reader.read::<InitialPublication>(head)?.q
        } else {
            let q_ref = if head.object_type == "echo.transaction-v4" {
                reader.read::<CoreTransactionV4>(head)?.staged_q
            } else {
                reader.event_snapshot(head)?.staged_q
            };
            reader.read::<InitialRuntimeRoot>(&q_ref)?
        };
        disconnected(&q)?;
        let status = Status::unmodulated(Lineage::REHEARSAL, Some(&q));
        // No modulation was applied in this disconnected rehearsal.
        Ok(status.terminal(Lineage::REHEARSAL, self.mode, false, None))
    }
}

fn runtime_error(error: crate::Error) -> gateway::Error {
    match error {
        crate::Error::Store(error) => gateway::Error::Store(error),
        _ => gateway::Error::Rejected("rehearsal event preparation failed"),
    }
}
