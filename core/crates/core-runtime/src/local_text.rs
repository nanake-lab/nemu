//! Bind the local authentication service's evidence to exactly the durable event.
use crate::{Error, Result, check};
use core_store::{LineageKind, PreparedObject, ReadTicket, Store};
use core_types::{generated::*, permission::check_direct_user_text, scalar::DecimalU64};

pub(crate) fn author_layer<L: LineageKind>(
    store: &Store<L>,
    ticket: &ReadTicket<L>,
    event: &EventEnvelope,
    user: &LocalUserBinding,
    pins: &mut Vec<PreparedObject<L>>,
) -> Result<Option<QuoteLayer>> {
    check(
        event.session == user.session_id && event.device == user.installation_id,
        "local authentication belongs to another ingress",
    )?;
    let source: SourceMetadata = store.read_bound(ticket, &event.provenance)?;
    // W02 added the device, session, receipt time and D position at ingress.
    // Reapply the same ADR 0047 predicate to the original terminal fields.
    let original = TerminalEvent {
        schema_version: event.schema_version,
        id: event.id.clone(),
        stable_source_key: event.stable_source_key.clone(),
        kind: event.kind,
        event_time_ns: event.event_time_ns,
        parents: event.parents.clone(),
        payload_ref: event.payload_ref.clone(),
        provenance: event.provenance.clone(),
        payload_digest: event.payload_digest.clone(),
        action_id: event.action_id.clone(),
        trial_id: event.trial_id.clone(),
        correction_of: event.correction_of.clone(),
        derivation: event.derivation,
    };
    check_direct_user_text(&original, &source).map_err(Error::Contract)?;
    let payload: UserText = store.read_bound(ticket, &event.payload_ref)?;
    if payload.text.is_empty() {
        return Ok(None); // ADR 0046: a quote requires a nonempty UTF-8 span.
    }
    Ok(Some(QuoteLayer {
        schema_version: 1,
        source: RecordVersion {
            schema_version: 1,
            branch_id: ticket.binding().branch_id.clone(),
            record_id: event.id.clone(),
            content: event.payload_ref.clone(),
        },
        principal_id: user.principal_id.clone(),
        identity_evidence: crate::store::pin(store, user, pins)?,
        visible: Utf8Range {
            schema_version: 1,
            start: DecimalU64(0),
            end: DecimalU64(payload.text.len() as u64),
        },
    }))
}
