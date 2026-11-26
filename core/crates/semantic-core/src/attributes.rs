//! Deterministic event attributes (ADR 0048 G05, V2-71/72/77, EV2/3/9).
use crate::{Fault, Result, aliases::Aliases, graph::epistemic, irl::IrlReadView};
use core_types::{
    contract::{self, Limits},
    generated::*,
    semantic::{VisibleContent, check_applied_rule, resolve_location},
};

/// The core authenticates the account and supplies the aliases from the named
/// mapping resource. Non-person sources have their own governed key.
pub enum SourceIdentity<'a> {
    Account {
        account: &'a str,
        aliases: &'a Aliases,
        mapping: &'a ObjectRef,
    },
    Other(&'a str),
}

/// Authority-side observations at this event and prefix. The owner verifies
/// episode identity, causal and direction facts, each independent assessment,
/// and fixes target requirements before evaluating outcomes. No worker DTO or
/// user-provided text can serve as this capability.
pub struct AttributeFacts<'a> {
    pub source: AttributeSource,
    pub identity: SourceIdentity<'a>,
    pub metadata: &'a SourceMetadataV2,
    pub table: &'a AttributeRuleTable,
    pub pinned_table: &'a ObjectRef,
    pub governance: &'a ObjectRef,
    pub inputs: &'a [RecordVersion],
    pub unit: Option<&'a RecordVersion>,
    pub directed_to_n: Option<&'a RecordVersion>,
    pub cause: Option<&'a RecordVersion>,
    pub episode_phase: Option<&'a ObjectRef>,
    pub duration: Option<&'a ObjectRef>,
    pub occurred: &'a [ScopedVerification],
    pub truth: &'a [ScopedVerification],
    pub subjective: &'a [ScopedVerification],
    pub targets: &'a [TargetPremise],
}

pub trait AttributeView: IrlReadView {
    fn attribute_facts(&self, event: &RecordVersion) -> contract::Result<AttributeFacts<'_>>;
}

/// Prepare the V2 record for the outer transaction. The rule selector uses all
/// three governed fields, including for events with identical payload bytes.
pub fn stage_attributes(
    view: &impl AttributeView,
    event: &RecordVersion,
    limits: Limits,
) -> Result<EventAttributesRecordV2> {
    let VisibleContent::Event(envelope, _) = view.content(event).map_err(|_| Fault::Source)? else {
        return Err(Fault::Source);
    };
    let facts = view.attribute_facts(event).map_err(|_| Fault::Source)?;
    let rule = facts
        .table
        .rules
        .iter()
        .find(|r| {
            r.source == facts.source
                && r.event_kind == envelope.kind
                && r.payload_type == event.content.object_type
        })
        .ok_or(Fault::UnsupportedSource)?;
    view.authorize_provenance(facts.metadata)
        .map_err(|_| Fault::Source)?;
    for root in &facts.metadata.roots {
        let origin = view.source_facts(root).map_err(|_| Fault::Source)?;
        epistemic(
            facts.metadata.epistemic_type,
            facts.metadata.derivation,
            &origin,
        )?;
    }
    let resolve = |loc: &SourceLocation| {
        resolve_location(view, view.base_commit(), view.transaction(), loc, limits)
            .map(|_| ())
            .map_err(|_| Fault::Source)
    };
    for input in facts.inputs {
        resolve(&SourceLocation {
            schema_version: 1,
            source: input.clone(),
            path: vec![],
            text: None,
        })?;
    }
    for loc in &facts.metadata.locations {
        resolve(loc)?;
    }
    for observation in facts
        .occurred
        .iter()
        .chain(facts.truth)
        .chain(facts.subjective)
    {
        resolve(&observation.scope)?;
    }
    let (source_key, mapping) = match facts.identity {
        SourceIdentity::Account {
            account,
            aliases,
            mapping,
        } => (aliases.representative(account)?, Some(mapping.clone())),
        SourceIdentity::Other(key) => (key, None),
    };
    let attributes = EventAttributesRecordV2 {
        schema_version: 1,
        event_id: event.record_id.clone(),
        event: event.clone(),
        rule_table_version: facts.pinned_table.resource_version.clone(),
        rule_table: facts.pinned_table.clone(),
        applied: rule.clone(),
        source_key: source_key.into(),
        exogenous: rule.exogenous,
        self_narrative: rule.self_narrative,
        source: facts.metadata.clone(),
        governance: facts.governance.clone(),
        c_unit: facts.unit.cloned(),
        directed_to_n: facts.directed_to_n.cloned(),
        person_mapping: mapping,
        cause: facts.cause.cloned(),
        episode_phase_fact: facts.episode_phase.cloned(),
        duration_fact: facts.duration.cloned(),
        occurred: facts.occurred.to_vec(),
        truth: facts.truth.to_vec(),
        subjective: facts.subjective.to_vec(),
        inputs: facts.inputs.to_vec(),
        targets: facts.targets.to_vec(),
    };
    check_applied_rule(
        &attributes,
        facts.table,
        envelope,
        facts.source,
        source_key,
        facts.unit,
        limits,
    )
    .map_err(|_| Fault::Source)?;
    Ok(attributes)
}
