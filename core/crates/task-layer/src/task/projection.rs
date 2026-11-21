use super::speech::{self, MissingHint, Point, SpeechSettings, VisibleQuote};
use crate::cause_projection::{
    self as cp, AssemblyInput, Delivered, Outcome, RecordId, RequestCodec,
};
use core_types::generated::QuoteLayer;
use core_types::generated::{Lang, ObjectRef, QueryStatus, RecordVersion, TaskInput};
use core_types::{WireMessage, contract::Limits};
use std::collections::BTreeMap;

/// Selected by P6-04 from the signed P5-17 bank. Exact selected bytes, the
/// immutable modulation reference, presentation and ordering are replay inputs.
#[derive(Debug, Clone, PartialEq)]
pub struct Samples {
    pub bank: ObjectRef,
    pub modulation: ObjectRef,
    pub metric: ObjectRef,
    pub selection_rule: ObjectRef,
    pub presentation: ObjectRef,
    pub k: usize,
    pub language: Lang,
    pub ordered_ids: Vec<String>,
    pub rendered: String,
}

pub trait SampleSelector {
    /// The immutable modulation resource identifies the actual vector; zero
    /// selects neutral samples. The selector resolves only a signed bank.
    fn select(&self, modulation: &ObjectRef, language: Lang, k: usize) -> Result<Samples, String>;
}

/// A local snapshot adapter resolves content bytes and speaker evidence before
/// assembly. It must prove the complete frontier and provide only authorized records.
pub struct SampleRequest<'a> {
    pub selector: &'a dyn SampleSelector,
    pub modulation: &'a ObjectRef,
    pub language: Lang,
    pub k: usize,
}
pub struct CloudMaterial<'a> {
    pub prefix: String,
    pub snapshot_id: String,
    pub latest: RecordVersion,
    pub input: AssemblyInput,
    pub settings: SpeechSettings,
    pub samples: Option<SampleRequest<'a>>,
    /// Historical IDs for which the trusted reader identified a material gap.
    pub gaps: Vec<String>,
}

pub struct CloudRequest {
    pub(super) bytes: Vec<u8>,
    pub binding: cp::Binding,
    pub note: cp::SentNote,
    settings: SpeechSettings,
    pub samples: Option<Samples>,
    task: TaskInput,
    latest: RecordVersion,
    visible: BTreeMap<String, (Vec<u8>, Vec<u8>, std::ops::Range<usize>)>,
}

/// Only a trusted governance adapter resolves these layers. The raw model
/// cannot contribute an identity assertion or mark its own text as supported.
pub trait Governance {
    fn layers(&self, id: &str) -> Option<(&RecordVersion, &[QuoteLayer])>;
    fn self_id(&self) -> &str;
    fn supported(&self, point: &Point, evidence: &[&VisibleQuote<'_>]) -> bool;
}
impl CloudRequest {
    pub(super) fn matches(&self, input: &TaskInput, branch: &str) -> bool {
        self.task == *input && self.latest.branch_id == branch
    }
    pub(super) fn latest(&self) -> &RecordVersion {
        &self.latest
    }
    pub fn checked_text(
        &self,
        outline: &str,
        governance: &dyn Governance,
    ) -> Result<String, String> {
        let mut quotes = BTreeMap::new();
        for (id, (version, body, range)) in &self.visible {
            let Some((record, layers)) = governance.layers(id) else {
                continue;
            };
            if record.branch_id != self.latest.branch_id
                || record.record_id != *id
                || record.content.digest.0 != *version
            {
                continue;
            }
            let Ok(text) = std::str::from_utf8(body) else {
                continue;
            };
            quotes.insert(
                id.clone(),
                layers
                    .iter()
                    .filter_map(|layer| speech::visible_quote(record, layer, text, range.clone()))
                    .collect(),
            );
        }
        speech::join_checked(
            outline,
            self.settings,
            governance.self_id(),
            &quotes,
            |p, e| governance.supported(p, e),
        )
    }
}

/// The P6-04 renderer serializes the complete vendor request, including these
/// format instructions. Its meter must bound the actual model and wrapping (D5).
pub trait SpeechCodec {
    fn encode(
        &self,
        plan: &cp::RequestPlan,
        records: &cp::RecordTable,
        instructions: &str,
        hints: &[String],
    ) -> Vec<u8>;
    fn measure(&self, bytes: &[u8]) -> u64;
}
struct MeasuredSpeech<'a, C> {
    codec: &'a C,
    settings: SpeechSettings,
    gaps: &'a [String],
}
impl<C: SpeechCodec> RequestCodec for MeasuredSpeech<'_, C> {
    fn encode(&self, plan: &cp::RequestPlan, records: &cp::RecordTable) -> Vec<u8> {
        let ids: std::collections::BTreeSet<_> = plan
            .dialogue
            .iter()
            .map(|d| &d.id)
            .chain(plan.experience.iter().map(|e| &e.id))
            .chain(std::iter::once(&plan.latest))
            .collect();
        let selected = self
            .gaps
            .iter()
            .filter(|id| ids.contains(&RecordId(id.as_bytes().to_vec())))
            .cloned()
            .collect::<Vec<_>>();
        self.codec.encode(plan,records,
          r#"Return JSON {"points":[{"type":"past_event|past_self|now|other","text":"...","cites":[]}]}. Cite visible record IDs for past claims. Use empty cites for current thoughts and other speech. Each text is the exact speech segment, in order."#,
          &missing_hints(self.settings.missing,&selected))
    }
    fn measure(&self, bytes: &[u8]) -> u64 {
        self.codec.measure(bytes)
    }
}

