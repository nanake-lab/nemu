//! P5-43. These checks validate a proposed record. Only the outer transaction
//! can establish where it came from or publish it.
mod registry;
mod source;
mod stages;

pub use registry::{check_graph_registry, check_registry_successor, text_adapter};
pub use source::{SemanticReadView, VisibleContent, check_graph_sources, resolve_location};
pub use stages::{
    StageExecution, check_applied_rule, check_reinterpretation, check_stage_execution,
};

use crate::contract::{Limits, Result, Schema, require};
use crate::generated::*;
use serde::de::DeserializeOwned;
use serde_json::Value;

fn typed<T: DeserializeOwned>(value: &Value) -> Result<T> {
    serde_json::from_value(value.clone()).map_err(|err| err.to_string())
}

fn location(loc: &SourceLocation) -> Result<()> {
    if let Some(range) = &loc.text {
        require(range.start.0 < range.end.0, "empty source interval")?;
    }
    Ok(())
}

fn provenance(meta: &SourceMetadataV2) -> Result<()> {
    require(
        !meta.locations.is_empty() && !meta.roots.is_empty(),
        "missing provenance",
    )?;
    let branch = &meta.roots[0].branch_id;
    require(
        meta.roots.iter().all(|r| &r.branch_id == branch),
        "cross-branch roots",
    )?;
    let mut reachable: Vec<&RecordVersion> = meta.roots.iter().collect();
    for step in &meta.transformations {
        require(
            !step.inputs.is_empty()
                && step.inputs.iter().all(|input| reachable.contains(&input))
                && !reachable.contains(&&step.output)
                && &step.output.branch_id == branch,
            "broken or cyclic transform chain",
        )?;
        reachable.push(&step.output);
    }
    require(
        meta.locations
            .iter()
            .all(|loc| reachable.contains(&&loc.source)),
        "unbound location",
    )?;
    if let Some(speaker) = &meta.speaker {
        require(
            speaker
                .layers
                .first()
                .is_some_and(|layer| meta.locations.iter().any(|loc| loc.source == layer.source)),
            "speaker does not describe this source",
        )?;
    }
    Ok(())
}

fn local(name: &str, v: &Value) -> Result<()> {
    match name {
        "SourceLocation" => location(&typed(v)?),
        "SourceMetadataV2" => provenance(&typed(v)?),
        "ConceptRevision" => registry::revision(&typed(v)?),
        "ConceptRegistry" => registry::registry(&typed(v)?),
        "ClaimV2" => {
            let c: ClaimV2 = typed(v)?;
            require(
                c.reported_text.text.is_some() && c.source.locations.contains(&c.reported_text),
                "reported text lacks its source range",
            )?;
            require(
                (c.time_kind == ClaimTimeKind::CLAIM_TIME_FACT) == c.time_fact.is_some(),
                "claim time fact presence",
            )
        }
        "TextAdapterEntry" => registry::entry(&typed(v)?),
        "TextAdapterTable" => registry::table(&typed(v)?),
        "SemanticGraphV2" => registry::nodes(&typed(v)?).map(|_| ()),
        "StageCoverage" => stages::coverage(&typed(v)?),
        "StageAdoption" => stages::stage(&typed(v)?),
        "AttributeRule" => stages::rule(&typed(v)?),
        "AttributeRuleTable" => stages::rule_table(&typed(v)?),
        "ScopedVerification" => stages::verification(&typed(v)?),
        "VerificationRequirement" => {
            let r: VerificationRequirement = typed(v)?;
            require(
                r.expected != VerificationValue::VERIFY_VALUE_UNKNOWN,
                "unknown cannot establish a prerequisite",
            )
        }
        "TargetPremise" => {
            let p: TargetPremise = typed(v)?;
            require(
                !(p.occurrence_scopes.is_empty()
                    && p.truth_scopes.is_empty()
                    && p.subjective_scopes.is_empty()),
                "empty target premise",
            )
        }
        "EventAttributesRecordV2" => stages::attributes(&typed(v)?),
        _ => Ok(()),
    }
}

fn walk(schema: &Schema, name: &str, v: &Value, limits: Limits) -> Result<()> {
    let message = schema.message(name)?;
    if message.file == "speak_think.proto" {
        return super::check_speak_think_message(name, v, limits);
    }
    if message.file != "semantic_evidence.proto" {
        return Ok(());
    }
    for field in &message.fields {
        if !schema.messages.contains_key(&field.r#type) {
            continue;
        }
        if let Some(child) = v.get(&field.name) {
            if let Some(items) = child.as_array() {
                for item in items {
                    walk(schema, &field.r#type, item, limits)?;
                }
            } else {
                walk(schema, &field.r#type, child, limits)?;
            }
        }
    }
    local(name, v)
}

pub fn check_semantic_evidence_message(name: &str, v: &Value, limits: Limits) -> Result<()> {
    let schema = crate::codec_schema(name)?;
    if schema.message(name)?.file != "semantic_evidence.proto" {
        return Ok(());
    }
    let normalized = schema.normalize(name, v, limits, 0)?;
    walk(schema, name, &normalized, limits)
}

// Verify the immutable object actually supplied by the trusted reader.
fn bound_object(
    schema: &Schema,
    name: &str,
    v: &Value,
    r: &ObjectRef,
    limits: Limits,
) -> Result<()> {
    require(
        r.schema_version == 1
            && r.object_type == schema.message(name)?.tag
            && !r.resource_version.is_empty()
            && r.byte_length.0 == schema.encode_canonical(name, v, limits)?.len() as u64
            && crate::scalar::hex(&r.digest.0) == schema.digest(name, v, limits)?,
        "object binding",
    )
}
