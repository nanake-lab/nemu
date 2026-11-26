//! P5-53 2/4. Only stage inputs are synthetic. D, the initial root, publication,
//! verification and restart all use the regular writer and independent readers.
mod common;
use common::*;
#[path = "handler/support.rs"]
mod support;
use core_runtime::handler::RehearsalHandler;
use core_store::{Fault, FaultPoint, Reader, Rehearsal, Store, TemporaryRoot};
use core_types::{generated::*, scalar::DecimalU64};
use gateway::{
    Grants,
    hooks::{CoreHandler, Decision},
};
use support::{config, initial, inputs};

struct NoTerminal;
impl CoreHandler<Rehearsal> for NoTerminal {
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

#[test]
fn failed_publication_keeps_d_for_reopen_then_commits_once_from_initial_root() {
    let temp = TemporaryRoot::new().unwrap();
    let (mut writer, cap) = initial(&temp);
    let old = writer.head().unwrap();
    let p = writer
        .prepare(
            &UserText {
                schema_version: 1,
                text: "synthetic".into(),
            },
            "1",
        )
        .unwrap();
    let mut event: EventEnvelope = with_refs(p.reference());
    event.payload_digest = event.payload_ref.digest.clone();
    event.raw_log_position = DecimalU64(0);
    writer.ingest(&event).unwrap();
    assert!(writer.take_commit_capability().is_err());
    let mut handler = RehearsalHandler::new(
        cap,
        NoTerminal,
        inputs,
        limits(),
        SpeechMode::B_REMOTE_PROJECTED,
    );
    writer.set_fault(Fault::DiskFull(FaultPoint::BeforePublish));
    assert!(matches!(handler.process_next(&mut writer),
        Err(gateway::Error::Store(core_store::Error::Io(ref e))) if e.kind() == std::io::ErrorKind::StorageFull));
    let reader = Reader::<Rehearsal>::open_temporary(&temp, "handler", config()).unwrap();
    assert_eq!(writer.head().unwrap(), old);
    assert!(reader.facts(4).unwrap().is_empty());
    drop((reader, p, handler, writer));

    let mut writer = Store::<Rehearsal>::open_temporary(&temp, "handler", config()).unwrap();
    assert_eq!(writer.next_raw().unwrap(), Some(event.clone()));
    let cap = writer.take_commit_capability().unwrap();
    let mut handler = RehearsalHandler::new(
        cap,
        NoTerminal,
        inputs,
        limits(),
        SpeechMode::B_REMOTE_PROJECTED,
    );
    let results = handler.process_next(&mut writer).unwrap();
    assert_eq!(results.len(), 1);
    let reader = Reader::<Rehearsal>::open_temporary(&temp, "handler", config()).unwrap();
    let (tx, committed) = reader
        .read_published_transaction(&results[0].snapshot)
        .unwrap();
    assert_eq!(committed, event);
    assert_eq!(results[0].event_id, event.id);
    let q: InitialRuntimeRoot = reader.read(&tx.staged_q).unwrap();
    let state = handler
        .committed_run_state(&reader, &results[0].snapshot)
        .unwrap()
        .unwrap();
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
    assert!(handler.process_next(&mut writer).unwrap().is_empty());
}

#[cfg(windows)]
#[path = "handler/windows.rs"]
mod windows;
