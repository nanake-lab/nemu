//! One live reply attempt in the outer transaction owner. Restores/replays never
//! dispatch from this API. The owner persists Q and arbitrates the final CAS.
use super::projection::{CloudRequest, Governance};
use core_types::{WireMessage, contract::Limits, generated::*};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

/// Bound Q inputs captured when generation actually starts. A new observation,
/// protocol, permission, branch or snapshot invalidates unregistered output.
#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub branch: String,
    pub history_prefix: String,
    pub processed_through: u64,
    pub snapshot: String,
    pub frontier: Vec<String>,
    pub protocol_version: String,
    pub permission: ObjectRef,
    pub q_revision: String,
}

impl Binding {
    /// Apply to the owner's staged Q before commit. Only plans still bound to
    /// this complete runtime binding remain eligible for registration.
    pub fn bind_protocol(&mut self, protocol: &super::protocol::Protocol) {
        self.protocol_version = protocol.version().to_owned();
    }
}

#[derive(Debug)]
pub enum Modulation<'a> {
    Zero(FailureReason),
    Compiled(&'a ModulationEnvelopeV2),
}
pub trait ModulationProvider {
    /// Immutable value used by the sample selector (D10). A provider without a
    /// retrievable value can only service requests which do not bind samples.
    fn resource(&self, _snapshot: &str, _generation_ns: u64) -> Option<&ObjectRef> {
        None
    }
    fn at_generation(&self, snapshot: &str, generation_ns: u64) -> Result<Modulation<'_>, String>;
}
pub struct DisabledModulation;
impl ModulationProvider for DisabledModulation {
    fn at_generation(&self, _: &str, _: u64) -> Result<Modulation<'_>, String> {
        Ok(Modulation::Zero(FailureReason {
            schema_version: 1,
            code: FailureCode::MODULATION_DISABLED,
            detail_ref: None,
        }))
    }
}

/// Selected by the trusted runtime only for a separately permitted zero-mode
/// response. A failure never implicitly selects another generator or mode.
pub struct CarrierUnavailable(pub Option<ObjectRef>);
impl ModulationProvider for CarrierUnavailable {
    fn at_generation(&self, _: &str, _: u64) -> Result<Modulation<'_>, String> {
        Ok(Modulation::Zero(FailureReason {
            schema_version: 1,
            code: FailureCode::CARRIER_UNAVAILABLE,
            detail_ref: self.0.clone(),
        }))
    }
}

/// Registered adapters are trusted in-process clients of the separate model.
/// The cloud transport only receives the preassembled bytes. Receipts stay local.
pub enum ModelInput<'a> {
    Local {
        task: &'a TaskInput,
        modulation: &'a Modulation<'a>,
    },
    Projected(&'a [u8]),
}
pub struct Generated {
    pub receipt: PlanAndCarrierReceipt,
    pub used_output_tokens: u64,
    /// The once-generated JSON outline is required for mode B.
    pub outline: Option<String>,
}
pub trait Generator {
    /// Apply both limits during generation and stop the actual model at its
    /// deadline. The host also checks wall time and usage before registration.
    fn generate(
        &mut self,
        input: ModelInput<'_>,
        budget: super::protocol::GenerationBudget,
    ) -> Result<Generated, FailureReason>;
}
#[derive(Default)]
pub struct Generators(BTreeMap<u32, Box<dyn Generator>>);
impl Generators {
    pub fn register(&mut self, mode: SpeechMode, generator: Box<dyn Generator>) {
        self.0.insert(mode.into(), generator);
    }
}

pub enum Interpretation {
    Pending,
    Failed(FailureReason),
    /// H position of the effective interpretation, including a valid no-effect
    /// outcome, after its ordered commit. A failed placeholder is not a commit here.
    Committed {
        input_position: u64,
    },
}
#[derive(Debug, PartialEq)]
pub enum ReplyState {
    Waiting,
    Failed(FailureReason),
    Generated,
    Closed,
}

impl ReplyState {
    /// Persist this generated status with the owning Q transaction. A failed
    /// pre-generation turn has no carrier response or delivery receipt.
    pub fn fault_receipt(&self) -> Option<DegradedStatus> {
        let Self::Failed(reason) = self else {
            return None;
        };
        let state = if reason.code == FailureCode::IRL_FAILED {
            RunStatus::IRL_DEGRADED
        } else {
            RunStatus::CORE_UNMODULATED
        };
        Some(DegradedStatus {
            schema_version: 1,
            states: vec![state],
            reasons: vec![reason.clone()],
        })
    }
}

