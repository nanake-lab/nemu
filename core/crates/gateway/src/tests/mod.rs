use super::*;
use core_types::{
    WireMessage,
    contract::Limits,
    generated::*,
    scalar::{DecimalI64, DecimalU64},
};
use std::{sync::OnceLock, time::Duration};
mod frames;
mod ingress;
mod validation;
#[cfg(windows)]
mod windows;

fn limits() -> Limits {
    Limits {
        max_bytes: 4 * 1024 * 1024,
        max_items: 4096,
        max_depth: 64,
    }
}
fn fixture<T: WireMessage>() -> T {
    static VALUES: OnceLock<serde_json::Value> = OnceLock::new();
    let values = VALUES.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../contracts/golden/legal.json")).unwrap()
    });
    let value = values
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["request"]["name"] == T::NAME)
        .unwrap();
    serde_json::from_value(value["request"]["value"].clone()).unwrap()
}
fn session() -> AuthenticatedSession {
    TerminalToken::synthetic()
        .authenticate(&[0xA5; 32], "device", "session")
        .unwrap()
}
fn config() -> core_store::StoreConfig {
    core_store::StoreConfig {
        encoding: limits(),
        busy_timeout: Duration::from_millis(50),
        max_pending_records: 8,
        max_pending_bytes: 1024 * 1024,
        max_object_bytes: 4 * 1024 * 1024,
        max_store_bytes: 64 * 1024 * 1024,
        max_sql_pages: 8192,
    }
}
fn header<L: core_store::LineageKind>() -> LineageHeaderV2 {
    LineageHeaderV2 {
        schema_version: 1,
        lineage: L::KIND,
        created_at_ns: DecimalI64(1),
        genesis_manifest_digest: None,
        world_id: (L::KIND == Lineage::EXPERIMENT).then(|| "synthetic-world".into()),
        world_seed: (L::KIND == Lineage::EXPERIMENT).then_some(DecimalU64(1)),
    }
}
fn event(payload: &TerminalPayload, n: u64) -> (EventEnvelope, SourceMetadata) {
    // Synthetic provenance; source identity resolution is a runtime prerequisite.
    let mut source: SourceMetadata = fixture();
    source.source_refs = vec![];
    source.source_span = payload.reference(limits()).unwrap();
    source.epistemic_type = match payload {
        TerminalPayload::Text(_)
        | TerminalPayload::Utterance(_)
        | TerminalPayload::Retraction(_) => EpistemicType::USER_ASSERTION,
        TerminalPayload::Codex(_) => EpistemicType::TOOL_OUTPUT,
        _ => EpistemicType::SYSTEM_RECORD,
    };
    let mut event: EventEnvelope = fixture();
    event.id = format!("synthetic-event-{n}");
    event.raw_log_position = DecimalU64(n);
    event.device = "device".into();
    event.session = "session".into();
    event.stable_source_key = format!("01ARZ3NDEKTSV4RRFFQ{:07}", n);
    event.parents.clear();
    event.correction_of.clear();
    event.action_id = match payload {
        TerminalPayload::Body(p) => Some(p.action_id.clone()),
        TerminalPayload::Speech(p) => Some(p.action_id.clone()),
        _ => None,
    };
    event.trial_id = None;
    event.kind = match payload {
        TerminalPayload::Body(_) | TerminalPayload::Speech(_) => EventKind::EXECUTION_RESULT,
        TerminalPayload::Maintenance(_) => EventKind::MAINTENANCE,
        _ => EventKind::OBSERVATION,
    };
    event.derivation = source.derivation;
    event.payload_ref = payload.reference(limits()).unwrap();
    event.payload_digest = event.payload_ref.digest.clone();
    // Reuse generated canonical hashing without publishing an object.
    let schema = core_types::contract::Schema::builtin().unwrap();
    let value = serde_json::to_value(&source).unwrap();
    event.provenance = ObjectRef {
        schema_version: 1,
        digest: core_types::scalar::HexBytes(
            core_types::scalar::unhex(
                &schema
                    .digest(SourceMetadata::NAME, &value, limits())
                    .unwrap(),
            )
            .unwrap(),
        ),
        object_type: schema.message(SourceMetadata::NAME).unwrap().tag.clone(),
        resource_version: "1".into(),
        byte_length: DecimalU64(source.canonical_bytes(limits()).unwrap().len() as u64),
    };
    (event, source)
}
fn text() -> TerminalPayload {
    TerminalPayload::Text(UserText {
        schema_version: 1,
        text: "synthetic input".into(),
    })
}
