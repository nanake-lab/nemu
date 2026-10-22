//! Redundant durability receipts, not a second state database. Ordinary opening
//! never deletes them. They live beside the store so replacing an old store backup
//! cannot silently rewind acknowledged D/commits. W14 must retain/reconcile them.
use crate::{Error, Result, error::require, objects::Objects, platform, schema::unsigned};
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
pub(crate) fn create(root: &Path) -> Result<()> {
    let receipts = path(root)?;
    require(
        !receipts.exists(),
        "prior durability receipts exist; explicit recovery required",
    )?;
    fs::create_dir(&receipts)?;
    for kind in ["d", "commits"] {
        fs::create_dir(receipts.join(kind))?;
    }
    platform::sync_dir(&receipts)?;
    platform::sync_dir(
        receipts
            .parent()
            .ok_or(Error::Rejected("missing receipt parent"))?,
    )
}
pub(crate) fn record(
    root: &Path,
    kind: &str,
    position: u64,
    r: &ObjectRef,
    objects: &Objects,
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
        temp.as_file().sync_all()?;
        temp.persist_noclobber(&target)
            .map_err(|e| Error::Io(e.error))?;
    }
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&target)?
        .sync_all()?;
    platform::sync_dir(&dir)
}
pub(crate) fn expected(
    db: &Connection,
    objects: &Objects,
    kind: &str,
) -> Result<BTreeMap<u64, ObjectRef>> {
    let mut result = BTreeMap::new();
    let sql = if kind == "d" {
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
        let reference = if is_snapshot {
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
pub(crate) fn reconcile(
    db: &Connection,
    root: &Path,
    objects: &Objects,
    repair_tail: bool,
) -> Result<()> {
    for kind in ["d", "commits"] {
        let dir = path(root)?.join(kind);
        platform::no_links(&dir)?;
        let expected = expected(db, objects, kind)?;
        let mut found = BTreeMap::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            platform::no_links(&entry.path())?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".tmp") {
                continue;
            }
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
                expected.get(&position) == Some(&r),
                "database predates acknowledged history; explicit tail recovery or lossy branch required",
            )?;
            found.insert(position, r);
        }
        if repair_tail {
            for (position, r) in expected {
                if !found.contains_key(&position) {
                    record(root, kind, position, &r, objects)?;
                }
            }
        }
    }
    Ok(())
}
