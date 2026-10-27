//! Pure connection-contract checks. No sockets, credentials database or persistence.
//! A decoded session is NOT authenticated. The gateway supplies the session bound
//! to its currently authenticated connection, never one taken from a request.
use crate::{
    WireMessage,
    contract::{Limits, Result, Schema, require},
    generated::*,
    scalar::hex,
};

pub fn check_auth_pair(
    request: &TerminalAuthenticate,
    reply: &TerminalAuthResult,
    limits: Limits,
) -> Result<()> {
    request.canonical_bytes(limits)?;
    reply.canonical_bytes(limits)?;
    require(
        request.request_id == reply.request_id,
        "authentication request mismatch",
    )?;
    if let Some(session) = &reply.session {
        require(
            session.installation_id == request.installation_id
                && request
                    .protocol_versions
                    .contains(&session.protocol_version),
            "authentication negotiation mismatch",
        )?;
        check_session(session, limits)?;
    }
    Ok(())
}

pub fn check_session(session: &TerminalSession, limits: Limits) -> Result<()> {
    session.canonical_bytes(limits)?;
    let b = &session.budget;
    require(
        b.max_chunk_bytes <= b.max_object_bytes
            && b.max_object_bytes <= b.max_upload_bytes
            && b.max_chunk_bytes < b.max_frame_bytes,
        "inconsistent terminal budget",
    )
}

pub fn check_request(
    request: &TerminalRequest,
    authenticated: Option<&TerminalSession>,
    limits: Limits,
) -> Result<()> {
    let bytes = request.protobuf_bytes(limits)?;
    let session = authenticated.ok_or("unauthenticated dispatch")?;
    check_session(session, limits)?;
    require(
        request.session_id == session.session_id
            && request.protocol_version == session.protocol_version,
        "connection session mismatch",
    )?;
    require(
        session.capabilities.contains(&request.kind),
        "capability denied",
    )?;
    require(
        bytes.len() <= session.budget.max_frame_bytes as usize,
        "frame budget exceeded",
    )?;
    if let Some(event) = &request.event {
        require(
            event.payload_digest == event.payload_ref.digest,
            "event payload mismatch",
        )?;
    }
    if let Some(upload) = &request.upload {
        check_chunk(upload, &session.budget, limits)?;
    }
    Ok(())
}

pub fn check_response(
    request: &TerminalRequest,
    reply: &TerminalResponse,
    session: &TerminalSession,
    limits: Limits,
) -> Result<()> {
    check_request(request, Some(session), limits)?;
    let bytes = reply.protobuf_bytes(limits)?;
    require(
        bytes.len() <= session.budget.max_frame_bytes as usize,
        "response frame budget exceeded",
    )?;
    require(
        request.request_id == reply.request_id
            && request.session_id == reply.session_id
            && request.protocol_version == reply.protocol_version
            && request.kind == reply.reply_to,
        "response pairing mismatch",
    )?;
    if reply.kind == IngressResponseKind::INGRESS_REJECTED {
        return Ok(());
    }
    let paired = match request.kind {
        IngressKind::INGRESS_EVENT => {
            reply.kind == IngressResponseKind::EVENT_RECEIPT
                && request
                    .event
                    .as_ref()
                    .zip(reply.receipt.as_ref())
                    .is_some_and(|(a, b)| {
                        Some(&a.id) == b.event_id.as_ref()
                            && a.stable_source_key == b.stable_source_key
                    })
        }
        IngressKind::INGRESS_UPLOAD => {
            reply.kind == IngressResponseKind::UPLOAD_RECEIPT
                && request
                    .upload
                    .as_ref()
                    .zip(reply.upload.as_ref())
                    .is_some_and(|(a, b)| {
                        a.upload_id == b.upload_id
                            && a.object == b.object
                            && a.offset.0.checked_add(a.data.0.len() as u64)
                                == Some(b.next_offset.0)
                            && (b.status == UploadStatus::UPLOAD_COMPLETE) == a.final_chunk
                    })
        }
        IngressKind::INGRESS_QUERY => {
            reply.kind == IngressResponseKind::QUERY_REPLY
                && request
                    .query
                    .as_ref()
                    .zip(reply.query.as_ref())
                    .is_some_and(|(a, b)| a.query_id == b.query_id)
        }
        _ => reply.kind == IngressResponseKind::CONTROL_ACK,
    };
    require(paired, "business response mismatch")
}

