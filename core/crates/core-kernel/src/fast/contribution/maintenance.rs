use super::*;
use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::*,
    scalar::hex,
};

impl State {
    /// §4.7: aliases merge only their h, in first-effect order; x and anchor
    /// stay unchanged. Canonical representative/validity are governed in H.
    pub fn merge_aliases(
        &self,
        members: &[HistoryBasis<'_>],
        limits: Limits,
    ) -> Result<Self, FastError> {
        if members.is_empty() {
            return Err(FastError::OutOfRange("empty alias group"));
        }
        let mut ordered: Vec<_> = members.iter().collect();
        ordered.sort_by_key(|h| h.first_position);
        for (i, h) in ordered.iter().enumerate() {
            if h.target.record().basis.kind == FastBasisKind::FAST_BASIS_INTERNAL_BRANCH
                || ordered[..i].iter().any(|p| {
                    p.first_position == h.first_position
                        || p.target.record().basis == h.target.record().basis
                })
            {
                return Err(FastError::OutOfRange("alias membership/order"));
            }
            if h.target.record().law_version != self.snapshot.law_version {
                return Err(FastError::OutOfRange("law binding"));
            }
        }
        // Revision itself must not change targets: use the recorded target of
        // the earliest member; deferred reinterpretation belongs to revisit.
        let earliest = ordered[0].target.record();
        let target = codec::align(&earliest.target, &self.snapshot.learned);
        let mut sum = vec![0.0; 4 + target.learned.len()];
        for member in &ordered {
            if let Some(slot) = self
                .snapshot
                .slots
                .iter()
                .find(|s| s.basis == member.target.record().basis)
            {
                for (sum, h) in sum.iter_mut().zip(codec::values(&slot.current)) {
                    *sum = finite(*sum + h, "alias sum")?;
                }
            }
        }
        let mut next = self.clone();
        next.snapshot
            .slots
            .retain(|s| !ordered.iter().any(|m| s.basis == m.target.record().basis));
        let mut current = target.clone();
        codec::set_values(&mut current, &sum);
        next.snapshot.slots.push(FastBasisSlot {
            schema_version: 1,
            basis: earliest.basis.clone(),
            current,
            target,
        });
        codec::sort_slots(&mut next, limits)?;
        Ok(next)
    }

    /// §4.6 B7: no ε and no hard capacity. H retains the target for re-entry.
    pub fn remove_zero_slots(&self) -> Self {
        let mut next = self.clone();
        next.snapshot
            .slots
            .retain(|s| codec::values(&s.current).iter().any(|v| *v != 0.0));
        next
    }

    /// §4.3/4.7: F_out has no extra event or time step. A removed then
    /// re-entered learned coordinate starts at +0; H still holds old targets.
    pub fn f_out(&self, effects: &EffectiveEffects) -> Result<Self, FastError> {
        let x = super::super::f_out(&self.visible(), effects)?;
        let mut next = self.clone();
        next.set_visible(&x);
        for slot in &mut next.snapshot.slots {
            slot.current = codec::align(&slot.current, &next.snapshot.learned);
            slot.target = codec::align(&slot.target, &next.snapshot.learned);
        }
        Ok(next)
    }

    /// §4.8: pure migration preparation. Caller supplies the actual snapshot
    /// references and commits the returned record in H/L atomically (P5-75).
    pub fn migrate(
        old: &FastStateSnapshotV2,
        law: &ObjectRef,
        record: FastStateMigrationRecord,
        limits: Limits,
    ) -> Result<(Self, FastStateMigrationRecord), FastError> {
        codec::check(old, limits)?;
        codec::check(&record, limits)?;
        if old.representation_version != super::super::REPRESENTATION_VERSION {
            return Err(FastError::RepresentationVersion(
                old.representation_version.clone(),
            ));
        }
        let next = Self::from_snapshot(
            FastStateSnapshotV3 {
                schema_version: 1,
                revision: old.revision.clone(),
                representation_version: REPRESENTATION_VERSION.into(),
                anchor_time_ns: old.anchor_time_ns,
                nature: old.nature.clone(),
                learned: old.learned.clone(),
                slots: Vec::new(),
                law_version: law.clone(),
            },
            law,
            limits,
        )?;
        // Content references must describe the exact old/new bytes. No storage
        // access is necessary to verify this binding (SYS17 pure boundary).
        for (reference, name, bytes) in [
            (
                &record.previous_snapshot,
                FastStateSnapshotV2::NAME,
                old.canonical_bytes(limits).map_err(FastError::Codec)?,
            ),
            (
                &record.migrated_snapshot,
                FastStateSnapshotV3::NAME,
                next.canonical_bytes(limits)?,
            ),
        ] {
            let schema = Schema::builtin().map_err(FastError::Codec)?;
            let value = schema
                .decode_canonical(name, &bytes, limits, 0)
                .map_err(FastError::Codec)?;
            let digest = schema
                .digest(name, &value, limits)
                .map_err(FastError::Codec)?;
            if reference.byte_length.0 != bytes.len() as u64 || hex(&reference.digest.0) != digest {
                return Err(FastError::OutOfRange("migration reference content"));
            }
        }
        Ok((next, record))
    }
}
