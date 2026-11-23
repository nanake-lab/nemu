use crate::{AuthenticatedSession, Error, Result};
use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::*,
};

/// Current process-local T1 decisions supplied by the action gate for this
/// request. Never retained in a connection or deserialized from a peer.
#[derive(Clone, Copy, Default)]
pub struct Grants {
    pub desktop_titles: bool,
    pub codex_sessions: bool,
}

/// A local sum of generated messages, not another wire schema.
#[derive(Clone)]
pub enum TerminalPayload {
    Utterance(UserUtterance),
    Text(UserText),
    Retraction(UserRetraction),
    Touch(TouchObservation),
    Desktop(DesktopObservation),
    Codex(CodexSessionObservation),
    Speech(SpeechDelivery),
    Body(BodyActionResult),
    Maintenance(TerminalMaintenance),
}
impl TerminalPayload {
    pub fn decode(reference: &ObjectRef, bytes: &[u8], limits: Limits) -> Result<Self> {
        macro_rules! decode {
            ($($t:ty => $variant:ident),+ $(,)?) => {
                match reference.object_type.as_str() {
                    $(tag if tag == Schema::builtin().map_err(|_| Error::Contract)?
                        .message(<$t>::NAME).map_err(|_| Error::Contract)?.tag =>
                        Ok(Self::$variant(<$t>::from_protobuf(bytes, limits).map_err(|_| Error::Contract)?)),)+
                    _ => Err(Error::Rejected("unsupported terminal payload")),
                }
            }
        }
        decode!(UserUtterance => Utterance, UserText => Text,
            UserRetraction => Retraction, TouchObservation => Touch,
            DesktopObservation => Desktop, CodexSessionObservation => Codex,
            SpeechDelivery => Speech, BodyActionResult => Body,
            TerminalMaintenance => Maintenance)
    }
    pub(crate) fn prepare<L: core_store::LineageKind>(
        &self,
        store: &core_store::Store<L>,
        version: &str,
    ) -> Result<core_store::PreparedObject<L>> {
        macro_rules! prepare {
            ($($variant:ident),+) => { match self {
                $(Self::$variant(value) => Ok(store.prepare(value, version)?)),+
            }}
        }
        prepare!(
            Utterance,
            Text,
            Retraction,
            Touch,
            Desktop,
            Codex,
            Speech,
            Body,
            Maintenance
        )
    }
    pub(crate) fn reference(&self, limits: Limits) -> Result<ObjectRef> {
        macro_rules! reference {
            ($($variant:ident),+) => { match self {
                $(Self::$variant(value) => object_ref(value, limits)),+
            }}
        }
        reference!(
            Utterance,
            Text,
            Retraction,
            Touch,
            Desktop,
            Codex,
            Speech,
            Body,
            Maintenance
        )
    }
}
fn object_ref<T: WireMessage>(value: &T, limits: Limits) -> Result<ObjectRef> {
    let schema = Schema::builtin().map_err(|_| Error::Contract)?;
    let value = serde_json_value(value)?;
    let bytes = schema
        .encode_canonical(T::NAME, &value, limits)
        .map_err(|_| Error::Contract)?;
    let digest = schema
        .digest(T::NAME, &value, limits)
        .map_err(|_| Error::Contract)?;
    Ok(ObjectRef {
        schema_version: 1,
        digest: core_types::scalar::HexBytes(
            core_types::scalar::unhex(&digest).map_err(|_| Error::Contract)?,
        ),
        object_type: schema
            .message(T::NAME)
            .map_err(|_| Error::Contract)?
            .tag
            .clone(),
        resource_version: "1".into(),
        byte_length: core_types::scalar::DecimalU64(bytes.len() as u64),
    })
}
fn serde_json_value<T: WireMessage>(value: &T) -> Result<serde_json::Value> {
    serde_json::to_value(value).map_err(|_| Error::Contract)
}

/// Immutable, one-use process-local validation result. Fields cannot be replaced
/// after validation or created from a peer's claim that it was already validated.
pub struct ValidatedEvent {
    pub(crate) event: EventEnvelope,
    pub(crate) payload: TerminalPayload,
    pub(crate) provenance: SourceMetadata,
}