pub struct ReplyTurn {
    input: RecordVersion,
    mode: SpeechMode,
    state: ReplyState,
}
pub struct Generation<'a> {
    pub input: &'a TaskInput,
    pub binding: &'a Binding,
    pub projected_at_ns: u64,
    pub modulation: &'a dyn ModulationProvider,
    pub cloud: Option<(&'a CloudRequest, &'a dyn Governance)>,
    pub limits: Limits,
}

/// Kept by the host alongside the generated receipt. No carrier receipt is
/// fabricated when F is disabled or a generator could not run.
pub struct PendingPlan {
    bound: Binding,
    receipt: PlanAndCarrierReceipt,
    zero_reason: Option<FailureReason>,
}

#[derive(Debug)]
pub enum GenerationError {
    Fault(FailureReason),
    Invalid(String),
}
impl From<String> for GenerationError {
    fn from(s: String) -> Self {
        Self::Invalid(s)
    }
}
impl From<&str> for GenerationError {
    fn from(s: &str) -> Self {
        Self::Invalid(s.into())
    }
}

/// P5-14 owns registration/send-time revalidation, idempotence, numbered chunks
/// and UNKNOWN. Authorizing all payloads and registering them must be atomic.
pub trait Outbox {
    fn register(
        &mut self,
        binding: &Binding,
        chunks: &[OutputChunk],
        action: Option<&ActionPlan>,
    ) -> Result<(), String>;
}

/// The trusted boundary table pairs a payload kind with its allowed scope.
/// A model cannot grant itself a new kind by copying an allowed scope.
pub struct AllowedAction<'a> {
    pub payload_type: &'a str,
    pub scope: &'a ObjectRef,
}

impl PendingPlan {
    pub fn modulation_status(&self) -> Option<DegradedStatus> {
        self.zero_reason.as_ref().map(|reason| DegradedStatus {
            schema_version: 1,
            states: if reason.code == FailureCode::MODULATION_DISABLED {
                vec![
                    RunStatus::CORE_UNMODULATED,
                    RunStatus::MODULATION_NOT_ENABLED,
                ]
            } else {
                vec![RunStatus::CORE_UNMODULATED]
            },
            reasons: vec![reason.clone()],
        })
    }
    pub fn receipt(&self) -> &PlanAndCarrierReceipt {
        &self.receipt
    }
    pub fn zero_reason(&self) -> Option<&FailureReason> {
        self.zero_reason.as_ref()
    }
    /// Consume once. The outbox must compare the supplied binding atomically
    /// with current Q, including after this preliminary read.
    pub fn submit(
        self,
        current: &Binding,
        allowed: &[AllowedAction<'_>],
        outbox: &mut impl Outbox,
    ) -> Result<(), String> {
        if &self.bound != current {
            return Err("stale plan".into());
        }
        if let Some(action) = &self.receipt.action_plan
            && !allowed.iter().any(|entry| {
                entry.scope == &action.expected_scope
                    && entry.payload_type == action.output_or_tool_parameters.object_type
            })
        {
            return Err("action outside allowed scope".into());
        }
        if self.receipt.output_chunks.is_empty() && self.receipt.action_plan.is_none() {
            return Ok(());
        }
        outbox.register(
            &self.bound,
            &self.receipt.output_chunks,
            self.receipt.action_plan.as_ref(),
        )
    }
}

impl ReplyTurn {
    pub fn new(input: RecordVersion, mode: SpeechMode) -> Self {
        Self::resume(input, mode, ReplyState::Waiting)
    }
    /// The outer owner restores the state recorded for this exact input. A
    /// completed or failed attempt cannot be rearmed by late interpretation.
    pub fn resume(input: RecordVersion, mode: SpeechMode, state: ReplyState) -> Self {
        Self { input, mode, state }
    }
    pub fn state(&self) -> &ReplyState {
        &self.state
    }
    /// No duration, deadline or timeout on this transition. The owner calls it
    /// with committed interpretation of the same input, including no-effect.
    pub fn advance<'a>(
        &mut self,
        input: &RecordVersion,
        progress: Interpretation,
        generators: &mut Generators,
        prepare: impl FnOnce() -> Result<Generation<'a>, String>,
    ) -> Result<Option<PendingPlan>, GenerationError> {
        if self.state != ReplyState::Waiting || &self.input != input {
            return Ok(None);
        }
        match progress {
            Interpretation::Pending => Ok(None),
            Interpretation::Failed(reason) => {
                self.state = ReplyState::Failed(reason);
                Ok(None)
            }
            Interpretation::Committed { input_position } => {
                self.state = ReplyState::Closed;
                let generation = prepare()?;
                if generation.binding.processed_through < input_position
                    || generation.input.c_task.history_prefix != generation.binding.history_prefix
                    || generation.input.current_input != self.input.content
                    || generation.binding.branch != self.input.branch_id
                    || generation
                        .cloud
                        .is_some_and(|(c, _)| c.latest() != &self.input)
                {
                    return Err("reply observation changed".into());
                }
                let plan = match generate(self.mode, generation, generators) {
                    Err(GenerationError::Fault(reason)) => {
                        self.state = ReplyState::Failed(reason.clone());
                        return Err(GenerationError::Fault(reason));
                    }
                    other => other?,
                };
                self.state = ReplyState::Generated;
                Ok(Some(plan))
            }
        }
    }
}

