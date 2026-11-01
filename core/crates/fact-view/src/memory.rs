//! Local, explicit presentation of historical interpretations (V2-88).
//! These are not IPC types; authority references retain their generated types.
use crate::{Result, source};
use core_store::{LineageKind, Reader};
use core_types::{
    generated::*,
    scalar::{DecimalI64, hex},
};

pub const PAST_REACTION_MARKER: &str = "这是她当时的反应";

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct MemoryEntry {
    pub content: ObjectRef,
    /// The event being recalled, rather than the interpretation's record ID.
    pub event_id: String,
    pub at_ns: DecimalI64,
    /// A later interpretation keeps its separate recording time.
    pub recorded_at_ns: DecimalI64,
    pub kind: String,
    pub source: String,
    pub original: String,
    pub marker: Option<String>,
}
impl MemoryEntry {
    pub fn marked_text(&self) -> String {
        match &self.marker {
            Some(marker) => format!("【{marker}】\n{}", self.original),
            None => self.original.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ReactionOrigin {
    pub kind: &'static str,
    pub recorded_event: EventEnvelope,
    pub recorded_event_ref: ObjectRef,
}

/// Read only the six interpretation types named by V2-88, never an arbitrary
/// reference closure. Binding/state references are verification metadata, not
/// memory prose. All semantic fields are preserved without model summarization.
pub(crate) fn reaction<L: LineageKind>(
    reader: &Reader<L>,
    reference: &ObjectRef,
) -> Result<Option<(String, &'static str, String)>> {
    macro_rules! read {
        ($($ty:ty => $kind:literal),* $(,)?) => {$(
            if reference.object_type == source::tag::<$ty>()? {
                let record: $ty = reader.read(reference)?;
                let mut value = serde_json::to_value(&record)?;
                value.as_object_mut().expect("generated message").remove("binding");
                return Ok(Some((record.event_id, $kind, serde_json::to_string(&value)?)));
            }
        )*};
    }
    read!(
        NatureAppraisalRecord => "nature_appraisal",
        SituationMatchRecord => "situation_match",
        TendencyCandidateRecord => "tendency_candidate",
        TeachingAnnotation => "teaching_annotation",
        LieAnnotation => "lie_annotation",
        SameMatterAnnotation => "same_matter_annotation",
    );
    Ok(None)
}

pub(crate) fn append_reactions<L: LineageKind>(
    reader: &Reader<L>,
    transactions: &[CoreTransactionV2],
    records: &mut Vec<source::Record>,
) -> Result<()> {
    let events = transactions
        .iter()
        .map(|tx| {
            Ok((
                reader.read::<EventEnvelope>(&tx.staged_h)?,
                tx.staged_h.clone(),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut seen = std::collections::BTreeSet::new();
    for (tx, (recorded_event, recorded_event_ref)) in transactions.iter().zip(&events) {
        for reference in &tx.accepted_results {
            let Some((event_id, kind, text)) = reaction(reader, reference)? else {
                continue;
            };
            if !seen.insert(hex(&reference.digest.0)) {
                continue;
            }
            // Backfill can refer to an older event. Its permissions, time and
            // withdrawals come from that event, never from the backfill event.
            let Some(original) = records.iter().find(|r| r.event.id == event_id).cloned() else {
                // No allowlisted source in this prefix: fail closed.
                continue;
            };
            let mut record = original;
            record.id = format!("{event_id}/reaction/{}", hex(&reference.digest.0));
            record.content = reference.clone();
            record.text = text;
            record.reaction = Some(ReactionOrigin {
                kind,
                recorded_event: recorded_event.clone(),
                recorded_event_ref: recorded_event_ref.clone(),
            });
            records.push(record);
        }
    }
    Ok(())
}

impl source::Record {
    pub(crate) fn memory(&self) -> Result<MemoryEntry> {
        Ok(MemoryEntry {
            content: self.content.clone(),
            event_id: self.event.id.clone(),
            at_ns: self.event.event_time_ns,
            recorded_at_ns: DecimalI64(
                self.reaction
                    .as_ref()
                    .map_or(self.event.event_time_ns.0, |r| {
                        r.recorded_event.event_time_ns.0
                    }),
            ),
            kind: match &self.reaction {
                Some(r) => format!("past_reaction:{}", r.kind),
                None => source::payload_name(&self.content)?,
            },
            source: self.event.device.clone(),
            original: self.text.clone(),
            marker: self.reaction.as_ref().map(|_| PAST_REACTION_MARKER.into()),
        })
    }

    pub(crate) fn verified<L: LineageKind>(&self, reader: &Reader<L>) -> bool {
        if reader.read::<EventEnvelope>(&self.event_ref).as_ref().ok() != Some(&self.event) {
            return false;
        }
        match &self.reaction {
            Some(origin) => {
                reader
                    .read::<EventEnvelope>(&origin.recorded_event_ref)
                    .as_ref()
                    .ok()
                    == Some(&origin.recorded_event)
                    && matches!(reaction(reader, &self.content), Ok(Some((event_id, kind, text)))
                        if event_id == self.event.id && kind == origin.kind && text == self.text)
            }
            None => {
                matches!(source::text(reader, &self.content), Ok(Some(text)) if text == self.text)
            }
        }
    }
}
