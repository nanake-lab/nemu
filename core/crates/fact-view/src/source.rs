//! Sole authority adapter. No direct SQLite/file access to the core store.
use crate::{Error, Result};
use core_store::{LineageKind, Reader};
use core_types::{WireMessage, contract::Schema, generated::*, scalar::hex};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

#[derive(Clone)]
pub(crate) struct Record {
    pub id: String,
    pub event: EventEnvelope,
    pub event_ref: ObjectRef,
    pub content: ObjectRef,
    pub reaction: Option<crate::memory::ReactionOrigin>,
    pub text: String,
    pub entities: BTreeSet<String>,
    pub mentions: Vec<(String, String)>,
    pub retracted: Vec<String>,
    pub corrected: Vec<String>,
}

pub(crate) fn tag<T: WireMessage>() -> Result<String> {
    // Built-in descriptors are immutable for this binary. This cache cannot
    // change the query plan or substitute for authority-object verification.
    static SCHEMA: OnceLock<std::result::Result<Schema, String>> = OnceLock::new();
    let schema = SCHEMA
        .get_or_init(Schema::builtin)
        .as_ref()
        .map_err(|e| Error::Invalid(e.clone()))?;
    Ok(schema.message(T::NAME)?.tag.clone())
}

pub(crate) fn payload_tag(name: &str) -> Result<Option<String>> {
    Ok(match name {
        "user_text" => Some(tag::<UserText>()?),
        "user_utterance" => Some(tag::<UserUtterance>()?),
        "touch" => Some(tag::<TouchObservation>()?),
        "desktop" => Some(tag::<DesktopObservation>()?),
        "codex_session" => Some(tag::<CodexSessionObservation>()?),
        "speech_delivery" => Some(tag::<SpeechDelivery>()?),
        "body_action" => Some(tag::<BodyActionResult>()?),
        "terminal" => Some(tag::<TerminalMaintenance>()?),
        "nature_appraisal" | "past_reaction:nature_appraisal" => {
            Some(tag::<NatureAppraisalRecord>()?)
        }
        "situation_match" | "past_reaction:situation_match" => Some(tag::<SituationMatchRecord>()?),
        "tendency_candidate" | "past_reaction:tendency_candidate" => {
            Some(tag::<TendencyCandidateRecord>()?)
        }
        "teaching_annotation" | "past_reaction:teaching_annotation" => {
            Some(tag::<TeachingAnnotation>()?)
        }
        "lie_annotation" | "past_reaction:lie_annotation" => Some(tag::<LieAnnotation>()?),
        "same_matter_annotation" | "past_reaction:same_matter_annotation" => {
            Some(tag::<SameMatterAnnotation>()?)
        }
        _ => None,
    })
}
pub(crate) fn payload_name(r: &ObjectRef) -> Result<String> {
    for name in [
        "user_text",
        "user_utterance",
        "touch",
        "desktop",
        "codex_session",
        "speech_delivery",
        "body_action",
        "terminal",
    ] {
        if payload_tag(name)?.as_deref() == Some(&r.object_type) {
            return Ok(name.into());
        }
    }
    Err(Error::Invalid("unsupported fact type".into()))
}

/// Positive allowlist. Never traverse arbitrary object closure into appraisal,
/// situation matches, candidates, annotations, state, ledger or semantic graphs.
pub(crate) fn text<L: LineageKind>(reader: &Reader<L>, r: &ObjectRef) -> Result<Option<String>> {
    if r.object_type == tag::<UserText>()? {
        return Ok(Some(reader.read::<UserText>(r)?.text));
    }
    if r.object_type == tag::<UserUtterance>()? {
        return Ok(Some(reader.read::<UserUtterance>(r)?.text));
    }
    macro_rules! raw {
        ($($t:ty),*) => {$(if r.object_type == tag::<$t>()? { return Ok(Some(serde_json::to_string(&reader.read::<$t>(r)?)?)); })*};
    }
    raw!(
        TouchObservation,
        DesktopObservation,
        CodexSessionObservation,
        SpeechDelivery,
        BodyActionResult,
        TerminalMaintenance
    );
    Ok(None)
}

