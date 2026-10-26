use crate::{
    Error, FaultPoint, LineageKind, PreparedObject, Result, Store,
    error::require,
    ingress::{fingerprint, next_raw, prior_fingerprint, record_decision},
    publication,
    schema::integer,
    store::{head, publish_refs},
};
use core_types::{
    WireMessage,
    contract::Schema,
    generated::{
        CoreCommitResultV2, CoreTransactionV2, DegradedStatus, EventEnvelope, FastStateSnapshotV2,
        ObjectRef, ProposalBindingV2, RSnapshotV3, ThetaSnapshot,
    },
    scalar::hex,
};
use rusqlite::params;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// The runtime takes this once. Workers receive permitted views/proposals, never
/// a Store or this capability. It is private, non-Clone and specific to one opening.
///
/// ```compile_fail
/// use core_store::{CommitCapability, Rehearsal};
/// let forged = CommitCapability::<Rehearsal> {};
/// ```
pub struct CommitCapability<L: LineageKind> {
    owner: Arc<Mutex<BTreeMap<String, usize>>>,
    marker: std::marker::PhantomData<L>,
}
pub struct ReadTicket<L: LineageKind> {
    binding: ProposalBindingV2,
    pinned: PreparedObject<L>,
    event: EventEnvelope,
}
impl<L: LineageKind> ReadTicket<L> {
    pub fn binding(&self) -> &ProposalBindingV2 {
        &self.binding
    }
}
/// A prepared transaction cannot be passed into another lineage's commit API.
/// ```compile_fail
/// use core_store::{Store, Life, Rehearsal, CommitCapability, PreparedTransaction};
/// use core_types::generated::DegradedStatus;
/// fn cross_lineage(s: &mut Store<Life>, c: &CommitCapability<Life>,
///     t: PreparedTransaction<Rehearsal>, status: DegradedStatus) {
///     s.commit(c, t, status);
/// }
/// ```
pub struct PreparedTransaction<L: LineageKind> {
    value: CoreTransactionV2,
    snapshot: PreparedObject<L>,
    ticket: ReadTicket<L>,
    ledger: Vec<PreparedObject<L>>,
}

