use super::*;

/// A fresh W02 reader recomputes every derived row from the canonical manifests.
/// Require a real registered online row first, so two empty projections cannot
/// pass. Python's standard sqlite3 is only an offline read-only test probe.
#[test]
fn rebuilt_outbox_rows_equal_online_rows() {
    let mut f = Fixture::new(false, false);
    f.registered();
    let head = f.store.base_commit().unwrap();
    let current = f.store.current().unwrap().unwrap();
    let q: CognitionState = f.store.reader().unwrap().read(&current.staged_q).unwrap();
    assert_eq!(q.inflight.len(), 1);
    let expected = q.inflight[0].canonical_bytes(limits()).unwrap();
    let check = r#"import sqlite3, sys
with sqlite3.connect('file:store.sqlite?mode=ro', uri=True) as db:
    rows = db.execute('SELECT commit_id, action_id, reference FROM outbox ORDER BY commit_id, action_id').fetchall()
    assert rows == [(sys.argv[1], 'a', bytes.fromhex(sys.argv[2]))], rows
"#;
    let probe = std::process::Command::new("python")
        .args(["-I", "-c", check])
        .current_dir(f.store.path())
        .arg(head)
        .arg(core_types::scalar::hex(&expected))
        .output()
        .unwrap();
    assert!(
        probe.status.success(),
        "{}",
        String::from_utf8_lossy(&probe.stderr)
    );
    // W02 derived::verify compares every column of every sorted online row
    // against its in-memory reconstruction, including outbox.reference.
    f.store.verify().unwrap();
}

#[test]
fn stale_and_missing_index_hints_use_q_and_leave_diagnostics() {
    let mut f = Fixture::new(false, false);
    f.registered();
    assert_eq!(f.index.diagnostics, [IndexIssue::Missing]);
    let q: CognitionState = f
        .store
        .reader()
        .unwrap()
        .read(&f.store.current().unwrap().unwrap().staged_q)
        .unwrap();
    let mut incorrect = q.inflight[0].clone();
    incorrect.status = ActionStatus::SUCCEEDED;
    f.index.hint = Some(incorrect);
    f.authorized();
    assert_eq!(f.status().status, ActionStatus::AUTHORIZED);
    assert_eq!(f.index.diagnostics.last(), Some(&IndexIssue::Disagrees));
    assert_eq!(pending_dispatch(&f.store, &mut f.index).unwrap(), ["a"]);
    f.index.hint = None;
    assert_eq!(pending_dispatch(&f.store, &mut f.index).unwrap(), ["a"]);
    assert_eq!(f.index.diagnostics.last(), Some(&IndexIssue::Missing));
    f.index.hint = Some(q.inflight[0].clone());
    let mut empty = q;
    empty.inflight.clear();
    assert!(index::locate(&empty, "head", "a", &mut f.index).is_err());
}

#[test]
fn failed_consume_never_calls_sender_even_if_sql_committed() {
    for point in [FaultPoint::BeforePublish, FaultPoint::SqlCommitted] {
        let mut f = Fixture::new(false, false);
        f.registered();
        f.authorized();
        let frame = f.frame(EventKind::MAINTENANCE, &f.leaf.clone(), "consume");
        let expected = f.request.clone();
        f.store.set_fault(Fault::DiskFull(point));
        let auth = authority(&expected, &f.settings, &f.issuer);
        let mut tx =
            ActionTransaction::open(&mut f.store, &f.cap, frame, limits(), 32, &mut f.index)
                .unwrap();
        assert!(
            outbox::dispatch(
                &mut tx,
                &expected,
                &auth,
                limits(),
                || Some(40),
                |_| panic!("ambiguous commit let a send escape")
            )
            .is_err()
        );
        drop(tx);
        let f = f.reopen();
        let expected = if point == FaultPoint::BeforePublish {
            ActionStatus::AUTHORIZED
        } else {
            ActionStatus::DISPATCHING
        };
        assert_eq!(f.status().status, expected);
        assert_eq!(
            unresolved_after_restart(&f.store).unwrap().len(),
            usize::from(point == FaultPoint::SqlCommitted)
        );
    }
}

#[test]
fn sent_body_can_fail_and_pending_action_can_cancel() {
    let mut f = Fixture::new(false, false);
    f.registered();
    let frame = f.frame(EventKind::CANCEL, &f.leaf.clone(), "cancel");
    f.transaction(frame).cancel().unwrap();
    assert_eq!(f.status().status, ActionStatus::CANCELLED_ACTION);

    let mut f = Fixture::new(false, false);
    f.registered();
    f.authorized();
    let frame = f.frame(EventKind::MAINTENANCE, &f.leaf.clone(), "consume");
    let auth = authority(&f.request, &f.settings, &f.issuer);
    let mut tx =
        ActionTransaction::open(&mut f.store, &f.cap, frame, limits(), 32, &mut f.index).unwrap();
    outbox::dispatch(
        &mut tx,
        &f.request,
        &auth,
        limits(),
        || Some(40),
        |_| Ok(()),
    )
    .unwrap();
    let receipt = BodyActionResult {
        schema_version: 1,
        action_id: "a".into(),
        status: ActionStatus::FAILED,
        detail: Some("fixture motion failed".into()),
    };
    let r = f.store.prepare(&receipt, "1").unwrap();
    let frame = f.frame(EventKind::EXECUTION_RESULT, r.reference(), "failed");
    f.transaction(frame).settle().unwrap();
    assert_eq!(f.status().status, ActionStatus::FAILED);
}

pub(super) fn deliver_then_exit(body: &ActionRequestV2) {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "action::tests::recovery::synthetic_body_executor",
            "--nocapture",
        ])
        .env("NEMU_W13_SYNTHETIC_CHILD", "yes")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&body.action_payload.0)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(86));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("executed motion=wave")
    );
}

#[test]
fn synthetic_body_executor() {
    use std::io::{Read, Write};
    if std::env::var("NEMU_W13_SYNTHETIC_CHILD").as_deref() != Ok("yes") {
        return;
    }
    let mut bytes = vec![];
    std::io::stdin()
        .take(limits().max_bytes as u64)
        .read_to_end(&mut bytes)
        .unwrap();
    let body = BodyAction::from_canonical(&bytes, limits()).unwrap();
    assert_eq!(body.kind, BodyActionKind::MOTION);
    println!("executed motion={}", body.motion_id.unwrap());
    std::io::stdout().flush().unwrap();
    // No terminal receipt, unwinding or normal shutdown after the simulated effect.
    std::process::exit(86);
}

#[test]
fn elapsed_approval_window_cancels_the_durable_request() {
    use std::time::{Duration, Instant};
    // A short user-selected fixture window, independent of the 60 s default.
    let window = Duration::from_millis(25);
    let mut f = Fixture::with_approval_deadline(false, true, window.as_nanos() as i64);
    f.registered();
    let user = f.request.grant.consent.as_ref().unwrap().user.clone();
    let shown = Instant::now();
    std::thread::sleep(window);
    let decision_at = i64::try_from(shown.elapsed().as_nanos()).unwrap();
    let frame = f.frame(
        EventKind::MAINTENANCE,
        &f.leaf.clone(),
        "wall-clock-timeout",
    );
    f.transaction(frame).decision(&user, decision_at).unwrap();
    assert!(decision_at >= window.as_nanos() as i64);
    assert_eq!(f.status().status, ActionStatus::CANCELLED_ACTION);
    println!(
        "approval_window_ns={} elapsed_at_timeout_ns={} durable_cancellation_seen_ms={}",
        window.as_nanos(),
        decision_at,
        shown.elapsed().as_millis()
    );
}
