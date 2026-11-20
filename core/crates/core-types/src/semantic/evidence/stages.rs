use super::{bound_object, check_semantic_evidence_message};
use crate::contract::{Limits, Result, require};
use crate::generated::*;
use std::collections::BTreeSet;

pub(super) fn coverage(c: &StageCoverage) -> Result<()> {
    require(
        usize::from(c.accepted.is_some())
            + usize::from(c.mu.is_some())
            + usize::from(c.unknown_reason.is_some())
            == 1,
        "ambiguous coverage result",
    )
}

pub(super) fn stage(s: &StageAdoption) -> Result<()> {
    use InterpretationStage::*;
    use StageOutcome::*;
    let b = &s.binding;
    require(
        s.event.branch_id == b.branch_id
            && s.interpretation.branch_id == b.branch_id
            && s.interpretation != s.event,
        "interpretation endpoints",
    )?;
    require(
        s.registry_before == b.environment.registry
            && s.context.digest == b.c_core_digest
            && b.read_set.contains(&s.context)
            && s.definitions.iter().all(|d| b.read_set.contains(d)),
        "unbound read set",
    )?;
    let resource = match s.stage {
        INTERPRET_STAGE_IRL => Some(&b.environment.irl),
        INTERPRET_STAGE_A => b.environment.nature_appraiser.as_ref(),
        INTERPRET_STAGE_M => b.environment.situation_matcher.as_ref(),
    };
    require(
        resource == s.producer_resource.as_ref()
            && (resource.is_some() || s.outcome == STAGE_OUTCOME_FAILED),
        "stage producer mismatch",
    )?;
    require(
        s.stage == INTERPRET_STAGE_M || s.definitions.is_empty(),
        "unexpected concept definitions",
    )?;
    require(
        s.registry_after.is_none() == s.staged_revisions.is_empty()
            && (s.registry_after.is_none() || s.stage == INTERPRET_STAGE_IRL),
        "registry change outside IRL",
    )?;
    require(
        (s.outcome != STAGE_OUTCOME_VALID) == s.reason.is_some(),
        "stage reason",
    )?;
    if s.outcome == STAGE_OUTCOME_FAILED {
        require(
            s.appraisal.is_none()
                && s.coverage.is_empty()
                && s.registry_after.is_none()
                && s.revision.is_none(),
            "failed stage published results",
        )?;
    } else {
        require(
            s.actual_output.is_some() && s.candidate.is_some(),
            "missing stage input or output",
        )?;
        match s.stage {
            INTERPRET_STAGE_A => require(
                s.outcome == STAGE_OUTCOME_VALID && s.appraisal.is_some() && s.coverage.is_empty(),
                "A requires a complete vector",
            )?,
            INTERPRET_STAGE_IRL | INTERPRET_STAGE_M => {
                require(
                    s.appraisal.is_none()
                        && (s.outcome != STAGE_OUTCOME_PARTIAL || !s.coverage.is_empty()),
                    "partial stage lacks coverage",
                )?;
                for c in &s.coverage {
                    require(
                        if s.stage == INTERPRET_STAGE_IRL {
                            c.mu.is_none()
                        } else {
                            c.accepted.is_none()
                        },
                        "coverage from wrong stage",
                    )?;
                }
            }
        }
    }
    if let Some(edge) = &s.revision {
        require(
            edge.kind == RelationFact::REL_FACT_REINTERPRETATION
                && edge.origin == s.interpretation
                && edge.target != edge.origin
                && edge.target.branch_id == b.branch_id
                && !s.replaced_targets.is_empty(),
            "invalid reinterpretation",
        )?;
    } else {
        require(
            s.replaced_targets.is_empty(),
            "replacement without predecessor",
        )?;
    }
    if let Some(q) = &s.relations {
        let endpoint = s.revision.as_ref().map(|e| &e.target).unwrap_or(&s.event);
        require(
            q.result.endpoint == *endpoint
                && q.result.frontier == b.base_commit
                && s.revision
                    .as_ref()
                    .is_none_or(|e| e.purpose == q.result.purpose),
            "relation receipt scope",
        )?;
    }
    Ok(())
}

/// Trusted facts from the actual outer transaction and its isolated dispatch.
pub struct StageExecution<'a> {
    pub binding: &'a ProposalBindingV2,
    pub transaction_id: &'a str,
    pub event: &'a RecordVersion,
    pub interpretation: &'a RecordVersion,
    pub stage: InterpretationStage,
    pub candidate: Option<&'a ObjectRef>,
    pub actual_output: Option<&'a ObjectRef>,
    pub isolation: &'a ObjectRef,
    /// Manifest fixed independently at the requested endpoint, purpose and frontier.
    pub relation_manifest: Option<&'a ObjectRef>,
}

