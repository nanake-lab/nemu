use super::*;
use crate::connection::Authentication;
use tokio::sync::{mpsc, oneshot};

struct Hook {
    outcome: Result<Decision<()>>,
    called: Vec<TerminalSession>,
}

impl CoreHandler<Rehearsal> for Hook {
    fn authenticated(
        &mut self,
        _: &mut Store<Rehearsal>,
        session: &TerminalSession,
    ) -> Result<Decision<()>> {
        self.called.push(session.clone());
        std::mem::replace(&mut self.outcome, Err(Error::Contract))
    }

    fn authorize_event(
        &mut self,
        _: &Store<Rehearsal>,
        _: &TerminalSession,
        _: &TerminalEvent,
        _: &SourceMetadata,
    ) -> Decision<Grants> {
        Err(IngressReason::CAPABILITY_DENIED)
    }
}

fn check(outcome: Result<Decision<()>>, reason: Option<IngressReason>) {
    let root = TemporaryRoot::new().unwrap();
    let store = Store::<Rehearsal>::create_temporary(
        &root,
        "hook",
        header::<Rehearsal>(),
        "hook",
        config(),
    )
    .unwrap();
    let secret = TerminalToken::synthetic();
    let mut request: TerminalAuthenticate = fixture();
    request.token = HexBytes(secret.test_bytes().to_vec());
    request.protocol_versions = vec![1];
    let install = Installation::new(request.installation_id.clone(), secret, CAPS.to_vec());
    let mut service = Gateway::new(
        store,
        Hook {
            outcome,
            called: vec![],
        },
        vec![install],
        deployment(),
    )
    .unwrap();
    let (reply, mut rx) = oneshot::channel();
    let (output, _outputs) = mpsc::channel(1);
    service
        .authenticate(1, Authentication(request), reply, output)
        .unwrap();
    let response = rx.try_recv().unwrap();
    assert_eq!(service.handler.called.len(), 1);
    assert_eq!(response.reason, reason);
    let audit = service.store().ingress_rejections().unwrap();
    if let Some(reason) = reason {
        assert_eq!(response.status, AuthStatus::AUTH_REJECTED);
        assert!(response.session.is_none() && service.connections.is_empty());
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].reason, reason);
        assert_eq!(audit[0].request_kind, None);
    } else {
        assert_eq!(response.status, AuthStatus::AUTHENTICATED);
        assert_eq!(response.session.as_ref(), service.handler.called.first());
        assert_eq!(service.connections[&1].session, response.session.unwrap());
        assert!(audit.is_empty());
    }
}

#[test]
fn successful_hook_registers_the_exact_accepted_session_once() {
    check(Ok(Ok(())), None);
}

#[test]
fn denied_or_failed_hook_audits_and_never_installs_a_session() {
    check(
        Ok(Err(IngressReason::CAPABILITY_DENIED)),
        Some(IngressReason::CAPABILITY_DENIED),
    );
    check(Err(Error::Contract), Some(IngressReason::AUTH_FAILED));
}
