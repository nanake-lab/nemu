use crate::{Result, StoredEvidenceGroup, error::require, objects::Objects, r_state::StoredR};
use core_types::{
    WireMessage,
    contract::Schema,
    generated::{
        CoreTransactionV2, EvidenceAllocation, EvidenceAllocationV2, EvidenceGroup,
        EvidenceGroupV2, EvidenceGroupV3, LawVersion, ObjectRef, RSnapshot, RSnapshotV2, Tendency,
        TendencyV2, TendencyV3,
    },
};
use rusqlite::{Connection, params};
use std::collections::{BTreeMap, BTreeSet};

/// Stream latest versions, optionally restricted to a current tendency's index.
/// Select the version before filtering associations, so empty successors win.
/// Historical row count is not a per-message encoding budget.
pub(crate) fn visit_latest_groups(
    db: &Connection,
    objects: &Objects,
    tendency: Option<&str>,
    mut visit: impl FnMut(StoredEvidenceGroup) -> Result<()>,
) -> Result<()> {
    let mut statement = db.prepare("WITH latest AS (SELECT g.commit_id,g.group_id,g.reference,row_number() OVER (PARTITION BY g.group_id ORDER BY c.position DESC) AS rank FROM evidence_groups g JOIN commits c ON c.id=g.commit_id) SELECT l.reference FROM latest l WHERE l.rank=1 AND (?1 IS NULL OR EXISTS (SELECT 1 FROM evidence_index i WHERE i.commit_id=l.commit_id AND i.group_id=l.group_id AND i.tendency_id=?1)) ORDER BY l.group_id")?;
    let rows = statement.query_map([tendency], |row| row.get::<_, Vec<u8>>(0))?;
    for row in rows {
        let reference = ObjectRef::from_canonical(&row?, objects.config.encoding)?;
        visit(StoredEvidenceGroup::read(objects, &reference)?)?;
    }
    Ok(())
}

pub(crate) fn validate_new(
    db: &Connection,
    objects: &Objects,
    value: &CoreTransactionV2,
) -> Result<()> {
    // Bound the collections actually being published, before any derived lookup.
    value.canonical_bytes(objects.config.encoding)?;
    let StoredR::V3(r) = StoredR::read(objects, &value.staged_r)? else {
        return Err(crate::Error::Rejected("new R publications require V3"));
    };
    require(
        value.binding.environment.law_version.as_ref() == Some(&r.law_version),
        "R law differs from bound environment",
    )?;
    let schema = Schema::builtin()?;
    let law = LawVersion::from_canonical(&objects.read(&r.law_version)?, objects.config.encoding)?;
    let old_tags = [
        Tendency::NAME,
        TendencyV2::NAME,
        EvidenceAllocation::NAME,
        EvidenceAllocationV2::NAME,
        EvidenceGroup::NAME,
        EvidenceGroupV2::NAME,
        RSnapshot::NAME,
        RSnapshotV2::NAME,
    ]
    .into_iter()
    .map(|name| Ok(schema.message(name)?.tag.as_str()))
    .collect::<std::result::Result<Vec<_>, String>>()?;
    let mut changed = BTreeMap::new();
    for reference in &value.accepted_results {
        require(
            !old_tags.contains(&reference.object_type.as_str()),
            "new accepted results cannot publish legacy R types",
        )?;
        if reference.object_type == schema.message(EvidenceGroupV3::NAME)?.tag {
            let group = EvidenceGroupV3::from_canonical(
                &objects.read(reference)?,
                objects.config.encoding,
            )?;
            for allocation in &group.allocations {
                core_types::semantic::check_allocation_v3(allocation)?;
            }
            require(
                changed.insert(group.group_id.clone(), group).is_none(),
                "multiple versions of one evidence group in a transaction",
            )?;
        } else if reference.object_type == schema.message(TendencyV3::NAME)?.tag {
            let tendency =
                TendencyV3::from_canonical(&objects.read(reference)?, objects.config.encoding)?;
            core_types::semantic::check_tendency_v3_with_parameters(&tendency, &law.parameters)?;
        }
    }
    for tendency in &r.tendencies {
        let mut ordinals = BTreeSet::new();
        let mut add = |group: &EvidenceGroupV3| -> Result<()> {
            for allocation in &group.allocations {
                if allocation.tendency_id == tendency.tendency_id {
                    require(
                        ordinals.insert(allocation.ordinal.0),
                        "duplicate allocation ordinal",
                    )?;
                }
            }
            Ok(())
        };
        visit_latest_groups(db, objects, Some(&tendency.tendency_id), |group| {
            if changed.contains_key(group.id()) {
                return Ok(());
            }
            match group {
                StoredEvidenceGroup::V3(g) => add(&g),
                StoredEvidenceGroup::V2(_) => Err(crate::Error::Rejected(
                    "active legacy evidence needs H replay",
                )),
            }
        })?;
        for group in changed.values() {
            add(group)?;
        }
        require(
            u64::try_from(ordinals.len()).is_ok_and(|n| n == tendency.group_count.0),
            "allocation count differs from the group count",
        )?;
        require(
            ordinals.into_iter().eq(0..tendency.group_count.0),
            "allocation ordinals are not 0..N-1 exactly once",
        )?;
    }
    Ok(())
}

/// Reconcile both historical V2 and current V3 index rows with canonical objects.
/// This never upgrades or rewrites a historical object.
pub(crate) fn verify_indexes(
    db: &Connection,
    objects: &Objects,
    commit: &str,
    value: &CoreTransactionV2,
) -> Result<()> {
    let limits = objects.config.encoding;
    let r = StoredR::read(objects, &value.staged_r)?;
    if let StoredR::V3(r) = &r {
        require(
            value.binding.environment.law_version.as_ref() == Some(&r.law_version),
            "historical R law differs from bound environment",
        )?;
    }
    let expected = r
        .tendency_refs(objects)?
        .into_iter()
        .map(|(id, reference)| Ok((id, reference.canonical_bytes(limits)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut statement = db.prepare(
        "SELECT object_key,reference FROM snapshot_objects WHERE commit_id=?1 AND role='tendency'",
    )?;
    let actual = statement
        .query_map([commit], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?
        .collect::<std::result::Result<BTreeMap<_, _>, _>>()?;
    require(
        actual == expected,
        "tendency index differs from canonical R",
    )?;
    let mut expected_groups = BTreeMap::new();
    let mut expected_allocations = BTreeSet::new();
    for reference in &value.accepted_results {
        if crate::r_state::is_group(reference)? {
            let group = StoredEvidenceGroup::read(objects, reference)?;
            let bytes = reference.canonical_bytes(limits)?;
            require(
                expected_groups
                    .insert(group.id().to_owned(), bytes.clone())
                    .is_none(),
                "ambiguous historical evidence group",
            )?;
            for id in group.tendencies() {
                expected_allocations.insert((id.to_owned(), group.id().to_owned(), bytes.clone()));
            }
        }
    }
    let mut statement =
        db.prepare("SELECT group_id,reference FROM evidence_groups WHERE commit_id=?1")?;
    let actual_groups = statement
        .query_map([commit], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?
        .collect::<std::result::Result<BTreeMap<_, _>, _>>()?;
    require(
        actual_groups == expected_groups,
        "evidence group index differs from canonical transaction",
    )?;
    let mut statement =
        db.prepare("SELECT tendency_id,group_id,reference FROM evidence_index WHERE commit_id=?1")?;
    let actual_allocations = statement
        .query_map(params![commit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })?
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    require(
        actual_allocations == expected_allocations,
        "allocation index differs from canonical transaction",
    )
}
