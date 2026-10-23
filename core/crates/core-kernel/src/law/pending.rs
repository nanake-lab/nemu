//! Temporary in-process state until P5-34 supplies successor IDL types.
//! No wire implementation and no production serialization. V and q must never
//! be discarded when passing this state between G calls.

use core_types::generated::{
    ConceptRef, EvidenceSign, NatureVector, ObjectRef, PersonRelation, TrilingualText,
};
use core_types::scalar::DecimalU64;
#[derive(Clone, Debug, PartialEq)]
pub struct Tendency {
    pub schema_version: u32,
    pub tendency_id: String,
    pub concept: ConceptRef,
    pub predicate: Vec<TrilingualText>,
    pub direction: NatureVector,
    pub evidence_a: f64,
    pub evidence_b: f64,
    pub h_hat: f64,
    pub group_count: DecimalU64,
    pub flip_count: DecimalU64,
    pub last_sign: Option<EvidenceSign>,
    pub pi: f64,
    pub typical_magnitude: f64,
    pub relation: Option<PersonRelation>,
    pub attachment_prior: Option<f64>,
    pub magnitude_sum: f64,
    pub magnitude_weight: f64,
    pub evidence_v: f64,
}
impl Tendency {
    pub(crate) fn legacy(&self) -> core_types::generated::TendencyV2 {
        core_types::generated::TendencyV2 {
            schema_version: self.schema_version,
            tendency_id: self.tendency_id.clone(),
            concept: self.concept.clone(),
            predicate: self.predicate.clone(),
            direction: self.direction.clone(),
            evidence_a: self.evidence_a,
            evidence_b: self.evidence_b,
            h_hat: self.h_hat,
            group_count: self.group_count,
            flip_count: self.flip_count,
            last_sign: self.last_sign,
            pi: self.pi,
            typical_magnitude: self.typical_magnitude,
            relation: self.relation.clone(),
            attachment_prior: self.attachment_prior,
            magnitude_sum: self.magnitude_sum,
            magnitude_weight: self.magnitude_weight,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Allocation {
    pub schema_version: u32,
    pub tendency_id: String,
    pub support: f64,
    pub intensity: f64,
    pub share: f64,
    pub ordinal: DecimalU64,
    pub mu: f64,
    pub projection: Option<f64>,
    pub actual_retention: f64,
}
#[cfg(test)]
impl Allocation {
    pub(crate) fn legacy(&self) -> core_types::generated::EvidenceAllocationV2 {
        core_types::generated::EvidenceAllocationV2 {
            schema_version: self.schema_version,
            tendency_id: self.tendency_id.clone(),
            support: self.support,
            intensity: self.intensity,
            share: self.share,
            ordinal: self.ordinal,
            mu: self.mu,
            projection: self.projection,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceGroup {
    pub schema_version: u32,
    pub group_id: String,
    pub members: Vec<String>,
    pub source_key: String,
    pub credibility: f64,
    pub verifiability: f64,
    pub allocations: Vec<Allocation>,
}
#[cfg(test)]
impl EvidenceGroup {
    pub(crate) fn legacy(&self) -> core_types::generated::EvidenceGroupV2 {
        core_types::generated::EvidenceGroupV2 {
            schema_version: self.schema_version,
            group_id: self.group_id.clone(),
            members: self.members.clone(),
            source_key: self.source_key.clone(),
            credibility: self.credibility,
            verifiability: self.verifiability,
            allocations: self.allocations.iter().map(|v| v.legacy()).collect(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub schema_version: u32,
    pub revision: String,
    pub law_version: ObjectRef,
    pub tendencies: Vec<Tendency>,
}
#[cfg(test)]
impl Snapshot {
    pub(crate) fn legacy(&self) -> core_types::generated::RSnapshotV2 {
        core_types::generated::RSnapshotV2 {
            schema_version: self.schema_version,
            revision: self.revision.clone(),
            law_version: self.law_version.clone(),
            tendencies: self.tendencies.iter().map(|v| v.legacy()).collect(),
        }
    }
}
