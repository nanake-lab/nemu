use crate::{
    Error, FaultPoint, LineageKind, Result, Store,
    error::require,
    schema::{integer, unsigned},
};
use core_types::{
    WireMessage,
    generated::*,
    scalar::{DecimalI64, DecimalU64},
    terminal_ingress::check_session,
};
use rusqlite::params;

fn server_time() -> Result<DecimalI64> {
    let ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| Error::Rejected("server clock before epoch"))?
        .as_nanos();
    Ok(DecimalI64(
        i64::try_from(ns).map_err(|_| Error::Rejected("server clock overflow"))?,
    ))
}

impl<L: LineageKind> Store<L> {
    /// The caller supplies its server-authenticated connection context. A wire
    /// session or decoded request alone is not authentication (ADR 0024).
    pub fn ingest_terminal(
        &mut self,
        input: &TerminalEvent,
        authenticated: &TerminalSession,
    ) -> Result<ReceiptV2> {
        let limits = self.objects.config.encoding;
        input.canonical_bytes(limits)?;
        check_session(authenticated, limits)?;
        require(
            authenticated
                .capabilities
                .contains(&IngressKind::INGRESS_EVENT),
            "event capability denied",
        )?;
        let position = unsigned(self.db.query_row(
            "SELECT count(*) FROM raw_ingress",
            [],
            |r| r.get(0),
        )?)?;
        let time = server_time()?;
        self.fault.hit(FaultPoint::IngressAllocated)?;
        // Exclusive Store writer + &mut self spans allocation and durable ingest.
        // No next-position counter survives a failed or interrupted call.
        let event = EventEnvelope {
            schema_version: 1,
            id: input.id.clone(),
            kind: input.kind,
            event_time_ns: input.event_time_ns,
            ingest_time_ns: time,
            raw_log_position: DecimalU64(position),
            parents: input.parents.clone(),
            payload_ref: input.payload_ref.clone(),
            provenance: input.provenance.clone(),
            device: authenticated.installation_id.clone(),
            session: authenticated.session_id.clone(),
            stable_source_key: input.stable_source_key.clone(),
            payload_digest: input.payload_digest.clone(),
            action_id: input.action_id.clone(),
            trial_id: input.trial_id.clone(),
            correction_of: input.correction_of.clone(),
            derivation: input.derivation,
        };
        self.ingest(&event)?;
        Ok(ReceiptV2 {
            schema_version: 1,
            stable_source_key: input.stable_source_key.clone(),
            stage: ReceiptStage::RECEIVED_DURABLE,
            event_id: Some(input.id.clone()),
            reason_code: None,
            raw_log_position: Some(DecimalU64(position)),
            ingest_time_ns: Some(time),
        })
    }

    /// No arbitrary client text, IDs, raw frames or credential digests are accepted.
    /// Returns only after both the independent row and its witness are durable.
    pub fn audit_ingress_rejection(
        &mut self,
        reason: IngressReason,
        kind: Option<IngressKind>,
        budget: &IngressAuditBudget,
    ) -> Result<IngressRejection> {
        budget.canonical_bytes(self.objects.config.encoding)?;
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let (count, size): (i64, i64) = tx.query_row(
            "SELECT count(*),coalesce(sum(length(record)),0) FROM ingress_rejections",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let record = IngressRejection {
            schema_version: 1,
            position: DecimalU64(unsigned(count)?),
            recorded_at_ns: server_time()?,
            reason,
            request_kind: kind,
        };
        let bytes = record.canonical_bytes(self.objects.config.encoding)?;
        if unsigned(count)? >= u64::from(budget.max_records)
            || unsigned(size)?
                .checked_add(bytes.len() as u64)
                .is_none_or(|n| n > u64::from(budget.max_bytes))
        {
            return Err(Error::Backpressure(IngressReason::AUDIT_FULL));
        }
        tx.execute(
            "INSERT INTO ingress_rejections VALUES(?1,?2)",
            params![integer(record.position.0)?, bytes],
        )?;
        self.fault.hit(FaultPoint::AuditBeforeSql)?;
        tx.commit()?;
        self.fault.hit(FaultPoint::AuditSqlCommitted)?;
        crate::witness::record(
            &self.root,
            "reject",
            record.position.0,
            &self.objects.reference(&record, "1")?,
            &self.objects,
            &self.fault,
        )?;
        Ok(record)
    }
    pub fn ingress_rejections(&self) -> Result<Vec<IngressRejection>> {
        let mut stmt = self
            .db
            .prepare("SELECT record FROM ingress_rejections ORDER BY position")?;
        stmt.query_map([], |r| r.get::<_, Vec<u8>>(0))?
            .map(|row| {
                Ok(IngressRejection::from_canonical(
                    &row?,
                    self.objects.config.encoding,
                )?)
            })
            .collect()
    }
}

pub(crate) fn verify_audit(
    db: &rusqlite::Connection,
    limits: core_types::contract::Limits,
) -> Result<()> {
    if crate::migration::format(db)? == 1 {
        return Ok(());
    }
    let mut stmt =
        db.prepare("SELECT position,record FROM ingress_rejections ORDER BY position")?;
    for (expected, row) in stmt
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?)))?
        .enumerate()
    {
        let (position, bytes) = row?;
        let record = IngressRejection::from_canonical(&bytes, limits)?;
        require(
            unsigned(position)? == expected as u64 && record.position.0 == expected as u64,
            "rejection audit has a gap or wrong position",
        )?;
    }
    Ok(())
}
