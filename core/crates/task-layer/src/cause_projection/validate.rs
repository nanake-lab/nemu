//! Input validation and global availability checks (ADR 0042 §§1.2, 1.3, 3.2; Steps 5.1–5.2).

use std::collections::{BTreeMap, BTreeSet};

use super::types::{
    AssemblyInput, BasisId, Delivered, InvalidInput, Record, RecordId, RecordTable, Slot,
    Unavailable,
};

/// Result of Steps 5.1–5.2: either a validated `RecordTable` or an early global `Unavailable`.
pub(crate) enum ValidatedOrUnavailable {
    Ready(RecordTable),
    Unavailable(Unavailable),
}

/// Validates caller invariants (Step 5.1) and checks global availability (Step 5.2).
///
/// Per Step 0 Q6 Reading B, global `frontier_complete` and `decomposition_unknown` checks
/// (ADR 0042 §§1.2, 3.2) run before locked base-record existence checks so truncated frontiers
/// report `ProjectionUnavailable` rather than `Err(InvalidInput)`.
pub(crate) fn validate_and_check_global(
    input: &AssemblyInput,
) -> Result<ValidatedOrUnavailable, InvalidInput> {
    validate_slots(&input.slots)?;

    let map = build_validated_record_map(&input.records)?;

    // Step 5.2 (ADR 0042 §§1.2, 3.2; Step 0 Q6 Reading B): global availability precedes
    // locked base-record existence checks.
    if !input.frontier_complete {
        return Ok(ValidatedOrUnavailable::Unavailable(
            Unavailable::FrontierIncomplete,
        ));
    }
    if input.decomposition_unknown {
        return Ok(ValidatedOrUnavailable::Unavailable(
            Unavailable::DecompositionUnknown,
        ));
    }

    validate_base_records_exist(input, &map)?;
    validate_partial_ranges(&input.dialogue, &map)?;

    Ok(ValidatedOrUnavailable::Ready(RecordTable::new(map)))
}

/// Validates slot `basis` uniqueness, `h` finiteness, and non-empty `members`
/// (ADR 0042 §1.2, §1.3; Step 5.1; Step 0 Q2 Reading A).
fn validate_slots(slots: &[Slot]) -> Result<(), InvalidInput> {
    let mut by_basis: BTreeMap<&BasisId, &Slot> = BTreeMap::new();
    let mut duplicate_bases: BTreeSet<BasisId> = BTreeSet::new();

    for slot in slots {
        if by_basis.insert(&slot.basis, slot).is_some() {
            duplicate_bases.insert(slot.basis.clone());
        }
    }

    if let Some(first_dup) = duplicate_bases.into_iter().next() {
        return Err(InvalidInput::DuplicateSlotBasis(first_dup));
    }

    // Iterate in canonical `BasisId` byte order for input-order independence.
    for (basis, slot) in &by_basis {
        for (index, coord) in slot.h.iter().copied().enumerate() {
            if !coord.is_finite() {
                return Err(InvalidInput::NonFiniteH {
                    basis: (*basis).clone(),
                    index,
                });
            }
        }
        if slot.members.is_empty() {
            return Err(InvalidInput::EmptySlotMembers((*basis).clone()));
        }
    }

    Ok(())
}

/// Deduplicates identical records and rejects conflicting payloads for the same `RecordId`
/// (ADR 0042 §1.3; Step 5.1).
fn build_validated_record_map(
    records: &[Record],
) -> Result<BTreeMap<RecordId, Record>, InvalidInput> {
    let mut map: BTreeMap<RecordId, Record> = BTreeMap::new();
    let mut conflicts: BTreeSet<RecordId> = BTreeSet::new();

    for rec in records {
        if let Some(existing) = map.get(&rec.id) {
            if existing != rec {
                conflicts.insert(rec.id.clone());
            }
        } else {
            map.insert(rec.id.clone(), rec.clone());
        }
    }

    if let Some(first_conflict) = conflicts.into_iter().next() {
        return Err(InvalidInput::ConflictingDuplicateRecord(first_conflict));
    }

    Ok(map)
}

/// Verifies all locked base-material `RecordId`s exist in `records` (ADR 0042 §2.1; Step 5.1).
fn validate_base_records_exist(
    input: &AssemblyInput,
    map: &BTreeMap<RecordId, Record>,
) -> Result<(), InvalidInput> {
    if !map.contains_key(&input.latest) {
        return Err(InvalidInput::MissingBaseRecord(input.latest.clone()));
    }
    for d in &input.dialogue {
        if !map.contains_key(&d.id) {
            return Err(InvalidInput::MissingBaseRecord(d.id.clone()));
        }
    }
    for id in &input.required_facts {
        if !map.contains_key(id) {
            return Err(InvalidInput::MissingBaseRecord(id.clone()));
        }
    }
    for id in &input.protocol_cited {
        if !map.contains_key(id) {
            return Err(InvalidInput::MissingBaseRecord(id.clone()));
        }
    }
    Ok(())
}

/// Validates `Delivered::Partial { start, end }` byte ranges against record body lengths (Step 5.1).
fn validate_partial_ranges(
    dialogue: &[super::types::DItem],
    map: &BTreeMap<RecordId, Record>,
) -> Result<(), InvalidInput> {
    for d in dialogue {
        if let Delivered::Partial { start, end } = d.delivered {
            let Some(rec) = map.get(&d.id) else {
                return Err(InvalidInput::MissingBaseRecord(d.id.clone()));
            };
            let body_len = u64::try_from(rec.body.len()).unwrap_or(u64::MAX);
            if start > end || u128::from(end) > (rec.body.len() as u128) {
                return Err(InvalidInput::InvalidPartialRange {
                    id: d.id.clone(),
                    start,
                    end,
                    body_len,
                });
            }
        }
    }
    Ok(())
}
