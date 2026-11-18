#![forbid(unsafe_code)]
//! CLI bridge example for DEMO-CAUSE-01 (`task_layer::cause_projection::assemble`).

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};
use task_layer::cause_projection::{
    AssemblyInput, BaseFailure, BasisId, Budget, DItem, Delivered, Eligibility, HPos, InvalidInput,
    Member, Outcome, PackEntry, PackFate, Record, RecordId, RecordTable, Relation, RelationKind,
    RequestCodec, RequestPlan, SentNote, Slot, Unavailable, assemble,
};

const DEFAULT_PROTOCOL: &str = "请用中文回复最新输入。输出为对话正文。已有对话保留原说话人；本轮来由是历史记录，记录中的话保留原有来源身份。";
const IDENTITY_LINE_1: &str = "本轮说话者是 Nemu，对话对象是用户竹。";
const IDENTITY_LINE_2: &str = "你知道的过去，只有下面记录里的内容。";

fn default_true() -> bool {
    true
}

fn default_protocol() -> String {
    DEFAULT_PROTOCOL.to_string()
}

#[derive(Deserialize)]
#[serde(untagged)]
enum HValueIn {
    Scalar(f64),
    Vec(Vec<f64>),
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum EligibilityIn {
    #[default]
    Eligible,
    Undetermined,
    Ineligible {
        reason: String,
    },
}

#[derive(Deserialize)]
struct RecordIn {
    id: String,
    h_pos: u64,
    #[serde(default)]
    time: String,
    speaker: String,
    text: String,
    #[serde(default)]
    eligibility: EligibilityIn,
}

#[derive(Deserialize)]
struct RelationIn {
    from: String,
    to: String,
    kind: String,
}

#[derive(Deserialize)]
struct SlotIn {
    basis: String,
    h: HValueIn,
    first_f_pos: u64,
    sources: Vec<String>,
    #[serde(default)]
    merge_undecided: bool,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum DeliveredIn {
    #[default]
    Full,
    Partial {
        start: u64,
        end: u64,
    },
}

#[derive(Deserialize)]
struct DialogueIn {
    id: String,
    #[serde(default)]
    delivered: DeliveredIn,
}

#[derive(Deserialize)]
struct BudgetIn {
    capacity: u64,
    output_reserve: u64,
}

#[derive(Deserialize)]
struct DemoInput {
    #[serde(default = "default_protocol")]
    protocol: String,
    #[serde(default)]
    protocol_cited: Vec<String>,
    #[serde(default)]
    dialogue: Vec<DialogueIn>,
    latest: String,
    #[serde(default)]
    required_facts: Vec<String>,
    records: Vec<RecordIn>,
    #[serde(default)]
    relations: Vec<RelationIn>,
    #[serde(default = "default_true")]
    frontier_complete: bool,
    #[serde(default)]
    slots: Vec<SlotIn>,
    #[serde(default)]
    decomposition_unknown: bool,
    #[serde(default)]
    gap_facts: bool,
    budget: BudgetIn,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct PackOut {
    basis: String,
    records: Vec<String>,
    fate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_records: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    record: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Serialize)]
struct BindingOut {
    base_records: Vec<String>,
    accepted: u32,
    packs: Vec<PackOut>,
}

#[derive(Serialize)]
struct DemoOutput {
    outcome: String,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    messages: Option<Vec<ChatMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    binding: Option<BindingOut>,
    #[serde(skip_serializing_if = "Option::is_none")]
    measured: Option<u64>,
}

impl DemoOutput {
    fn err(outcome: &str, reason: String) -> Self {
        Self {
            outcome: outcome.to_string(),
            reason,
            messages: None,
            binding: None,
            measured: None,
        }
    }
}

/// 演示用，不是发布的投影资源 (Demo only; not a published projection resource).
struct DemoCodec {
    speakers: BTreeMap<RecordId, String>,
    times: BTreeMap<RecordId, String>,
    slot_sources: Vec<Vec<RecordId>>,
    gap_facts: bool,
}

impl DemoCodec {
    fn append_gap_facts(&self, content: &mut String, plan: &RequestPlan, records: &RecordTable) {
        let delivered: BTreeSet<_> = plan
            .dialogue
            .iter()
            .map(|item| &item.id)
            .chain(plan.experience.iter().map(|item| &item.id))
            .chain(std::iter::once(&plan.latest))
            .collect();
        let mut anchors = BTreeSet::new();
        for sources in &self.slot_sources {
            if !sources.iter().all(|id| {
                delivered.contains(id)
                    && self
                        .speakers
                        .get(id)
                        .is_some_and(|speaker| !speaker.starts_with("Nemu"))
            }) {
                continue;
            }
            if let Some(anchor) = sources
                .iter()
                .filter_map(|id| records.get(id))
                .min_by_key(|record| (record.h_pos, &record.id))
                && let Some(time) = self.times.get(&anchor.id)
            {
                anchors.insert((anchor.h_pos, time));
            }
        }
        let mut written = BTreeSet::new();
        for (_, time) in anchors {
            if written.insert(time) {
                content.push_str(&format!("\n关于 [{time}] 那件事，你当时的想法没有记录。"));
            }
        }
    }

