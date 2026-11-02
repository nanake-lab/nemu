use super::*;
use crate::{
    framing::{FrameConfig, Framed},
    terminal::{validate_approval, validate_query_result},
};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

fn frames() -> FrameConfig {
    FrameConfig {
        encoding: limits(),
        deadline: Duration::from_secs(2),
    }
}
async fn roundtrip<T: WireMessage + PartialEq + std::fmt::Debug>() {
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("../../../../contracts/golden/legal.json")).unwrap();
    let mut count = 0;
    for v in vectors
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["request"]["name"] == T::NAME)
    {
        let value: T = serde_json::from_value(v["request"]["value"].clone()).unwrap();
        let bytes = value.protobuf_bytes(limits()).unwrap();
        // Sets are normalized by the approved codec. Golden expected values,
        // unlike input fixtures, already have canonical set order.
        let expected: T = serde_json::from_value(v["expected"]["value"].clone()).unwrap();
        assert_eq!(
            bytes,
            core_types::scalar::unhex(v["expected"]["protobuf"].as_str().unwrap()).unwrap()
        );
        let (a, b) = tokio::io::duplex(bytes.len() + 8);
        let mut sender = Framed::new(a, frames()).unwrap();
        let mut receiver = Framed::new(b, frames()).unwrap();
        sender.write(&value).await.unwrap();
        assert_eq!(receiver.read::<T>().await.unwrap(), expected);
        count += 1;
    }
    assert!(count >= 3, "{} golden coverage", T::NAME);
}
async fn rejects<T: WireMessage>(bytes: Vec<u8>) {
    let (mut a, b) = tokio::io::duplex(bytes.len() + 8);
    a.write_all(&(bytes.len() as u32).to_le_bytes())
        .await
        .unwrap();
    a.write_all(&bytes).await.unwrap();
    let mut receiver = Framed::new(b, frames()).unwrap();
    assert!(
        receiver.read::<T>().await.is_err(),
        "{} accepted malformed frame",
        T::NAME
    );
    assert!(matches!(
        receiver.read::<UserText>().await,
        Err(Error::Rejected("connection is closed"))
    ));
}

#[tokio::test]
async fn p522_successors_and_notice_growth_control_roundtrip_all_golden_variants() {
    roundtrip::<ReadoutV2>().await; // Both migration v1 and production v2 branches.
    roundtrip::<RunStateV2>().await;
    roundtrip::<QueryV2>().await;
    roundtrip::<QueryResultV2>().await;
    roundtrip::<ApprovalRequestV2>().await;
    roundtrip::<ApprovalDecisionV2>().await;
    roundtrip::<Notice>().await;
    roundtrip::<GrowthControl>().await;
}

async fn reject_old<T: WireMessage>() {
    let old: T = fixture();
    let bytes = old.protobuf_bytes(limits()).unwrap();
    assert!(T::from_protobuf(&bytes, limits()).is_ok()); // Historical bytes stay decodable.
    let (a, _b) = tokio::io::duplex(4096);
    assert!(matches!(
        Framed::new(a, frames()).unwrap().write(&old).await,
        Err(Error::Rejected("superseded outbound message"))
    ));
    rejects::<T>(bytes).await;
}
#[tokio::test]
async fn p522_old_types_remain_decodable_but_cannot_enter_or_leave_live_frames() {
    reject_old::<Readout>().await;
    reject_old::<RunState>().await;
    reject_old::<Query>().await;
    reject_old::<QueryResult>().await;
    reject_old::<ApprovalRequest>().await;
    reject_old::<ApprovalDecision>().await;
}

#[tokio::test]
async fn p522_rejects_experiment_votes_unknown_fields_versions_and_invalid_oneof() {
    let mut state = fixture::<RunStateV2>().protobuf_bytes(limits()).unwrap();
    assert_eq!(state[state.len() - 2], 9 << 3);
    *state.last_mut().unwrap() = 3; // Internal EXPERIMENT has no terminal lineage.
    rejects::<RunStateV2>(state).await;
    let mut growth = fixture::<GrowthControl>().protobuf_bytes(limits()).unwrap();
    *growth.last_mut().unwrap() = 3; // No review/vote action.
    rejects::<GrowthControl>(growth).await;
    let mut notice = fixture::<Notice>().protobuf_bytes(limits()).unwrap();
    notice.extend([0x38, 1]); // Unknown response/vote field.
    rejects::<Notice>(notice).await;
    let mut decision = fixture::<ApprovalDecisionV2>()
        .protobuf_bytes(limits())
        .unwrap();
    decision.extend([8, 1]); // Duplicate schema_version.
    rejects::<ApprovalDecisionV2>(decision).await;
    let mut query = fixture::<QueryV2>().protobuf_bytes(limits()).unwrap();
    assert_eq!(&query[..2], &[8, 1]);
    query[1] = 2;
    rejects::<QueryV2>(query).await;
    let mut readout = fixture::<ReadoutV2>();
    readout.v1 = None;
    readout.v2 = Some(fixture());
    let mut bytes = readout.protobuf_bytes(limits()).unwrap();
    // Append the other oneof arm using independently constructed protobuf TLV.
    let legacy = fixture::<BodyReadout>().protobuf_bytes(limits()).unwrap();
    bytes.push(5 << 3 | 2);
    let mut length = legacy.len();
    while length >= 128 {
        bytes.push((length as u8 & 127) | 128);
        length >>= 7;
    }
    bytes.push(length as u8);
    bytes.extend(legacy);
    rejects::<ReadoutV2>(bytes).await;
    rejects::<ReadoutV2>(vec![8, 1, 26, 1, b's', 32, 0]).await; // Missing body.
    let mut invalid_body: BodyReadoutV2 = fixture();
    for invalid in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
        invalid_body.facing = invalid;
        assert!(invalid_body.protobuf_bytes(limits()).is_err());
    }
}

