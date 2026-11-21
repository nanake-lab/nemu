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
    Committed,
}
#[derive(Debug, PartialEq)]
pub enum ReplyState {
    Waiting,
    Failed(FailureReason),
    Generated,
    Closed,
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

impl PendingPlan {
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
        allowed_scopes: &[ObjectRef],
        outbox: &mut impl Outbox,
    ) -> Result<(), String> {
        if &self.bound != current {
            return Err("stale plan".into());
        }
        if let Some(action) = &self.receipt.action_plan
            && !allowed_scopes.contains(&action.expected_scope)
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
        Self {
            input,
            mode,
            state: ReplyState::Waiting,
        }
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
            Interpretation::Committed => {
                self.state = ReplyState::Closed;
                let generation = prepare()?;
                if generation.input.current_input != self.input.content
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
            if !request.matches(input, &g.binding.branch) {
                return Err("stale cloud request".into());
            }
            ModelInput::Projected(&request.bytes)
        }
        _ => ModelInput::Local {
            task: input,
            modulation: &modulation,
        },
    };
    let adapter = generators
        .0
        .get_mut(&u32::from(mode))
        .ok_or("generator unavailable")?;
    let budget = super::protocol::GenerationBudget {
        tokens: input.max_tokens.0,
        duration_ns: input.max_duration_ns.0,
    };
    let start = Instant::now();
    let generated = adapter
        .generate(model_input, budget)
        .map_err(GenerationError::Fault)?;
    let r = generated.receipt;
    r.canonical_bytes(g.limits)?;
    if generated.used_output_tokens > budget.tokens || r.speech_mode != mode {
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
        if r.output_chunks
            .iter()
            .map(|c| c.text.as_str())
            .collect::<String>()
            != approved
        {
            return Err("speech differs from checked outline".into());
        }
    } else if r.carrier_receipt.is_none()
        || (mode == SpeechMode::C_DRAFT_REWRITE && r.intermediate_results.is_empty())
    {
        return Err("missing actual carrier or draft receipt".into());
    }
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
