//! Context checks for the P5-22 generated successors. No RPC envelope, query
//! execution, permission grant, or persistent state is defined here. The trusted
//! adapter must correlate requests and call these checks after typed frame decode.
use crate::{Error, Result};
use core_types::{
    WireMessage,
    contract::Limits,
    generated::{
        ApprovalChoice, ApprovalDecisionV2, ApprovalRequestV2, QueryKindV2, QueryResultV2, QueryV2,
        ToolTier,
    },
    scalar::DecimalI64,
};
use sha2::{Digest, Sha256};

/// Only user_view results: L entries are preserved, never remapped to QueryItem
/// or sent into a task/core projection. No status is replaced with an empty OK.
pub fn validate_query_result(
    query: &QueryV2,
    result: &QueryResultV2,
    limits: Limits,
) -> Result<()> {
    query.canonical_bytes(limits).map_err(|_| Error::Contract)?;
    result
        .canonical_bytes(limits)
        .map_err(|_| Error::Contract)?;
    if query.query_id != result.query_id {
        return Err(Error::Rejected("query correlation mismatch"));
    }
    if (query.kind == QueryKindV2::QUERY_GROWTH && !result.items.is_empty())
        || (query.kind != QueryKindV2::QUERY_GROWTH && !result.growth_entries.is_empty())
    {
        return Err(Error::Rejected("query result kind mismatch"));
    }
    Ok(())
}

/// Immutable, one-use payload check, NOT an authenticated identity or execution
/// credential. P5-05 still owns authentication, action authority and auditing.
/// No Clone/Debug/Serialize: the actual checked bytes remain privately owned.
pub struct CheckedApproval {
    request: ApprovalRequestV2,
    decision: ApprovalDecisionV2,
    checked_at: DecimalI64,
}
impl CheckedApproval {
    /// Fresh trusted time is checked at consumption. Even a failing/panicking
    /// consumer consumes self. Only the same immutable payload snapshot is given
    /// to the consumer; this does not authorize that consumer to execute anything.
    pub fn consume<T>(
        self,
        now_ns: DecimalI64,
        consumer: impl FnOnce(&ApprovalRequestV2, &ApprovalDecisionV2) -> T,
    ) -> Result<T> {
        if now_ns.0 < self.checked_at.0 || now_ns.0 >= self.request.expires_at_ns.0 {
            return Err(Error::Rejected("approval unavailable"));
        }
        Ok(consumer(&self.request, &self.decision))
    }
}

/// Inputs are owned before any forwarding. expected_action_id, execution_payload
/// and time must come from the trusted pending action, not peer claims. DENY is
/// valid on the wire but cannot yield a payload check for execution/forwarding.
pub fn validate_approval(
    request: ApprovalRequestV2,
    decision: ApprovalDecisionV2,
    expected_action_id: &str,
    execution_payload: &[u8],
    now_ns: DecimalI64,
    limits: Limits,
) -> Result<CheckedApproval> {
    request
        .canonical_bytes(limits)
        .map_err(|_| Error::Contract)?;
    decision
        .canonical_bytes(limits)
        .map_err(|_| Error::Contract)?;
    if request.approval_id != decision.approval_id
        || request.action_id != expected_action_id
        || request.action_payload.0 != execution_payload
        || request.payload_digest != decision.payload_digest
        || request.payload_digest.0.as_slice()
            != Sha256::digest(&request.action_payload.0).as_slice()
        || now_ns.0 >= request.expires_at_ns.0
        || decision.decision == ApprovalChoice::DENY
        || (request.tier == ToolTier::T3 && decision.decision == ApprovalChoice::ALLOW_ALWAYS)
    {
        return Err(Error::Rejected("approval binding mismatch"));
    }
    Ok(CheckedApproval {
        request,
        decision,
        checked_at: now_ns,
    })
}
