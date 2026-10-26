use crate::{
    Result,
    objects::{Objects, collect_refs},
};
use core_types::{
    WireMessage,
    contract::Schema,
    generated::{ActionLifecycle, CoreTransactionV2, EvidenceGroupV2, ObjectRef, RSnapshotV2},
    scalar::hex,
};
use rusqlite::{Connection, params};

/// All derived indexes publish in the same SQLite transaction as branch_head.
pub(crate) fn index(
    db: &Connection,
    objects: &Objects,
    commit: &str,
    value: &CoreTransactionV2,
    refs: &[ObjectRef],
) -> Result<()> {
    let limits = objects.config.encoding;
    for (role, r) in [
        ("H", &value.staged_h),
        ("x", &value.staged_x),
        ("R", &value.staged_r),
        ("theta", &value.staged_theta),
        ("L", &value.staged_l),
        ("Q", &value.staged_q),
        ("U", &value.staged_u),
    ] {
        db.execute(
            "INSERT INTO snapshot_objects VALUES(?1,?2,'',?3)",
            params![commit, role, r.canonical_bytes(limits)?],
        )?;
    }
    let r = RSnapshotV2::from_canonical(&objects.read(&value.staged_r)?, limits)?;
    for tendency in &r.tendencies {
        let reference = objects.reference(tendency, "1")?;
        db.execute(
            "INSERT INTO snapshot_objects VALUES(?1,'tendency',?2,?3)",
            params![
                commit,
                tendency.tendency_id,
                reference.canonical_bytes(limits)?
            ],
        )?;
    }
    for reference in &value.accepted_results {
        db.execute(
            "INSERT INTO accepted_results VALUES(?1,?2)",
            params![commit, reference.canonical_bytes(limits)?],
        )?;
    }
    for reference in &value.staged_concepts {
        db.execute(
            "INSERT INTO snapshot_objects VALUES(?1,'concept',?2,?3)",
            params![
                commit,
                hex(&reference.digest.0),
                reference.canonical_bytes(limits)?
            ],
        )?;
    }
    for reference in &value.audit_refs {
        db.execute(
            "INSERT INTO runtime_records VALUES(?1,?2)",
            params![commit, reference.canonical_bytes(limits)?],
        )?;
    }
    let schema = Schema::builtin()?;
    // Index only groups accepted in this transaction, never unaccepted candidates.
    for reference in &value.accepted_results {
        if reference.object_type == schema.message(EvidenceGroupV2::NAME)?.tag {
            let group = EvidenceGroupV2::from_canonical(&objects.read(reference)?, limits)?;
            db.execute(
                "INSERT INTO evidence_groups VALUES(?1,?2,?3)",
                params![commit, group.group_id, reference.canonical_bytes(limits)?],
            )?;
            for allocation in &group.allocations {
                db.execute(
                    "INSERT INTO evidence_index VALUES(?1,?2,?3,?4)",
                    params![
                        commit,
                        allocation.tendency_id,
                        group.group_id,
                        reference.canonical_bytes(limits)?
                    ],
                )?;
            }
        }
    }
    if value.staged_q.object_type == schema.message(ActionLifecycle::NAME)?.tag {
        let action = ActionLifecycle::from_canonical(&objects.read(&value.staged_q)?, limits)?;
        db.execute(
            "INSERT INTO outbox VALUES(?1,?2,?3)",
            params![
                commit,
                action.action_id,
                value.staged_q.canonical_bytes(limits)?
            ],
        )?;
    }
    for r in refs {
        let (name, _) = schema
            .messages
            .iter()
            .find(|(_, m)| m.tag == r.object_type)
            .ok_or(crate::Error::Rejected("unknown object type"))?;
        let bytes = objects.read(r)?;
        let value = schema.decode_canonical(name, &bytes, limits, 0)?;
        let mut dependencies = Vec::new();
        collect_refs(&schema, name, &value, &mut dependencies)?;
        for dependency in dependencies {
            db.execute(
                "INSERT OR IGNORE INTO dependency_index VALUES(?1,?2,?3)",
                params![commit, hex(&r.digest.0), hex(&dependency.digest.0)],
            )?;
        }
    }
    Ok(())
}
