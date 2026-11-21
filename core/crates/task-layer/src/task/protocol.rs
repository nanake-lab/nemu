use core_types::generated::{FactProjection, Lang, ObjectRef, RunStateV2, TaskInput};
use core_types::scalar::{DecimalU64, hex};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sections {
    facts: String,
    boundaries: String,
    format: String,
    language: String,
}

#[derive(Deserialize)]
struct Manifest {
    version: String,
    status: String,
    files: Vec<String>,
}

/// Selected by the bound deployment configuration. LIFE uses its signed resource.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Source {
    Candidate,
    Final,
}

#[derive(Debug)]
pub struct Protocol {
    version: String,
    content: String,
    source: Source,
}

impl Protocol {
    /// W19 supplies only a purpose=Task projection. Its query and read set
    /// accompany the proposal so the transaction owner can bind the audit.
    pub fn from_retrieval(
        &self,
        current_input: ObjectRef,
        result: &fact_view::Execution,
        snapshot_id: String,
        budget: GenerationBudget,
    ) -> Result<TaskInput, String> {
        if result.trace.query.purpose != fact_view::Purpose::Task
            || result.trace.query.history_prefix != result.projection.history_prefix
        {
            return Err("expected W19 task projection at the bound prefix".into());
        }
        Ok(self.task_input(
            current_input,
            result.projection.clone(),
            snapshot_id,
            budget,
        ))
    }
    /// Slots come from the validated asset manifest intersected with the contracts'
    /// performable slots. Only slot identifiers may enter the trusted format section.
    pub fn load(root: &Path, source: Source, lang: Lang, slots: &[&str]) -> Result<Self, String> {
        let (folder, status) = match source {
            Source::Candidate => ("_candidates/core/protocol", "candidate"),
            Source::Final => ("nemu/core/protocol", "final"),
        };
        let dir = root.join(folder);
        let name = match lang {
            Lang::ZH => "zh.json",
            Lang::EN => "en.json",
            Lang::JA => "ja.json",
        };
        let read = |name: &str| fs::read(dir.join(name)).map_err(|e| e.to_string());
        let manifest: Manifest =
            serde_json::from_slice(&read("manifest.json")?).map_err(|e| e.to_string())?;
        if manifest.status != status
            || manifest.version.is_empty()
            || !manifest.files.iter().any(|f| f == name)
        {
            return Err("protocol resource does not match deployment selection".into());
        }
        let Sections {
            facts,
            boundaries,
            format,
            language,
        } = serde_json::from_slice(&read(name)?).map_err(|e| e.to_string())?;
        if [&facts, &boundaries, &format, &language]
            .iter()
            .any(|s| s.trim().is_empty())
            || format.matches("{acts}").count() != 1
            || slots.iter().any(|s| {
                s.is_empty()
                    || !s
                        .bytes()
                        .all(|c| c.is_ascii_lowercase() || c == b'_' || c.is_ascii_digit())
            })
        {
            return Err("invalid protocol sections or slot identifier".into());
        }
        let mut slots = slots.to_vec();
        slots.sort_unstable();
        slots.dedup();
        let content = [
            facts,
            boundaries,
            format.replace("{acts}", &slots.join(", ")),
            language,
        ]
        .join("\n\n");
        let version = format!(
            "{}/{status}/{name}/{}",
            manifest.version,
            hex(&Sha256::digest(content.as_bytes()))
        );
        Ok(Self {
            version,
            content,
            source,
        })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn task_input(
        &self,
        current_input: ObjectRef,
        c_task: FactProjection,
        snapshot_id: String,
        budget: GenerationBudget,
    ) -> TaskInput {
        TaskInput {
            schema_version: 1,
            current_input,
            protocol_version: self.version.clone(),
            protocol_content: self.content.clone(),
            c_task,
            snapshot_id,
            max_tokens: DecimalU64(budget.tokens),
            max_duration_ns: DecimalU64(budget.duration_ns),
        }
    }

    /// The owner supplies the current Q/lineage projection on every update.
    pub fn push_status<E>(
        &self,
        mut current: RunStateV2,
        previous: &mut Option<RunStateV2>,
        send: impl FnOnce(&RunStateV2) -> Result<(), E>,
    ) -> Result<(), E> {
        if self.source == Source::Candidate && !current.reasons.iter().any(|s| s == "协议草稿")
        {
            current.reasons.push("协议草稿".into());
        }
        if previous.as_ref() != Some(&current) {
            send(&current)?;
            *previous = Some(current);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct GenerationBudget {
    pub tokens: u64,
    pub duration_ns: u64,
}

/// Explicit deployment resource. The approved D3/D7 defaults are the only
/// implicit choices; the owner supplies all numeric settings.
#[derive(Clone, Copy, Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskSettings {
    pub generation: GenerationBudget,
    #[serde(default)]
    pub speech: super::speech::SpeechSettings,
    pub k: usize,
}
impl TaskSettings {
    pub fn load(path: &Path) -> Result<Self, String> {
        serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }
}
