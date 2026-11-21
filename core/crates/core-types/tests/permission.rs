use core_types::{
    contract::Limits,
    generated::{
        ActionRequestV2, IngressKind, ManagementEntry, TerminalRequestV2, TerminalSession,
    },
    permission::{check_dispatch_binding, check_management_binding},
};
use serde_json::Value;

fn fixture<T: serde::de::DeserializeOwned>(name: &str) -> T {
    let vectors: Vec<Value> =
        serde_json::from_str(include_str!("../../../contracts/golden/legal.json")).unwrap();
    let request = vectors
        .iter()
        .map(|v| &v["request"])
        .find(|q| q["name"] == name)
        .unwrap();
    serde_json::from_value(request["value"].clone()).unwrap()
}

const LIMITS: Limits = Limits {
    max_bytes: 1048576,
    max_items: 4096,
    max_depth: 32,
};

#[test]
fn dispatch_matches_issuance_holder_and_half_open_validity() {
    let request: ActionRequestV2 = fixture("ActionRequestV2");
    let issued = &request.grant;
    let check = |g, holder, time| check_dispatch_binding(&request, g, holder, time, LIMITS);
    check(issued, &issued.executor_session_id, issued.issued_at_ns.0).unwrap();
    assert!(check(issued, "unrelated", issued.issued_at_ns.0).is_err());
    assert!(check(issued, &issued.executor_session_id, issued.expires_at_ns.0).is_err());
    assert!(
        check(
            issued,
            &issued.executor_session_id,
            issued.issued_at_ns.0 - 1
        )
        .is_err()
    );
    let mut other = issued.clone();
    other.revocation_version.0 += 1;
    assert!(check(&other, &issued.executor_session_id, issued.issued_at_ns.0).is_err());
}

#[test]
fn management_matches_authenticated_installation_person_entry_and_frame_budget() {
    let request: TerminalRequestV2 = fixture("TerminalRequestV2");
    let m = request.management.as_ref().unwrap();
    let mut session: TerminalSession = fixture("TerminalSession");
    session.session_id.clone_from(&request.session_id);
    session.installation_id.clone_from(&m.user.installation_id);
    session.protocol_version = request.protocol_version;
    session.capabilities = vec![IngressKind::INGRESS_GROWTH];
    session.budget.max_chunk_bytes = 1;
    session.budget.max_object_bytes = 2;
    session.budget.max_upload_bytes = 3;
    session.budget.max_frame_bytes = 65536;
    let check = |s, u, entry| check_management_binding(&request, s, u, entry, LIMITS);
    check(&session, &m.user, m.entry).unwrap();
    assert!(check(&session, &m.user, ManagementEntry::LOCAL_ECHOCTL).is_err());
    let mut stranger = m.user.clone();
    stranger.principal_id.push('2');
    assert!(check(&session, &stranger, m.entry).is_err());
    let mut small_frame = session.clone();
    small_frame.budget.max_frame_bytes = 2;
    assert!(check(&small_frame, &m.user, m.entry).is_err());
    let mut no_growth = session.clone();
    no_growth.capabilities.clear();
    assert!(check(&no_growth, &m.user, m.entry).is_err());
}
