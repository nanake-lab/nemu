//! ADR 0051. These checks consume an already authenticated local pipe. Only the
//! host knows its peer, launch instance and the core's transaction snapshot.
use crate::contract::{Limits, Result, require};
use crate::generated::*;
use crate::scalar::HexBytes;
use crate::{WireMessage, codec_schema};
use sha2::{Digest, Sha256};

/// Check the length prefix before allocating the frame body (P5-07 transport).
pub fn frame_length(bytes: u32, budget: &IrlPipeBudget) -> Result<usize> {
    require(bytes > 0 && bytes <= budget.frame_bytes, "IRL frame budget")?;
    Ok(bytes as usize)
}

pub fn verify_material(value: &IrlMaterial, limits: Limits) -> Result<()> {
    let r = &value.object;
    let bytes = &value.data.0;
    require(
        r.byte_length.0 == bytes.len() as u64 && bytes.len() <= limits.max_bytes,
        "IRL object length",
    )?;
    // ADR 0046 exact application bytes use the store's raw SHA-256 domain.
    if r.object_type == "nemu.storage.raw-bytes-v1" {
        return require(
            r.resource_version == "1" && Sha256::digest(bytes).as_slice() == r.digest.0,
            "IRL raw resource",
        );
    }
    let schema = codec_schema(IrlMaterial::NAME)?;
    let (name, _) = schema
        .messages
        .iter()
        .find(|(_, m)| m.tag == r.object_type && !m.transient)
        .ok_or("unknown or transient IRL object")?;
    let decoded = schema.decode_canonical(name, bytes, limits, 0)?;
    require(
        schema.digest(name, &decoded, limits)? == crate::scalar::hex(&r.digest.0),
        "IRL object digest",
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    ToCallee,
    ToCaller,
}

/// Ephemeral correlation for one authenticated request. A transport result can
/// only close this call. The U6 reader consumes a core commit (ADR 0042 §3.1).
pub struct PipeCall<'a> {
    binding: Option<&'a ProposalBindingV2>,
    opening: IrlPipeFrame,
    link: IrlPipeLink,
    frame_bytes: usize,
    phase: Phase,
    objects_left: u64,
    output_left: u64,
    pending: Vec<IrlBorrow>,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    New,
    Active,
    Cancelling,
    Closed,
}

impl<'a> PipeCall<'a> {
    /// The host supplies the approved opening frame from its local call record.
    pub fn new(
        opening: IrlPipeFrame,
        link: IrlPipeLink,
        binding: Option<&'a ProposalBindingV2>,
        budget: &IrlPipeBudget,
    ) -> Result<Self> {
        require(
            (link == IrlPipeLink::IRL_CORE_WORKER) == binding.is_some(),
            "IRL binding scope",
        )?;
        if let Some(binding) = binding {
            let dispatch = opening.dispatch.as_ref().ok_or("missing IRL dispatch")?;
            require(
                dispatch.target.record_id == binding.event_id
                    && dispatch.target.branch_id == binding.branch_id
                    && dispatch.target.content == dispatch.payload.object
                    && dispatch.facts.object.digest == binding.c_core_digest
                    && dispatch.registry.object == binding.environment.registry
                    && [&dispatch.payload, &dispatch.facts, &dispatch.registry]
                        .iter()
                        .all(|loan| binding.read_set.contains(&loan.object)),
                "IRL dispatch outside core binding",
            )?;
        }
        Ok(Self {
            binding,
            opening,
            link,
            frame_bytes: budget.frame_bytes as usize,
            phase: Phase::New,
            objects_left: budget.object_bytes.0,
            output_left: budget.output_bytes.0,
            pending: Vec::new(),
        })
    }

