//! Trusted local inputs for P6-04. No model, outbound request or write authority.
use crate::{
    Access, Embedder, Error, Execution, FactQuery, FactView, Purpose, Result,
    source::{Record, tag},
};
use core_store::Rehearsal;
use core_types::{generated::*, scalar::DecimalI64, semantic::resolved_text_range};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub struct ModeBText {
    pub version: RecordVersion,
    pub h_position: u64,
    pub event_time_ns: DecimalI64,
    pub text: String,
    pub speaker: SpeakerAttribution,
}

pub struct ModeBInput {
    /// Preserve both the actual projection and its W19 replay/audit trace.
    pub task_facts: Execution,
    pub latest: RecordVersion,
    /// Only direct local text with committed authors, keyed by H ID (including latest).
    pub records: BTreeMap<String, ModeBText>,
}

impl FactView<'_, Rehearsal> {
    /// The current input requires a committed author. Other verified facts may
    /// remain solely in the W19 projection when they have no attributable text.
    pub fn mode_b_input(
        &self,
        latest: &RecordVersion,
        query: &FactQuery,
        access: &Access,
        embedder: Option<&mut dyn Embedder>,
    ) -> Result<ModeBInput> {
        if query.purpose != Purpose::Task || query.after.is_some() {
            return Err(Error::Invalid(
                "mode B requires a complete task query".into(),
            ));
        }
        let task_facts = self.execute(query, access, embedder)?;
        let facts = &task_facts.projection;
        if facts.truncated || !matches!(facts.status, QueryStatus::OK | QueryStatus::NO_MATCH) {
            return Err(Error::Invalid(
                "required task projection unavailable".into(),
            ));
        }
        let current = self.mode_b_text(&latest.record_id, access)?;
        if current.version != *latest {
            return Err(Error::Invalid("current input version mismatch".into()));
        }
        let mut records = BTreeMap::from([(latest.record_id.clone(), current)]);
        for item in &facts.items {
            let record = self.task_record(&item.source_id, access)?;
            if item.availability != QueryStatus::OK || item.content != record.content {
                return Err(Error::Invalid("required fact version mismatch".into()));
            }
            if let std::collections::btree_map::Entry::Vacant(entry) =
                records.entry(item.source_id.clone())
                && let Some(text) = self.authored_text(record)?
            {
                entry.insert(text);
            }
        }
        Ok(ModeBInput {
            task_facts,
            latest: latest.clone(),
            records,
        })
    }

    /// Resolve another dialogue/source ID on this same W19 view. A local
    /// authentication record attests only to the outer author of direct text.
    pub fn mode_b_text(&self, id: &str, access: &Access) -> Result<ModeBText> {
        self.authored_text(self.task_record(id, access)?)?
            .ok_or_else(|| Error::Invalid("local text with committed author required".into()))
    }

    fn task_record(&self, id: &str, access: &Access) -> Result<&Record> {
        let r = self
            .records
            .iter()
            .find(|r| r.id == id && r.reaction.is_none())
            .ok_or_else(|| Error::Invalid("missing committed text".into()))?;
        if !access.allowed
            || (!access.sources.is_empty() && !access.sources.contains(&r.event.device))
            || !r.corrected.is_empty()
            || !r.retracted.is_empty()
        {
            return Err(Error::Invalid("text unavailable for task use".into()));
        }
        Ok(r)
    }

    fn authored_text(&self, r: &Record) -> Result<Option<ModeBText>> {
        let (h_position, governance, tx) = self.reader.published_event_at(&self.prefix, &r.id)?;
        if tx.staged_h != r.event_ref {
            return Err(Error::Invalid("different H event".into()));
        }
        if r.content.object_type != tag::<UserText>()? {
            return Ok(None);
        }
        let version = RecordVersion {
            schema_version: 1,
            branch_id: tx.binding.branch_id,
            record_id: r.id.clone(),
            content: r.content.clone(),
        };
        let layer_tag = tag::<QuoteLayer>()?;
        let mut layers = tx.audit_refs.iter().filter(|r| r.object_type == layer_tag);
        let Some(layer_ref) = layers.next() else {
            return Ok(None);
        };
        let layer: QuoteLayer = self.reader.read(layer_ref)?;
        if layers.next().is_some()
            || layer.source != version
            || layer.identity_evidence.object_type != tag::<LocalUserBinding>()?
            || resolved_text_range(&r.text, &layer.visible)? != r.text
        {
            return Err(Error::Invalid("unverified outer quote layer".into()));
        }
        let user: LocalUserBinding = self.reader.read(&layer.identity_evidence)?;
        if user.principal_id != layer.principal_id
            || user.session_id != r.event.session
            || user.installation_id != r.event.device
        {
            return Err(Error::Invalid(
                "speaker evidence belongs to another source".into(),
            ));
        }
        Ok(Some(ModeBText {
            version,
            h_position,
            event_time_ns: r.event.event_time_ns,
            text: r.text.clone(),
            speaker: SpeakerAttribution {
                schema_version: 1,
                layers: vec![layer],
                governance,
            },
        }))
    }
}
