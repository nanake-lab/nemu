//! Rebuild disposable SQL projections from canonical D and committed manifests.
use crate::{Result, error::require, objects::Objects};
use core_types::{
    WireMessage,
    generated::{EventEnvelope, ObjectRef},
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
    let projected = empty(db)?;
    extend(&projected, db, objects, 0, 0, 0)?;
    Ok(projected)
}

pub(crate) fn empty(db: &Connection) -> Result<Connection> {
    let projected = Connection::open_in_memory()?;
    // Foreign keys are intentionally off in this disposable projection: authority
    // tables are verified by recovery, never copied from possibly damaged indexes.
    projected.pragma_update(None, "foreign_keys", false)?;
    projected.execute_batch(crate::schema::INITIAL)?;
    if crate::migration::format(db)? == 3 {
        projected.execute_batch(crate::migration::ATTEMPTS_SQL)?;
    }
    Ok(projected)
}

pub(crate) fn extend(
    projected: &Connection,
    db: &Connection,
    objects: &Objects,
    inputs: u64,
    commits: u64,
    attempts: u64,
) -> Result<()> {
    let limits = objects.config.encoding;
    let mut input =
        db.prepare("SELECT event FROM raw_ingress WHERE position>=?1 ORDER BY position")?;
    let mut rows = input.query([crate::schema::integer(inputs)?])?;
    while let Some(row) = rows.next()? {
        let event = EventEnvelope::from_canonical(&row.get::<_, Vec<u8>>(0)?, limits)?;
        let root = objects.reference(&event, "1")?;
        crate::store::publish_refs(projected, &objects.closure(&root)?, objects.config)?;
    }
    let mut stmt =
        db.prepare("SELECT id,snapshot,decision FROM commits WHERE decision IN ('initial','accepted','activity') AND position>=?1 ORDER BY position")?;
    let mut rows = stmt.query([crate::schema::integer(commits)?])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let root = ObjectRef::from_canonical(&row.get::<_, Vec<u8>>(1)?, limits)?;
        if row.get::<_, String>(2)? == "initial" {
            let p = crate::initial::decode(objects, &root)?;
            crate::initial::project(projected, objects, &root, &p)?;
            continue;
        }
        let value = crate::manifest::decode(objects, &root)?;
        let mut refs = objects.closure(&root)?;
        for entry in &value.staged_ledger {
            refs.extend(objects.closure(&objects.reference(entry, "1")?)?);
        }
        crate::publication::visible(projected, objects, &refs)?;
        crate::publication::index(projected, objects, &id, &value, &refs)?;
        let encoded = root.canonical_bytes(limits)?;
        projected.execute("INSERT INTO snapshots VALUES(?1,?2)", params![id, encoded])?;
        if value.activity_commit.is_some() {
            continue;
        }
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
    if crate::migration::format(db)? == 3 {
        crate::activity_audit::rebuild(db, projected, objects, attempts)?;
    }
    let mut processed: i64 = projected.query_row(
        "SELECT processed FROM branch_head WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    let mut stmt =
        db.prepare("SELECT id,position,anchor,snapshot,decision FROM commits WHERE position>=?1 ORDER BY position")?;
    let mut rows = stmt.query([crate::schema::integer(commits)?])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let anchor: i64 = row.get(2)?;
        let snapshot: Option<Vec<u8>> = row.get(3)?;
        let decision: String = row.get(4)?;
        if !matches!(decision.as_str(), "activity" | "initial") {
            let bytes: Vec<u8> = db.query_row(
                "SELECT event FROM raw_ingress WHERE position=?1",
                [processed],
                |r| r.get(0),
            )?;
            let event = EventEnvelope::from_canonical(&bytes, limits)?;
            crate::ingress::record_decision(
                projected,
                &event,
                &id,
                &decision,
                &crate::ingress::fingerprint(&event, limits)?,
            )?;
            processed += 1;
        }
        projected.execute("UPDATE branch_head SET commit_id=?1,processed=?2,anchor=?3,snapshot=?4 WHERE singleton=1", params![id, processed, anchor, snapshot])?;
    }
    Ok(())
}

pub(crate) fn rows(db: &Connection, table: &str) -> Result<Vec<Vec<Value>>> {
    rows_since(db, table, None)
}

fn rows_since(db: &Connection, table: &str, frontier: Option<i64>) -> Result<Vec<Vec<Value>>> {
    require(
        TABLES.contains(&table) || table == "activity_keys",
        "unknown derived table",
    )?;
    let columns = db
        .prepare(&format!("SELECT * FROM {table}"))?
        .column_count();
    let order = (1..=columns)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let filter = if frontier.is_some() {
        "WHERE rowid > ?1"
    } else {
        ""
    };
    let mut stmt = db.prepare(&format!("SELECT * FROM {table} {filter} ORDER BY {order}"))?;
    Ok(stmt
        .query_map(rusqlite::params_from_iter(frontier), |r| {
            (0..columns).map(|i| r.get(i)).collect()
        })?
        .collect::<std::result::Result<_, _>>()?)
}

/// Each database allocates its own rowids. The Store only appends to these
/// projections, except for the singleton head which we always compare in full.
#[derive(Default)]
pub(crate) struct Frontiers(std::collections::BTreeMap<&'static str, (Option<i64>, Option<i64>)>);

impl Frontiers {
    pub fn compare(&mut self, db: &Connection, projected: &Connection) -> Result<()> {
        let activity = (crate::migration::format(db)? == 3).then_some("activity_keys");
        for table in TABLES.iter().copied().chain(activity) {
            let (actual, expected) = self.0.get(table).copied().unwrap_or_default();
            require(
                rows_since(db, table, actual)? == rows_since(projected, table, expected)?,
                "derived index differs from authoritative history; explicit index recovery required",
            )?;
            if table != "branch_head" {
                let sql = format!("SELECT MAX(rowid) FROM {table}");
                self.0.insert(
                    table,
                    (
                        db.query_row(&sql, [], |r| r.get(0))?,
                        projected.query_row(&sql, [], |r| r.get(0))?,
                    ),
                );
            }
        }
        Ok(())
    }
}

pub(crate) fn verify(db: &Connection, objects: &Objects) -> Result<()> {
    let projected = rebuild(db, objects)?;
    if crate::migration::format(db)? == 3 {
        require(
            rows(db, "activity_keys")? == rows(&projected, "activity_keys")?,
            "internal activity index mismatch",
        )?;
    }
    for table in TABLES {
        require(
            rows(db, table)? == rows(&projected, table)?,
            "derived index differs from authoritative history; explicit index recovery required",
        )?;
    }
    Ok(())
}
