//! Attempts are audit history, separate from H, the branch head and Q.
use crate::{
    FaultPoint, LineageKind, PreparedObject, Result, Store, error::require, objects::Objects,
    publication::visible,
};
use core_types::{
    WireMessage,
    generated::{AuditExtension, InternalActivityRecordV2, ObjectRef},
    scalar::HexBytes,
};
use rusqlite::{Connection, OptionalExtension, params};

fn decision(
    r: &ObjectRef,
    why: &str,
    limits: core_types::contract::Limits,
) -> Result<AuditExtension> {
    let mut payload = r.canonical_bytes(limits)?;
    payload.extend_from_slice(why.as_bytes());
    Ok(AuditExtension {
        schema_version: 1,
        audit_schema_tag: "cognition-attempt".into(),
        audit_schema_version: 1,
        payload: HexBytes(payload),
    })
}

pub(crate) fn record<L: LineageKind>(
    store: &mut Store<L>,
    activity: &PreparedObject<L>,
    request: &str,
    why: &'static str,
) -> Result<()> {
    let limits = store.objects.config.encoding;
    let activity_bytes = activity.reference().canonical_bytes(limits)?;
    let prior: Option<(i64, Vec<u8>)> = store
        .db
        .query_row(
            "SELECT position,record FROM cognitive_attempts WHERE activity=?1",
            [&activity_bytes],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if let Some((p, bytes)) = prior {
        return crate::witness::record(
            &store.root,
            "attempts",
            p as u64,
            &ObjectRef::from_canonical(&bytes, limits)?,
            &store.objects,
            &store.fault,
        );
    }
    let note = store.prepare(&decision(activity.reference(), why, limits)?, "1")?;
    let reference = note.reference().canonical_bytes(limits)?;
    let tx = store
        .db
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let position = {
        let p: i64 = tx.query_row(
            "SELECT coalesce(max(position),-1)+1 FROM cognitive_attempts",
            [],
            |r| r.get(0),
        )?;
        visible(
            &tx,
            &store.objects,
            &store.objects.closure(activity.reference())?,
        )?;
        visible(
            &tx,
            &store.objects,
            &store.objects.closure(note.reference())?,
        )?;
        tx.execute(
            "INSERT INTO cognitive_attempts VALUES(?1,?2,?3,?4)",
            params![
                p,
                reference,
                activity.reference().canonical_bytes(limits)?,
                request
            ],
        )?;
        p
    };
    store.fault.hit(FaultPoint::BeforePublish)?;
    tx.commit()?;
    store.fault.hit(FaultPoint::SqlCommitted)?;
    crate::witness::record(
        &store.root,
        "attempts",
        position as u64,
        note.reference(),
        &store.objects,
        &store.fault,
    )?;
    store.fault.hit(FaultPoint::AfterPublish)
}

pub(crate) fn rebuild(db: &Connection, projected: &Connection, objects: &Objects) -> Result<()> {
    let limits = objects.config.encoding;
    let mut stmt = db.prepare(
        "SELECT position,record,activity,request_id FROM cognitive_attempts ORDER BY position",
    )?;
    let mut rows = stmt.query([])?;
    let mut pos = 0i64;
    while let Some(row) = rows.next()? {
        let note = ObjectRef::from_canonical(&row.get::<_, Vec<u8>>(1)?, limits)?;
        let record = ObjectRef::from_canonical(&row.get::<_, Vec<u8>>(2)?, limits)?;
        let r = InternalActivityRecordV2::from_canonical(&objects.read(&record)?, limits)?;
        crate::activity::check_record(objects, &r)?;
        let binding = crate::activity::binding(&r)?;
        let stored = AuditExtension::from_canonical(&objects.read(&note)?, limits)?;
        require(
            row.get::<_, i64>(0)? == pos
                && row.get::<_, String>(3)? == binding.request_id
                && [
                    "stale",
                    "rejected",
                    "no-text",
                    "incomplete",
                    "invalid-content",
                ]
                .iter()
                .any(|why| decision(&record, why, limits).is_ok_and(|d| d == stored)),
            "attempt index disagrees with audit record",
        )?;
        visible(projected, objects, &objects.closure(&record)?)?;
        visible(projected, objects, &objects.closure(&note)?)?;
        pos += 1;
    }
    Ok(())
}