pub fn check_stage_execution(
    s: &StageAdoption,
    seen: &StageExecution<'_>,
    limits: Limits,
) -> Result<()> {
    use crate::WireMessage;
    check_semantic_evidence_message(
        "StageAdoption",
        &serde_json::to_value(s).map_err(|e| e.to_string())?,
        limits,
    )?;
    require(
        s.binding.canonical_bytes(limits)? == seen.binding.canonical_bytes(limits)?
            && s.transaction_id == seen.transaction_id
            && &s.event == seen.event
            && &s.interpretation == seen.interpretation
            && s.stage == seen.stage
            && s.candidate.as_ref() == seen.candidate
            && s.actual_output.as_ref() == seen.actual_output
            && &s.isolation == seen.isolation
            && s.relations.as_ref().map(|q| &q.manifest) == seen.relation_manifest,
        "stage differs from trusted execution",
    )
}

/// Prior is resolved from the trusted prefix. First-time coverage need not
/// have had a value in the prior result; it must be named in the new result.
pub fn check_reinterpretation(
    s: &StageAdoption,
    prior: &StageAdoption,
    limits: Limits,
) -> Result<()> {
    for record in [s, prior] {
        check_semantic_evidence_message(
            "StageAdoption",
            &serde_json::to_value(record).map_err(|e| e.to_string())?,
            limits,
        )?;
    }
    let edge = s.revision.as_ref().ok_or("missing reinterpretation edge")?;
    require(
        edge.target == prior.interpretation && s.event == prior.event && s.stage == prior.stage,
        "different event or stage",
    )?;
    require(
        s.replaced_targets.iter().all(|target| match s.stage {
            InterpretationStage::INTERPRET_STAGE_A => {
                target == "appraisal" && s.appraisal.is_some()
            }
            _ => s.coverage.iter().any(|c| c.target == *target),
        }),
        "replacement outside coverage",
    )
}

pub(super) fn rule(r: &AttributeRule) -> Result<()> {
    use AttributeSource::*;
    use EvidenceAccess::*;
    let partition = match r.source {
        ATTR_SOURCE_HUMAN_RESPONSE | ATTR_SOURCE_DIRECTED_TOUCH | ATTR_SOURCE_ABSENCE => {
            (true, false, EVIDENCE_ACCESS_TARGET_DEPENDENT)
        }
        ATTR_SOURCE_DESKTOP => (false, false, EVIDENCE_ACCESS_TARGET_DEPENDENT),
        ATTR_SOURCE_SELF_RESULT => (false, false, EVIDENCE_ACCESS_COMPARISON_REQUIRED),
        ATTR_SOURCE_INTERNAL => (false, true, EVIDENCE_ACCESS_NONE),
        ATTR_SOURCE_MAINTENANCE => (false, false, EVIDENCE_ACCESS_NONE),
    };
    require(
        (r.exogenous, r.self_narrative, r.growth_access) == partition,
        "source partition",
    )?;
    // Prevent relabeling an established terminal payload to change learning eligibility.
    let source = match r.payload_type.as_str() {
        "echo.user-text" | "echo.user-utterance" => Some(ATTR_SOURCE_HUMAN_RESPONSE),
        "echo.touch-v2" => Some(ATTR_SOURCE_DIRECTED_TOUCH),
        "echo.desktop-observation" => Some(ATTR_SOURCE_DESKTOP),
        "echo.body-action-result" => Some(ATTR_SOURCE_SELF_RESULT),
        "echo.terminal-maintenance" | "echo.system-maintenance-record" => {
            Some(ATTR_SOURCE_MAINTENANCE)
        }
        "echo.internal-activity-record" | "echo.internal-activity-record-v2" => {
            Some(ATTR_SOURCE_INTERNAL)
        }
        _ => None,
    };
    require(
        source.is_none_or(|kind| r.source == kind),
        "source contradicts payload type",
    )?;
    require(
        r.event_kind != EventKind::MAINTENANCE || r.source == ATTR_SOURCE_MAINTENANCE,
        "maintenance relabeled",
    )?;
    require(
        r.event_kind != EventKind::EXECUTION_RESULT || r.source == ATTR_SOURCE_SELF_RESULT,
        "self result relabeled",
    )
}

pub(super) fn rule_table(t: &AttributeRuleTable) -> Result<()> {
    let mut keys = BTreeSet::new();
    require(
        !t.rules.is_empty()
            && t.rules.iter().all(|r| {
                keys.insert((
                    u32::from(r.source),
                    u32::from(r.event_kind),
                    &r.payload_type,
                ))
            }),
        "ambiguous or empty rule table",
    )
}

