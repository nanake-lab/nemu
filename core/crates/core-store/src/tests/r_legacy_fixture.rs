//! Synthetic historical publication only. These raw writes exist under cfg(test)
//! so the public Store never gains a superseded-write escape hatch.
use super::*;
use core_types::{contract::append_blob, scalar::hex};
use rusqlite::params;

fn raw<T: WireMessage>(store: &Store<Rehearsal>, value: &T, version: &str) -> ObjectRef {
    let reference = store.objects.reference(value, version).unwrap();
    std::fs::write(
        store.objects.path(&reference).unwrap(),
        value.canonical_bytes(config().encoding).unwrap(),
    )
    .unwrap();
    reference
}

pub(super) fn seed(store: &mut Store<Rehearsal>) -> (String, ObjectRef, ObjectRef) {
    assert_eq!(store.head().unwrap(), (None, 0, 0));
    let e = event(store, 0);
    let (_ticket, mut value, _pins) = proposal(store, &e);
    let mut tendency: TendencyV2 = fixture();
    tendency.tendency_id = "legacy-t".into();
    tendency.direction.connection = 1.0;
    tendency.evidence_b = 4.0;
    tendency.group_count = DecimalU64(1);
    let child = raw(store, &tendency, "1");
    let r = RSnapshotV2 {
        schema_version: 1,
        revision: "legacy-r".into(),
        law_version: value.binding.environment.law_version.clone().unwrap(),
        tendencies: vec![tendency],
    };
    let r_ref = store.objects.reference(&r, &r.revision).unwrap();
    let mut empty = r.clone();
    empty.tendencies.clear();
    let mut prefix = empty.canonical_bytes(config().encoding).unwrap();
    prefix.truncate(prefix.len() - 4);
    prefix.extend(1u32.to_le_bytes());
    let mut physical = b"NSI1".to_vec();
    append_blob(&mut physical, &prefix).unwrap();
    append_blob(
        &mut physical,
        &child.canonical_bytes(config().encoding).unwrap(),
    )
    .unwrap();
    std::fs::write(store.objects.path(&r_ref).unwrap(), physical).unwrap();
    let mut group: EvidenceGroupV2 = fixture();
    group.group_id = "legacy-g".into();
    group.members = vec![e.id.clone()];
    let mut allocation: EvidenceAllocationV2 = fixture();
    allocation.tendency_id = "legacy-t".into();
    group.allocations = vec![allocation];
    let group_ref = raw(store, &group, "1");
    value.staged_r = r_ref.clone();
    value.accepted_results.push(group_ref.clone());
    let manifest = raw(store, &value, "1");
    let mut refs = store.objects.closure(&manifest).unwrap();
    for entry in &value.staged_ledger {
        refs.push(raw(store, entry, "1"));
    }
    let id = hex(&manifest.digest.0);
    let encoded = manifest.canonical_bytes(config().encoding).unwrap();
    let tx = store.db.transaction().unwrap();
    crate::store::publish_refs(&tx, &refs, config()).unwrap();
    for r in &refs {
        tx.execute(
            "INSERT OR IGNORE INTO visible_objects VALUES(?1)",
            [hex(&r.digest.0)],
        )
        .unwrap();
    }
    tx.execute(
        "INSERT INTO commits VALUES(?1,NULL,0,1,?2,'accepted')",
        params![id, encoded],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO facts VALUES(?1,0,?2,?3)",
        params![
            e.id,
            id,
            value.staged_h.canonical_bytes(config().encoding).unwrap()
        ],
    )
    .unwrap();
    crate::ingress::record_decision(
        &tx,
        &e,
        &id,
        "accepted",
        &crate::ingress::fingerprint(&e, config().encoding).unwrap(),
    )
    .unwrap();
    tx.execute("INSERT INTO snapshots VALUES(?1,?2)", params![id, encoded])
        .unwrap();
    crate::publication::index(&tx, &store.objects, &id, &value, &refs).unwrap();
    for entry in &value.staged_ledger {
        let r = store.objects.reference(entry, "1").unwrap();
        tx.execute(
            "INSERT INTO ledger_index VALUES(?1,?2,?3,?4)",
            params![
                entry.entry_id,
                e.id,
                id,
                r.canonical_bytes(config().encoding).unwrap()
            ],
        )
        .unwrap();
    }
    tx.execute(
        "UPDATE branch_head SET commit_id=?1,processed=1,anchor=1,snapshot=?2 WHERE singleton=1",
        params![id, encoded],
    )
    .unwrap();
    tx.commit().unwrap();
    crate::witness::record(
        &store.root,
        "commits",
        0,
        &manifest,
        &store.objects,
        &Fault::None,
    )
    .unwrap();
    store.verify().unwrap();
    (id, r_ref, group_ref)
}
