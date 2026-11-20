use core_types::{generated::Utf8Range, scalar::DecimalU64, semantic::resolved_text_range};

#[test]
fn resolved_range_returns_only_the_delivered_unicode_text() {
    let text = "甲🙂未投递";
    let range = |start, end| Utf8Range {
        r#schema_version: 1,
        r#start: DecimalU64(start),
        r#end: DecimalU64(end),
    };
    assert_eq!(resolved_text_range(text, &range(3, 7)).unwrap(), "🙂");
    for invalid in [range(3, 4), range(4, 7), range(7, 7), range(7, u64::MAX)] {
        assert!(resolved_text_range(text, &invalid).is_err());
    }
}

#[test]
fn contract_registration_does_not_enable_terminal_uploads() {
    let schema = core_types::contract::Schema::builtin().unwrap();
    let upload = core_types::terminal_ingress::upload_type;
    for message in schema
        .messages
        .values()
        .filter(|m| m.file == "speak_think.proto")
    {
        assert!(upload(&message.tag).is_err(), "{}", message.tag);
    }
    assert_eq!(upload("echo.user-text").unwrap(), "UserText");
}
