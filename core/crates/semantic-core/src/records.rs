//! Record governance never changes a model value, guesses a speaker or infers facts.
use crate::{
    Fault, Result,
    graph::{Snapshot, source},
    objects::{check_binding, check_object},
    wire,
};
use core_types::{contract::Limits, generated::*, semantic};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub enum Record<'a> {
    Appraisal(&'a NatureAppraisalRecord),
    Match(&'a SituationMatchRecord),
    Candidate(&'a TendencyCandidateRecord),
    Teaching(&'a TeachingAnnotation),
    Lie(&'a LieAnnotation),
    SameMatter(&'a SameMatterAnnotation),
    Reinterpretation(&'a ReinterpretationLink),
}
pub struct Context<'a> {
    pub snapshot: Snapshot<'a>,
    /// Trusted dispatch expectation, including proposal payload and read-set digests.
    pub binding: &'a ProposalBindingV2,
    pub producer: &'a str,
    pub producer_version: &'a str,
    /// Core-selected target of this interpretation (can differ from dispatch event).
    pub target_event: &'a str,
    pub known_tendencies: &'a BTreeSet<String>,
    pub known_persons: &'a BTreeSet<String>,
    pub prior_record_ids: &'a BTreeSet<String>,
    pub limits: Limits,
}

pub fn preflight(record: Record<'_>, context: &Context<'_>) -> Result<()> {
    let (id, event, binding, producer, version) = match record {
        Record::Appraisal(r) => {
            wire(r, context.limits)?;
            if r.persons.iter().any(|p| !context.known_persons.contains(p)) {
                return Err(Fault::Reference);
            }
            (
                &r.record_id,
                &r.event_id,
                &r.binding,
                &r.producer,
                &r.producer_version,
            )
        }
        Record::Match(r) => {
            wire(r, context.limits)?;
            semantic::check_situation_match(r).map_err(|_| Fault::Numeric)?;
            if r.matches
                .iter()
                .any(|m| !context.known_tendencies.contains(&m.tendency_id))
            {
                return Err(Fault::Reference);
            }
            (
                &r.record_id,
                &r.event_id,
                &r.binding,
                &r.producer,
                &r.producer_version,
            )
        }
        Record::Candidate(r) => {
            wire(r, context.limits)?;
            semantic::check_candidate(r).map_err(|_| Fault::Numeric)?;
            if r.relation
                .as_ref()
                .is_some_and(|relation| !context.known_persons.contains(&relation.person_id))
            {
                return Err(Fault::Reference);
            }
            // ID reservation and registry staging belong to the outer IRL transaction.
            (
                &r.record_id,
                &r.event_id,
                &r.binding,
                &r.producer,
                &r.producer_version,
            )
        }
        Record::Teaching(r) => {
            wire(r, context.limits)?;
            let s = source(&context.snapshot, &r.event_id)?;
            if s.self_narrative
                || s.speaker.as_deref() != Some(r.speaker.as_str())
                || !context.known_persons.contains(&r.speaker)
                || !matches!(s.epistemic, EpistemicType::USER_ASSERTION)
                || !context.known_tendencies.contains(&r.tendency_id)
            {
                return Err(Fault::Source);
            }
            (
                &r.record_id,
                &r.event_id,
                &r.binding,
                &r.producer,
                &r.producer_version,
            )
        }
        Record::Lie(r) => {
            wire(r, context.limits)?;
            let statement = source(&context.snapshot, &r.statement)?;
            let fact = source(&context.snapshot, &r.contradicts)?;
            if r.statement == r.contradicts
                || statement.self_narrative
                || fact.self_narrative
                || statement.epistemic == EpistemicType::MODEL_INFERENCE
                || fact.epistemic == EpistemicType::MODEL_INFERENCE
                || !fact.verifiable
            {
                return Err(Fault::Source);
            }
            (
                &r.record_id,
                &r.event_id,
                &r.binding,
                &r.producer,
                &r.producer_version,
            )
        }
        Record::SameMatter(r) => {
            wire(r, context.limits)?;
            for id in &r.same_as {
                source(&context.snapshot, id)?;
            }
            (
                &r.record_id,
                &r.event_id,
                &r.binding,
                &r.producer,
                &r.producer_version,
            )
        }
        Record::Reinterpretation(r) => {
            wire(r, context.limits)?;
            if !context.prior_record_ids.contains(&r.new_record)
                || r.prior_records.iter().any(|id| {
                    !context.prior_record_ids.contains(id)
                        || id == &r.new_record
                        || id == &r.record_id
                })
                || r.interpreter_version != context.producer_version
            {
                return Err(Fault::Reference);
            }
            (
                &r.record_id,
                &r.event_id,
                &r.binding,
                &r.producer,
                &r.producer_version,
            )
        }
    };
    check_binding(binding, context.binding, context.limits)?;
    source(&context.snapshot, event)?;
    if event != context.target_event
        || producer != context.producer
        || version != context.producer_version
    {
        return Err(Fault::Binding);
    }
    if id == event
        || context.prior_record_ids.contains(id)
        || context.snapshot.sources.contains_key(id)
    {
        return Err(Fault::Duplicate);
    }
    Ok(())
}

/// Prepare an auxiliary interpretation for the outer transaction. `assigned`
/// is the core-reserved endpoint for the actual returned record. A candidate
/// tendency stays a hypothesis and an annotation cannot grant evidence access.
/// A/M and revisions have their own stage and receipt checks.
/// For a lie annotation the core also resolves her confirmed statement from
/// the authorized read set. A worker-supplied speaker or matching text is insufficient.
pub fn stage_auxiliary_record(
    record: Record<'_>,
    assigned: &RecordVersion,
    context: &Context<'_>,
    confirmed_own_statement: Option<&RecordVersion>,
) -> Result<RecordVersion> {
    let actual_id = match record {
        Record::Candidate(r) => {
            check_object(&assigned.content, r, context.limits)?;
            &r.record_id
        }
        Record::Teaching(r) => {
            check_object(&assigned.content, r, context.limits)?;
            &r.record_id
        }
        Record::Lie(r) => {
            let statement = confirmed_own_statement.ok_or(Fault::Source)?;
            if statement.record_id != r.statement
                || statement.branch_id != assigned.branch_id
                || !context.binding.read_set.contains(&statement.content)
            {
                return Err(Fault::Source);
            }
            check_object(&assigned.content, r, context.limits)?;
            &r.record_id
        }
        Record::SameMatter(r) => {
            check_object(&assigned.content, r, context.limits)?;
            &r.record_id
        }
        _ => return Err(Fault::Binding),
    };
    wire(assigned, context.limits)?;
    if assigned.branch_id != context.binding.branch_id || actual_id != &assigned.record_id {
        return Err(Fault::Binding);
    }
    preflight(record, context)?;
    Ok(assigned.clone())
}
