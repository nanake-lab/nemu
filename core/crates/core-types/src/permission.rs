//! P5-40 comparison interfaces. The service supplies trusted arguments independently
//! of the request. These checks have no side effects and never consume a credential.
use crate::contract::{Limits, Result, require};
use crate::generated::{
    ActionRequestV2, ExecutionCredential, GrowthManagementRequest, IngressKind, LocalUserBinding,
    ManagementEntry, TerminalRequestV2, TerminalSession,
};
use crate::{WireMessage, semantic::check_permission_message, terminal_ingress::check_session};

fn check_fields<T: WireMessage>(value: &T, limits: Limits) -> Result<()> {
    let value = serde_json::to_value(value).map_err(|e| e.to_string())?;
    check_permission_message(T::NAME, &value, limits)
}

/// Match a proposed dispatch to the live issuer lookup and authenticated executor.
/// The consumer must atomically recheck policy, scope, revocation and remaining budget.
pub fn check_dispatch_binding(
    request: &ActionRequestV2,
    registered: &ExecutionCredential,
    executor_session_id: &str,
    now_ns: i64,
    limits: Limits,
) -> Result<()> {
    check_fields(request, limits)?;
    require(
        &request.grant == registered && registered.executor_session_id == executor_session_id,
        "wrong issuance or executor",
    )?;
    require(
        registered.issued_at_ns.0 <= now_ns && now_ns < registered.expires_at_ns.0,
        "outside credential validity",
    )
}

/// The local service resolves u* and the entry point from the authenticated
/// connection. Panel confirmation still needs a live, single-use service lookup.
pub fn check_management_binding<'a>(
    request: &'a TerminalRequestV2,
    session: &TerminalSession,
    current_user: &LocalUserBinding,
    entry: ManagementEntry,
    limits: Limits,
) -> Result<&'a GrowthManagementRequest> {
    check_session(session, limits)?;
    check_fields(request, limits)?;
    let m = request
        .management
        .as_ref()
        .ok_or("management request required")?;
    require(
        request.kind == IngressKind::INGRESS_GROWTH && session.capabilities.contains(&request.kind),
        "management capability required",
    )?;
    require(
        request.session_id == session.session_id
            && request.protocol_version == session.protocol_version
            && current_user.installation_id == session.installation_id
            && &m.user == current_user
            && m.entry == entry,
        "management identity or entry mismatch",
    )?;
    require(
        request.protobuf_bytes(limits)?.len() <= session.budget.max_frame_bytes as usize,
        "management frame exceeds budget",
    )?;
    Ok(m)
}