#[test]
fn p522_growth_query_preserves_ledger_and_rejects_mixed_or_mismatched_results() {
    let mut query: QueryV2 = fixture();
    query.kind = QueryKindV2::QUERY_GROWTH;
    let mut result: QueryResultV2 = fixture();
    result.query_id = query.query_id.clone();
    result.items.clear();
    result.growth_entries = vec![fixture()];
    let before = result.canonical_bytes(limits()).unwrap();
    for status in [
        QueryStatus::OK,
        QueryStatus::NO_MATCH,
        QueryStatus::INDEX_LAGGING,
        QueryStatus::DENIED,
        QueryStatus::OBJECT_MISSING,
        QueryStatus::BUDGET_EXHAUSTED,
    ] {
        result.status = status;
        validate_query_result(&query, &result, limits()).unwrap();
        assert_eq!(result.status, status);
    }
    result.status = QueryStatus::OK;
    assert_eq!(result.canonical_bytes(limits()).unwrap(), before);
    result.items.push(fixture());
    assert!(validate_query_result(&query, &result, limits()).is_err());
    result.items.clear();
    for kind in [
        QueryKindV2::QUERY_MEMORY,
        QueryKindV2::QUERY_ACTIONS,
        QueryKindV2::QUERY_RUN_HISTORY,
    ] {
        query.kind = kind;
        assert!(validate_query_result(&query, &result, limits()).is_err());
    }
    result.growth_entries.clear();
    result.items.push(fixture());
    validate_query_result(&query, &result, limits()).unwrap();
    result.query_id.push('x');
    assert!(validate_query_result(&query, &result, limits()).is_err());
}

fn approval() -> (ApprovalRequestV2, ApprovalDecisionV2) {
    let mut request: ApprovalRequestV2 = fixture();
    request.action_id = "synthetic-action".into();
    request.action_payload = core_types::scalar::HexBytes(b"synthetic exact bytes".to_vec());
    request.payload_digest =
        core_types::scalar::HexBytes(Sha256::digest(&request.action_payload.0).to_vec());
    request.tier = ToolTier::T2;
    request.expires_at_ns = DecimalI64(100);
    let decision = ApprovalDecisionV2 {
        schema_version: 1,
        approval_id: request.approval_id.clone(),
        decision: ApprovalChoice::ALLOW_ALWAYS,
        payload_digest: request.payload_digest.clone(),
    };
    (request, decision)
}
fn check(
    request: ApprovalRequestV2,
    decision: ApprovalDecisionV2,
) -> Result<crate::terminal::CheckedApproval> {
    validate_approval(
        request,
        decision,
        "synthetic-action",
        b"synthetic exact bytes",
        DecimalI64(50),
        limits(),
    )
}
#[test]
fn p522_approval_snapshot_binds_payload_action_digest_decision_and_expiry() {
    let (request, decision) = approval();
    let mut original = request.clone();
    let checked = check(request, decision.clone()).unwrap();
    original.action_payload.0.fill(0);
    assert_eq!(
        checked
            .consume(DecimalI64(99), |r, d| (
                r.action_payload.0.clone(),
                d.decision
            ))
            .unwrap(),
        (
            b"synthetic exact bytes".to_vec(),
            ApprovalChoice::ALLOW_ALWAYS
        )
    );
    let (request, decision) = approval();
    for change in 0..7 {
        let mut r = request.clone();
        let mut d = decision.clone();
        match change {
            0 => r.action_payload.0.push(0),
            1 => r.payload_digest.0[0] ^= 1,
            2 => d.payload_digest.0[0] ^= 1,
            3 => d.approval_id.push('x'),
            4 => r.action_id.push('x'),
            5 => r.expires_at_ns = DecimalI64(50),
            _ => d.decision = ApprovalChoice::DENY,
        }
        assert!(check(r, d).is_err(), "mutation {change}");
    }
    let mut t3 = request.clone();
    t3.tier = ToolTier::T3;
    assert!(check(t3.clone(), decision.clone()).is_err());
    let mut once = decision.clone();
    once.decision = ApprovalChoice::ALLOW_ONCE;
    check(t3, once).unwrap();
    for now in [49, 100, 101] {
        let checked = check(request.clone(), decision.clone()).unwrap();
        assert!(
            checked
                .consume(DecimalI64(now), |_, _| panic!("expired consumer called"))
                .is_err()
        );
    }
    let checked = check(request, decision).unwrap();
    assert!(
        checked
            .consume(DecimalI64(51), |_, _| Err::<(), _>("synthetic failure"))
            .unwrap()
            .is_err()
    );
}
