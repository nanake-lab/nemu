//! Test comparisons preserve zero signs; CanonicalV1 deliberately normalizes them.
use crate::law::state::{Allocation, EvidenceGroup, Snapshot, Tendency};

pub(super) fn tendency_bits_eq(a: &Tendency, b: &Tendency) -> bool {
    fn floats(t: &Tendency) -> [u64; 12] {
        [
            t.direction.connection,
            t.direction.empathy,
            t.direction.curiosity,
            t.direction.achievement,
            t.evidence_a,
            t.evidence_b,
            t.evidence_absolute,
            t.h_hat,
            t.pi,
            t.typical_magnitude,
            t.magnitude_sum,
            t.magnitude_weight,
        ]
        .map(f64::to_bits)
    }
    a == b
        && floats(a) == floats(b)
        && a.attachment_prior.map(f64::to_bits) == b.attachment_prior.map(f64::to_bits)
}

pub(super) fn snapshot_bits_eq(a: &Snapshot, b: &Snapshot) -> bool {
    a == b
        && a.tendencies
            .iter()
            .zip(&b.tendencies)
            .all(|(a, b)| tendency_bits_eq(a, b))
}

pub(super) fn group_bits_eq(a: &EvidenceGroup, b: &EvidenceGroup) -> bool {
    fn floats(a: &Allocation) -> [u64; 5] {
        [a.support, a.intensity, a.share, a.mu, a.actual_retention].map(f64::to_bits)
    }
    a == b
        && a.credibility.to_bits() == b.credibility.to_bits()
        && a.verifiability.to_bits() == b.verifiability.to_bits()
        && a.allocations.iter().zip(&b.allocations).all(|(a, b)| {
            floats(a) == floats(b)
                && a.projection.map(f64::to_bits) == b.projection.map(f64::to_bits)
        })
}

#[test]
fn raw_comparison_rejects_negative_zero_in_state_and_allocations() {
    let f = super::world::Fixture::new(8);
    let t = super::fixture::tendency(&f.law, "k", None);
    let original = f.snapshot(vec![t]);
    let mut changed = original.clone();
    changed.tendencies[0].magnitude_sum = -0.0;
    assert_eq!(original, changed); // Ordinary float equality is insufficient.
    assert!(!snapshot_bits_eq(&original, &changed));
    let group = EvidenceGroup {
        schema_version: 1,
        group_id: "g".into(),
        members: vec![],
        source_key: "source".into(),
        credibility: 1.0,
        verifiability: 1.0,
        allocations: vec![Allocation {
            schema_version: 1,
            tendency_id: "k".into(),
            support: 0.0,
            intensity: 1.0,
            share: 0.0,
            ordinal: core_types::scalar::DecimalU64(0),
            mu: 0.0,
            projection: Some(0.0),
            actual_retention: 1.0,
        }],
    };
    let mut changed = group.clone();
    changed.allocations[0].projection = Some(-0.0);
    assert_eq!(group, changed);
    assert!(!group_bits_eq(&group, &changed));
}
