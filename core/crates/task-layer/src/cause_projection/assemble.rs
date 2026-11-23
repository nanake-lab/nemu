//! Four-section cause-projection request assembly and whole-pack budgeting
//! (ADR 0042 §§2.1–2.4, 3.2, 3.3, 5.1, 5.3; Steps 5.5–5.9).

use std::collections::BTreeSet;

use super::RequestCodec;
use super::closure::{ClosureOutput, RelationIndex, compute_closure};
use super::rank::rank_candidate_slots;
use super::types::{
    AssemblyInput, BaseFailure, BasisId, Binding, Budget, Delivered, HPos, Outcome, PackEntry,
    PackFate, PresentedD, PresentedE, RecordId, RecordTable, RequestPlan, SentNote, Unavailable,
};
use super::validate::{ValidatedOrUnavailable, validate_and_check_global};

/// Pre-checked candidate pack awaiting budget packing in Step 5.9 (ADR 0042 §§1.3, 2.2).
struct CandidatePack {
    basis: BasisId,
    ordered: Vec<(HPos, RecordId)>,
    excluded: Option<(Option<RecordId>, String)>,
}

/// Shared read-only context passed during candidate packing (ADR 0042 §2.2).
struct PackContext<'a> {
    input: &'a AssemblyInput,
    records: &'a RecordTable,
    rels: &'a RelationIndex,
    presented_d: &'a [PresentedD],
    presented_full: &'a BTreeSet<RecordId>,
    latest_continuation_refs: &'a [RecordId],
}

/// Executes Steps 5.1–5.9 to produce a deterministic `Outcome` (ADR 0042 §§1.2–3.3).
pub(crate) fn assemble_impl<C: RequestCodec>(
    input: &AssemblyInput,
    codec: &C,
) -> Result<Outcome, super::types::InvalidInput> {
    // Steps 5.1–5.2 (Step 0 Q2A, Q6B).
    let records = match validate_and_check_global(input)? {
        ValidatedOrUnavailable::Ready(table) => table,
        ValidatedOrUnavailable::Unavailable(u) => return Ok(Outcome::ProjectionUnavailable(u)),
    };
    let rels = RelationIndex::new(&input.relations);

    // Step 5.3 (ADR 0042 §1.2, U1): rank non-zero candidate slots.
    let ranked_slots = rank_candidate_slots(&input.slots);

    // Step 5.5 & Step 5.7(2): base material closure.
    let base_seeds = collect_base_seeds(input);
    let base_closure = match compute_closure(&base_seeds, &records, &rels) {
        Ok(c) => c,
        Err(u) => return Ok(Outcome::ProjectionUnavailable(u)),
    };

    // Step 5.6 & Step 5.7(3) (Step 0 Q2A, Q4A, Q7A): evaluate all candidate closures upfront.
    let candidates = match build_candidates(&ranked_slots, &records, &rels) {
        Ok(c) => c,
        Err(u) => return Ok(Outcome::ProjectionUnavailable(u)),
    };

    // Step 5.7(4): fail if any record in the base closure is `Ineligible`.
    if let Some((record, reason)) = base_closure.first_ineligible {
        return Ok(Outcome::BaseRequestFailed(BaseFailure::Ineligible {
            record,
            reason,
        }));
    }

    // Step 5.8 (ADR 0042 §2.1; Step 0 Q1A, Q5A): build D, presented_full, and E_base.
    let presented_d = build_presented_d(input, &rels);
    let presented_full = collect_presented_full(&input.latest, &presented_d);
    let latest_continuation_refs = rels.continuation_refs(&input.latest);
    let mut current_e = build_base_e(input, &records, &base_closure, &presented_full);

    let ctx = PackContext {
        input,
        records: &records,
        rels: &rels,
        presented_d: &presented_d,
        presented_full: &presented_full,
        latest_continuation_refs: &latest_continuation_refs,
    };

    let base_plan = build_plan(&ctx, &current_e);
    let base_bytes = codec.encode(&base_plan, &records);
    let base_measured = codec.measure(&base_bytes);
    if !fits_budget(base_measured, input.budget) {
        return Ok(Outcome::BaseRequestFailed(BaseFailure::OverCapacity {
            measured: base_measured,
            output_reserve: input.budget.output_reserve,
            capacity: input.budget.capacity,
        }));
    }

    // Step 5.9 (ADR 0042 §2.2, U2; Step 0 Q8A): whole-pack greedy packing in rank order.
    let (packs, accepted, has_non_excluded, overflowed) =
        pack_candidates(&ctx, candidates, &mut current_e, codec);

    let plan = build_plan(&ctx, &current_e);
    let bytes = codec.encode(&plan, &records);
    let measured = codec.measure(&bytes);

    let note = if !has_non_excluded {
        SentNote::NoEligibleExperience
    } else if overflowed {
        SentNote::OptionalOverBudget
    } else {
        SentNote::AllFit
    };

    let base_records = base_closure.ordered.into_iter().map(|(_, id)| id).collect();

    Ok(Outcome::Sent {
        plan,
        bytes,
        binding: Binding {
            base_records,
            packs,
            accepted,
            measured,
        },
        note,
    })
}