impl<L: LineageKind> Store<L> {
    pub fn take_commit_capability(&mut self) -> Result<CommitCapability<L>> {
        require(!self.capability_issued, "commit capability already issued")?;
        self.capability_issued = true;
        Ok(CommitCapability {
            owner: Arc::clone(&self.objects.pins),
            marker: std::marker::PhantomData,
        })
    }
    /// Before the first commit, the canonical lineage header identifies the base.
    pub fn base_commit(&self) -> Result<String> {
        Ok(self
            .head()?
            .0
            .unwrap_or(hex(&self.objects.reference(&self.header, "1")?.digest.0)))
    }
    /// Called by the trusted runtime BEFORE dispatch. The runtime supplies the
    /// projected-state/accepted-Z/c_core digests; W02 preserves and checks their
    /// exact binding, while W04 computes them. Every runtime read must be declared.
    pub fn bind(&self, binding: ProposalBindingV2) -> Result<ReadTicket<L>> {
        self.check_binding(&binding)?;
        let event = self
            .next_raw()?
            .ok_or(Error::Rejected("no durable input"))?;
        let pinned = self.objects.prepare(&binding, "1", &self.fault)?;
        Ok(ReadTicket {
            binding,
            pinned,
            event,
        })
    }
    pub fn read_bound<T: WireMessage>(
        &self,
        ticket: &ReadTicket<L>,
        reference: &ObjectRef,
    ) -> Result<T> {
        require(
            Arc::ptr_eq(&ticket.pinned.pins, &self.objects.pins),
            "foreign read capability",
        )?;
        require(
            ticket.binding.read_set.contains(reference),
            "read absent from declared read set",
        )?;
        Ok(T::from_canonical(
            &self.objects.read(reference)?,
            self.objects.config.encoding,
        )?)
    }
    pub fn stage(
        &self,
        ticket: ReadTicket<L>,
        value: CoreTransactionV2,
    ) -> Result<PreparedTransaction<L>> {
        let limits = self.objects.config.encoding;
        require(
            Arc::ptr_eq(&ticket.pinned.pins, &self.objects.pins),
            "foreign staging capability",
        )?;
        require(
            value.binding.canonical_bytes(limits)? == ticket.binding.canonical_bytes(limits)?,
            "proposal binding changed after dispatch",
        )?;
        self.check_binding(&ticket.binding)?;
        let next = ticket
            .event
            .raw_log_position
            .0
            .checked_add(1)
            .ok_or(Error::Rejected("processed overflow"))?;
        require(
            value.processed_position.0 == next
                && value.anchor_time_ns == value.binding.process_time_ns,
            "position or time anchor mismatch",
        )?;
        integer(next)?;
        integer(value.anchor_time_ns.0)?;
        require(
            value.staged_h == self.objects.reference(&ticket.event, "1")?,
            "H does not identify the next durable input",
        )?;
        let x = FastStateSnapshotV2::from_canonical(&self.objects.read(&value.staged_x)?, limits)?;
        require(
            x.anchor_time_ns == value.anchor_time_ns,
            "staged x has wrong anchor",
        )?;
        RSnapshotV3::from_canonical(&self.objects.read(&value.staged_r)?, limits)?;
        self.fault.hit(FaultPoint::GStaged)?;
        ThetaSnapshot::from_canonical(&self.objects.read(&value.staged_theta)?, limits)?;
        self.fault.hit(FaultPoint::CStaged)?;
        let mut ledger = Vec::new();
        for entry in &value.staged_ledger {
            require(
                entry.event_id == ticket.event.id,
                "ledger entry belongs to another event",
            )?;
            ledger.push(self.objects.prepare(entry, "1", &self.fault)?);
        }
        let snapshot = self.objects.prepare(&value, "1", &self.fault)?;
        Ok(PreparedTransaction {
            value,
            snapshot,
            ticket,
            ledger,
        })
    }
    pub fn commit(
        &mut self,
        capability: &CommitCapability<L>,
        staged: PreparedTransaction<L>,
        status: DegradedStatus,
    ) -> Result<CoreCommitResultV2> {
        require(
            Arc::ptr_eq(&capability.owner, &self.objects.pins)
                && Arc::ptr_eq(&staged.snapshot.pins, &self.objects.pins),
            "foreign commit capability",
        )?;
        let cfg = self.objects.config;
        status.canonical_bytes(cfg.encoding)?;
        let guard = self
            .objects
            .pins
            .lock()
            .map_err(|_| Error::Rejected("retention lock poisoned"))?;
        self.check_binding(&staged.ticket.binding)?;
        // Re-read every byte in the retained closure immediately before publish.
        // No other writer/GC can interleave while this store owns the writer lock.
        let mut refs = self.objects.closure(&staged.snapshot.reference)?;
        for entry in &staged.ledger {
            refs.extend(self.objects.closure(&entry.reference)?);
        }
        let event = &staged.ticket.event;
        let id = hex(&staged.snapshot.reference.digest.0);
        let snapshot_bytes = staged.snapshot.reference.canonical_bytes(cfg.encoding)?;
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let (parent, processed, anchor) = head(&tx)?;
        require(
            processed == event.raw_log_position.0
                && anchor == staged.ticket.binding.anchor_time_ns.0,
            "stale input frontier",
        )?;
        require(
            prior_fingerprint(&tx, event)?.is_none(),
            "stable key must be settled before state transition",
        )?;
        publish_refs(&tx, &refs, cfg)?;
        for r in &refs {
            tx.execute(
                "INSERT OR IGNORE INTO visible_objects VALUES(?1)",
                [hex(&r.digest.0)],
            )?;
        }
        tx.execute(
            "INSERT INTO commits VALUES(?1,?2,?3,?4,?5,'accepted')",
            params![
                id,
                parent,
                integer(processed)?,
                integer(staged.value.anchor_time_ns.0)?,
                snapshot_bytes
            ],
        )?;
        tx.execute(
            "INSERT INTO facts VALUES(?1,?2,?3,?4)",
            params![
                event.id,
                integer(processed)?,
                id,
                staged.value.staged_h.canonical_bytes(cfg.encoding)?
            ],
        )?;
        for correction in &event.correction_of {
            tx.execute(
                "INSERT INTO fact_corrections VALUES(?1,?2)",
                params![event.id, correction],
            )?;
        }
        record_decision(
            &tx,
            event,
            &id,
            "accepted",
            &fingerprint(event, cfg.encoding)?,
        )?;
        tx.execute(
            "INSERT INTO snapshots VALUES(?1,?2)",
            params![id, snapshot_bytes],
        )?;
        publication::index(&tx, &self.objects, &id, &staged.value, &refs)?;
        for (entry, object) in staged.value.staged_ledger.iter().zip(&staged.ledger) {
            tx.execute(
                "INSERT INTO ledger_index VALUES(?1,?2,?3,?4)",
                params![
                    entry.entry_id,
                    event.id,
                    id,
                    object.reference.canonical_bytes(cfg.encoding)?
                ],
            )?;
            self.fault.hit(FaultPoint::LedgerWritten)?;
        }
        tx.execute("UPDATE branch_head SET commit_id=?1,processed=?2,anchor=?3,snapshot=?4 WHERE singleton=1", params![id, integer(staged.value.processed_position.0)?, integer(staged.value.anchor_time_ns.0)?, snapshot_bytes])?;
        self.fault.hit(FaultPoint::BeforePublish)?;
        tx.commit()?;
        self.fault.hit(FaultPoint::SqlCommitted)?;
        crate::witness::record(
            &self.root,
            "commits",
            processed,
            &staged.snapshot.reference,
            &self.objects,
            &self.fault,
        )?;
        self.fault.hit(FaultPoint::AfterPublish)?;
        drop(guard);
        tracing::debug!(position = processed, "core transaction committed");
        Ok(CoreCommitResultV2 {
            schema_version: 1,
            branch_id: self.branch.clone(),
            event_id: event.id.clone(),
            commit_id: id,
            committed: true,
            snapshot: staged.snapshot.reference.clone(),
            status,
            ledger_entries: staged
                .value
                .staged_ledger
                .iter()
                .map(|e| e.entry_id.clone())
                .collect(),
        })
    }
    fn check_binding(&self, binding: &ProposalBindingV2) -> Result<()> {
        let limits = self.objects.config.encoding;
        binding.canonical_bytes(limits)?;
        let (_, processed, anchor) = self.head()?;
        require(
            binding.branch_id == self.branch && binding.base_commit == self.base_commit()?,
            "stale branch head",
        )?;
        require(
            binding.raw_log_position.0 == processed
                && binding.anchor_time_ns.0 == anchor
                && binding.process_time_ns.0 >= anchor,
            "stale position or anchor",
        )?;
        integer(binding.process_time_ns.0)?;
        let event = next_raw(&self.db, limits)?.ok_or(Error::Rejected("no durable input"))?;
        require(
            binding.event_id == event.id && prior_fingerprint(&self.db, &event)?.is_none(),
            "input identity or stable key conflict",
        )?;
        let schema = Schema::builtin()?;
        let mut required = vec![binding.validation_snapshot.clone()];
        crate::objects::collect_refs(
            &schema,
            core_types::generated::VersionedEnvironmentV2::NAME,
            &serde_json::to_value(&binding.environment)?,
            &mut required,
        )?;
        if let Some(old) = self.current()? {
            let x =
                FastStateSnapshotV2::from_canonical(&self.objects.read(&old.staged_x)?, limits)?;
            let r = RSnapshotV3::from_canonical(&self.objects.read(&old.staged_r)?, limits)?;
            let theta =
                ThetaSnapshot::from_canonical(&self.objects.read(&old.staged_theta)?, limits)?;
            require(
                binding.x_revision == x.revision
                    && binding.r_revision == r.revision
                    && binding.theta_revision == theta.revision,
                "stale state revisions",
            )?;
            required.extend([
                old.staged_h,
                old.staged_x,
                old.staged_r,
                old.staged_theta,
                old.staged_l,
                old.staged_q,
                old.staged_u,
            ]);
        }
        for r in required {
            require(binding.read_set.contains(&r), "incomplete read set")?;
        }
        for r in &binding.read_set {
            self.objects.closure(r)?;
        }
        Ok(())
    }
}