/// Call only after the runtime assigned ingest_time/raw_log_position from D and
/// resolved provenance. This function does not allocate a source/root identity.
pub fn validate_event(
    auth: &AuthenticatedSession,
    grants: Grants,
    event: EventEnvelope,
    payload: TerminalPayload,
    provenance: SourceMetadata,
    limits: Limits,
) -> Result<ValidatedEvent> {
    event.canonical_bytes(limits).map_err(|_| Error::Contract)?;
    provenance
        .canonical_bytes(limits)
        .map_err(|_| Error::Contract)?;
    let key = event.stable_source_key.as_bytes();
    if key.len() != 26
        || key[0] > b'7'
        || !key
            .iter()
            .all(|c| b"0123456789ABCDEFGHJKMNPQRSTVWXYZ".contains(c))
    {
        return Err(Error::Rejected("stable source key must be a ULID"));
    }
    if event.device != auth.device || event.session != auth.session {
        return Err(Error::Rejected("terminal source identity mismatch"));
    }
    if event.payload_ref != payload.reference(limits)?
        || event.payload_digest != event.payload_ref.digest
        || event.provenance != object_ref(&provenance, limits)?
        || event.derivation != provenance.derivation
    {
        return Err(Error::Rejected("event object binding mismatch"));
    }
    let epistemic = provenance.epistemic_type;
    let mut action = None;
    let (kind, source) = match &payload {
        TerminalPayload::Utterance(_) | TerminalPayload::Text(_) => (
            EventKind::OBSERVATION,
            epistemic == EpistemicType::USER_ASSERTION
                || epistemic == EpistemicType::USER_SELF_REPORT,
        ),
        TerminalPayload::Retraction(p) => {
            if p.target_refs.is_empty() {
                return Err(Error::Rejected("empty retraction"));
            }
            (
                EventKind::OBSERVATION,
                epistemic == EpistemicType::USER_ASSERTION,
            )
        }
        TerminalPayload::Touch(p) => {
            if !["head", "ears", "tail", "body", "handL", "handR", "skirt"]
                .contains(&p.region.as_str())
            {
                return Err(Error::Rejected("invalid touch region"));
            }
            (
                EventKind::OBSERVATION,
                epistemic == EpistemicType::SYSTEM_RECORD,
            )
        }
        TerminalPayload::Desktop(p) => {
            if p.title.is_some() && !grants.desktop_titles {
                return Err(Error::Rejected("desktop title permission required"));
            }
            (
                EventKind::OBSERVATION,
                epistemic == EpistemicType::SYSTEM_RECORD,
            )
        }
        TerminalPayload::Codex(_) => {
            if !grants.codex_sessions {
                return Err(Error::Rejected("Codex session permission required"));
            }
            (
                EventKind::OBSERVATION,
                epistemic == EpistemicType::TOOL_OUTPUT,
            )
        }
        TerminalPayload::Speech(p) => {
            action = Some(&p.action_id);
            (
                EventKind::EXECUTION_RESULT,
                epistemic == EpistemicType::SYSTEM_RECORD,
            )
        }
        TerminalPayload::Body(p) => {
            if !matches!(
                p.status,
                ActionStatus::SUCCEEDED
                    | ActionStatus::FAILED
                    | ActionStatus::CANCELLED_ACTION
                    | ActionStatus::UNKNOWN_ACTION
            ) {
                return Err(Error::Rejected("body result must be terminal or unknown"));
            }
            action = Some(&p.action_id);
            (
                EventKind::EXECUTION_RESULT,
                epistemic == EpistemicType::SYSTEM_RECORD,
            )
        }
        TerminalPayload::Maintenance(_) => (
            EventKind::MAINTENANCE,
            epistemic == EpistemicType::SYSTEM_RECORD,
        ),
    };
    if event.kind != kind
        || !source
        || event.action_id.as_ref() != action
        || event.trial_id.is_some()
    {
        return Err(Error::Rejected(
            "terminal event kind or provenance mismatch",
        ));
    }
    Ok(ValidatedEvent {
        event,
        payload,
        provenance,
    })
}
