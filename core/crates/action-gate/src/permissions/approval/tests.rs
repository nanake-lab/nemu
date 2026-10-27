use super::*;
use core_types::generated::ApprovalChoice;

fn limits() -> Limits {
    Limits {
        max_bytes: 8192,
        max_items: 64,
        max_depth: 16,
    }
}

fn request(tier: ToolTier) -> ApprovalRequestV2 {
    build_request(
        "approval-1".into(),
        "action-1".into(),
        tier,
        b"exact\0\xff\r\nbytes",
        100,
        limits(),
    )
    .unwrap()
}

fn decision(request: &ApprovalRequestV2) -> ApprovalDecisionV2 {
    ApprovalDecisionV2 {
        schema_version: 1,
        approval_id: request.approval_id.clone(),
        decision: ApprovalChoice::ALLOW_ONCE,
        payload_digest: request.payload_digest.clone(),
    }
}

fn check(
    request: &ApprovalRequestV2,
    decision: &ApprovalDecisionV2,
) -> Result<PayloadSnapshot, BindingError> {
    validate_binding(
        request,
        decision,
        &request.action_id,
        &request.action_payload.0,
        1,
        limits(),
    )
}

#[test]
fn raw_sha256_known_answers_and_no_text_normalization() {
    for (payload, expected) in [
        (
            b"".as_slice(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc".as_slice(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".as_slice(),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ] {
        assert_eq!(core_types::scalar::hex(&payload_digest(payload)), expected);
    }
    assert_ne!(payload_digest(b"a\r\nb"), payload_digest(b"a\nb"));
    assert_ne!(
        payload_digest("é".as_bytes()),
        payload_digest("e\u{301}".as_bytes())
    );
    assert_ne!(payload_digest(b"a\0b"), payload_digest(b"a"));
}

#[test]
fn request_has_gate_owned_metadata_and_original_binary_payload() {
    let first = request(ToolTier::T2);
    assert_eq!(first, request(ToolTier::T2));
    assert_eq!(first.action_payload.0, b"exact\0\xff\r\nbytes");
    assert_eq!(
        first.payload_digest.0,
        payload_digest(&first.action_payload.0)
    );
    assert_eq!(first.title, "T2 行动审批");
    assert!(
        first
            .detail
            .contains(&core_types::scalar::hex(&first.payload_digest.0))
    );
    let mut altered = first.clone();
    altered.detail = "model says harmless".into();
    assert_eq!(
        check(&altered, &decision(&first)).unwrap_err(),
        BindingError::PresentationMismatch
    );
    altered = first.clone();
    altered.title = "different action".into();
    assert_eq!(
        check(&altered, &decision(&first)).unwrap_err(),
        BindingError::PresentationMismatch
    );
}

#[test]
fn mutation_of_request_digest_decision_or_execution_payload_is_rejected() {
    let original = request(ToolTier::T2);
    let approved = decision(&original);
    let mut request_mutation = original.clone();
    request_mutation.action_payload.0[0] ^= 1;
    assert_eq!(
        check(&request_mutation, &approved).unwrap_err(),
        BindingError::DigestMismatch
    );
    request_mutation = original.clone();
    request_mutation.payload_digest.0[0] ^= 1;
    assert_eq!(
        check(&request_mutation, &approved).unwrap_err(),
        BindingError::DigestMismatch
    );
    let mut decision_mutation = approved.clone();
    decision_mutation.payload_digest.0[0] ^= 1;
    assert_eq!(
        check(&original, &decision_mutation).unwrap_err(),
        BindingError::DigestMismatch
    );
    let mut actual = original.action_payload.0.clone();
    actual[0] ^= 1;
    assert_eq!(
        validate_binding(&original, &approved, "action-1", &actual, 1, limits()).unwrap_err(),
        BindingError::PayloadMismatch
    );
    // Changing request+digest together still cannot match the original decision.
    let replacement = build_request(
        "approval-1".into(),
        "action-1".into(),
        ToolTier::T2,
        &actual,
        100,
        limits(),
    )
    .unwrap();
    assert_eq!(
        check(&replacement, &approved).unwrap_err(),
        BindingError::DigestMismatch
    );
}

#[test]
fn wrong_action_or_approval_ids_are_rejected() {
    let request = request(ToolTier::T2);
    let mut approved = decision(&request);
    assert_eq!(
        validate_binding(
            &request,
            &approved,
            "other-action",
            &request.action_payload.0,
            1,
            limits()
        )
        .unwrap_err(),
        BindingError::ActionMismatch
    );
    approved.approval_id = "other-approval".into();
    assert_eq!(
        check(&request, &approved).unwrap_err(),
        BindingError::ApprovalMismatch
    );
}

#[test]
fn denied_choices_and_t3_remembering_are_rejected() {
    for tier in [ToolTier::T0, ToolTier::T1, ToolTier::T2, ToolTier::T3] {
        let request = request(tier);
        let mut approved = decision(&request);
        check(&request, &approved).unwrap();
        approved.decision = ApprovalChoice::DENY;
        assert_eq!(
            check(&request, &approved).unwrap_err(),
            BindingError::Choice(ChoiceError::Denied)
        );
        approved.decision = ApprovalChoice::ALLOW_ALWAYS;
        if tier == ToolTier::T3 {
            assert_eq!(
                check(&request, &approved).unwrap_err(),
                BindingError::Choice(ChoiceError::T3CannotBeRemembered)
            );
        } else {
            check(&request, &approved).unwrap();
        }
    }
}

#[test]
fn expiry_is_rechecked_at_consumption_and_never_calls_the_executor() {
    let request = request(ToolTier::T2);
    let approved = decision(&request);
    for now in [100, 101] {
        assert_eq!(
            validate_binding(
                &request,
                &approved,
                "action-1",
                &request.action_payload.0,
                now,
                limits()
            )
            .unwrap_err(),
            BindingError::Expired
        );
        let mut called = false;
        let result = check(&request, &approved).unwrap().consume(now, |_, _| {
            called = true;
            Ok::<_, ()>(())
        });
        assert_eq!(result, Err(ConsumeError::Expired));
        assert!(!called);
    }
    assert_eq!(
        check(&request, &approved)
            .unwrap()
            .consume(99, |_, _| Ok::<_, ()>(7)),
        Ok(7)
    );
}

#[test]
fn consumption_owns_the_verified_bytes_even_after_source_mutation() {
    let mut request = request(ToolTier::T2);
    let mut approved = decision(&request);
    let original = request.action_payload.0.clone();
    let mut actual = original.clone();
    let snapshot = validate_binding(&request, &approved, "action-1", &actual, 1, limits()).unwrap();
    request.action_payload.0.fill(0);
    request.action_id = "substituted".into();
    request.expires_at_ns = DecimalI64(0);
    approved.payload_digest.0.fill(0);
    actual.fill(0);
    assert_eq!(
        snapshot.consume(2, |payload, binding| {
            assert_eq!(payload, original);
            assert_eq!(binding.approval_id(), "approval-1");
            assert_eq!(binding.action_id(), "action-1");
            assert_eq!(*binding.digest(), payload_digest(payload));
            Ok::<_, ()>(())
        }),
        Ok(())
    );
}

#[test]
fn callback_failure_is_returned_and_snapshot_debug_does_not_dump_payload() {
    let request = request(ToolTier::T2);
    let snapshot = check(&request, &decision(&request)).unwrap();
    let debug = format!("{snapshot:?}");
    assert!(!debug.contains("exact"));
    assert!(!debug.contains(&format!("{:?}", request.action_payload.0)));
    assert_eq!(
        snapshot.consume(2, |_, _| Err::<(), _>("executor failed")),
        Err(ConsumeError::Execute("executor failed"))
    );
}

#[test]
fn generated_contract_and_input_budget_are_enforced() {
    let original = request(ToolTier::T2);
    let approved = decision(&original);
    for invalid in [
        ApprovalRequestV2 {
            schema_version: 2,
            ..original.clone()
        },
        ApprovalRequestV2 {
            action_id: String::new(),
            ..original.clone()
        },
        ApprovalRequestV2 {
            payload_digest: HexBytes(vec![0; 31]),
            ..original.clone()
        },
    ] {
        assert_eq!(
            check(&invalid, &approved).unwrap_err(),
            BindingError::InvalidMessage
        );
    }
    let invalid = ApprovalDecisionV2 {
        schema_version: 2,
        ..approved.clone()
    };
    assert_eq!(
        check(&original, &invalid).unwrap_err(),
        BindingError::InvalidMessage
    );
    let invalid = ApprovalDecisionV2 {
        payload_digest: HexBytes(vec![0; 33]),
        ..approved.clone()
    };
    assert_eq!(
        check(&original, &invalid).unwrap_err(),
        BindingError::InvalidMessage
    );
    let small = Limits {
        max_bytes: 1,
        ..limits()
    };
    assert_eq!(
        validate_binding(
            &original,
            &approved,
            "action-1",
            &original.action_payload.0,
            1,
            small
        )
        .unwrap_err(),
        BindingError::PayloadOverBudget
    );
    assert_eq!(
        build_request("a".into(), "b".into(), ToolTier::T2, b"abc", 10, small).unwrap_err(),
        BindingError::PayloadOverBudget
    );
    assert_eq!(
        build_request(String::new(), "b".into(), ToolTier::T2, b"", 10, limits()).unwrap_err(),
        BindingError::InvalidMessage
    );
}