pub(crate) fn records<L: LineageKind>(
    reader: &Reader<L>,
    transactions: &[CoreTransactionV2],
) -> Result<Vec<Record>> {
    let mut records = Vec::new();
    let mut aliases: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut retractions = Vec::new();
    let mut corrections = Vec::new();
    for tx in transactions {
        let event: EventEnvelope = reader.read(&tx.staged_h)?;
        for key in [
            event.id.clone(),
            hex(&tx.staged_h.digest.0),
            hex(&event.payload_ref.digest.0),
        ] {
            aliases.entry(key).or_default().insert(event.id.clone());
        }
        if event.payload_ref.object_type == tag::<UserRetraction>()? {
            let r: UserRetraction = reader.read(&event.payload_ref)?;
            retractions.push((event.id.clone(), r.target_refs));
            // Control observations (including the note) cannot reintroduce the
            // forgotten text through any retrieval channel.
            continue;
        }
        if !event.correction_of.is_empty() {
            corrections.push((event.id.clone(), event.correction_of.clone()));
        }
        let Some(original) = text(reader, &event.payload_ref)? else {
            continue;
        };
        let mut entities = BTreeSet::new();
        let mut mentions = Vec::new();
        for r in &tx.accepted_results {
            if let Some((event_id, _, _)) = crate::memory::reaction(reader, r)? {
                aliases
                    .entry(hex(&r.digest.0))
                    .or_default()
                    .insert(event_id);
                continue;
            }
            if r.object_type != tag::<SemanticGraph>()? {
                continue;
            }
            let graph: SemanticGraph = reader.read(r)?;
            aliases
                .entry(hex(&r.digest.0))
                .or_default()
                .insert(event.id.clone());
            for node in graph.refs {
                // Use only accepted identity references, never appraisal,
                // polarity, match strength, candidate text or annotations.
                if !node.metadata.source_refs.contains(&event.id) {
                    continue;
                }
                aliases.entry(node.id).or_default().insert(event.id.clone());
                if let Some(c) = node.referent {
                    if let Some(mention) = text(reader, &node.metadata.source_span)?
                        && !mention.is_empty()
                        && original.contains(&mention)
                    {
                        mentions.push((c.concept_id.clone(), mention));
                    }
                    entities.insert(c.concept_id);
                }
            }
        }
        records.push(Record {
            id: event.id.clone(),
            content: event.payload_ref.clone(),
            reaction: None,
            event,
            event_ref: tx.staged_h.clone(),
            text: original,
            entities,
            mentions,
            retracted: vec![],
            corrected: vec![],
        });
    }
    // Reverse correction ancestry. Retracting a replacement also suppresses
    // its predecessors; a later correction must not resurrect withdrawn text.
    let mut ancestry: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (id, targets) in &corrections {
        let mut set = BTreeSet::new();
        for target in targets {
            if let Some(ids) = aliases.get(target) {
                for old in ids {
                    set.insert(old.clone());
                    if let Some(parents) = ancestry.get(old) {
                        set.extend(parents.clone());
                    }
                }
            }
        }
        ancestry.insert(id.clone(), set);
    }
    for (id, targets) in corrections.iter().chain(&retractions) {
        let is_retraction = retractions.iter().any(|(r, _)| r == id);
        let mut affected = BTreeSet::new();
        for target in targets {
            if let Some(ids) = aliases.get(target) {
                affected.extend(ids.clone());
                for old in ids {
                    if let Some(parents) = ancestry.get(old) {
                        affected.extend(parents.clone());
                    }
                }
            }
        }
        // A withdrawal of the original also covers all its replacements.
        if is_retraction {
            for (replacement, parents) in &ancestry {
                if parents.iter().any(|p| affected.contains(p)) {
                    affected.insert(replacement.clone());
                }
            }
        }
        for record in &mut records {
            if affected.contains(&record.event.id) {
                if is_retraction {
                    record.retracted.push(id.clone());
                } else {
                    record.corrected.push(id.clone());
                }
            }
        }
    }
    if transactions.is_empty() {
        return Err(Error::Invalid("empty accepted prefix".into()));
    }
    crate::memory::append_reactions(reader, transactions, &mut records)?;
    Ok(records)
}
