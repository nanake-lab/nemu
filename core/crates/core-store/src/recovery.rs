use crate::{
    Result,
    error::require,
    ingress::fingerprint,
    objects::Objects,
    schema::{integer, unsigned},
    store::head,
};
use core_types::{
    WireMessage,
    generated::{EventEnvelope, LedgerEntry, LineageHeaderV2, ObjectRef},
    scalar::hex,
};
use rusqlite::{Connection, params};
use std::collections::BTreeMap;

/// SQLite page integrity alone cannot prove that valid pointers form one snapshot.
/// Reconcile the physical indexes against their canonical transaction manifests.
pub(crate) fn verify(
    db: &Connection,
    objects: &Objects,
    header: &LineageHeaderV2,
    branch: &str,
) -> Result<()> {
    let mut prefix = Prefix::default();
    verify_tail(db, objects, header, branch, &mut prefix)?;
    crate::derived::verify(db, objects)
}

#[derive(Default)]
pub(crate) struct Prefix {
    pub inputs: u64,
    position: u64,
    pub commits: u64,
    pub attempts: u64,
    activities: i64,
    has_initial: bool,
    record_ids: std::collections::BTreeSet<String>,
    request_ids: std::collections::BTreeSet<String>,
    step_ids: std::collections::BTreeSet<(String, String)>,
    pub audit: u64,
    parent: Option<String>,
    anchor: u64,
    snapshot: Option<Vec<u8>>,
    accepted: i64,
    ledger_count: i64,
    seen: BTreeMap<(String, String), Vec<u8>>,
}

