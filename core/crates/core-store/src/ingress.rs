use crate::{
    FaultPoint, LineageKind, Result, Store,
    error::require,
    schema::integer,
    store::{head, publish_refs},
};
use core_types::{
    WireMessage,
    generated::{EventEnvelope, Receipt, ReceiptStage},
    scalar::{DecimalI64, DecimalU64, hex},
};
use rusqlite::{Connection, OptionalExtension, params};

impl<L: LineageKind> Store<L> {
    /// Acknowledges only after D and its complete object closure have committed.
    pub fn ingest(&mut self, event: &EventEnvelope) -> Result<Receipt> {
        let cfg = self.objects.config;
        let bytes = event.canonical_bytes(cfg.encoding)?;
        require(
            event.payload_digest == event.payload_ref.digest,
            "payload digest does not match object",
        )?;
        let prepared = self.objects.prepare::<L, _>(event, "1", self.fault)?;
        let guard = self
            .objects
            .pins
            .lock()
            .map_err(|_| crate::Error::Rejected("retention lock poisoned"))?;
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let (_, processed, _) = head(&tx)?;
        let count = crate::schema::unsigned(tx.query_row(
            "SELECT count(*) FROM raw_ingress",
            [],
            |r| r.get(0),
        )?)?;
        require(
            event.raw_log_position.0 == count,
            "raw position must be the next durable position",
        )?;
        integer(
            count
                .checked_add(1)
                .ok_or(crate::Error::Rejected("raw position overflow"))?,
        )?;
        let pending_bytes = crate::schema::unsigned(tx.query_row(
            "SELECT coalesce(sum(size),0) FROM raw_ingress WHERE position>=?1",
            [integer(processed)?],
            |r| r.get(0),
        )?)?;
        require(
            count - processed < cfg.max_pending_records
                && pending_bytes
                    .checked_add(bytes.len() as u64)
                    .is_some_and(|n| n <= cfg.max_pending_bytes),
            "durable ingress backpressure",
        )?;
        publish_refs(&tx, &prepared.closure, cfg)?;
        tx.execute(
            "INSERT INTO raw_ingress VALUES(?1,?2,?3)",
            params![integer(count)?, bytes, integer(bytes.len() as u64)?],
        )?;
        tx.commit()?;
        crate::witness::record(&self.root, "d", count, &prepared.reference, &self.objects)?;
        drop(guard);
        tracing::debug!(position = count, "durable ingress committed");
        Ok(Receipt {
            schema_version: 1,
            stable_source_key: event.stable_source_key.clone(),
            stage: ReceiptStage::RECEIVED_DURABLE,
            event_id: Some(event.id.clone()),
            reason: None,
        })
    }
    pub fn next_raw(&self) -> Result<Option<EventEnvelope>> {
        next_raw(&self.db, self.objects.config.encoding)
    }

    /// Deterministic idempotence settlement. New observations return None for the
    /// runtime to process; duplicate/conflicting observations settle only D/U/Q.
    pub fn settle_duplicate(&mut self) -> Result<Option<Receipt>> {
        let cfg = self.objects.config;
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let Some(event) = next_raw(&tx, cfg.encoding)? else {
            return Ok(None);
        };
        let Some(old) = prior_fingerprint(&tx, &event)? else {
            return Ok(None);
        };
        let fingerprint = fingerprint(&event, cfg.encoding)?;
        let identical = old == fingerprint;
        let decision = if identical { "filtered" } else { "conflict" };
        let (parent, processed, anchor) = head(&tx)?;
        let id = hex(&self.objects.reference(&event, "1")?.digest.0);
        let snapshot: Option<Vec<u8>> = tx.query_row(
            "SELECT snapshot FROM branch_head WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        tx.execute(
            "INSERT INTO commits VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                id,
                parent,
                integer(processed)?,
                integer(anchor)?,
                snapshot,
                decision
            ],
        )?;
        record_decision(&tx, &event, &id, decision, &fingerprint)?;
        tx.execute(
            "UPDATE branch_head SET commit_id=?1,processed=?2 WHERE singleton=1",
            params![
                id,
                integer(
                    processed
                        .checked_add(1)
                        .ok_or(crate::Error::Rejected("processed overflow"))?
                )?
            ],
        )?;
        self.fault.hit(FaultPoint::BeforePublish)?;
        tx.commit()?;
        self.fault.hit(FaultPoint::SqlCommitted)?;
        crate::witness::record(
            &self.root,
            "commits",
            processed,
            &self.objects.reference(&event, "1")?,
            &self.objects,
        )?;
        self.fault.hit(FaultPoint::AfterPublish)?;
        Ok(Some(Receipt {
            schema_version: 1,
            stable_source_key: event.stable_source_key,
            stage: if identical {
                ReceiptStage::FILTERED
            } else {
                ReceiptStage::REJECTED
            },
            event_id: Some(event.id),
            reason: if identical {
                None
            } else {
                Some("STABLE_KEY_CONFLICT".into())
            },
        }))
    }
}
pub(crate) fn next_raw(
    db: &Connection,
    limits: core_types::contract::Limits,
) -> Result<Option<EventEnvelope>> {
    let (_, processed, _) = head(db)?;
    let bytes: Option<Vec<u8>> = db
        .query_row(
            "SELECT event FROM raw_ingress WHERE position=?1",
            [integer(processed)?],
            |r| r.get(0),
        )
        .optional()?;
    bytes
        .map(|v| Ok(EventEnvelope::from_canonical(&v, limits)?))
        .transpose()
}
pub(crate) fn fingerprint(
    event: &EventEnvelope,
    limits: core_types::contract::Limits,
) -> Result<Vec<u8>> {
    let mut normalized = event.clone();
    // Transport retry position/receipt time/session can differ. The source identity,
    // occurrence time, payload, provenance and causal/correction links cannot.
    normalized.raw_log_position = DecimalU64(0);
    normalized.ingest_time_ns = DecimalI64(0);
    normalized.session.clear();
    // Session is a required nonempty ID; a fixed sentinel is only used for hashing.
    normalized.session = "transport-independent".into();
    Ok(normalized.canonical_bytes(limits)?)
}
pub(crate) fn prior_fingerprint(db: &Connection, event: &EventEnvelope) -> Result<Option<Vec<u8>>> {
    Ok(db.query_row("SELECT fingerprint FROM ingress_decisions WHERE device=?1 AND stable_key=?2 AND decision='accepted'", params![event.device, event.stable_source_key], |r| r.get(0)).optional()?)
}
pub(crate) fn record_decision(
    db: &Connection,
    event: &EventEnvelope,
    commit: &str,
    decision: &str,
    fingerprint: &[u8],
) -> Result<()> {
    db.execute(
        "INSERT INTO ingress_decisions VALUES(?1,?2,?3,?4,?5,?6)",
        params![
            integer(event.raw_log_position.0)?,
            commit,
            decision,
            event.device,
            event.stable_source_key,
            fingerprint
        ],
    )?;
    Ok(())
}