    fn encode_messages(&self, plan: &RequestPlan, records: &RecordTable) -> Vec<ChatMessage> {
        let mut msgs = vec![ChatMessage {
            role: "system".to_string(),
            content: String::from_utf8_lossy(&plan.protocol).into_owned(),
        }];
        for item in &plan.dialogue {
            let role = match self.speakers.get(&item.id).map(String::as_str) {
                Some("Nemu" | "assistant") => "assistant",
                _ => "user",
            };
            let content = records.get(&item.id).map_or_else(String::new, |rec| {
                let slice = match item.delivered {
                    Delivered::Full => rec.body.as_slice(),
                    Delivered::Partial { start, end } => {
                        let s = usize::try_from(start).unwrap_or(rec.body.len());
                        let e = usize::try_from(end).unwrap_or(rec.body.len());
                        rec.body.get(s..e).unwrap_or(&[])
                    }
                };
                String::from_utf8_lossy(slice).into_owned()
            });
            msgs.push(ChatMessage {
                role: role.to_string(),
                content,
            });
        }
        let mut e_content = format!("{IDENTITY_LINE_1}\n{IDENTITY_LINE_2}");
        if self.gap_facts {
            self.append_gap_facts(&mut e_content, plan, records);
        }
        for item in &plan.experience {
            if let Some(rec) = records.get(&item.id) {
                e_content.push('\n');
                e_content.push_str(&String::from_utf8_lossy(&rec.body));
            }
        }
        msgs.push(ChatMessage {
            role: "system".to_string(),
            content: e_content,
        });
        let latest = records.get(&plan.latest).map_or_else(String::new, |rec| {
            String::from_utf8_lossy(&rec.body).into_owned()
        });
        msgs.push(ChatMessage {
            role: "user".to_string(),
            content: latest,
        });
        msgs
    }
}

impl RequestCodec for DemoCodec {
    fn encode(&self, plan: &RequestPlan, records: &RecordTable) -> Vec<u8> {
        serde_json::to_vec(&self.encode_messages(plan, records)).unwrap_or_default()
    }

