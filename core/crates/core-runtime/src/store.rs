//! W02 integration over public APIs. ADR 0049 bootstrap and staged event computation.
use crate::{
    Error, Result,
    binding::digest,
    check,
    pipeline::{StagedEvent, Step, verify_order},
    stages::{Snapshot, StageSet},
    status::Status,
    validate,
};
use core_store::{LineageKind, PreparedObject, PreparedTransaction, ReadTicket, Store};
use core_types::{contract::Limits, generated::*, scalar::DecimalU64};

pub enum Next {
    Idle,
    Settled(Receipt),
    Event(Box<EventEnvelope>),
}
/// Gamma is settled before capturing time or running any mathematical stage.
pub fn next<L: LineageKind>(store: &mut Store<L>) -> Result<Next> {
    if let Some(receipt) = store.settle_duplicate()? {
        return Ok(Next::Settled(receipt));
    }
    Ok(store
        .next_raw()?
        .map(|event| Next::Event(Box::new(event)))
        .unwrap_or(Next::Idle))
}
/// Publish the supplied rehearsal starting state with no connected stages.
pub fn bootstrap_rehearsal(
    store: &mut Store<core_store::Rehearsal>,
    capability: &core_store::CommitCapability<core_store::Rehearsal>,
    environment: &VersionedEnvironmentV2,
    initial: &Snapshot,
) -> Result<ObjectRef> {
    // Detect a damaged prior publication before a retry could recreate its objects.
    store.verify()?;
    let x = store.prepare(&initial.x, &initial.x.revision)?;
    let r = store.prepare_r(&initial.r)?;
    let theta = store.prepare(&initial.theta, &initial.theta.revision)?;
    Ok(store.publish_rehearsal_initial(capability, environment, &x, &r, &theta)?)
}

pub fn read_state<L: LineageKind>(store: &Store<L>) -> Result<Snapshot> {
    Ok(committed_state(store)?.0)
}
fn committed_state<L: LineageKind>(
    store: &Store<L>,
) -> Result<(Snapshot, Option<InitialRuntimeRoot>)> {
    let reader = store.verified_reader()?;
    let reference = reader.current_snapshot()?.ok_or(Error::Rejected(
        "genesis/bootstrap publication contract required",
    ))?;
    let read_q = |q: &ObjectRef| -> core_store::Result<Option<InitialRuntimeRoot>> {
        if q.object_type == "echo.initial-runtime-root" {
            reader.read(q).map(Some)
        } else {
            Ok(None)
        }
    };
    let (x, r, theta, initial_q) = match reference.object_type.as_str() {
        "echo.initial-publication" => {
            let root = reader.read::<InitialPublication>(&reference)?;
            (root.x, root.r, root.theta, Some(root.q))
        }
        "echo.transaction-v4" => {
            let tx = reader.read::<CoreTransactionV4>(&reference)?;
            (
                tx.staged_x,
                tx.staged_r,
                tx.staged_theta,
                read_q(&tx.staged_q)?,
            )
        }
        _ => {
            let tx = reader.event_snapshot(&reference)?;
            (
                tx.staged_x,
                tx.staged_r,
                tx.staged_theta,
                read_q(&tx.staged_q)?,
            )
        }
    };
    Ok((
        Snapshot {
            x: reader.read(&x)?,
            r: reader.read(&r)?,
            theta: reader.read(&theta)?,
        },
        initial_q,
    ))
}

