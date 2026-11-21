//! W13 operations on an action in Q. Only the outer transaction publishes changes.
use crate::permissions::approval::{build_request, validate_binding};
use core_types::{
    contract::{Limits, Result, require},
    generated::*,
    scalar::DecimalI64,
};

/// ADR 0043 §4.1 and CONTRACTS §6. Other deployment values have no implicit default.
pub struct Settings {
    pub irreversible: DesktopTier,
    pub approval_timeout_ns: i64,
    pub credential_lifetime_ns: Option<i64>,
    pub max_dispatching: Option<usize>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            irreversible: DesktopTier::IRREVERSIBLE_ASK,
            approval_timeout_ns: 60_000_000_000, // CONTRACTS §6: 60 seconds.
            credential_lifetime_ns: None,
            max_dispatching: None,
        }
    }
}

/// The caller resolves the origin from committed Q. Never accept a second,
/// caller-supplied plan alongside the producer record (ADR 0045 D1).
#[derive(Clone, Copy)]
pub enum Origin<'a> {
    Local(&'a InternalActivityRecord),
    Receipt(&'a InternalActivityRecordV2),
}

pub fn local_plan(origin: Origin<'_>) -> Result<&CognitiveResultAndReceipt> {
    let local = match origin {
        Origin::Local(record) => &record.result,
        Origin::Receipt(record) => {
            require(
                record.remote.is_none(),
                "remote cognition cannot select an action",
            )?;
            record.local.as_ref().ok_or("missing local result")?
        }
    };
    require(
        local.completion == CognitiveCompletion::COG_COMPLETED && local.carrier_receipt.is_some(),
        "local selection did not complete",
    )?;
    let plan = local.action_plan.as_ref().ok_or("no local action")?;
    require(
        plan.snapshot_id == local.binding.snapshot_id
            && plan.task_version == local.binding.q_revision
            && plan.protocol_version == local.binding.protocol.resource_version,
        "stale local plan",
    )?;
    Ok(local)
}

pub fn request(
    action: &CognitiveInflightAction,
    event: &EventEnvelope,
    now_ns: i64,
) -> Result<ActionStatus> {
    require(
        action.status == ActionStatus::PLANNED
            && event.kind == EventKind::ACTION_REQUEST
            && event.action_id.as_deref() == Some(&action.action_id)
            && event.payload_ref == action.request,
        "request must register its pending action exactly once",
    )?;
    require(now_ns < action.plan.deadline_ns.0, "plan expired")?;
    Ok(ActionStatus::REQUESTED)
}

pub fn ask(
    action: &CognitiveInflightAction,
    approval_id: String,
    tier: ToolTier,
    bytes: &[u8],
    now_ns: i64,
    settings: &Settings,
    limits: Limits,
) -> Result<ApprovalRequestV2> {
    require(
        action.status == ActionStatus::REQUESTED && settings.approval_timeout_ns > 0,
        "invalid approval window",
    )?;
    let expires = now_ns
        .checked_add(settings.approval_timeout_ns)
        .ok_or("approval time overflow")?
        .min(action.plan.deadline_ns.0);
    require(now_ns < expires, "plan expired")?;
    build_request(
        approval_id,
        action.action_id.clone(),
        tier,
        bytes,
        expires,
        limits,
    )
    .map_err(|_| "invalid approval request".into())
}

pub enum Decision {
    Consented(ActionConsent),
    Cancelled(PermissionDenial),
}

/// Address a denied or expired *matching* approval. An unrelated reply cannot
/// cancel somebody else's action. Identity is supplied by the local authenticator.
pub fn decide(
    request: &ApprovalRequestV2,
    reply: Option<&ApprovalDecisionV2>,
    user: &LocalUserBinding,
    action: &CognitiveInflightAction,
    actual: &[u8],
    now_ns: i64,
    limits: Limits,
) -> Result<Decision> {
    use core_types::WireMessage;
    require(
        action.status == ActionStatus::REQUESTED
            && request.action_id == action.action_id
            && request.action_payload.0 == actual,
        "wrong pending approval",
    )?;
    request.canonical_bytes(limits)?;
    if let Some(reply) = reply {
        reply.canonical_bytes(limits)?;
        require(
            reply.approval_id == request.approval_id
                && reply.payload_digest == request.payload_digest,
            "wrong approval reply",
        )?;
    }
    if now_ns >= request.expires_at_ns.0 || now_ns >= action.plan.deadline_ns.0 {
        return Ok(Decision::Cancelled(PermissionDenial::GRANT_EXPIRED));
    }
    let reply = reply.ok_or("approval still pending")?;
    if reply.decision == ApprovalChoice::DENY {
        return Ok(Decision::Cancelled(PermissionDenial::CONSENT_MISSING));
    }
    validate_binding(request, reply, &action.action_id, actual, now_ns, limits)
        .map_err(|_| "approval binding mismatch")?;
    let consent = ActionConsent {
        schema_version: 1,
        user: user.clone(),
        confirmed_at_ns: DecimalI64(now_ns),
        approval_id: request.approval_id.clone(),
        action_id: action.action_id.clone(),
        payload_digest: request.payload_digest.clone(),
    };
    consent.canonical_bytes(limits)?;
    Ok(Decision::Consented(consent))
}

pub fn cancel(status: ActionStatus) -> Result<ActionStatus> {
    match status {
        ActionStatus::PLANNED | ActionStatus::REQUESTED | ActionStatus::AUTHORIZED => {
            Ok(ActionStatus::CANCELLED_ACTION)
        }
        ActionStatus::DISPATCHING | ActionStatus::UNKNOWN_ACTION => {
            Ok(ActionStatus::UNKNOWN_ACTION)
        }
        _ => Err("action already settled".into()),
    }
}
/// Create the PLANNED entry while publishing the originating local step in Q.
/// The caller supplies the origin's W02 object reference and registration time.
pub fn planned(
    origin: Origin<'_>,
    origin_ref: ObjectRef,
    id: String,
    request: ObjectRef,
    at_ns: u64,
) -> Result<CognitiveInflightAction> {
    let local = local_plan(origin)?;
    require(
        !id.is_empty()
            && origin_ref.object_type
                == match origin {
                    Origin::Local(_) => "echo.internal-activity-record",
                    Origin::Receipt(_) => "echo.internal-activity-record-v2",
                },
        "invalid action identity or origin",
    )?;
    let plan = local.action_plan.clone().ok_or("no local plan")?;
    require(
        i128::from(at_ns) < i128::from(plan.deadline_ns.0),
        "plan already expired",
    )?;
    Ok(CognitiveInflightAction {
        schema_version: 1,
        action_id: id,
        origin_record: origin_ref,
        origin_step_id: local.binding.step_id.clone(),
        round_index: local.binding.round_index,
        registered_at_ns: core_types::scalar::DecimalU64(at_ns),
        plan,
        status: ActionStatus::PLANNED,
        request,
        actual_result: None,
        credential: None,
    })
}
