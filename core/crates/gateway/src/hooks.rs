//! Trusted in-process core adapter. These are callbacks over generated messages,
//! not another protocol or permission database. Errors deny, never fake success.
use crate::{Grants, Result};
use core_store::{LineageKind, Store};
use core_types::generated::*;

pub type Decision<T> = std::result::Result<T, IngressReason>;
pub trait CoreHandler<L: LineageKind> {
    /// Current source/action/reference authority and per-event T1 grants. The
    /// implementation must resolve causal/correction and execution-result claims
    /// against trusted state, not against uploaded peer assertions alone.
    fn authorize_event(
        &mut self,
        store: &Store<L>,
        session: &TerminalSession,
        event: &TerminalEvent,
        provenance: &SourceMetadata,
    ) -> Decision<Grants>;
    fn query(
        &mut self,
        _store: &mut Store<L>,
        _session: &TerminalSession,
        _query: &QueryV2,
    ) -> Result<Decision<QueryResultV2>> {
        Ok(Err(IngressReason::CAPABILITY_DENIED))
    }
    /// Resolve the authoritative outstanding approval and actual execution bytes,
    /// then call terminal::validate_approval (and consume its one-use snapshot).
    /// A structurally valid decision alone never grants execution authority.
    fn approval(
        &mut self,
        _store: &mut Store<L>,
        _session: &TerminalSession,
        _decision: &ApprovalDecisionV2,
    ) -> Result<Decision<()>> {
        Ok(Err(IngressReason::CAPABILITY_DENIED))
    }
    fn cancel(
        &mut self,
        _store: &mut Store<L>,
        _session: &TerminalSession,
        _cancel: &OutputCancel,
    ) -> Result<Decision<()>> {
        Ok(Err(IngressReason::CAPABILITY_DENIED))
    }
    fn growth(
        &mut self,
        _store: &mut Store<L>,
        _session: &TerminalSession,
        _control: &GrowthControl,
    ) -> Result<Decision<()>> {
        Ok(Err(IngressReason::CAPABILITY_DENIED))
    }
    /// Process at most the current next_raw input through the actual outer
    /// transaction. Results may be delayed/reordered; every returned snapshot is
    /// checked against published, lineage-visible H before CORE_COMMITTED.
    fn process_next(&mut self, _store: &mut Store<L>) -> Result<Vec<CoreCommitResultV2>> {
        Ok(vec![])
    }
}