/// Pins survive until the caller finishes W02's commit or abort. No early z,
/// concept or x can be read through Reader. This is a structural preparation
/// boundary, NOT durable completion of the W03 clock/runtime publication.
pub struct ExistingPreparation<L: LineageKind> {
    pub transaction: PreparedTransaction<L>,
    pub status: DegradedStatus,
    pub pins: Vec<PreparedObject<L>>,
}
pub fn prepare_existing_head<L: LineageKind>(
    store: &Store<L>,
    ticket: ReadTicket<L>,
    staged: StagedEvent<L>,
    limits: Limits,
) -> Result<ExistingPreparation<L>> {
    check(
        crate::same(ticket.binding(), &staged.binding, limits)?,
        "ticket is not the staged computation",
    )?;
    verify_order(&staged.trace)?;
    check(
        staged.trace.last() == Some(&Step::Ledger),
        "computation incomplete",
    )?;
    let old = store.published_roots()?.ok_or(Error::Rejected(
        "genesis/bootstrap publication contract required",
    ))?;
    // No invented representation for a changed aggregate L or runtime Q.
    check(
        staged.ledger.is_empty(),
        "L root publication requires successor contract",
    )?;
    let previous = read_state(store)?;
    check(
        crate::same(&previous.x, &staged.base.x, limits)?
            && crate::same(&previous.r, &staged.base.r, limits)?
            && crate::same(&previous.theta, &staged.base.theta, limits)?,
        "input was not the committed base snapshot",
    )?;
    check(
        crate::same(&previous.r, &staged.state.r, limits)?
            && crate::same(&previous.theta, &staged.state.theta, limits)?,
        "growth publication requires successor L/Q contract",
    )?;
    let mut pins = vec![];
    let h = pin(store, &staged.event, &mut pins)?;
    let x = pin(store, &staged.state.x, &mut pins)?;
    let r_pin = store.prepare_r(&staged.state.r)?;
    let r = r_pin.reference().clone();
    pins.push(r_pin);
    // No growth here: preserve the committed reference version as well as all
    // estimator bytes (ADR 0029); Store::prepare enforces the successor gate.
    let theta_pin = store.prepare(&staged.state.theta, &old.theta.resource_version)?;
    let theta = theta_pin.reference().clone();
    pins.push(theta_pin);
    let status = staged.status.value();
    let audit = pin(store, &status, &mut pins)?;
    let mut accepted = vec![];
    let mut concepts = vec![];
    if let Some(z) = &staged.semantic {
        accepted.push(pin(store, &z.graph, &mut pins)?);
        for c in &z.concepts {
            concepts.push(pin(store, c, &mut pins)?);
        }
    }
    if let Some(a) = &staged.appraisal {
        accepted.push(pin(store, a, &mut pins)?);
    }
    if let Some(m) = &staged.matching {
        accepted.push(pin(store, &m.record, &mut pins)?);
        for c in &m.candidates {
            accepted.push(pin(store, c, &mut pins)?);
        }
    }
    let position = staged
        .event
        .raw_log_position
        .0
        .checked_add(1)
        .ok_or(Error::Rejected("position overflow"))?;
    let transaction = CoreTransactionV4 {
        schema_version: 1,
        staged_h: h,
        staged_x: x,
        staged_r: r,
        staged_theta: theta,
        staged_l: old.l,
        staged_q: old.q,
        staged_u: old.u,
        processed_position: DecimalU64(position),
        anchor_time_ns: DecimalU64(staged.time.ns()),
        staged_concepts: concepts,
        accepted_results: accepted,
        audit_refs: vec![audit],
        event_binding: Some(staged.binding),
        activity_commit: None,
        staged_ledger: vec![],
    };
    Ok(ExistingPreparation {
        transaction: store.stage_event_v4(ticket, transaction)?,
        status,
        pins,
    })
}
fn pin<L: LineageKind, T: core_types::WireMessage>(
    store: &Store<L>,
    value: &T,
    pins: &mut Vec<PreparedObject<L>>,
) -> Result<ObjectRef> {
    let pin = store.prepare(value, "1")?;
    let reference = pin.reference().clone();
    pins.push(pin);
    Ok(reference)
}
pub enum Emission {
    /// No forged modulation envelope/receipt when disabled or unavailable.
    Unmodulated(Status),
}
impl<L: LineageKind> StageSet<L> {
    /// Reads only a committed W02 snapshot. A queued observation blocks use of an
    /// older plan. Scheduling/continuation and actions belong to later cards.
    pub fn compile_committed(&self, store: &Store<L>, generation_ns: u64) -> Result<Emission> {
        check(
            store.next_raw()?.is_none(),
            "ingress frontier advanced before generation",
        )?;
        let (state, initial_q) = committed_state(store)?;
        check(
            generation_ns >= state.x.anchor_time_ns.0,
            "generation before committed anchor",
        )?;
        if L::KIND == Lineage::REHEARSAL || self.parts.j.is_none() || self.parts.f.is_none() {
            return Ok(Emission::Unmodulated(Status::unmodulated(
                L::KIND,
                initial_q.as_ref(),
            )));
        }
        // Reading a committed x alone does not prove that Q permits a new
        // generation. Do not produce nonzero m until Q publication is defined.
        // The pure J trait is available for its owner; no runtime service uses it.
        Err(Error::Rejected(
            "authoritative Q capability publication requires successor contract",
        ))
    }
}

/// Validation helper for the future J adapter. A valid envelope is data, never
/// permission to generate or act. This function has no publication side effects.
pub fn validate_compilation(
    envelope: &ModulationEnvelopeV2,
    snapshot_id: &str,
    state: &Snapshot,
    generation_ns: u64,
    environment: &VersionedEnvironmentV2,
    limits: Limits,
) -> Result<()> {
    validate(envelope, limits)?;
    check(
        envelope.snapshot_id == snapshot_id
            && envelope.projected_at_ns.0 == generation_ns
            && state.x.anchor_time_ns.0 == generation_ns
            && envelope.x_digest == digest(&state.x, limits)?
            && envelope.r_digest == digest(&state.r, limits)?
            && envelope.theta_digest == digest(&state.theta, limits)?
            && envelope.carrier == environment.carrier
            && envelope.model == environment.model
            && envelope.tokenizer == environment.tokenizer
            && Some(&envelope.direction_set) == environment.direction_set.as_ref(),
        "J binding mismatch",
    )
}

/// ADR 0031: every update, even an equal/zero m, clears KV, installs the vector,
/// then prefills. No old-cache recovery or silent retry after any failure.
pub trait CarrierUpdate {
    fn clear_kv(&mut self) -> Result<()>;
    fn install(&mut self, envelope: &ModulationEnvelopeV2) -> Result<()>;
    fn prefill(&mut self) -> Result<()>;
}
pub fn update_carrier(
    carrier: &mut impl CarrierUpdate,
    envelope: &ModulationEnvelopeV2,
    limits: Limits,
) -> Result<()> {
    validate(envelope, limits)?;
    carrier.clear_kv()?;
    carrier.install(envelope)?;
    carrier.prefill()
}