    fn measure(&self, bytes: &[u8]) -> u64 {
        let msgs: Vec<ChatMessage> = serde_json::from_slice(bytes).unwrap_or_default();
        msgs.iter().fold(0_u64, |acc, m| {
            acc.saturating_add(m.content.len() as u64 + 64)
        })
    }
}

fn id_str(id: &RecordId) -> String {
    String::from_utf8_lossy(&id.0).into_owned()
}

fn to_rids(ids: Vec<String>) -> Vec<RecordId> {
    ids.into_iter().map(|s| RecordId(s.into_bytes())).collect()
}

fn pack_out(entry: PackEntry) -> PackOut {
    let basis = String::from_utf8_lossy(&entry.basis.0).into_owned();
    let records = entry.records.iter().map(id_str).collect();
    let (fate, new_records, record, reason) = match entry.fate {
        PackFate::Accepted { new_records } => (
            "accepted",
            Some(new_records.iter().map(id_str).collect()),
            None,
            None,
        ),
        PackFate::FirstOverflow => ("first_overflow", None, None, None),
        PackFate::NotTried => ("not_tried", None, None, None),
        PackFate::Excluded { record, reason } => {
            ("excluded", None, record.as_ref().map(id_str), Some(reason))
        }
    };
    PackOut {
        basis,
        records,
        fate: fate.to_string(),
        new_records,
        record,
        reason,
    }
}

fn build_input(demo_in: DemoInput) -> Result<(AssemblyInput, DemoCodec), String> {
    let mut speakers = BTreeMap::new();
    let mut times = BTreeMap::new();
    let mut records = Vec::with_capacity(demo_in.records.len());
    for r in demo_in.records {
        let rid = RecordId(r.id.into_bytes());
        speakers.insert(rid.clone(), r.speaker.clone());
        let body = if r.time.is_empty() {
            r.text.into_bytes()
        } else {
            format!("[{}] {}：{}", r.time, r.speaker, r.text).into_bytes()
        };
        times.insert(rid.clone(), r.time);
        let eligibility = match r.eligibility {
            EligibilityIn::Eligible => Eligibility::Eligible,
            EligibilityIn::Undetermined => Eligibility::Undetermined,
            EligibilityIn::Ineligible { reason } => Eligibility::Ineligible { reason },
        };
        records.push(Record {
            id: rid,
            content_version: b"demo-v1".to_vec(),
            h_pos: HPos(r.h_pos),
            body,
            eligibility,
        });
    }
    let mut relations = Vec::with_capacity(demo_in.relations.len());
    for rel in demo_in.relations {
        let kind = match rel.kind.as_str() {
            "body_source" | "BodySource" => RelationKind::BodySource,
            "continuation" | "Continuation" => RelationKind::Continuation,
            "correction" | "Correction" => RelationKind::Correction,
            "local_only" | "LocalOnly" => RelationKind::LocalOnly,
            "unknown" | "Unknown" => RelationKind::Unknown,
            other => return Err(format!("unknown relation kind: {other}")),
        };
        relations.push(Relation {
            from: RecordId(rel.from.into_bytes()),
            to: RecordId(rel.to.into_bytes()),
            kind,
        });
    }
    let mut slot_sources = Vec::new();
    let slots = demo_in
        .slots
        .into_iter()
        .map(|s| {
            let basis = BasisId(s.basis.into_bytes());
            let h = match s.h {
                HValueIn::Scalar(v) => vec![v],
                HValueIn::Vec(v) => v,
            };
            let sources = to_rids(s.sources);
            if demo_in.gap_facts {
                slot_sources.push(sources.clone());
            }
            Slot {
                basis: basis.clone(),
                h,
                first_f_pos: HPos(s.first_f_pos),
                members: vec![Member { basis, sources }],
                merge_undecided: s.merge_undecided,
            }
        })
        .collect();
    let dialogue = demo_in
        .dialogue
        .into_iter()
        .map(|d| DItem {
            id: RecordId(d.id.into_bytes()),
            delivered: match d.delivered {
                DeliveredIn::Full => Delivered::Full,
                DeliveredIn::Partial { start, end } => Delivered::Partial { start, end },
            },
        })
        .collect();
    let input = AssemblyInput {
        protocol: demo_in.protocol.into_bytes(),
        protocol_cited: to_rids(demo_in.protocol_cited),
        dialogue,
        latest: RecordId(demo_in.latest.into_bytes()),
        required_facts: to_rids(demo_in.required_facts),
        records,
        relations,
        frontier_complete: demo_in.frontier_complete,
        slots,
        decomposition_unknown: demo_in.decomposition_unknown,
        budget: Budget {
            capacity: demo_in.budget.capacity,
            output_reserve: demo_in.budget.output_reserve,
        },
    };
    Ok((
        input,
        DemoCodec {
            speakers,
            times,
            slot_sources,
            gap_facts: demo_in.gap_facts,
        },
    ))
}

fn run(stdin_str: &str) -> DemoOutput {
    let demo_in: DemoInput = match serde_json::from_str(stdin_str) {
        Ok(v) => v,
        Err(err) => return DemoOutput::err("invalid_input", format!("json_parse_error: {err}")),
    };
    let (input, codec) = match build_input(demo_in) {
        Ok(pair) => pair,
        Err(msg) => return DemoOutput::err("invalid_input", msg),
    };
    match assemble(&input, &codec) {
        Ok(Outcome::Sent {
            bytes,
            binding,
            note,
            ..
        }) => {
            let reason = match note {
                SentNote::AllFit => "all_fit",
                SentNote::OptionalOverBudget => "optional_over_budget",
                SentNote::NoEligibleExperience => "no_eligible_experience",
            }
            .to_string();
            DemoOutput {
                outcome: "sent".to_string(),
                reason,
                messages: Some(serde_json::from_slice(&bytes).unwrap_or_default()),
                binding: Some(BindingOut {
                    base_records: binding.base_records.iter().map(id_str).collect(),
                    accepted: binding.accepted,
                    packs: binding.packs.into_iter().map(pack_out).collect(),
                }),
                measured: Some(binding.measured),
            }
        }
        Ok(Outcome::ProjectionUnavailable(u)) => {
            let msg = match u {
                Unavailable::FrontierIncomplete => "frontier_incomplete".to_string(),
                Unavailable::DecompositionUnknown => "decomposition_unknown".to_string(),
                Unavailable::MergeUndecided(b) => {
                    format!("merge_undecided({})", String::from_utf8_lossy(&b.0))
                }
                Unavailable::UnknownRelation { from, to } => {
                    format!("unknown_relation({} -> {})", id_str(&from), id_str(&to))
                }
                Unavailable::MissingRecord(r) => format!("missing_record({})", id_str(&r)),
                Unavailable::EligibilityUndetermined(r) => {
                    format!("eligibility_undetermined({})", id_str(&r))
                }
            };
            DemoOutput::err("projection_unavailable", msg)
        }
        Ok(Outcome::BaseRequestFailed(f)) => {
            let msg = match f {
                BaseFailure::Ineligible { record, reason } => {
                    format!("ineligible({}): {reason}", id_str(&record))
                }
                BaseFailure::OverCapacity {
                    measured,
                    output_reserve,
                    capacity,
                } => format!(
                    "over_capacity(measured={measured}, output_reserve={output_reserve}, capacity={capacity})"
                ),
            };
            DemoOutput::err("base_request_failed", msg)
        }
        Err(e) => {
            let msg = match e {
                InvalidInput::NonFiniteH { basis, index } => format!(
                    "non_finite_h({}, index={index})",
                    String::from_utf8_lossy(&basis.0)
                ),
                InvalidInput::ConflictingDuplicateRecord(r) => {
                    format!("conflicting_duplicate_record({})", id_str(&r))
                }
                InvalidInput::DuplicateSlotBasis(b) => {
                    format!("duplicate_slot_basis({})", String::from_utf8_lossy(&b.0))
                }
                InvalidInput::EmptySlotMembers(b) => {
                    format!("empty_slot_members({})", String::from_utf8_lossy(&b.0))
                }
                InvalidInput::MissingBaseRecord(r) => {
                    format!("missing_base_record({})", id_str(&r))
                }
                InvalidInput::InvalidPartialRange {
                    id,
                    start,
                    end,
                    body_len,
                } => format!(
                    "invalid_partial_range({}, {start}..{end}, len={body_len})",
                    id_str(&id)
                ),
            };
            DemoOutput::err("invalid_input", msg)
        }
    }
}

fn main() {
    let mut buf = String::new();
    let out = match io::stdin().read_to_string(&mut buf) {
        Ok(_) => run(&buf),
        Err(err) => DemoOutput::err("invalid_input", format!("stdin_read_error: {err}")),
    };
    if let Ok(json) = serde_json::to_string_pretty(&out) {
        let mut stdout = io::stdout().lock();
        let _ = stdout.write_all(json.as_bytes());
        let _ = stdout.write_all(b"\n");
    }
}
