//! Version dispatch is for historical reads only. New publications require V3.
use crate::{Result, error::require, objects::Objects};
use core_types::{
    WireMessage,
    contract::Schema,
    generated::{
        EvidenceGroupV2, EvidenceGroupV3, LawVersion, ObjectRef, RSnapshotV2, RSnapshotV3,
    },
    semantic::{check_allocation_v3, check_tendency_v3_with_parameters},
};

/// Historical evidence is never converted by guessing q or subtracting c from B.
#[derive(Debug, PartialEq)]
pub enum StoredEvidenceGroup {
    V2(EvidenceGroupV2),
    V3(EvidenceGroupV3),
}
impl StoredEvidenceGroup {
    pub(crate) fn read(objects: &Objects, reference: &ObjectRef) -> Result<Self> {
        let schema = Schema::builtin()?;
        let bytes = objects.read(reference)?;
        let limits = objects.config.encoding;
        if reference.object_type == schema.message(EvidenceGroupV2::NAME)?.tag {
            Ok(Self::V2(EvidenceGroupV2::from_canonical(&bytes, limits)?))
        } else {
            require(
                reference.object_type == schema.message(EvidenceGroupV3::NAME)?.tag,
                "unexpected evidence group type",
            )?;
            let group = EvidenceGroupV3::from_canonical(&bytes, limits)?;
            for allocation in &group.allocations {
                check_allocation_v3(allocation)?;
            }
            Ok(Self::V3(group))
        }
    }
    pub(crate) fn id(&self) -> &str {
        match self {
            Self::V2(g) => &g.group_id,
            Self::V3(g) => &g.group_id,
        }
    }
    pub(crate) fn tendencies(&self) -> Vec<&str> {
        match self {
            Self::V2(g) => g
                .allocations
                .iter()
                .map(|a| a.tendency_id.as_str())
                .collect(),
            Self::V3(g) => g
                .allocations
                .iter()
                .map(|a| a.tendency_id.as_str())
                .collect(),
        }
    }
}

pub(crate) enum StoredR {
    V2(RSnapshotV2),
    V3(RSnapshotV3),
}
impl StoredR {
    pub fn read(objects: &Objects, reference: &ObjectRef) -> Result<Self> {
        let schema = Schema::builtin()?;
        let bytes = objects.read(reference)?;
        let limits = objects.config.encoding;
        if reference.object_type == schema.message(RSnapshotV2::NAME)?.tag {
            // Preserve the old decoder and its checks, without applying V3 semantics.
            Ok(Self::V2(RSnapshotV2::from_canonical(&bytes, limits)?))
        } else {
            require(
                reference.object_type == schema.message(RSnapshotV3::NAME)?.tag,
                "unexpected R snapshot type",
            )?;
            let r = RSnapshotV3::from_canonical(&bytes, limits)?;
            validate_r(objects, &r)?;
            Ok(Self::V3(r))
        }
    }
    pub fn revision(&self) -> &str {
        match self {
            Self::V2(r) => &r.revision,
            Self::V3(r) => &r.revision,
        }
    }
    pub fn tendency_refs(&self, objects: &Objects) -> Result<Vec<(String, ObjectRef)>> {
        match self {
            Self::V2(r) => r
                .tendencies
                .iter()
                .map(|t| Ok((t.tendency_id.clone(), objects.reference(t, "1")?)))
                .collect(),
            Self::V3(r) => r
                .tendencies
                .iter()
                .map(|t| Ok((t.tendency_id.clone(), objects.reference(t, "1")?)))
                .collect(),
        }
    }
}

pub(crate) fn validate_r(objects: &Objects, r: &RSnapshotV3) -> Result<()> {
    // Read and hash-check the resource identified by R, not caller-supplied parameters.
    let law = LawVersion::from_canonical(&objects.read(&r.law_version)?, objects.config.encoding)?;
    core_types::semantic::check_law_parameters(&law.parameters)?;
    for tendency in &r.tendencies {
        check_tendency_v3_with_parameters(tendency, &law.parameters)?;
    }
    Ok(())
}

pub(crate) fn is_group(reference: &ObjectRef) -> Result<bool> {
    let schema = Schema::builtin()?;
    Ok(
        reference.object_type == schema.message(EvidenceGroupV2::NAME)?.tag
            || reference.object_type == schema.message(EvidenceGroupV3::NAME)?.tag,
    )
}