fn generate(
    mode: SpeechMode,
    generation: Generation<'_>,
    generators: &mut Generators,
) -> Result<PendingPlan, GenerationError> {
    let g = generation;
    let input = g.input;
    input.canonical_bytes(g.limits)?;
    if input.snapshot_id != g.binding.snapshot
        || input.protocol_version != g.binding.protocol_version
    {
        return Err("task and Q binding differ".into());
    }
    let modulation = g
        .modulation
        .at_generation(&input.snapshot_id, g.projected_at_ns)?;
    if let Modulation::Compiled(e) = &modulation
        && (e.snapshot_id != input.snapshot_id || e.projected_at_ns.0 != g.projected_at_ns)
    {
        return Err("stale modulation".into());
    }
    let cloud = g.cloud;
    let model_input = match mode {
        SpeechMode::B_REMOTE_PROJECTED => {
            let (request, _) = cloud.ok_or("assembled cloud request required")?;
            if !request.matches(input, g.binding)
                || request.samples().is_some_and(|s| {
                    g.modulation.resource(&input.snapshot_id, g.projected_at_ns)
                        != Some(&s.modulation)
                })
            {
                return Err("stale cloud request".into());
            }
            ModelInput::Projected(&request.bytes)
        }
        _ => ModelInput::Local {
            task: input,
            modulation: &modulation,
        },
    };
    let adapter = generators.0.get_mut(&u32::from(mode)).ok_or({
        GenerationError::Fault(FailureReason {
            schema_version: 1,
            code: FailureCode::CARRIER_UNAVAILABLE,
            detail_ref: None,
        })
    })?;
    let budget = super::protocol::GenerationBudget {
        tokens: input.max_tokens.0,
        duration_ns: input.max_duration_ns.0,
    };
    let start = Instant::now();
    let generated = adapter
        .generate(model_input, budget)
        .map_err(GenerationError::Fault)?;
    let mut r = generated.receipt;
    if generated.used_output_tokens > budget.tokens
        || r.speech_mode != mode
        || (matches!(modulation, Modulation::Zero(_)) && r.strict)
    {
        return Err("invalid generation budget or mode".into());
    }
    if mode == SpeechMode::B_REMOTE_PROJECTED {
        if r.strict
            || r.carrier_receipt.is_some()
            || r.projection_version.as_deref().is_none_or(str::is_empty)
            || r.action_plan.is_some()
        {
            return Err("projected path exceeded speech authority".into());
        }
        let (request, governance) = cloud.ok_or("missing cloud request")?;
        let approved = request.checked_text(
            generated
                .outline
                .as_deref()
                .ok_or("missing speech outline")?,
            governance,
        )?;
        // The adapter supplies output identity. Only the code-joined outline
        // becomes speech. No raw reply or second model pass is publishable.
        if approved.is_empty() {
            r.output_chunks.clear();
        } else {
            r.output_chunks.truncate(1);
            let chunk = r
                .output_chunks
                .first_mut()
                .ok_or("missing speech action identity")?;
            chunk.text = approved;
        }
    } else if r.carrier_receipt.is_none()
        || (mode == SpeechMode::C_DRAFT_REWRITE && r.intermediate_results.is_empty())
    {
        return Err("missing actual carrier or draft receipt".into());
    }
    r.canonical_bytes(g.limits)?;
    if r.output_chunks.iter().enumerate().any(|(i, c)| {
        c.snapshot_id != input.snapshot_id
            || c.speech_mode != mode
            || c.strict != r.strict
            || c.chunk_index.0 != i as u64
            || c.action_id != r.output_chunks[0].action_id
            || c.output_id != r.output_chunks[0].output_id
    }) || r.action_plan.as_ref().is_some_and(|p| {
        p.snapshot_id != input.snapshot_id || p.protocol_version != input.protocol_version
    }) {
        return Err("plan binding mismatch".into());
    }
    if start.elapsed() > Duration::from_nanos(budget.duration_ns) {
        return Err("generation duration exhausted".into());
    }
    let zero_reason = match modulation {
        Modulation::Zero(reason) => Some(reason),
        Modulation::Compiled(_) => None,
    };
    Ok(PendingPlan {
        bound: g.binding.clone(),
        receipt: r,
        zero_reason,
    })
}
