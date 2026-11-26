use super::*;
use core_types::WireMessage;
use gateway::{
    TerminalToken,
    gateway::{Gateway, Installation},
};
use std::{path::Path, process::Command};

#[path = "../../../nemu-core/src/authority.rs"]
mod authority;
#[path = "../../../nemu-core/src/deployment.rs"]
mod deployment;

#[tokio::test]
async fn authenticated_pipe_settlement_and_restart_match_the_committed_q() {
    let root = TemporaryRoot::new().unwrap();
    let (mut writer, cap) = initial(&root);
    let token_path = root.path().join("token.dpapi");
    let token = TerminalToken::create_protected(&token_path).unwrap();
    drop(token);
    let name = gateway::pipe::current_pipe_name().unwrap();
    let sid = name.strip_prefix(r"\\.\pipe\nemu-core-").unwrap();
    let first = writer.head().unwrap();
    writer.set_fault(Fault::DiskFull(FaultPoint::BeforePublish));
    let mut next = Some((writer, cap));
    for phase in ["failure", "restart"] {
        let (writer, cap) = next.take().unwrap_or_else(|| {
            let mut writer =
                Store::<Rehearsal>::open_temporary(&root, "handler", config()).unwrap();
            let cap = writer.take_commit_capability().unwrap();
            (writer, cap)
        });
        let local =
            authority::LocalAuthority::open(&root.path().join("auth.sqlite"), sid, limits())
                .unwrap();
        let handler =
            RehearsalHandler::new(cap, local, inputs, limits(), SpeechMode::B_REMOTE_PROJECTED);
        let token = TerminalToken::load_protected(&token_path).unwrap();
        use IngressKind::*;
        let install = Installation::new(
            "desktop".into(),
            token,
            vec![INGRESS_EVENT, INGRESS_UPLOAD, INGRESS_CANCEL],
        );
        let mut gateway =
            Gateway::new(writer, handler, vec![install], deployment::settings().1).unwrap();
        let (ready, start) = tokio::sync::oneshot::channel();
        let (done, stop) = tokio::sync::oneshot::channel();
        let run = gateway.serve(async {
            let _ = ready.send(());
            let _ = stop.await;
        });
        let client = async {
            start.await.expect("test owns the current SID pipe");
            let dir = root.path().to_owned();
            let endpoint = name.clone();
            let result = tokio::task::spawn_blocking(move || {
                Command::new("python")
                    .args(["-B", "-I"])
                    .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/handler/terminal.py"))
                    .arg(dir)
                    .arg(endpoint)
                    .arg(std::process::id().to_string())
                    .arg(phase)
                    .status()
                    .unwrap()
            })
            .await
            .unwrap();
            let _ = done.send(());
            assert!(result.success());
        };
        let (result, ()) = tokio::join!(run, client);
        if phase == "failure" {
            assert!(
                matches!(result, Err(gateway::Error::Store(core_store::Error::Io(ref e)))
                if e.kind() == std::io::ErrorKind::StorageFull),
                "{result:?}"
            );
        } else {
            result.unwrap();
        }
        let (store, _) = gateway.into_parts();
        let reader = Reader::<Rehearsal>::open_temporary(&root, "handler", config()).unwrap();
        let head = reader.current_snapshot().unwrap().unwrap();
        let q = if phase == "failure" {
            assert_eq!(store.head().unwrap(), first);
            assert_eq!(store.next_raw().unwrap().unwrap().id, "host-0");
            reader.read::<InitialPublication>(&head).unwrap().q
        } else {
            assert_eq!(reader.facts(8).unwrap().len(), 2);
            assert_eq!(store.head().unwrap().1, 3);
            assert!(store.next_raw().unwrap().is_none());
            let tx = reader.event_snapshot(&head).unwrap();
            assert_eq!(tx.binding.event_id, "host-1");
            reader.read::<InitialRuntimeRoot>(&tx.staged_q).unwrap()
        };
        if let Some(target) = std::env::var_os("NEMU_P5_53_EVIDENCE_DIR") {
            let target = Path::new(&target);
            std::fs::write(
                target.join(format!("{phase}-verified-head.pb")),
                head.protobuf_bytes(limits()).unwrap(),
            )
            .unwrap();
            std::fs::write(
                target.join(format!("{phase}-verified-q.pb")),
                q.protobuf_bytes(limits()).unwrap(),
            )
            .unwrap();
        }
        let wire = std::fs::read(root.path().join("state.pb")).unwrap();
        let state = RunStateV2::from_protobuf(&wire, limits()).unwrap();
        assert_eq!(
            state.states.contains(&RunStatus::IRL_DEGRADED),
            q.irl.is_none()
        );
        assert_eq!(
            state.states.contains(&RunStatus::A_DEGRADED),
            q.appraiser.is_none()
        );
        assert_eq!(
            state.states.contains(&RunStatus::M_DEGRADED),
            q.matcher.is_none()
        );
        assert_eq!(
            state.states.contains(&RunStatus::CORE_UNMODULATED),
            q.fast.is_none() || q.compiler.is_none()
        );
        assert_eq!(
            state.states.contains(&RunStatus::NO_GROWTH),
            q.growth.is_none() && q.dynamics.is_none()
        );
        assert_eq!(state.states.len(), 5);
        let mut reasons = state.reasons.clone();
        reasons.sort();
        assert_eq!(
            reasons,
            [
                "DEPENDENCY_FAULT",
                "GROWTH_OFF_REHEARSAL",
                "MODULATION_DISABLED"
            ]
        );
        assert_eq!(state.speech_mode, SpeechMode::B_REMOTE_PROJECTED);
        assert!(state.born_at.is_none());
        assert_eq!(state.lineage, TerminalLineage::TERMINAL_REHEARSAL);
        assert_eq!(state.autonomy, Autonomy::PAUSED);
        assert!(!state.strict);
    }
}
