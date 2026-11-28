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
        let path = root.path().join("auth.sqlite");
        let local = authority::LocalAuthority::open(&path, sid, limits()).unwrap();
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
            let prefix = store.base_commit().unwrap();
            let view = fact_view::FactView::rebuild(&reader, &prefix, 8, None).unwrap();
            let access = fact_view::Access {
                allowed: true,
                ..Default::default()
            };
            let prior = view.mode_b_text("host-0", &access).unwrap();
            let latest = view.mode_b_text("host-1", &access).unwrap();
            let query = fact_view::FactQuery::new(
                &prefix,
                fact_view::Purpose::Task,
                &latest.text,
                &fact_view::Budgets::default(),
            );
            let material = view
                .mode_b_input(&latest.version, &query, &access, None)
                .unwrap();
            assert_eq!(material.task_facts.projection.history_prefix, prefix);
            assert_eq!(latest.speaker.governance, head);
            let user = |record: &fact_view::ModeBText| {
                reader
                    .read::<LocalUserBinding>(&record.speaker.layers[0].identity_evidence)
                    .unwrap()
            };
            let earlier = user(&prior);
            let later = user(&latest);
            assert_ne!(earlier.session_id, later.session_id);
            assert_eq!(earlier.principal_id, later.principal_id);
            // Independently compare both committed bindings with the original
            // service rows across the disconnect; same-principal alone is weak.
            let issued = rusqlite::Connection::open_with_flags(
                &path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .unwrap();
            for binding in [earlier, later] {
                let matches: bool = issued
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM local_auth_v1 WHERE record_id=?1
                     AND principal_id=?2 AND session_id=?3 AND installation_id=?4)",
                        rusqlite::params![
                            binding.authentication_record_id,
                            binding.principal_id,
                            binding.session_id,
                            binding.installation_id
                        ],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert!(matches);
            }
            assert_eq!(prior.h_position, 1);
            assert_eq!(latest.h_position, 3); // One filtered redelivery between accepted events.
            eprintln!(
                "authenticated pipe: recovered author on first event, second session shares principal; H positions 1 and 3, frontier={prefix}"
            );
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