    /// Decode the transport frame under host limits. Direction comes from the connection.
    /// Failure leaves the previous state and all counters intact.
    pub fn receive<'permit>(
        &mut self,
        direction: Direction,
        bytes: &[u8],
        limits: Limits,
        // The host rechecks the parent call, producer, purpose and current
        // authorization version, returning only objects approved for this use.
        permitted_inputs: impl FnOnce(&IrlDelegation) -> Result<&'permit [ObjectRef]>,
    ) -> Result<IrlPipeFrame> {
        use Direction::*;
        use IrlPipeLink::*;
        use IrlPipeOperation::*;
        require(bytes.len() <= self.frame_bytes, "IRL frame budget")?;
        let frame = IrlPipeFrame::from_protobuf(bytes, limits)?;
        require(
            frame.call == self.opening.call,
            "different IRL session or request",
        )?;
        if self.phase == Phase::New {
            require(
                frame == self.opening,
                "IRL dispatch differs from host delegation",
            )?;
        }
        let core = self.link == IRL_CORE_WORKER;
        let active = self.phase == Phase::Active;
        let running = active || self.phase == Phase::Cancelling;
        match (direction, frame.operation) {
            (ToCallee, IRL_DISPATCH) if self.phase == Phase::New && core => {
                self.phase = Phase::Active;
            }
            (ToCallee, IRL_DELEGATE) if self.phase == Phase::New && !core => {
                let d = frame.delegation.as_ref().ok_or("missing IRL delegation")?;
                require(d.link == self.link, "wrong IRL endpoint")?;
                let permitted = permitted_inputs(d)?;
                let mut remaining = self.objects_left;
                for input in &d.inputs {
                    require(
                        permitted.contains(&input.object),
                        "IRL material permission expired",
                    )?;
                    remaining = remaining
                        .checked_sub(input.object.byte_length.0)
                        .ok_or("IRL input budget")?;
                    verify_material(input, limits)?;
                }
                self.objects_left = remaining;
                self.phase = Phase::Active;
            }
            (ToCaller, IRL_FETCH) if core && active => {
                let loan = frame.borrow.as_ref().ok_or("missing IRL loan")?;
                require(
                    self.offered(loan)
                        && !self.pending.contains(loan)
                        && loan.object.byte_length.0 <= self.objects_left,
                    "IRL fetch pending or over budget",
                )?;
                self.pending.push(loan.clone());
            }
            (ToCallee, IRL_MATERIAL) if core && active => {
                let loan = frame.borrow.as_ref().ok_or("missing IRL loan")?;
                let material = frame.material.as_ref().ok_or("missing IRL material")?;
                let idx = self
                    .pending
                    .iter()
                    .position(|b| b == loan)
                    .ok_or("unsolicited IRL material")?;
                require(material.object == loan.object, "IRL material substitution")?;
                let left = self
                    .objects_left
                    .checked_sub(material.object.byte_length.0)
                    .ok_or("IRL object budget")?;
                verify_material(material, limits)?;
                self.pending.swap_remove(idx);
                self.objects_left = left;
            }
            (ToCallee, IRL_CANCEL_REQUEST) if active => self.phase = Phase::Cancelling,
            (ToCaller, IRL_CANCEL_EFFECTIVE | IRL_FAILURE | IRL_STALE) if running => {
                self.phase = Phase::Closed
            }
            (ToCaller, IRL_NOT_SENT | IRL_SENT_UNKNOWN)
                if running && self.link != IRL_WORKER_ADAPTER =>
            {
                self.phase = Phase::Closed
            }
            (ToCaller, IRL_CANDIDATE | IRL_OUTPUT | IRL_TRUNCATED) if running => {
                require(
                    match frame.operation {
                        IRL_CANDIDATE => core,
                        IRL_OUTPUT => !core,
                        _ => true,
                    },
                    "IRL result direction",
                )?;
                if let Some(output) = &frame.output {
                    let left = self
                        .output_left
                        .checked_sub(output.object.byte_length.0)
                        .ok_or("IRL output budget")?;
                    verify_material(output, limits)?;
                    self.output_left = left;
                }
                self.phase = Phase::Closed;
            }
            _ => return Err("IRL operation outside authenticated link or call phase".into()),
        }
        Ok(frame)
    }

    fn offered(&self, loan: &IrlBorrow) -> bool {
        self.opening
            .dispatch
            .as_ref()
            .is_some_and(|d| [&d.payload, &d.facts, &d.registry].contains(&loan))
    }

    /// The host supplies a reader scoped to the *current* authorized frontier and
    /// this transaction's explicitly loaned staged objects. Successful service
    /// consumes the pending fetch and debits this peer before returning the bytes.
    pub fn fetch(
        &mut self,
        loan: &IrlBorrow,
        read_visible: impl FnOnce(&ProposalBindingV2, &ObjectRef) -> Result<Vec<u8>>,
        limits: Limits,
    ) -> Result<IrlMaterial> {
        require(self.phase == Phase::Active, "no pending IRL fetch")?;
        let index = self
            .pending
            .iter()
            .position(|pending| pending == loan)
            .ok_or("no pending IRL loan")?;
        let binding = self.binding.ok_or("missing IRL binding")?;
        let r = &loan.object;
        require(
            binding.read_set.contains(r),
            "object outside bound IRL loan",
        )?;
        require(r.byte_length.0 <= self.objects_left, "IRL object budget")?;
        let data = read_visible(binding, r)?;
        let material = IrlMaterial {
            schema_version: 1,
            object: r.clone(),
            data: HexBytes(data),
        };
        verify_material(&material, limits)?;
        self.objects_left -= r.byte_length.0;
        self.pending.swap_remove(index);
        Ok(material)
    }
}
