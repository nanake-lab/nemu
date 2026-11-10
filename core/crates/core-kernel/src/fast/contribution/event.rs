use super::*;
use core_types::{WireMessage, contract::Limits, generated::*};

/// Checked F record. Target construction uses recorded inputs and their actual
/// prestate, never the current state of a later revisit (ADR 0035 §4.3).
#[derive(Clone, Debug)]
pub struct Effect {
    record: FastEffectRecord,
}

impl Effect {
    pub fn new(
        record: FastEffectRecord,
        target: &Target,
        limits: Limits,
    ) -> Result<Self, FastError> {
        codec::check(&record, limits)?;
        if record.law_version != target.law_version || record.target != codec::vector(&target.drive)
        {
            return Err(FastError::OutOfRange("target binding"));
        }
        // A revisit is assembled only from a resolved history below.
        if record.role == FastEffectRole::FAST_EFFECT_REVISIT {
            return Err(FastError::OutOfRange("revisit needs history"));
        }
        let record = FastEffectRecord::from_canonical(
            &record.canonical_bytes(limits).map_err(FastError::Codec)?,
            limits,
        )
        .map_err(FastError::Codec)?;
        Ok(Self { record })
    }
    pub fn record(&self) -> &FastEffectRecord {
        &self.record
    }
}

/// A read-only view resolved from H by the caller, not new persistent state.
/// `target` is the latest target-setting record, NOT the last revisit.
/// `reinterpreted` is its new interpretation computed with that same prestate.
/// The caller verifies references, provenance and current interpreter version;
/// P5-75 owns that resolution and its atomic commit with H/L.
pub struct HistoryBasis<'a> {
    pub first_position: u64,
    pub target: &'a Effect,
    pub reinterpreted: Option<&'a Effect>,
    pub active: bool,
    /// Governance of the current revisit, not the old target-setting event.
    pub revisit_governance: &'a [ObjectRef],
}

impl HistoryBasis<'_> {
    pub(super) fn selected(&self) -> Result<&FastEffectRecord, FastError> {
        let original = &self.target.record;
        let Some(revised) = self.reinterpreted else {
            return Ok(original);
        };
        let r = &revised.record;
        // §4.7 / §9(12): θ₂ and its R+ view, not θ₀/θ₁ or today's θ.
        if r.basis != original.basis
            || r.theta_before != original.theta_before
            || r.effects_before != original.effects_before
            || r.law_version != original.law_version
        {
            return Err(FastError::OutOfRange("reinterpretation prestate"));
        }
        Ok(r)
    }
}

impl State {
    /// New basis, target update or internal branch. The role/identity pair has
    /// passed the P5-68 boundary, including internal TARGET_UPDATE (§5.2).
    /// Consumption eligibility and target ownership are caller obligations.
    pub fn apply(&self, effect: &Effect, limits: Limits) -> Result<Self, FastError> {
        self.apply_record(&effect.record, limits)
    }

    fn apply_record(&self, record: &FastEffectRecord, limits: Limits) -> Result<Self, FastError> {
        if record.law_version != self.snapshot.law_version {
            return Err(FastError::OutOfRange("law binding"));
        }
        let mut next = self.clone();
        let target = codec::align(&record.target, &self.snapshot.learned);
        let index = match next
            .snapshot
            .slots
            .iter()
            .position(|s| s.basis == record.basis)
        {
            Some(index) => {
                if record.role == FastEffectRole::FAST_EFFECT_NEW_BASIS
                    && next.snapshot.slots[index].target != target
                {
                    return Err(FastError::OutOfRange("existing basis needs target update"));
                }
                index
            }
            None => {
                let mut current = target.clone();
                codec::set_values(&mut current, &vec![0.0; 4 + target.learned.len()]);
                next.snapshot.slots.push(FastBasisSlot {
                    schema_version: 1,
                    basis: record.basis.clone(),
                    current,
                    target: target.clone(),
                });
                next.snapshot.slots.len() - 1
            }
        };
        let h = codec::values(&next.snapshot.slots[index].current);
        let v = codec::values(&target);
        let mut x = self.visible();
        let mut current = Vec::with_capacity(h.len());
        for ((x, h), v) in x
            .nature
            .iter_mut()
            .chain(x.learned.iter_mut().map(|l| &mut l.value))
            .zip(h)
            .zip(v)
        {
            let (new_x, new_h) = coordinate(*x, h, v)?;
            *x = new_x;
            current.push(new_h);
        }
        next.set_visible(&x);
        let slot = &mut next.snapshot.slots[index];
        codec::set_values(&mut slot.current, &current);
        slot.target = target;
        codec::sort_slots(&mut next, limits)?;
        Ok(next)
    }

    /// §4.2: multi-basis revisits in first-H-effect order (not slot order).
    /// Invalidated bases do not act; interpreter-revision commits themselves
    /// have no F operation. Returns staged H receipts alongside staged x.
    pub fn revisit(
        &self,
        history: &[HistoryBasis<'_>],
        limits: Limits,
    ) -> Result<(Self, Vec<FastEffectRecord>), FastError> {
        let mut ordered: Vec<_> = history.iter().filter(|h| h.active).collect();
        ordered.sort_by_key(|h| h.first_position);
        for (i, h) in ordered.iter().enumerate() {
            if ordered[..i].iter().any(|p| {
                p.first_position == h.first_position
                    || p.target.record.basis == h.target.record.basis
            }) {
                return Err(FastError::OutOfRange("duplicate revisit basis/order"));
            }
        }
        let mut next = self.clone();
        let mut receipts = Vec::new();
        for h in ordered {
            let mut r = h.selected()?.clone();
            if r.basis.kind == FastBasisKind::FAST_BASIS_INTERNAL_BRANCH {
                return Err(FastError::OutOfRange("internal revisit"));
            }
            r.role = FastEffectRole::FAST_EFFECT_REVISIT;
            r.governance_refs = h.revisit_governance.to_vec();
            codec::check(&r, limits)?;
            next = next.apply_record(&r, limits)?;
            receipts.push(r);
        }
        Ok((next, receipts))
    }
}
