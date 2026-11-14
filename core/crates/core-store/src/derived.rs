//! Rebuild disposable SQL projections from canonical D and committed manifests.
use crate::{Result, error::require, objects::Objects};
use core_types::{
    WireMessage,
    generated::{CoreTransactionV2, EventEnvelope, ObjectRef},
    scalar::hex,
};
use rusqlite::{Connection, params, types::Value};

pub(crate) const TABLES: &[&str] = &[
    "object_refs",
    "visible_objects",
    "snapshot_objects",
    "accepted_results",
    "runtime_records",
    "outbox",
    "dependency_index",
    "evidence_groups",
    "evidence_index",
    "fact_corrections",
    "facts",
    "snapshots",
    "ledger_index",
    "ingress_decisions",
    "branch_head",
];

pub(crate) fn rebuild(db: &Connection, objects: &Objects) -> Result<Connection> {
    let projected = Connection::open_in_memory()?;
    // Foreign keys are intentionally off in this disposable projection: authority
    // tables are verified by recovery, never copied from possibly damaged indexes.
    projected.pragma_update(None, "foreign_keys", false)?;
    projected.execute_batch(crate::schema::INITIAL)?;
    let limits = objects.config.encoding;
    let mut input = db.prepare("SELECT event FROM raw_ingress ORDER BY position")?;
    let mut rows = input.query([])?;
    while let Some(row) = rows.next()? {
        let event = EventEnvelope::from_canonical(&row.get::<_, Vec<u8>>(0)?, limits)?;
        let root = objects.reference(&event, "1")?;
        crate::store::publish_refs(&projected, &objects.closure(&root)?, objects.config)?;
    }
    let mut stmt =
        db.prepare("SELECT id,snapshot FROM commits WHERE decision='accepted' ORDER BY position")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let root = ObjectRef::from_canonical(&row.get::<_, Vec<u8>>(1)?, limits)?;
        let value = CoreTransactionV2::from_canonical(&objects.read(&root)?, limits)?;
        let mut refs = objects.closure(&root)?;
        for entry in &value.staged_ledger {
            refs.extend(objects.closure(&objects.reference(entry, "1")?)?);
        }
        crate::store::publish_refs(&projected, &refs, objects.config)?;
        for r in &refs {
            projected.execute(
                "INSERT OR IGNORE INTO visible_objects VALUES(?1)",
                [hex(&r.digest.0)],
            )?;
        }
        crate::publication::index(&projected, objects, &id, &value, &refs)?;
        let encoded = root.canonical_bytes(limits)?;
        projected.execute("INSERT INTO snapshots VALUES(?1,?2)", params![id, encoded])?;
        let event = EventEnvelope::from_canonical(&objects.read(&value.staged_h)?, limits)?;
        projected.execute(
            "INSERT INTO facts VALUES(?1,?2,?3,?4)",
            params![
                event.id,
                crate::schema::integer(event.raw_log_position.0)?,
                id,
                value.staged_h.canonical_bytes(limits)?
            ],
        )?;
        for entry in &value.staged_ledger {
            projected.execute(
                "INSERT INTO ledger_index VALUES(?1,?2,?3,?4)",
                params![
                    entry.entry_id,
                    event.id,
                    id,
                    objects.reference(entry, "1")?.canonical_bytes(limits)?
                ],
            )?;
        }

        for correction in event.correction_of {
            projected.execute(
                "INSERT INTO fact_corrections VALUES(?1,?2)",
                params![event.id, correction],
            )?;
        }
    }
    let mut stmt =
        db.prepare("SELECT id,position,anchor,snapshot,decision FROM commits ORDER BY position")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let position: i64 = row.get(1)?;
        let anchor: i64 = row.get(2)?;
        let snapshot: Option<Vec<u8>> = row.get(3)?;
        let decision: String = row.get(4)?;
        let bytes: Vec<u8> = db.query_row(
            "SELECT event FROM raw_ingress WHERE position=?1",
            [position],
            |r| r.get(0),
        )?;
        let event = EventEnvelope::from_canonical(&bytes, limits)?;
        crate::ingress::record_decision(
            &projected,
            &event,
            &id,
            &decision,
            &crate::ingress::fingerprint(&event, limits)?,
        )?;
        projected.execute("UPDATE branch_head SET commit_id=?1,processed=?2,anchor=?3,snapshot=?4 WHERE singleton=1", params![id, position + 1, anchor, snapshot])?;
    }
    Ok(projected)
}

pub(crate) fn rows(db: &Connection, table: &str) -> Result<Vec<Vec<Value>>> {
    // table is selected exclusively from the fixed TABLES list.
    require(TABLES.contains(&table), "unknown derived table")?;
    let stmt = db.prepare(&format!("SELECT * FROM {table}"))?;
    let columns = stmt.column_count();
    let order = (1..=columns)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let mut stmt = db.prepare(&format!("SELECT * FROM {table} ORDER BY {order}"))?;
    Ok(stmt
        .query_map([], |r| (0..columns).map(|i| r.get(i)).collect())?
        .collect::<std::result::Result<_, _>>()?)
}

pub(crate) fn verify(db: &Connection, objects: &Objects) -> Result<()> {
    let projected = rebuild(db, objects)?;
    for table in TABLES {
        require(
            rows(db, table)? == rows(&projected, table)?,
            "derived index differs from authoritative history; explicit index recovery required",
        )?;
    }
    Ok(())
}
