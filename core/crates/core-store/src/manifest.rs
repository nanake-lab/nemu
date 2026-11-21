//! Decode each immutable manifest with its own descriptor. V2-shaped event views
//! serve existing event consumers; only the original reference identifies the commit.
use crate::{Error, Result, objects::Objects};
use core_types::{
    WireMessage,
    generated::{CoreTransactionV2, CoreTransactionV4, ObjectRef},
};
pub(crate) fn from_event(value: CoreTransactionV2) -> CoreTransactionV4 {
    CoreTransactionV4 {
        schema_version: value.schema_version,
        staged_h: value.staged_h,
        staged_x: value.staged_x,
        staged_r: value.staged_r,
        staged_theta: value.staged_theta,
        staged_l: value.staged_l,
        staged_q: value.staged_q,
        staged_u: value.staged_u,
        processed_position: value.processed_position,
        anchor_time_ns: value.anchor_time_ns,
        staged_concepts: value.staged_concepts,
        accepted_results: value.accepted_results,
        audit_refs: value.audit_refs,
        staged_ledger: value.staged_ledger,
        event_binding: Some(value.binding),
        activity_commit: None,
    }
}
pub(crate) fn event_view(value: CoreTransactionV4) -> Result<CoreTransactionV2> {
    crate::error::require(
        value.activity_commit.is_none(),
        "activity cannot acknowledge ingress",
    )?;
    Ok(CoreTransactionV2 {
        schema_version: value.schema_version,
        staged_h: value.staged_h,
        staged_x: value.staged_x,
        staged_r: value.staged_r,
        staged_theta: value.staged_theta,
        staged_l: value.staged_l,
        staged_q: value.staged_q,
        staged_u: value.staged_u,
        processed_position: value.processed_position,
        anchor_time_ns: value.anchor_time_ns,
        staged_concepts: value.staged_concepts,
        accepted_results: value.accepted_results,
        audit_refs: value.audit_refs,
        staged_ledger: value.staged_ledger,
        binding: value
            .event_binding
            .ok_or(Error::Rejected("activity is not an event receipt"))?,
    })
}
pub(crate) fn decode(objects: &Objects, r: &ObjectRef) -> Result<CoreTransactionV4> {
    let bytes = objects.read(r)?;
    let limits = objects.config.encoding;
    match r.object_type.as_str() {
        "echo.transaction-v2" => Ok(from_event(CoreTransactionV2::from_canonical(
            &bytes, limits,
        )?)),
        "echo.transaction-v4" => Ok(CoreTransactionV4::from_canonical(&bytes, limits)?),
        _ => Err(Error::Rejected("unsupported published transaction tag")),
    }
}
