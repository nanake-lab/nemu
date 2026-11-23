//! Resolves against a prefix or a transaction-scoped view supplied by the trusted core.
use super::{bound_object, location};
use crate::contract::{Limits, Result, require};
use crate::generated::{
    EventEnvelope, ObjectRef, RecordVersion, SemanticGraphV2, SourceLocation, SourceMetadataV2,
};
use crate::semantic::resolved_text_range;
use serde_json::Value;

pub enum VisibleContent<'a> {
    Event(&'a EventEnvelope, &'a [u8]),
    Adopted(&'a RecordVersion, &'a [u8]),
}

/// A store/transaction capability, never a serialized worker assertion. Implementors
/// must expose only permitted objects in this prefix plus this transaction's staged set.
pub trait SemanticReadView {
    fn branch(&self) -> &str;
    fn base_commit(&self) -> &str;
    fn transaction(&self) -> Option<&str>;
    /// Check the source-validity revision for this purpose, including transform receipts,
    /// per-layer principal evidence and revocations through this prefix.
    fn authorize_provenance(&self, meta: &SourceMetadataV2) -> Result<()>;
    fn content(&self, source: &RecordVersion) -> Result<VisibleContent<'_>>;
}

/// Verify every registered node's source in the same trusted read view. Each quoted
/// layer identifies its own text object; UTF-8 boundaries are checked on that field.
/// `time_facts` is supplied by the core's formal time reader for this prefix.
pub fn check_graph_sources(
    g: &SemanticGraphV2,
    view: &impl SemanticReadView,
    base: &str,
    transaction: Option<&str>,
    time_facts: &[ObjectRef],
    limits: Limits,
) -> Result<()> {
    super::check_semantic_evidence_message(
        "SemanticGraphV2",
        &serde_json::to_value(g).map_err(|e| e.to_string())?,
        limits,
    )?;
    let nodes = g
        .references
        .iter()
        .map(|n| &n.source)
        .chain(g.statements.iter().map(|n| &n.source))
        .chain(g.actions.iter().map(|n| &n.source));
    for meta in nodes {
        resolve_metadata(meta, view, base, transaction, limits)?;
    }
    for c in &g.statements {
        require(
            c.time_fact.as_ref().is_none_or(|f| time_facts.contains(f)),
            "unverified formal time fact",
        )?;
    }
    Ok(())
}

fn resolve_metadata(
    meta: &SourceMetadataV2,
    view: &impl SemanticReadView,
    base: &str,
    transaction: Option<&str>,
    limits: Limits,
) -> Result<()> {
    view.authorize_provenance(meta)?;
    for loc in &meta.locations {
        resolve_location(view, base, transaction, loc, limits)?;
    }
    if let Some(speaker) = &meta.speaker {
        for layer in &speaker.layers {
            // An attribution must identify a unique field on its source. Ranges in different
            // string fields of the same payload can have the same numeric offsets.
            let mut fields = meta.locations.iter().filter(|loc| {
                loc.source == layer.source
                    && loc.text.as_ref().is_some_and(|r| {
                        r.start.0 <= layer.visible.start.0 && layer.visible.end.0 <= r.end.0
                    })
            });
            let first = fields
                .next()
                .ok_or("quote layer lacks supporting location")?;
            require(
                fields.all(|loc| loc.path == first.path),
                "ambiguous quoted field",
            )?;
            let mut loc = first.clone();
            loc.text = Some(layer.visible.clone());
            resolve_location(view, base, transaction, &loc, limits)?;
        }
    }
    Ok(())
}

pub fn resolve_location(
    view: &impl SemanticReadView,
    base_commit: &str,
    transaction: Option<&str>,
    loc: &SourceLocation,
    limits: Limits,
) -> Result<Value> {
    require(
        view.branch() == loc.source.branch_id
            && view.base_commit() == base_commit
            && view.transaction() == transaction,
        "source outside read view",
    )?;
    let schema = crate::codec_schema("SourceLocation")?;
    schema.normalize(
        "SourceLocation",
        &serde_json::to_value(loc).map_err(|e| e.to_string())?,
        limits,
        0,
    )?;
    location(loc)?;
    let (reference, bytes): (&ObjectRef, &[u8]) = match view.content(&loc.source)? {
        VisibleContent::Event(event, bytes) => {
            require(
                event.id == loc.source.record_id
                    && event.payload_digest == event.payload_ref.digest,
                "event payload mismatch",
            )?;
            (&event.payload_ref, bytes)
        }
        VisibleContent::Adopted(record, bytes) => {
            require(record == &loc.source, "different adopted product")?;
            (&record.content, bytes)
        }
    };
    require(
        reference == &loc.source.content && reference.byte_length.0 == bytes.len() as u64,
        "source content mismatch",
    )?;
    let mut name = schema
        .messages
        .iter()
        .find(|(_, m)| m.tag == reference.object_type)
        .map(|(n, _)| n.as_str())
        .ok_or("unsupported source content type")?;
    let decoded = schema.decode_canonical(name, bytes, limits, 0)?;
    bound_object(schema, name, &decoded, reference, limits)?;
    require(
        schema.encode_canonical(name, &decoded, limits)? == bytes,
        "noncanonical source bytes",
    )?;
    let mut selected = &decoded;
    for step in &loc.path {
        let field = schema
            .message(name)?
            .fields
            .iter()
            .find(|f| f.number == step.field_number)
            .ok_or("unknown source field")?;
        selected = selected.get(&field.name).ok_or("absent source field")?;
        require(
            field.repeated == step.element.is_some(),
            "field index presence",
        )?;
        if let Some(index) = &step.element {
            let i = usize::try_from(index.0).map_err(|_| "field index overflow")?;
            selected = selected.get(i).ok_or("field index out of bounds")?;
        }
        name = &field.r#type;
    }
    if let Some(range) = &loc.text {
        require(name == "string", "range requires a UTF-8 field")?;
        return Ok(Value::from(resolved_text_range(
            selected.as_str().ok_or("text field")?,
            range,
        )?));
    }
    Ok(selected.clone())
}
