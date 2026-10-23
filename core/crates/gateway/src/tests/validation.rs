use super::*;

#[test]
fn all_nine_terminal_payloads_validate_and_bind_generated_objects() {
    let mut touch: TouchObservation = fixture();
    touch.region = "ears".into();
    let mut body: BodyActionResult = fixture();
    body.status = ActionStatus::SUCCEEDED;
    let retraction = UserRetraction {
        schema_version: 1,
        target_refs: vec!["synthetic-event".into()],
        note: None,
    };
    let payloads = vec![
        TerminalPayload::Utterance(fixture()),
        text(),
        TerminalPayload::Retraction(retraction),
        TerminalPayload::Touch(touch),
        TerminalPayload::Desktop(fixture()),
        TerminalPayload::Codex(fixture()),
        TerminalPayload::Speech(fixture()),
        TerminalPayload::Body(body),
        TerminalPayload::Maintenance(fixture()),
    ];
    let auth = session(Grants {
        desktop_titles: true,
        codex_sessions: true,
    });
    for payload in payloads {
        let (e, source) = event(&payload, 0);
        validate_event(&auth, e.clone(), payload.clone(), source.clone(), limits()).unwrap();
        let mut wrong = e.clone();
        wrong.kind = EventKind::CONSEQUENCE;
        assert!(validate_event(&auth, wrong, payload.clone(), source.clone(), limits()).is_err());
        let mut wrong = e;
        wrong.payload_ref.byte_length.0 += 1;
        assert!(validate_event(&auth, wrong, payload, source, limits()).is_err());
    }
}
#[test]
fn rejects_source_spoofing_payload_changes_and_privilege_claims() {
    let auth = session(Grants::default());
    let payload = text();
    let (e, source) = event(&payload, 0);
    let mut wrong = e.clone();
    wrong.device = "attacker".into();
    assert!(validate_event(&auth, wrong, payload.clone(), source.clone(), limits()).is_err());
    let mut wrong = e.clone();
    wrong.session = "other".into();
    assert!(validate_event(&auth, wrong, payload.clone(), source.clone(), limits()).is_err());
    let changed = TerminalPayload::Text(UserText {
        schema_version: 1,
        text: "different".into(),
    });
    assert!(validate_event(&auth, e.clone(), changed, source.clone(), limits()).is_err());
    let mut wrong = e.clone();
    wrong.payload_digest.0[0] ^= 1;
    assert!(validate_event(&auth, wrong, payload.clone(), source.clone(), limits()).is_err());
    let mut wrong = e;
    wrong.stable_source_key = "same-text-is-not-an-id".into();
    assert!(validate_event(&auth, wrong, payload, source, limits()).is_err());
    let mut desktop: DesktopObservation = fixture();
    desktop.title = Some("synthetic-title".into());
    let payload = TerminalPayload::Desktop(desktop);
    let (e, source) = event(&payload, 0);
    assert!(validate_event(&auth, e, payload, source, limits()).is_err());
    let payload = TerminalPayload::Codex(fixture());
    let (e, source) = event(&payload, 0);
    assert!(validate_event(&auth, e, payload, source, limits()).is_err());
}
#[test]
fn rejects_bad_fields_unknown_protobuf_and_nonfinite_values() {
    let payload = text();
    let r = payload.reference(limits()).unwrap();
    let bytes = UserText {
        schema_version: 1,
        text: "synthetic input".into(),
    }
    .protobuf_bytes(limits())
    .unwrap();
    assert!(TerminalPayload::decode(&r, &bytes, limits()).is_ok());
    let mut unknown = bytes.clone();
    unknown.extend([0x78, 1]);
    assert!(TerminalPayload::decode(&r, &unknown, limits()).is_err());
    let mut duplicate = bytes;
    duplicate.extend([8, 1]);
    assert!(TerminalPayload::decode(&r, &duplicate, limits()).is_err());
    let mut utterance: UserUtterance = fixture();
    utterance.asr_confidence = f64::NAN;
    assert!(
        TerminalPayload::Utterance(utterance)
            .reference(limits())
            .is_err()
    );
    let mut body: BodyActionResult = fixture();
    body.status = ActionStatus::PLANNED;
    let payload = TerminalPayload::Body(body);
    let (e, source) = event(&payload, 0);
    assert!(validate_event(&session(Grants::default()), e, payload, source, limits()).is_err());
    let payload = TerminalPayload::Retraction(UserRetraction {
        schema_version: 1,
        target_refs: vec![],
        note: None,
    });
    let (e, source) = event(&payload, 0);
    assert!(validate_event(&session(Grants::default()), e, payload, source, limits()).is_err());
}
#[test]
fn authentication_never_accepts_short_wrong_or_empty_identity() {
    let token = TerminalToken::synthetic();
    for bytes in [&[][..], &[0xA5; 31][..], &[0; 32][..], &[0xA5; 33][..]] {
        assert!(
            token
                .authenticate(bytes, "device", "session", Grants::default())
                .is_err()
        );
    }
    assert!(
        token
            .authenticate(&[0xA5; 32], "", "session", Grants::default())
            .is_err()
    );
}
