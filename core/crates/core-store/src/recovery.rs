use crate::{
    LineageKind, Result, Store,
    error::require,
    ingress::fingerprint,
    schema::{integer, unsigned},
};
use core_types::{
    WireMessage,
    generated::{CoreTransactionV2, EventEnvelope, LedgerEntry, ObjectRef},
    scalar::hex,
};
use rusqlite::params;
use std::collections::BTreeMap;

/// SQLite page integrity alone cannot prove that valid pointers form one snapshot.
/// Reconcile the physical indexes against their canonical transaction manifests.
pub(crate) fn verify<L: LineageKind>(store: &Store<L>) -> Result<()> {
    let db = &store.db;
    let limits = store.objects.config.encoding;
    let mut input = db.prepare("SELECT position,event FROM raw_ingress ORDER BY position")?;
    let mut rows = input.query([])?;
    let mut count = 0u64;
    while let Some(row) = rows.next()? {
        let event = EventEnvelope::from_canonical(&row.get::<_, Vec<u8>>(1)?, limits)?;
        require(
            unsigned(row.get(0)?)? == count && event.raw_log_position.0 == count,
            "durable input has a gap or mismatched position",
        )?;
        require(
            event.payload_digest == event.payload_ref.digest,
            "durable input payload mismatch",
        )?;
        count = count
            .checked_add(1)
            .ok_or(crate::Error::Rejected("durable input overflow"))?;
    }
    let mut stmt = db.prepare(
        "SELECT id,parent,position,anchor,snapshot,decision FROM commits ORDER BY position",
    )?;
    let mut rows = stmt.query([])?;
    let mut parent: Option<String> = None;
    let mut position = 0u64;
    let mut anchor = 0u64;
    let mut snapshot: Option<Vec<u8>> = None;
    let mut accepted = 0i64;
    let mut ledger_count = 0i64;
    let mut seen = BTreeMap::new();
    let initial = hex(&store.objects.reference(&store.header, "1")?.digest.0);
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let next_anchor = unsigned(row.get(3)?)?;
        let next_snapshot: Option<Vec<u8>> = row.get(4)?;
        let decision: String = row.get(5)?;
        require(
            row.get::<_, Option<String>>(1)? == parent && unsigned(row.get(2)?)? == position,
            "commit chain is broken",
        )?;
        let event_bytes: Vec<u8> = db.query_row(
            "SELECT event FROM raw_ingress WHERE position=?1",
            [integer(position)?],
            |r| r.get(0),
        )?;
        let event = EventEnvelope::from_canonical(&event_bytes, limits)?;
        let event_ref = store.objects.reference(&event, "1")?;
        let fp = fingerprint(&event, limits)?;
        let key = (event.device.clone(), event.stable_source_key.clone());
        if decision == "accepted" {
            require(!seen.contains_key(&key), "stable key accepted twice")?;
            seen.insert(key, fp.clone());
            let bytes = next_snapshot
                .as_ref()
                .ok_or(crate::Error::Rejected("accepted commit lacks snapshot"))?;
            let reference = ObjectRef::from_canonical(bytes, limits)?;
            let value =
                CoreTransactionV2::from_canonical(&store.objects.read(&reference)?, limits)?;
            require(
                hex(&reference.digest.0) == id
                    && value.binding.base_commit == parent.as_ref().unwrap_or(&initial).as_str()
                    && value.binding.branch_id == store.branch
                    && value.binding.event_id == event.id
                    && value.binding.raw_log_position.0 == position
                    && value.binding.anchor_time_ns.0 == anchor,
                "canonical commit binding differs from commit chain",
            )?;
            require(
                value.staged_h == event_ref
                    && value.processed_position.0 == position + 1
                    && value.anchor_time_ns.0 == next_anchor
                    && value.binding.process_time_ns.0 == next_anchor
                    && next_anchor >= anchor,
                "canonical snapshot differs from committed frontier",
            )?;
            let stored: Vec<u8> = db.query_row(
                "SELECT reference FROM snapshots WHERE commit_id=?1",
                [&id],
                |r| r.get(0),
            )?;
            require(stored == *bytes, "snapshot index differs from commit")?;
            let stored: Vec<u8> = db.query_row(
                "SELECT reference FROM facts WHERE event_id=?1 AND position=?2 AND commit_id=?3",
                params![event.id, integer(position)?, id],
                |r| r.get(0),
            )?;
            require(
                stored == event_ref.canonical_bytes(limits)?,
                "fact index differs from commit",
            )?;
            for entry in &value.staged_ledger {
                let stored: Vec<u8> = db.query_row("SELECT reference FROM ledger_index WHERE entry_id=?1 AND event_id=?2 AND commit_id=?3", params![entry.entry_id, event.id, id], |r| r.get(0))?;
                let reference = ObjectRef::from_canonical(&stored, limits)?;
                require(
                    LedgerEntry::from_canonical(&store.objects.read(&reference)?, limits)?
                        .canonical_bytes(limits)?
                        == entry.canonical_bytes(limits)?,
                    "ledger index differs from canonical transaction",
                )?;
                ledger_count += 1;
            }
            accepted += 1;
        } else {
            let old = seen.get(&key).ok_or(crate::Error::Rejected(
                "settlement has no earlier accepted source",
            ))?;
            require(
                (decision == "filtered" && old == &fp) || (decision == "conflict" && old != &fp),
                "incorrect duplicate/conflict decision",
            )?;
            require(
                hex(&event_ref.digest.0) == id
                    && next_anchor == anchor
                    && next_snapshot == snapshot,
                "filtered input changed authority state",
            )?;
        }
        let stored: (String,String,String,String,Vec<u8>) = db.query_row("SELECT commit_id,decision,device,stable_key,fingerprint FROM ingress_decisions WHERE position=?1", [integer(position)?], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;
        require(
            stored
                == (
                    id.clone(),
                    decision,
                    event.device,
                    event.stable_source_key,
                    fp,
                ),
            "ingress decision differs from canonical history",
        )?;
        parent = Some(id);
        anchor = next_anchor;
        snapshot = next_snapshot;
        position += 1;
    }
    let head = store.head()?;
    let head_snapshot: Option<Vec<u8>> = db.query_row(
        "SELECT snapshot FROM branch_head WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    require(
        head == (parent, position, anchor) && head_snapshot == snapshot && position <= count,
        "branch head mixes committed snapshots",
    )?;
    for (table, expected) in [
        ("facts", accepted),
        ("snapshots", accepted),
        ("ledger_index", ledger_count),
        ("ingress_decisions", integer(position)?),
    ] {
        require(
            db.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| {
                r.get::<_, i64>(0)
            })? == expected,
            "authority index count differs from commit manifests",
        )?;
    }
    Ok(())
}