/// Collects locked base-material seed IDs (ADR 0042 §2.1; Step 5.5).
fn collect_base_seeds(input: &AssemblyInput) -> BTreeSet<RecordId> {
    let mut seeds = BTreeSet::new();
    seeds.insert(input.latest.clone());
    for d in &input.dialogue {
        seeds.insert(d.id.clone());
    }
    for id in &input.required_facts {
        seeds.insert(id.clone());
    }
    for id in &input.protocol_cited {
        seeds.insert(id.clone());
    }
    seeds
}

/// Evaluates candidate slot closures in rank order (ADR 0042 §1.3; Step 5.6; Step 0 Q2A, Q4A).
fn build_candidates(
    ranked_slots: &[&super::types::Slot],
    records: &RecordTable,
    rels: &RelationIndex,
) -> Result<Vec<CandidatePack>, Unavailable> {
    let mut candidates = Vec::with_capacity(ranked_slots.len());

    for slot in ranked_slots {
        if slot.merge_undecided {
            return Err(Unavailable::MergeUndecided(slot.basis.clone()));
        }

        let has_empty_member_sources = slot.members.iter().any(|m| m.sources.is_empty());
        let mut pack_seeds = BTreeSet::new();
        for member in &slot.members {
            for src in &member.sources {
                pack_seeds.insert(src.clone());
            }
        }

        let closure = compute_closure(&pack_seeds, records, rels)?;
        let excluded = if let Some((id, reason)) = closure.first_ineligible {
            Some((Some(id), reason))
        } else if has_empty_member_sources {
            Some((None, "no_external_source".to_string()))
        } else {
            None
        };

        candidates.push(CandidatePack {
            basis: slot.basis.clone(),
            ordered: closure.ordered,
            excluded,
        });
    }

    Ok(candidates)
}

/// Deduplicates `D` keeping first occurrence while dropping `latest` (ADR 0042 §2.1; Step 5.8).
fn build_presented_d(input: &AssemblyInput, rels: &RelationIndex) -> Vec<PresentedD> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for item in &input.dialogue {
        if item.id == input.latest {
            continue;
        }
        if seen.insert(item.id.clone()) {
            out.push(PresentedD {
                id: item.id.clone(),
                delivered: item.delivered,
                continuation_refs: rels.continuation_refs(&item.id),
            });
        }
    }
    out
}

/// Collects record IDs already presented in full in `o` or `D` (ADR 0042 §2.1; Step 5.8).
fn collect_presented_full(latest: &RecordId, presented_d: &[PresentedD]) -> BTreeSet<RecordId> {
    let mut full = BTreeSet::new();
    full.insert(latest.clone());
    for item in presented_d {
        if item.delivered == Delivered::Full {
            full.insert(item.id.clone());
        }
    }
    full
}