pub(super) fn verification(v: &ScopedVerification) -> Result<()> {
    match v.value {
        VerificationValue::VERIFY_VALUE_UNKNOWN => {
            require(v.reason.is_some(), "unknown without reason")
        }
        _ => require(!v.evidence.is_empty(), "judgment without evidence"),
    }
}

pub(super) fn attributes(a: &EventAttributesRecordV2) -> Result<()> {
    use AttributeSource::*;
    let r = &a.applied;
    require(
        a.event_id == a.event.record_id
            && r.payload_type == a.event.content.object_type
            && a.rule_table.object_type == "echo.attribute-rule-table"
            && a.rule_table_version == a.rule_table.resource_version
            && a.exogenous == r.exogenous
            && a.self_narrative == r.self_narrative
            && a.c_unit.is_some() == r.exogenous,
        "event attributes disagree with source rule",
    )?;
    require(
        a.inputs.contains(&a.event)
            && a.inputs.iter().all(|e| e.branch_id == a.event.branch_id)
            && a.source.locations.iter().any(|l| l.source == a.event),
        "attribute inputs unbound",
    )?;
    for e in [&a.c_unit, &a.directed_to_n, &a.cause]
        .into_iter()
        .flatten()
    {
        require(a.inputs.contains(e), "missing classification evidence")?;
    }
    match r.source {
        ATTR_SOURCE_HUMAN_RESPONSE | ATTR_SOURCE_DIRECTED_TOUCH => require(
            a.directed_to_n.is_some()
                && (r.source != ATTR_SOURCE_HUMAN_RESPONSE || a.source.speaker.is_some())
                && a.person_mapping.is_some(),
            "unverified actor or direction",
        )?,
        ATTR_SOURCE_SELF_RESULT => require(a.cause.is_some(), "unknown action origin")?,
        _ => {}
    }
    if matches!(r.source, ATTR_SOURCE_ABSENCE | ATTR_SOURCE_DIRECTED_TOUCH) {
        require(
            a.episode_phase_fact.is_some(),
            "missing episode identity or phase",
        )?;
    }
    for axis in [&a.occurred, &a.truth, &a.subjective] {
        require(!axis.is_empty(), "unassessed verification axis")?;
        for item in axis {
            require(
                a.inputs.contains(&item.scope.source)
                    && item.evidence.iter().all(|e| a.inputs.contains(e)),
                "unbound verification scope or evidence",
            )?;
        }
    }
    require(
        a.occurred
            .iter()
            .chain(&a.truth)
            .all(|v| v.experiencer.is_none())
            && a.subjective.iter().all(|v| {
                v.value != VerificationValue::VERIFY_VALUE_TRUE || v.experiencer.is_some()
            }),
        "verification experiencer",
    )?;
    for t in &a.targets {
        for (requirements, assessed) in [
            (&t.occurrence_scopes, &a.occurred),
            (&t.truth_scopes, &a.truth),
            (&t.subjective_scopes, &a.subjective),
        ] {
            require(
                requirements
                    .iter()
                    .all(|p| assessed.iter().any(|v| v.scope == p.scope)),
                "target premise lacks scoped observation",
            )?;
        }
    }
    Ok(())
}

/// Structural checks plus a match against the independently resolved table
/// and the event actually being classified. Caller authenticates input facts.
pub fn check_applied_rule(
    a: &EventAttributesRecordV2,
    t: &AttributeRuleTable,
    event: &EventEnvelope,
    source: AttributeSource,
    canonical_source_key: &str,
    observed_unit: Option<&RecordVersion>,
    limits: Limits,
) -> Result<()> {
    let value = serde_json::to_value(t).map_err(|e| e.to_string())?;
    check_semantic_evidence_message("AttributeRuleTable", &value, limits)?;
    check_semantic_evidence_message(
        "EventAttributesRecordV2",
        &serde_json::to_value(a).map_err(|e| e.to_string())?,
        limits,
    )?;
    bound_object(
        crate::codec_schema("AttributeRuleTable")?,
        "AttributeRuleTable",
        &value,
        &a.rule_table,
        limits,
    )?;
    require(
        t.rules.contains(&a.applied)
            && event.kind == a.applied.event_kind
            && event.id == a.event_id
            && event.payload_ref == a.event.content
            && event.payload_digest == a.event.content.digest
            && source == a.applied.source
            && canonical_source_key == a.source_key
            && a.c_unit.as_ref() == observed_unit,
        "rule or source event mismatch",
    )
}
