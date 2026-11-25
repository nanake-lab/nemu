//! Reference verification uses the existing CanonicalV1 domain-separated digest.
use crate::{Fault, Result, wire};
use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::*,
};

/// Checks the actual object, never trusting a worker's declared digest/type/size.
pub fn check_object<T: WireMessage>(
    reference: &ObjectRef,
    value: &T,
    limits: Limits,
) -> Result<()> {
    wire(reference, limits)?;
    if reference != &object_ref(value, &reference.resource_version, limits)? {
        return Err(Fault::Object);
    }
    Ok(())
}

pub(crate) fn object_ref<T: WireMessage>(
    value: &T,
    version: &str,
    limits: Limits,
) -> Result<ObjectRef> {
    if T::SUPERSEDED || !T::PERSISTABLE {
        return Err(Fault::Object);
    }
    let schema = Schema::builtin().map_err(|_| Fault::Schema)?;
    let descriptor = schema.message(T::NAME).map_err(|_| Fault::Schema)?;
    let json = serde_json::to_value(value).map_err(|_| Fault::Schema)?;
    let digest = schema
        .digest(T::NAME, &json, limits)
        .map_err(|_| Fault::Schema)?;
    Ok(ObjectRef {
        schema_version: 1,
        object_type: descriptor.tag.clone(),
        resource_version: version.into(),
        byte_length: core_types::scalar::DecimalU64(wire(value, limits)?.len() as u64),
        digest: core_types::scalar::HexBytes(
            core_types::scalar::unhex(&digest).map_err(|_| Fault::Object)?,
        ),
    })
}

/// Expected binding comes from the core's immutable dispatch snapshot, not the
/// candidate. Exact canonical comparison also checks read-set/environment changes.
pub fn check_binding(
    actual: &ProposalBindingV2,
    expected: &ProposalBindingV2,
    limits: Limits,
) -> Result<()> {
    if wire(actual, limits)? != wire(expected, limits)? {
        return Err(Fault::Binding);
    }
    Ok(())
}

pub fn check_candidate(
    candidate: &SemanticCandidateV2,
    graph: &SemanticGraph,
    expected: &ProposalBindingV2,
    producer: (&str, &str),
    limits: Limits,
) -> Result<()> {
    wire(candidate, limits)?;
    check_binding(&candidate.binding, expected, limits)?;
    if candidate.producer != producer.0 || candidate.producer_version != producer.1 {
        return Err(Fault::Binding);
    }
    check_object(&candidate.candidate_graph, graph, limits)?;
    // actual_output still requires an authority object read; this function does
    // not claim to authenticate it or publish an accepted proposal.
    Ok(())
}

/// A deterministic adapter can validate and preserve existing typed payloads.
/// These payload preflights do not register graph predicates or confer action authority.
pub enum StructuredPayload<'a> {
    Touch(&'a TouchObservation),
    TouchV2(&'a TouchObservationV2),
    Desktop(&'a DesktopObservation),
    Execution(&'a BodyActionResult),
    Delivery(&'a SpeechDelivery),
    Codex(&'a CodexSessionObservation),
    Maintenance(&'a TerminalMaintenance),
}
pub fn check_structured(
    event: &EventEnvelope,
    payload: StructuredPayload<'_>,
    limits: Limits,
) -> Result<()> {
    wire(event, limits)?;
    if event.payload_digest != event.payload_ref.digest {
        return Err(Fault::Object);
    }
    match payload {
        StructuredPayload::Touch(v) => check_object(&event.payload_ref, v, limits),
        StructuredPayload::TouchV2(v) => check_object(&event.payload_ref, v, limits),
        StructuredPayload::Desktop(v) => check_object(&event.payload_ref, v, limits),
        StructuredPayload::Execution(v) => {
            if event.action_id.as_deref() != Some(v.action_id.as_str()) {
                return Err(Fault::Binding);
            }
            check_object(&event.payload_ref, v, limits)
        }
        StructuredPayload::Delivery(v) => check_object(&event.payload_ref, v, limits),
        StructuredPayload::Codex(v) => check_object(&event.payload_ref, v, limits),
        StructuredPayload::Maintenance(v) => check_object(&event.payload_ref, v, limits),
    }
}