pub fn prepare(
    task: &TaskInput,
    mut material: CloudMaterial<'_>,
    codec: &impl SpeechCodec,
    limits: Limits,
) -> Result<CloudRequest, String> {
    task.canonical_bytes(limits)?;
    let facts = &task.c_task;
    if task.snapshot_id != material.snapshot_id
        || material.prefix != facts.history_prefix
        || task.current_input != material.latest.content
        || !matches!(facts.status, QueryStatus::OK | QueryStatus::NO_MATCH)
        || facts.truncated
    {
        return Err("cloud input binding or required fact query unavailable".into());
    }
    let input = &mut material.input;
    input.latest = RecordId(material.latest.record_id.as_bytes().to_vec());
    input.protocol = task.protocol_content.as_bytes().to_vec();
    let samples = material
        .samples
        .map(|request| {
            let s = request
                .selector
                .select(request.modulation, request.language, request.k)?;
            if s.modulation != *request.modulation
                || s.language != request.language
                || s.k != request.k
                || s.ordered_ids.len() > s.k
                || s.rendered.is_empty()
            {
                return Err("invalid selected samples".to_string());
            }
            Ok(s)
        })
        .transpose()?;
    if let Some(s) = &samples {
        input
            .protocol
            .extend_from_slice("\n\n【说话样例】\n".as_bytes());
        input.protocol.extend_from_slice(s.rendered.as_bytes());
    }
    let records: BTreeMap<_, _> = input.records.iter().map(|r| (&r.id, r)).collect();
    let check = |id: &RecordId, obj: &ObjectRef| {
        records
            .get(id)
            .is_some_and(|r| r.content_version == obj.digest.0)
    };
    if !check(&input.latest, &task.current_input) {
        return Err("latest version missing".into());
    }
    input.required_facts.clear();
    for item in &facts.items {
        let id = RecordId(item.source_id.as_bytes().to_vec());
        if item.availability != QueryStatus::OK || !check(&id, &item.content) {
            return Err("required fact version unavailable".into());
        }
        input.required_facts.push(id);
        input.required_facts.extend(
            item.correction_chain
                .iter()
                .map(|s| RecordId(s.as_bytes().to_vec())),
        );
    }
    let outcome = cp::assemble(
        input,
        &MeasuredSpeech {
            codec,
            settings: material.settings,
            gaps: &material.gaps,
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    let Outcome::Sent {
        plan,
        bytes,
        binding,
        note,
    } = outcome
    else {
        return Err(format!("{outcome:?}"));
    };
    let mut visible = BTreeMap::new();
    let mut add = |id: &RecordId, delivered: Delivered| -> Result<(), String> {
        let r = records.get(id).ok_or("unresolved material")?;
        let range = match delivered {
            Delivered::Full => 0..r.body.len(),
            Delivered::Partial { start, end } => {
                usize::try_from(start).map_err(|_| "range")?
                    ..usize::try_from(end).map_err(|_| "range")?
            }
        };
        let name = String::from_utf8(id.0.clone()).map_err(|_| "invalid source ID")?;
        visible
            .entry(name)
            .and_modify(|v: &mut (Vec<u8>, Vec<u8>, std::ops::Range<usize>)| {
                if delivered == Delivered::Full {
                    v.2 = range.clone()
                }
            })
            .or_insert_with(|| (r.content_version.clone(), r.body.clone(), range));
        Ok(())
    };
    for d in &plan.dialogue {
        add(&d.id, d.delivered)?
    }
    for e in &plan.experience {
        add(&e.id, Delivered::Full)?
    }
    add(&plan.latest, Delivered::Full)?;
    Ok(CloudRequest {
        bytes,
        binding,
        note,
        settings: material.settings,
        samples,
        visible,
        task: task.clone(),
        latest: material.latest,
    })
}

/// The hint is request-scoped. Record labels are encoded as data by the codec.
pub fn missing_hints(setting: MissingHint, selected_gaps: &[String]) -> Vec<String> {
    if selected_gaps.is_empty() {
        return vec![];
    }
    match setting {
        MissingHint::None => vec![],
        MissingHint::General => vec!["本轮材料未提供部分相关记录。".into()],
        MissingHint::PerEvent => selected_gaps
            .iter()
            .map(|id| format!("本轮材料未提供与 {id} 对应的记录。"))
            .collect(),
    }
}
