//! ADR 0048 G04. Results are prepared in memory for the caller's transaction.
use crate::{
    Fault, Result,
    irl::adopted,
    objects::check_object,
    records::{self, Context, Record},
};
use core_types::{
    contract::{self, Limits},
    generated::*,
    semantic::{SemanticReadView, StageExecution, check_reinterpretation, check_stage_execution},
};

/// Additional facts captured by the core during this isolated A or M execution.
/// The context and definitions are actual authorized reads. `seen.interpretation`
/// is the core-reserved identity and content reference of the result.
pub struct JudgmentDispatch<'a> {
    pub seen: StageExecution<'a>,
    pub context: &'a ObjectRef,
    pub definitions: &'a [ObjectRef],
    pub governance: &'a ObjectRef,
    /// M coverage captured by the core with the actual output. The legacy
    /// sparse candidate itself cannot attest zero or unknown for a target.
    pub assessed_without_positive_match: &'a [StageCoverage],
}

/// M's original sparse record stores positive values only. Explicit zeroes and
/// assessed unknowns come from the actual scoped output of that dispatch.
/// Never turn an omitted target into an observation (ADR 0048 §4).
pub enum Judgment<'a> {
    Appraisal(&'a NatureAppraisalRecord),
    Match(&'a SituationMatchRecord),
    Failed(&'a DegradedStatus, &'a str),
}

pub fn stage_judgment(
    dispatch: &JudgmentDispatch<'_>,
    context: &Context<'_>,
    result: Judgment<'_>,
) -> Result<StageAdoption> {
    use InterpretationStage::*;
    use StageOutcome::*;
    let seen = &dispatch.seen;
    let limits = context.limits;
    let resource = match seen.stage {
        INTERPRET_STAGE_A => &seen.binding.environment.nature_appraiser,
        INTERPRET_STAGE_M => &seen.binding.environment.situation_matcher,
        INTERPRET_STAGE_IRL => return Err(Fault::Binding),
    };
    if context.target_event != seen.event.record_id
        || context.binding != seen.binding
        || !dispatch.assessed_without_positive_match.is_empty()
            && !matches!(result, Judgment::Match(_))
    {
        return Err(Fault::Binding);
    }
    let mut coverage = Vec::new();
    let mut appraisal = None;
    let mut outcome = STAGE_OUTCOME_VALID;
    let mut reason = None;
    match result {
        Judgment::Appraisal(r) if seen.stage == INTERPRET_STAGE_A => {
            records::preflight(Record::Appraisal(r), context)?;
            if r.record_id != seen.interpretation.record_id {
                return Err(Fault::Binding);
            }
            check_object(seen.candidate.ok_or(Fault::Binding)?, r, limits)?;
            check_object(&seen.interpretation.content, r, limits)?;
            appraisal = Some(r.appraisal.clone());
        }
        Judgment::Match(record) if seen.stage == INTERPRET_STAGE_M => {
            records::preflight(Record::Match(record), context)?;
            if record.record_id != seen.interpretation.record_id {
                return Err(Fault::Binding);
            }
            check_object(seen.candidate.ok_or(Fault::Binding)?, record, limits)?;
            check_object(&seen.interpretation.content, record, limits)?;
            for c in dispatch.assessed_without_positive_match {
                if !context.known_tendencies.contains(&c.target) || c.mu.is_some_and(|v| v != 0.0) {
                    return Err(Fault::Reference);
                }
                if c.unknown_reason.is_some() {
                    outcome = STAGE_OUTCOME_PARTIAL;
                    reason = Some("unresolved match targets".into());
                }
                coverage.push(c.clone());
            }
            coverage.extend(record.matches.iter().map(|m| StageCoverage {
                schema_version: 1,
                target: m.tendency_id.clone(),
                mu: Some(m.mu),
                accepted: None,
                unknown_reason: None,
            }));
        }
        Judgment::Failed(status, why) => {
            let expected = if seen.stage == INTERPRET_STAGE_A {
                RunStatus::A_DEGRADED
            } else {
                RunStatus::M_DEGRADED
            };
            if !status.states.contains(&expected) || status.reasons.is_empty() {
                return Err(Fault::Binding);
            }
            check_object(&seen.interpretation.content, status, limits)?;
            outcome = STAGE_OUTCOME_FAILED;
            reason = Some(why.to_owned());
        }
        _ => return Err(Fault::Binding),
    }
    let stage = StageAdoption {
        schema_version: 1,
        event: seen.event.clone(),
        interpretation: seen.interpretation.clone(),
        stage: seen.stage,
        outcome,
        binding: seen.binding.clone(),
        producer_resource: resource.clone(),
        candidate: seen.candidate.cloned(),
        actual_output: seen.actual_output.cloned(),
        context: dispatch.context.clone(),
        definitions: dispatch.definitions.to_vec(),
        isolation: seen.isolation.clone(),
        governance: dispatch.governance.clone(),
        coverage,
        appraisal,
        reason,
        registry_after: None,
        revision: None,
        relations: None,
        transaction_id: seen.transaction_id.into(),
        replaced_targets: vec![],
        registry_before: seen.binding.environment.registry.clone(),
        staged_revisions: vec![],
    };
    check_stage_execution(&stage, seen, limits).map_err(|_| Fault::Binding)?;
    Ok(stage)
}

/// Authorization for this exact new stage, issued by the trusted core. It
/// reserves a fresh interpretation ID, pins the committed predecessor and the
/// target subset, and obtains the manifest independently of the query reply.
pub struct RevisionApproval<'a> {
    pub prior_adoption: &'a RecordVersion,
    pub edge: &'a RecordRelation,
    pub targets: &'a [String],
    pub manifest: &'a ObjectRef,
}

pub trait RevisionPrefix: SemanticReadView {
    fn approved_revision(&self, staged: &StageAdoption) -> contract::Result<RevisionApproval<'_>>;
}

/// Attach a revision to a governed IRL, A or M result. The relation query is
/// always about the *old* endpoint at the base frontier; the new edge is staged.
pub fn stage_revision(
    mut staged: StageAdoption,
    prefix: &impl RevisionPrefix,
    receipt: RelationQueryReceipt,
    limits: Limits,
) -> Result<StageAdoption> {
    if prefix.transaction().is_some()
        || prefix.base_commit() != staged.binding.base_commit
        || prefix.branch() != staged.binding.branch_id
        || staged.revision.is_some()
        || staged.relations.is_some()
        || !staged.replaced_targets.is_empty()
    {
        return Err(Fault::Binding);
    }
    let approval = prefix
        .approved_revision(&staged)
        .map_err(|_| Fault::Binding)?;
    let prior: StageAdoption = adopted(prefix, approval.prior_adoption, limits)?;
    if approval.manifest != &receipt.manifest
        || prior.interpretation.record_id == staged.interpretation.record_id
    {
        return Err(Fault::Binding);
    }
    check_object(approval.manifest, &receipt.result, limits)?;
    staged.relations = Some(receipt);
    staged.revision = Some(approval.edge.clone());
    staged.replaced_targets = approval.targets.to_vec();
    check_reinterpretation(&staged, &prior, limits).map_err(|_| Fault::Binding)?;
    Ok(staged)
}