/// Builds initial `E_base` excluding `Partial`-only dialogue items unless independently required
/// (ADR 0042 §2.1, §3.2, §5.1; Step 0 Q1 Reading A).
fn build_base_e(
    input: &AssemblyInput,
    records: &RecordTable,
    base_closure: &ClosureOutput,
    presented_full: &BTreeSet<RecordId>,
) -> BTreeSet<(HPos, RecordId)> {
    let mut needed_full: BTreeSet<(HPos, RecordId)> = base_closure.expanded_targets.clone();

    for id in input
        .required_facts
        .iter()
        .chain(input.protocol_cited.iter())
    {
        if let Some(rec) = records.get(id) {
            needed_full.insert((rec.h_pos, rec.id.clone()));
        }
    }

    needed_full
        .into_iter()
        .filter(|(_, id)| !presented_full.contains(id))
        .collect()
}

/// Constructs a `RequestPlan` for a given `E` set with `strict = false` (ADR 0042 §2.1, §5.1).
fn build_plan(ctx: &PackContext<'_>, e_set: &BTreeSet<(HPos, RecordId)>) -> RequestPlan {
    let experience = e_set
        .iter()
        .map(|(_, id)| PresentedE {
            id: id.clone(),
            continuation_refs: ctx.rels.continuation_refs(id),
        })
        .collect();

    RequestPlan {
        protocol: ctx.input.protocol.clone(),
        dialogue: ctx.presented_d.to_vec(),
        experience,
        latest: ctx.input.latest.clone(),
        latest_continuation_refs: ctx.latest_continuation_refs.to_vec(),
        strict: false,
    }
}

/// Checks `measured + output_reserve <= capacity` using overflow-safe arithmetic (Step 5.8).
fn fits_budget(measured: u64, budget: Budget) -> bool {
    measured
        .checked_add(budget.output_reserve)
        .is_some_and(|total| total <= budget.capacity)
}

/// Packs ranked candidate slots whole-pack-or-nothing, stopping on first overflow
/// (ADR 0042 §2.2, U2; Step 5.9; Step 0 Q8 Reading A).
fn pack_candidates<C: RequestCodec>(
    ctx: &PackContext<'_>,
    candidates: Vec<CandidatePack>,
    current_e: &mut BTreeSet<(HPos, RecordId)>,
    codec: &C,
) -> (Vec<PackEntry>, u32, bool, bool) {
    let mut packs = Vec::with_capacity(candidates.len());
    let mut accepted: u32 = 0;
    let mut has_non_excluded = false;
    let mut overflowed = false;

    for cand in candidates {
        let pack_records: Vec<RecordId> = cand.ordered.iter().map(|(_, id)| id.clone()).collect();

        if let Some((record, reason)) = cand.excluded {
            // Step 0 Q8 Reading A: keep `Excluded` fate even after `FirstOverflow`.
            packs.push(PackEntry {
                basis: cand.basis,
                records: pack_records,
                fate: PackFate::Excluded { record, reason },
            });
            continue;
        }

        has_non_excluded = true;
        if overflowed {
            packs.push(PackEntry {
                basis: cand.basis,
                records: pack_records,
                fate: PackFate::NotTried,
            });
            continue;
        }

        let mut trial_e = current_e.clone();
        let mut new_records = Vec::new();
        for (pos, id) in &cand.ordered {
            if !ctx.presented_full.contains(id) && trial_e.insert((*pos, id.clone())) {
                new_records.push(id.clone());
            }
        }

        let trial_plan = build_plan(ctx, &trial_e);
        let trial_bytes = codec.encode(&trial_plan, ctx.records);
        let trial_measured = codec.measure(&trial_bytes);

        if fits_budget(trial_measured, ctx.input.budget) {
            *current_e = trial_e;
            accepted += 1;
            packs.push(PackEntry {
                basis: cand.basis,
                records: pack_records,
                fate: PackFate::Accepted { new_records },
            });
        } else {
            overflowed = true;
            packs.push(PackEntry {
                basis: cand.basis,
                records: pack_records,
                fate: PackFate::FirstOverflow,
            });
        }
    }

    (packs, accepted, has_non_excluded, overflowed)
}