pub fn check_chunk(
    chunk: &ObjectUploadChunk,
    budget: &TerminalBudget,
    limits: Limits,
) -> Result<()> {
    chunk.canonical_bytes(limits)?;
    budget.canonical_bytes(limits)?;
    require(
        !chunk.data.0.is_empty()
            && chunk.data.0.len() <= budget.max_chunk_bytes as usize
            && chunk.object.byte_length.0 <= u64::from(budget.max_object_bytes),
        "upload size budget exceeded",
    )?;
    let end = chunk
        .offset
        .0
        .checked_add(chunk.data.0.len() as u64)
        .ok_or("upload offset overflow")?;
    require(
        end <= chunk.object.byte_length.0
            && chunk.final_chunk == (end == chunk.object.byte_length.0),
        "upload extent mismatch",
    )?;
    upload_type(&chunk.object.object_type)?;
    Ok(())
}

/// All registered live object contracts are eligible. Ingress control frames
/// and transient credentials cannot become objects. Upload never publishes H.
pub fn upload_type(tag: &str) -> Result<String> {
    Schema::builtin()?
        .messages
        .into_iter()
        .find(|(_, m)| {
            m.tag == tag
                && !m.transient
                && m.superseded.is_none()
                && m.file != "terminal_ingress.proto"
        })
        .map(|(name, _)| name)
        .ok_or("object type is not terminal-uploadable".into())
}

/// Ephemeral SERVER bookkeeping, not a wire message or authoritative state.
pub struct UploadWindow {
    pub upload_id: String,
    pub object: ObjectRef,
    pub next_offset: u64,
    /// Includes this upload, reserved before accepting its first chunk.
    pub active_uploads: u32,
    /// Sum of declared object sizes, including this upload (checked by caller).
    pub reserved_bytes: u64,
}
pub fn check_upload_window(
    chunk: &ObjectUploadChunk,
    budget: &TerminalBudget,
    window: &UploadWindow,
    limits: Limits,
) -> Result<()> {
    check_chunk(chunk, budget, limits)?;
    require(
        chunk.upload_id == window.upload_id && chunk.object == window.object,
        "upload declaration changed",
    )?;
    require(chunk.offset.0 == window.next_offset, "nonsequential upload")?;
    require(
        window.active_uploads > 0
            && window.active_uploads <= budget.max_uploads
            && window.reserved_bytes >= chunk.object.byte_length.0
            && window.reserved_bytes <= u64::from(budget.max_upload_bytes),
        "session upload budget exceeded",
    )
}

/// Call only after bounded sequential reassembly. Hash includes CanonicalV1's
/// type/version domain, never a plain SHA256 of a JSON/protobuf representation.
pub fn check_uploaded_object(
    reference: &ObjectRef,
    bytes: &[u8],
    budget: &TerminalBudget,
    limits: Limits,
) -> Result<()> {
    reference.canonical_bytes(limits)?;
    budget.canonical_bytes(limits)?;
    require(
        bytes.len() as u64 == reference.byte_length.0
            && bytes.len() <= budget.max_object_bytes as usize,
        "object size mismatch",
    )?;
    let name = upload_type(&reference.object_type)?;
    let schema = Schema::builtin()?;
    let value = schema.decode_canonical(&name, bytes, limits, 0)?;
    require(
        schema.digest(&name, &value, limits)? == hex(&reference.digest.0),
        "object digest mismatch",
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryDisposition {
    SameSourceKeyAfterCapacity,
    Reauthenticate,
    DoNotRetryUnchanged,
}
pub fn rejected_receipt(event: &TerminalEvent, reason: IngressReason) -> ReceiptV2 {
    ReceiptV2 {
        schema_version: 1,
        stable_source_key: event.stable_source_key.clone(),
        stage: ReceiptStage::REJECTED,
        event_id: Some(event.id.clone()),
        reason_code: Some(reason),
        raw_log_position: None,
        ingest_time_ns: None,
    }
}
pub fn retry_disposition(reason: IngressReason) -> RetryDisposition {
    use IngressReason::*;
    match reason {
        PENDING_RECORDS_FULL | PENDING_BYTES_FULL | AUDIT_FULL | UPLOAD_BUDGET_FULL => {
            RetryDisposition::SameSourceKeyAfterCapacity
        }
        AUTH_FAILED | SESSION_EXPIRED => RetryDisposition::Reauthenticate,
        _ => RetryDisposition::DoNotRetryUnchanged,
    }
}