/// The caller discards the entire prefix if any check, including projection
/// or witness reconciliation, fails. Never publish a partially advanced proof.
pub(crate) fn verify_tail(
    db: &Connection,
    objects: &Objects,
    header: &LineageHeaderV2,
    branch: &str,
    prefix: &mut Prefix,
) -> Result<()> {
    require(
        db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))? == "ok",
        "database integrity failure",
    )?;
    require(
        db.prepare("PRAGMA foreign_key_check")?
            .query([])?
            .next()?
            .is_none(),
        "foreign key closure failure",
    )?;
    let limits = objects.config.encoding;
    prefix.audit = crate::terminal_ingress::verify_audit_from(db, limits, prefix.audit)?;
    let mut input =
        db.prepare("SELECT position,event FROM raw_ingress WHERE position>=?1 ORDER BY position")?;
    let mut rows = input.query([integer(prefix.inputs)?])?;
    let mut count = prefix.inputs;
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
        "SELECT id,parent,position,anchor,snapshot,decision FROM commits WHERE position>=?1 ORDER BY position",
    )?;
    let mut rows = stmt.query([integer(prefix.commits)?])?;
    let mut parent = prefix.parent.take();
    let mut position = prefix.position;
    let mut sequence = prefix.commits;
    let mut activities = prefix.activities;
    let mut anchor = prefix.anchor;
    let mut snapshot = prefix.snapshot.take();
    let mut accepted = prefix.accepted;
    let mut ledger_count = prefix.ledger_count;
    let seen = &mut prefix.seen;
    let initial = hex(&objects.reference(header, "1")?.digest.0);
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let next_anchor = unsigned(row.get(3)?)?;
        let next_snapshot: Option<Vec<u8>> = row.get(4)?;
        let decision: String = row.get(5)?;
        require(
            row.get::<_, Option<String>>(1)? == parent && unsigned(row.get(2)?)? == sequence,
            "commit chain is broken",
        )?;
        sequence += 1;
        if decision == "initial" {
            require(
                sequence == 1 && parent.is_none() && crate::migration::format(db)? == 3,
                "initial publication requires an empty format 3 prefix",
            )?;
            let reference = ObjectRef::from_canonical(
                next_snapshot
                    .as_deref()
                    .ok_or(crate::Error::Rejected("missing initial snapshot"))?,
                limits,
            )?;
            require(
                hex(&reference.digest.0) == id
                    && crate::initial::verify(objects, &reference, header, branch)? == next_anchor,
                "initial publication frontier mismatch",
            )?;
            prefix.has_initial = true;
            parent = Some(id);
            anchor = next_anchor;
            snapshot = next_snapshot;
            continue;
        }
        if decision == "activity" {
            require(
                crate::migration::format(db)? == 3,
                "internal activity before migration",
            )?;
            let decode_ref = |b: &Option<Vec<u8>>| -> Result<ObjectRef> {
                Ok(ObjectRef::from_canonical(
                    b.as_deref()
                        .ok_or(crate::Error::Rejected("missing activity snapshot"))?,
                    limits,
                )?)
            };
            let reference = decode_ref(&next_snapshot)?;
            let previous = crate::manifest::decode(objects, &decode_ref(&snapshot)?)?;
            let value = crate::manifest::decode(objects, &reference)?;
            let c = value
                .activity_commit
                .as_ref()
                .ok_or(crate::Error::Rejected("expected activity origin"))?;
            require(
                prefix.record_ids.insert(c.activity.record_id.clone())
                    && prefix.request_ids.insert(c.binding.request_id.clone())
                    && prefix
                        .step_ids
                        .insert((c.binding.activity_id.clone(), c.binding.step_id.clone())),
                "duplicate internal publication",
            )?;
            require(
                hex(&reference.digest.0) == id
                    && c.binding.branch_id == branch
                    && Some(&c.binding.base_commit) == parent.as_ref()
                    && value.processed_position.0 == position
                    && value.anchor_time_ns.0 == next_anchor
                    && next_anchor >= anchor,
                "internal publication frontier mismatch",
            )?;
            crate::activity::verify_transition(objects, &previous, &value)?;
            parent = Some(id);
            anchor = next_anchor;
            snapshot = next_snapshot;
            activities += 1;
            continue;
        }
        let event_bytes: Vec<u8> = db.query_row(
            "SELECT event FROM raw_ingress WHERE position=?1",
            [integer(position)?],
            |r| r.get(0),
        )?;
        let event = EventEnvelope::from_canonical(&event_bytes, limits)?;
        let event_ref = objects.reference(&event, "1")?;
        let fp = fingerprint(&event, limits)?;
        let key = (event.device.clone(), event.stable_source_key.clone());
        if decision == "accepted" {
            require(!seen.contains_key(&key), "stable key accepted twice")?;
            seen.insert(key, fp.clone());
            let bytes = next_snapshot
                .as_ref()
                .ok_or(crate::Error::Rejected("accepted commit lacks snapshot"))?;
            let reference = ObjectRef::from_canonical(bytes, limits)?;
            let value = crate::manifest::event_view(crate::manifest::decode(objects, &reference)?)?;
            crate::theta::read(objects, &value.staged_theta)?;
            require(
                hex(&reference.digest.0) == id
                    && value.binding.base_commit == parent.as_ref().unwrap_or(&initial).as_str()
                    && value.binding.branch_id == branch
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
                    LedgerEntry::from_canonical(&objects.read(&reference)?, limits)?
                        .canonical_bytes(limits)?
                        == entry.canonical_bytes(limits)?,
                    "ledger index differs from canonical transaction",
                )?;
                ledger_count += 1;
            }
            accepted += 1;
        } else {
            // The durable event can already belong to the verified input
            // prefix. Settlement still publishes its digest at this position.
            objects.closure(&event_ref)?;
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
    let head = head(db)?;
    let head_snapshot: Option<Vec<u8>> = db.query_row(
        "SELECT snapshot FROM branch_head WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    require(
        head.0 == parent
            && head.1 == position
            && head.2 == anchor
            && head_snapshot == snapshot
            && position <= count,
        "branch head mixes committed snapshots",
    )?;
    for (table, expected) in [
        ("facts", accepted),
        (
            "snapshots",
            accepted + activities + i64::from(prefix.has_initial),
        ),
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
    prefix.inputs = count;
    prefix.position = position;
    prefix.activities = activities;
    prefix.commits = sequence;
    prefix.anchor = anchor;
    prefix.parent = parent;
    prefix.snapshot = snapshot;
    prefix.accepted = accepted;
    prefix.ledger_count = ledger_count;
    Ok(())
}

/// The writer has finished the publications in these ranges before dispatching
/// the check. A later append cannot replace their immutable position receipts.
pub(crate) fn check_new_witnesses(
    db: &Connection,
    root: &std::path::Path,
    objects: &Objects,
    inputs: std::ops::Range<u64>,
    commits: std::ops::Range<u64>,
    attempts: std::ops::Range<u64>,
    audit: std::ops::Range<u64>,
) -> Result<()> {
    let receipts = crate::witness::path(root)?;
    let limits = objects.config.encoding;
    for (kind, range) in [
        ("d", inputs),
        ("commits", commits),
        ("reject", audit),
        ("attempts", attempts),
    ] {
        let dir = receipts.join(kind);
        for position in range {
            let path = dir.join(format!("{position:016x}.ref"));
            crate::platform::no_links(&path)?;
            let metadata = match std::fs::metadata(&path) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue, // SQL can precede ACK
                other => other?,
            };
            require(
                metadata.len() <= limits.max_bytes as u64,
                "oversized receipt",
            )?;
            let reference = ObjectRef::from_canonical(&std::fs::read(path)?, limits)?;
            let pos = integer(position)?;
            let expected = match kind {
                "d" => {
                    let bytes: Vec<u8> = db.query_row(
                        "SELECT event FROM raw_ingress WHERE position=?1",
                        [pos],
                        |r| r.get(0),
                    )?;
                    objects.reference(&EventEnvelope::from_canonical(&bytes, limits)?, "1")?
                }
                "attempts" => {
                    let bytes: Vec<u8> = db.query_row(
                        "SELECT record FROM cognitive_attempts WHERE position=?1",
                        [pos],
                        |r| r.get(0),
                    )?;
                    ObjectRef::from_canonical(&bytes, limits)?
                }
                "reject" => {
                    let bytes: Vec<u8> = db.query_row(
                        "SELECT record FROM ingress_rejections WHERE position=?1",
                        [pos],
                        |r| r.get(0),
                    )?;
                    objects.reference(
                        &core_types::generated::IngressRejection::from_canonical(&bytes, limits)?,
                        "1",
                    )?
                }
                _ => {
                    let (decision, bytes): (String, Vec<u8>) = db.query_row("SELECT c.decision,CASE WHEN c.decision IN ('initial','accepted','activity') THEN c.snapshot ELSE d.event END FROM commits c LEFT JOIN ingress_decisions i ON i.commit_id=c.id LEFT JOIN raw_ingress d ON d.position=i.position WHERE c.position=?1", [pos], |r| Ok((r.get(0)?, r.get(1)?)))?;
                    if matches!(decision.as_str(), "initial" | "accepted" | "activity") {
                        ObjectRef::from_canonical(&bytes, limits)?
                    } else {
                        objects.reference(&EventEnvelope::from_canonical(&bytes, limits)?, "1")?
                    }
                }
            };
            require(
                reference == expected,
                "durability receipt differs from publication",
            )?;
        }
    }
    Ok(())
}
