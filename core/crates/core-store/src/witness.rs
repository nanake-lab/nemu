//! Redundant durability receipts, not a second state database. Ordinary opening
//! never deletes them. They live beside the store so replacing an old store backup
//! cannot silently rewind acknowledged D/commits. W14 must retain/reconcile them.
use crate::{
    Error, Fault, FaultPoint, Result, error::require, objects::Objects, platform, schema::unsigned,
};
use core_types::{
    WireMessage,
    generated::{EventEnvelope, ObjectRef},
    scalar::hex,
};
use rusqlite::Connection;
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub(crate) fn path(root: &Path) -> Result<PathBuf> {
    let parent = root
        .parent()
        .ok_or(Error::Rejected("missing storage parent"))?;
    let name = root
        .file_name()
        .ok_or(Error::Rejected("missing store name"))?
        .to_string_lossy();
    Ok(parent.join(format!(".{name}.w02-receipts-v1")))
}
pub(crate) fn create(root: &Path, fault: &Fault) -> Result<()> {
    let receipts = path(root)?;
    require(
        !receipts.exists(),
        "prior durability receipts exist; explicit recovery required",
    )?;
    fs::create_dir(&receipts)?;
    fault.hit(FaultPoint::ReceiptRootCreated)?;
    for kind in ["d", "commits", "reject"] {
        fs::create_dir(receipts.join(kind))?;
        fault.hit(FaultPoint::ReceiptKindCreated)?;
        platform::sync_dir(&receipts.join(kind))?;
        fault.hit(FaultPoint::ReceiptKindSynced)?;
    }
    platform::sync_dir(&receipts)?;
    fault.hit(FaultPoint::ReceiptRootSynced)?;
    platform::sync_dir(
        receipts
            .parent()
            .ok_or(Error::Rejected("missing receipt parent"))?,
    )?;
    fault.hit(FaultPoint::ReceiptParentSynced)
}
pub(crate) fn record(
    root: &Path,
    kind: &str,
    position: u64,
    r: &ObjectRef,
    objects: &Objects,
    fault: &Fault,
) -> Result<()> {
    let dir = path(root)?.join(kind);
    platform::no_links(&dir)?;
    let target = dir.join(format!("{position:016x}.ref"));
    let bytes = r.canonical_bytes(objects.config.encoding)?;
    if target.exists() {
        require(fs::read(&target)? == bytes, "durability receipt conflict")?;
    } else {
        let mut temp = tempfile::Builder::new().suffix(".tmp").tempfile_in(&dir)?;
        temp.write_all(&bytes)?;
        fault.hit(FaultPoint::ReceiptWritten)?;
        fault.hit(FaultPoint::ReceiptFileSyncBefore)?;
        temp.as_file().sync_all()?;
        fault.hit(FaultPoint::ReceiptFileSynced)?;
        fault.hit(FaultPoint::ReceiptRenameBefore)?;
        platform::persist_noclobber(temp, &target)?;
        fault.hit(FaultPoint::ReceiptRenamed)?;
    }
    fault.hit(FaultPoint::ReceiptTargetSyncBefore)?;
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&target)?
        .sync_all()?;
    fault.hit(FaultPoint::ReceiptTargetSynced)?;
    fault.hit(FaultPoint::ReceiptDirectorySyncBefore)?;
    platform::sync_dir(&dir)?;
    fault.hit(FaultPoint::ReceiptDirectorySynced)
}
pub(crate) fn expected(
    db: &Connection,
    objects: &Objects,
    kind: &str,
) -> Result<BTreeMap<u64, ObjectRef>> {
    let mut result = BTreeMap::new();
    if kind == "reject" && crate::migration::format(db)? == 1 {
        return Ok(result);
    }
    let sql = if kind == "reject" {
        "SELECT position,record FROM ingress_rejections ORDER BY position"
    } else if kind == "d" {
        "SELECT position,event FROM raw_ingress ORDER BY position"
    } else {
        "SELECT c.position,CASE WHEN c.decision='accepted' THEN c.snapshot ELSE d.event END,c.decision,c.id FROM commits c JOIN raw_ingress d USING(position) ORDER BY c.position"
    };
    let mut stmt = db.prepare(sql)?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let position = unsigned(row.get(0)?)?;
        let bytes: Vec<u8> = row.get(1)?;
        let is_snapshot = kind == "commits" && row.get::<_, String>(2)? == "accepted";
        let reference = if kind == "reject" {
            objects.reference(
                &core_types::generated::IngressRejection::from_canonical(
                    &bytes,
                    objects.config.encoding,
                )?,
                "1",
            )?
        } else if is_snapshot {
            ObjectRef::from_canonical(&bytes, objects.config.encoding)?
        } else {
            objects.reference(
                &EventEnvelope::from_canonical(&bytes, objects.config.encoding)?,
                "1",
            )?
        };
        if kind == "commits" {
            require(
                hex(&reference.digest.0) == row.get::<_, String>(3)?,
                "commit digest differs from its canonical object",
            )?;
        }
        result.insert(position, reference);
    }
    Ok(result)
}
/// A DB ahead of receipts can result from a crash before ACK; verified SQL/object
/// closure is authoritative. A receipt ahead of DB is an old-backup rollback and
/// MUST stop. Restoring the entire device/receipt namespace needs W14's external
/// recovery provenance; no local program can detect rollback of all its evidence.
pub(crate) type Captured = BTreeMap<&'static str, BTreeMap<u64, ObjectRef>>;

pub(crate) fn capture(root: &Path, objects: &Objects) -> Result<Captured> {
    let mut captured = BTreeMap::new();
    for kind in ["d", "commits", "reject"] {
        let dir = path(root)?.join(kind);
        platform::no_links(&dir)?;
        let mut found = BTreeMap::new();
        if kind == "reject" && !dir.exists() {
            captured.insert(kind, found);
            continue;
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            // A writer may rename its temp concurrently. Never stat/read it.
            if name.ends_with(".tmp") {
                continue;
            }
            platform::no_links(&entry.path())?;
            let number = name
                .strip_suffix(".ref")
                .ok_or(Error::Rejected("unknown durability receipt format"))?;
            let position = u64::from_str_radix(number, 16)
                .map_err(|_| Error::Rejected("invalid receipt position"))?;
            require(
                entry.metadata()?.len() <= objects.config.encoding.max_bytes as u64,
                "oversized receipt",
            )?;
            let r = ObjectRef::from_canonical(&fs::read(entry.path())?, objects.config.encoding)?;
            require(
                found.insert(position, r).is_none(),
                "duplicate durability receipt position",
            )?;
        }
        captured.insert(kind, found);
    }
    Ok(captured)
}

pub(crate) fn check(db: &Connection, objects: &Objects, captured: &Captured) -> Result<()> {
    for (kind, found) in captured {
        let expected = expected(db, objects, kind)?;
        for (position, r) in found {
            require(
                expected.get(position) == Some(r),
                "database predates acknowledged history; explicit tail recovery or lossy branch required",
            )?;
        }
    }
    Ok(())
}

pub(crate) fn reconcile(
    db: &Connection,
    root: &Path,
    objects: &Objects,
    repair_tail: bool,
    fault: &Fault,
) -> Result<()> {
    let captured = capture(root, objects)?;
    check(db, objects, &captured)?;
    if repair_tail {
        for kind in ["d", "commits", "reject"] {
            for (position, r) in expected(db, objects, kind)? {
                if !captured[kind].contains_key(&position) {
                    record(root, kind, position, &r, objects, fault)?;
                }
            }
        }
    }
    Ok(())
}
