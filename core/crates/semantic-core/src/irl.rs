//! First interpretation of a turn (ADR 0048 G04). Only the outer transaction
//! can publish the returned stage and graph. Reinterpretation is a separate operation.
use crate::{
    Fault, Result,
    objects::{check_binding, check_object, object_ref},
};
use core_types::{
    WireMessage,
    contract::Limits,
    generated::*,
    semantic::{
        SemanticReadView, StageExecution, VisibleContent, check_stage_execution, text_adapter,
    },
};

use super::irl_graph::govern;

/// Captured by the core from its isolated IRL dispatch, including the actual
/// returned objects and authorized reads. Never deserialize this from a proposal.
pub struct IrlDispatch<'a> {
    pub binding: &'a ProposalBindingV2,
    pub event: &'a RecordVersion,
    pub transaction: &'a str,
    pub interpretation_id: &'a str,
    pub producer: &'a str,
    pub producer_version: &'a str,
    pub representation_version: &'a str,
    pub candidate: &'a ObjectRef,
    pub actual_output: &'a ObjectRef,
    pub context: &'a ObjectRef,
    pub facts: &'a FactProjection,
    /// Adapter resource pinned by the isolated dispatch.
    pub adapter: &'a TextAdapterTable,
    pub adapter_ref: &'a ObjectRef,
    pub isolation: &'a ObjectRef,
    pub governance: &'a ObjectRef,
    pub time_facts: &'a [ObjectRef],
}

/// Authority-side facts for an authenticated root in the dispatch read set.
pub trait IrlReadView: SemanticReadView {
    fn source_facts(
        &self,
        root: &RecordVersion,
    ) -> core_types::contract::Result<crate::graph::SourceFacts>;
}

/// Both objects belong to the same outer transaction. A quarantined or
/// unbound proposal returns Fault. An authenticated but failed interpretation
/// carries its formal degraded status as the recorded product, never a z.
pub struct StagedIrl {
    pub adoption: StageAdoption,
    pub product: IrlProduct,
}

pub enum IrlProduct {
    Graph(SemanticGraphV2),
    Degraded(DegradedStatus),
}

/// Govern the adapted candidate against this transaction's actual event,
/// authorized projection and pinned registry, then prepare its canonical stage.
pub fn stage_irl(
    dispatch: &IrlDispatch<'_>,
    view: &impl IrlReadView,
    candidate: &SemanticCandidateV2,
    proposed: &SemanticGraphV2,
    registry: &ConceptRegistry,
    limits: Limits,
) -> Result<StagedIrl> {
    let b = dispatch.binding;
    let event = dispatch.event;
    if view.transaction() != Some(dispatch.transaction)
        || view.branch() != b.branch_id
        || view.base_commit() != b.base_commit
        || dispatch.facts.history_prefix != b.base_commit
        || event.record_id != b.event_id
        || event.branch_id != b.branch_id
        || dispatch.interpretation_id == event.record_id
        || proposed.registry != b.environment.registry
        || candidate.producer != dispatch.producer
        || candidate.producer_version != dispatch.producer_version
        || &candidate.actual_output != dispatch.actual_output
    {
        return Err(Fault::Binding);
    }
    let raw = view.content(event).map_err(|_| Fault::Source)?;
    if !matches!(raw, VisibleContent::Event(e, _) if e.raw_log_position == b.raw_log_position) {
        return Err(Fault::Source);
    }
    check_object(dispatch.context, dispatch.facts, limits)?;
    if dispatch.adapter.registry != proposed.registry
        || !b.environment.implementations.contains(dispatch.adapter_ref)
            && !b.read_set.contains(dispatch.adapter_ref)
    {
        return Err(Fault::Binding);
    }
    check_object(dispatch.adapter_ref, dispatch.adapter, limits)?;
    let (mapping, field) = text_adapter(dispatch.adapter, registry, view, event, limits)
        .map_err(|_| Fault::Source)?
        .ok_or(Fault::UnsupportedSource)?;
    check_binding(&candidate.binding, b, limits)?;
    check_object(dispatch.candidate, candidate, limits)?;
    check_object(&candidate.candidate_graph, proposed, limits)?;

    let governed = govern(
        dispatch,
        view,
        proposed,
        registry,
        (mapping, &field),
        limits,
    );
    let mut coverage = Vec::new();
    let (product, content, outcome, reason) = match governed {
        Ok((graph, rejected)) => {
            let content = object_ref(&graph, dispatch.representation_version, limits)?;
            for id in super::irl_graph::ids(&graph) {
                coverage.push(StageCoverage {
                    schema_version: 1,
                    target: id.into(),
                    accepted: Some(content.clone()),
                    mu: None,
                    unknown_reason: None,
                });
            }
            let partial = !rejected.is_empty() || graph.edges.len() != proposed.edges.len();
            for (id, fault) in rejected {
                coverage.push(StageCoverage {
                    schema_version: 1,
                    target: id,
                    accepted: None,
                    mu: None,
                    unknown_reason: Some(format!("{fault:?}")),
                });
            }
            let (outcome, reason) = if partial {
                (
                    StageOutcome::STAGE_OUTCOME_PARTIAL,
                    Some("candidate contains quarantined nodes or edges".into()),
                )
            } else {
                (StageOutcome::STAGE_OUTCOME_VALID, None)
            };
            (IrlProduct::Graph(graph), content, outcome, reason)
        }
        Err(fault) => {
            let status = fault.irl_status();
            let content = object_ref(&status, dispatch.representation_version, limits)?;
            (
                IrlProduct::Degraded(status),
                content,
                StageOutcome::STAGE_OUTCOME_FAILED,
                Some(format!("{fault:?}")),
            )
        }
    };
    let interpretation = RecordVersion {
        schema_version: 1,
        branch_id: b.branch_id.clone(),
        record_id: dispatch.interpretation_id.into(),
        content,
    };
    let record = StageAdoption {
        schema_version: 1,
        event: event.clone(),
        interpretation: interpretation.clone(),
        stage: InterpretationStage::INTERPRET_STAGE_IRL,
        outcome,
        binding: b.clone(),
        producer_resource: Some(b.environment.irl.clone()),
        candidate: Some(dispatch.candidate.clone()),
        actual_output: Some(dispatch.actual_output.clone()),
        context: dispatch.context.clone(),
        definitions: vec![],
        isolation: dispatch.isolation.clone(),
        governance: dispatch.governance.clone(),
        coverage,
        appraisal: None,
        reason,
        registry_after: None,
        revision: None,
        relations: None,
        transaction_id: dispatch.transaction.into(),
        replaced_targets: vec![],
        registry_before: b.environment.registry.clone(),
        staged_revisions: vec![],
    };
    let seen = StageExecution {
        binding: b,
        transaction_id: dispatch.transaction,
        event,
        interpretation: &interpretation,
        stage: InterpretationStage::INTERPRET_STAGE_IRL,
        candidate: Some(dispatch.candidate),
        actual_output: Some(dispatch.actual_output),
        isolation: dispatch.isolation,
        relation_manifest: None,
    };
    check_stage_execution(&record, &seen, limits).map_err(|_| Fault::Binding)?;
    Ok(StagedIrl {
        adoption: record,
        product,
    })
}

