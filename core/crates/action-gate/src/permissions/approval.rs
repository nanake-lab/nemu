//! I26 byte binding using the existing ApprovalRequestV2/ApprovalDecisionV2 IDL.
//!
//! This is an in-process payload check, NOT authentication, an execution
//! credential, an authorization store, or global replay prevention (P5-40).

use core_types::{
    WireMessage,
    contract::Limits,
    generated::{ApprovalDecisionV2, ApprovalRequestV2, ToolTier},
    scalar::{DecimalI64, HexBytes},
};
use sha2::{Digest, Sha256};

use super::tier::{ChoiceError, check_choice};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingError {
    InvalidMessage,
    PayloadOverBudget,
    ActionMismatch,
    ApprovalMismatch,
    DigestMismatch,
    PayloadMismatch,
    PresentationMismatch,
    Expired,
    Choice(ChoiceError),
}

/// SHA256 of the original bytes. No text decoding, canonical encoding, object
/// digest domain, line-ending conversion, or Unicode normalization is applied.
pub fn payload_digest(payload: &[u8]) -> [u8; 32] {
    Sha256::digest(payload).into()
}

fn presentation(tier: ToolTier, payload: &[u8]) -> (String, String) {
    let tier_name = match tier {
        ToolTier::T0 => "T0",
        ToolTier::T1 => "T1",
        ToolTier::T2 => "T2",
        ToolTier::T3 => "T3",
    };
    let digest = core_types::scalar::hex(&payload_digest(payload));
    (
        format!("{tier_name} 行动审批"),
        format!("原样载荷：{} 字节；SHA256：{digest}", payload.len()),
    )
}

/// Build gate-owned metadata; the original payload remains in action_payload
/// for lossless presentation by the terminal. No model-written summary input.
pub fn build_request(
    approval_id: String,
    action_id: String,
    tier: ToolTier,
    payload: &[u8],
    expires_at_ns: i64,
    limits: Limits,
) -> Result<ApprovalRequestV2, BindingError> {
    if payload.len() > limits.max_bytes {
        return Err(BindingError::PayloadOverBudget);
    }
    let (title, detail) = presentation(tier, payload);
    let request = ApprovalRequestV2 {
        schema_version: 1,
        approval_id,
        action_id,
        tier,
        title,
        detail,
        expires_at_ns: DecimalI64(expires_at_ns),
        action_payload: HexBytes(payload.to_vec()),
        payload_digest: HexBytes(payload_digest(payload).to_vec()),
    };
    request
        .canonical_bytes(limits)
        .map_err(|_| BindingError::InvalidMessage)?;
    Ok(request)
}

/// Immutable local binding, not a capability. Its fields cannot be changed by
/// the callback to substitute another action or digest.
#[derive(Debug)]
pub struct PayloadBinding {
    approval_id: String,
    action_id: String,
    digest: [u8; 32],
}

impl PayloadBinding {
    pub fn approval_id(&self) -> &str {
        &self.approval_id
    }
    pub fn action_id(&self) -> &str {
        &self.action_id
    }
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

/// Owns the checked bytes. No Clone/serialization/public constructor is exposed.
/// Re-validating the same wire request is still possible; global replay and
/// identity/credential checks belong to the future authenticated service.
///
/// ```compile_fail
/// use action_gate::permissions::approval::PayloadSnapshot;
/// fn reuse(snapshot: PayloadSnapshot) {
///     let _ = snapshot.consume(0, |_, _| Ok::<_, ()>(()));
///     let _ = snapshot.consume(0, |_, _| Ok::<_, ()>(()));
/// }
/// ```
pub struct PayloadSnapshot {
    binding: PayloadBinding,
    payload: Vec<u8>,
    expires_at_ns: i64,
}

impl std::fmt::Debug for PayloadSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PayloadSnapshot")
            .field("binding", &self.binding)
            .field("payload_length", &self.payload.len())
            .field("expires_at_ns", &self.expires_at_ns)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConsumeError<E> {
    Expired,
    Execute(E),
}

impl PayloadSnapshot {
    /// Consume by value even if expired or the callback fails. The caller must
    /// supply the current trusted time; no clock or authority is read here.
    /// The callback receives the checked immutable bytes, not an external array.
    pub fn consume<T, E>(
        self,
        now_ns: i64,
        execute: impl FnOnce(&[u8], &PayloadBinding) -> Result<T, E>,
    ) -> Result<T, ConsumeError<E>> {
        if now_ns >= self.expires_at_ns {
            return Err(ConsumeError::Expired);
        }
        execute(&self.payload, &self.binding).map_err(ConsumeError::Execute)
    }
}

/// Validate existing IDL messages and copy the exact bytes into a single-use
/// local snapshot. Rust's immutable borrows prevent mutation during this
/// synchronous check; later mutations of the source do not change the snapshot.
/// The authenticated service must establish the request/decision source and
/// metadata (including tier and expiry). A payload hash is not a signature.
pub fn validate_binding(
    request: &ApprovalRequestV2,
    decision: &ApprovalDecisionV2,
    action_id: &str,
    actual_payload: &[u8],
    now_ns: i64,
    limits: Limits,
) -> Result<PayloadSnapshot, BindingError> {
    if request.action_payload.0.len() > limits.max_bytes || actual_payload.len() > limits.max_bytes
    {
        return Err(BindingError::PayloadOverBudget);
    }
    request
        .canonical_bytes(limits)
        .map_err(|_| BindingError::InvalidMessage)?;
    decision
        .canonical_bytes(limits)
        .map_err(|_| BindingError::InvalidMessage)?;
    if request.action_id != action_id {
        return Err(BindingError::ActionMismatch);
    }
    if request.approval_id != decision.approval_id {
        return Err(BindingError::ApprovalMismatch);
    }
    check_choice(request.tier, decision.decision).map_err(BindingError::Choice)?;
    if now_ns >= request.expires_at_ns.0 {
        return Err(BindingError::Expired);
    }
    let digest = payload_digest(&request.action_payload.0);
    if request.payload_digest.0.as_slice() != digest
        || decision.payload_digest.0.as_slice() != digest
    {
        return Err(BindingError::DigestMismatch);
    }
    if request.action_payload.0 != actual_payload {
        return Err(BindingError::PayloadMismatch);
    }
    let (title, detail) = presentation(request.tier, &request.action_payload.0);
    if request.title != title || request.detail != detail {
        return Err(BindingError::PresentationMismatch);
    }
    Ok(PayloadSnapshot {
        binding: PayloadBinding {
            approval_id: request.approval_id.clone(),
            action_id: request.action_id.clone(),
            digest,
        },
        payload: request.action_payload.0.clone(),
        expires_at_ns: request.expires_at_ns.0,
    })
}

#[cfg(test)]
mod tests;
