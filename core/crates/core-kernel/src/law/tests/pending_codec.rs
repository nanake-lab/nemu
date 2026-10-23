//! Test-only envelope: legacy CanonicalV1 bytes plus raw V/q bits.
//! This is not a production codec or a replacement for P5-34.
use crate::law::pending::{Allocation, EvidenceGroup, Snapshot, Tendency};
use core_types::{WireMessage, contract::Limits};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
impl Tendency {
    fn from_legacy(base: core_types::generated::TendencyV2, extra: &[u8]) -> Self {
        Self {
            schema_version: base.schema_version,
            tendency_id: base.tendency_id,
            concept: base.concept,
            predicate: base.predicate,
            direction: base.direction,
            evidence_a: base.evidence_a,
            evidence_b: base.evidence_b,
            h_hat: base.h_hat,
            group_count: base.group_count,
            flip_count: base.flip_count,
            last_sign: base.last_sign,
            pi: base.pi,
            typical_magnitude: base.typical_magnitude,
            relation: base.relation,
            attachment_prior: base.attachment_prior,
            magnitude_sum: base.magnitude_sum,
            magnitude_weight: base.magnitude_weight,
            evidence_v: f64::from_le_bytes(extra.try_into().unwrap()),
        }
    }
    pub(super) fn canonical_bytes(&self, limits: Limits) -> Result<Vec<u8>> {
        let base = self.legacy().canonical_bytes(limits)?;
        let mut out = (base.len() as u64).to_le_bytes().to_vec();
        out.extend(base);
        out.extend(self.evidence_v.to_le_bytes());
        Ok(out)
    }
    pub(super) fn from_canonical(bytes: &[u8], limits: Limits) -> Result<Self> {
        let n = u64::from_le_bytes(bytes[..8].try_into()?) as usize;
        let base = core_types::generated::TendencyV2::from_canonical(&bytes[8..8 + n], limits)?;
        Ok(Self::from_legacy(base, &bytes[8 + n..]))
    }
}
impl Allocation {
    fn from_legacy(base: core_types::generated::EvidenceAllocationV2, extra: &[u8]) -> Self {
        Self {
            schema_version: base.schema_version,
            tendency_id: base.tendency_id,
            support: base.support,
            intensity: base.intensity,
            share: base.share,
            ordinal: base.ordinal,
            mu: base.mu,
            projection: base.projection,
            actual_retention: f64::from_le_bytes(extra.try_into().unwrap()),
        }
    }
}
impl EvidenceGroup {
    fn from_legacy(base: core_types::generated::EvidenceGroupV2, extra: &[u8]) -> Self {
        Self {
            schema_version: base.schema_version,
            group_id: base.group_id,
            members: base.members,
            source_key: base.source_key,
            credibility: base.credibility,
            verifiability: base.verifiability,
            allocations: base
                .allocations
                .into_iter()
                .enumerate()
                .map(|(i, v)| Allocation::from_legacy(v, &extra[i * 8..(i + 1) * 8]))
                .collect(),
        }
    }
    pub(super) fn canonical_bytes(&self, limits: Limits) -> Result<Vec<u8>> {
        let base = self.legacy().canonical_bytes(limits)?;
        let mut out = (base.len() as u64).to_le_bytes().to_vec();
        out.extend(base);
        for a in &self.allocations {
            out.extend(a.actual_retention.to_le_bytes());
        }
        Ok(out)
    }
    pub(super) fn from_canonical(bytes: &[u8], limits: Limits) -> Result<Self> {
        let n = u64::from_le_bytes(bytes[..8].try_into()?) as usize;
        let base =
            core_types::generated::EvidenceGroupV2::from_canonical(&bytes[8..8 + n], limits)?;
        Ok(Self::from_legacy(base, &bytes[8 + n..]))
    }
}
impl Snapshot {
    fn from_legacy(base: core_types::generated::RSnapshotV2, extra: &[u8]) -> Self {
        Self {
            schema_version: base.schema_version,
            revision: base.revision,
            law_version: base.law_version,
            tendencies: base
                .tendencies
                .into_iter()
                .enumerate()
                .map(|(i, v)| Tendency::from_legacy(v, &extra[i * 8..(i + 1) * 8]))
                .collect(),
        }
    }
    pub(super) fn canonical_bytes(&self, limits: Limits) -> Result<Vec<u8>> {
        let base = self.legacy().canonical_bytes(limits)?;
        let mut out = (base.len() as u64).to_le_bytes().to_vec();
        out.extend(base);
        for t in &self.tendencies {
            out.extend(t.evidence_v.to_le_bytes());
        }
        Ok(out)
    }
    pub(super) fn from_canonical(bytes: &[u8], limits: Limits) -> Result<Self> {
        let n = u64::from_le_bytes(bytes[..8].try_into()?) as usize;
        let base = core_types::generated::RSnapshotV2::from_canonical(&bytes[8..8 + n], limits)?;
        Ok(Self::from_legacy(base, &bytes[8 + n..]))
    }
}