/// The runtime supplies only the initial IRL adoption effective for this event
/// in its ordered, committed prefix, after the outer state updates. A missing
/// record (including a rolled-back transaction) is None. A receipt cannot
/// implement this lookup. Task cancellation and delivery belong to orchestration.
pub trait CommittedIrlPrefix: SemanticReadView {
    fn initial_irl(
        &self,
        turn: &RecordVersion,
    ) -> core_types::contract::Result<Option<&StageAdoption>>;
}

/// Read the same-turn product from the prefix without repeating interpretation.
pub fn committed_irl(
    prefix: &impl CommittedIrlPrefix,
    turn: &RecordVersion,
    transaction_id: &str,
    limits: Limits,
) -> Result<Option<SemanticGraphV2>> {
    if prefix.transaction().is_some() || prefix.branch() != turn.branch_id {
        return Err(Fault::Binding);
    }
    let Some(stage) = prefix.initial_irl(turn).map_err(|_| Fault::Source)? else {
        return Ok(None);
    };
    core_types::semantic::check_semantic_evidence_message(
        StageAdoption::NAME,
        &serde_json::to_value(stage).map_err(|_| Fault::Schema)?,
        limits,
    )
    .map_err(|_| Fault::Schema)?;
    if stage.event != *turn
        || stage.transaction_id != transaction_id
        || stage.binding.event_id != turn.record_id
        || stage.stage != InterpretationStage::INTERPRET_STAGE_IRL
        || stage.revision.is_some()
    {
        return Err(Fault::Binding);
    }
    if stage.outcome == StageOutcome::STAGE_OUTCOME_FAILED {
        return Err(Fault::Source);
    }
    let VisibleContent::Adopted(version, bytes) = prefix
        .content(&stage.interpretation)
        .map_err(|_| Fault::Source)?
    else {
        return Err(Fault::Source);
    };
    if version != &stage.interpretation {
        return Err(Fault::Binding);
    }
    let graph = SemanticGraphV2::from_canonical(bytes, limits).map_err(|_| Fault::Object)?;
    check_object(&version.content, &graph, limits)?;
    if graph.registry != stage.registry_before
        || graph.canonical_bytes(limits).map_err(|_| Fault::Object)? != bytes
    {
        return Err(Fault::Object);
    }
    Ok(Some(graph))
}
