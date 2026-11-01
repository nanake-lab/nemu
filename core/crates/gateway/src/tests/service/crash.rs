//! Actual child exit/TerminateProcess, followed by a fresh gateway process.
//! The ONLY path accepted is a freshly owned parent tempfile sandbox. The child
//! uses process-local LOCALAPPDATA to exercise public Store open/startup APIs;
//! no system/user environment setting or real user store is ever read/changed.
use super::*;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

struct Running(Child);
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn spawn(root: &Path, case: &str, phase: &str, nonce: &str) -> Running {
    let permit = root.join(format!("permit-{phase}"));
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&permit)
        .unwrap();
    f.write_all(nonce.as_bytes()).unwrap();
    f.sync_all().unwrap();
    Running(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tests::service::crash::crash_peer",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("NEMU_W18_CHILD_ROOT", root)
            .env("NEMU_W18_CHILD_CASE", case)
            .env("NEMU_W18_CHILD_PHASE", phase)
            .env("NEMU_W18_CHILD_NONCE", nonce)
            .env("LOCALAPPDATA", root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    )
}
async fn finished(child: &mut Running, label: &str) -> std::process::ExitStatus {
    let until = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            let mut out = String::new();
            child
                .0
                .stdout
                .take()
                .unwrap()
                .read_to_string(&mut out)
                .unwrap();
            let mut err = String::new();
            child
                .0
                .stderr
                .take()
                .unwrap()
                .read_to_string(&mut err)
                .unwrap();
            if let Some(dir) = std::env::var_os("NEMU_W18_EVIDENCE_DIR") {
                let dir = PathBuf::from(dir);
                std::fs::create_dir_all(&dir).unwrap();
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(dir.join(format!("{label}-{}.stdout.log", child.0.id())))
                    .unwrap();
                f.write_all(out.as_bytes()).unwrap();
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(dir.join(format!("{label}-{}.stderr.log", child.0.id())))
                    .unwrap();
                f.write_all(err.as_bytes()).unwrap();
            }
            println!("SYS01 child {label}: exit={:?}\n{out}{err}", status.code());
            return status;
        }
        assert!(
            std::time::Instant::now() < until,
            "synthetic child did not exit"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
fn child_root() -> Option<(PathBuf, String, String, String)> {
    let root = PathBuf::from(std::env::var_os("NEMU_W18_CHILD_ROOT")?);
    let root = root.canonicalize().unwrap();
    let temp = std::env::temp_dir().canonicalize().unwrap();
    assert!(root.starts_with(&temp));
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("w18-crash-")
    );
    assert!(
        !std::fs::symlink_metadata(&root)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap())
            .canonicalize()
            .unwrap(),
        root
    );
    let case = std::env::var("NEMU_W18_CHILD_CASE").unwrap();
    assert!(["before-sql", "after-sql", "uncommitted"].contains(&case.as_str()));
    let phase = std::env::var("NEMU_W18_CHILD_PHASE").unwrap();
    assert!(["initial", "recovery"].contains(&phase.as_str()));
    let nonce = std::env::var("NEMU_W18_CHILD_NONCE").unwrap();
    assert_eq!(nonce.len(), 32);
    assert!(nonce.bytes().all(|b| b.is_ascii_hexdigit()));
    let permit = root.join(format!("permit-{phase}"));
    assert_eq!(std::fs::read(&permit).unwrap(), nonce.as_bytes());
    std::fs::remove_file(permit).unwrap();
    Some((root, case, phase, nonce))
}
#[test]
fn crash_peer() {
    let Some((root, case, phase, nonce)) = child_root() else {
        return;
    };
    let mut store = if phase == "initial" {
        assert!(!root.join("Nemu/core/rehearsal").exists());
        Store::<Rehearsal>::startup(
            header::<Rehearsal>(),
            super::super::runtime_fixture::BRANCH,
            config(),
        )
        .unwrap()
    } else {
        Store::<Rehearsal>::open_production(config()).unwrap()
    };
    assert!(store.path().starts_with(&root));
    let cap = store.take_commit_capability().unwrap();
    if phase == "initial" {
        match case.as_str() {
            "before-sql" => store.set_fault(core_store::Fault::Crash(
                core_store::FaultPoint::IngressBeforeSql,
            )),
            "after-sql" => store.set_fault(core_store::Fault::Crash(
                core_store::FaultPoint::IngressSqlCommitted,
            )),
            _ => {}
        }
    }
    let token = TerminalToken::load_protected(&root.join("test-token.dpapi")).unwrap();
    let handler = Core {
        cap,
        enabled: Arc::new(AtomicBool::new(phase == "recovery")),
        commits: Arc::new(AtomicUsize::new(0)),
        reverse_pair: false,
        held: vec![],
        calls: vec![],
        controls: false,
        titles: Arc::new(AtomicBool::new(false)),
        fabricate: false,
    };
    let mut gateway = Gateway::new(
        store,
        handler,
        vec![Installation::new("device".into(), token, CAPS.to_vec())],
        deployment(),
    )
    .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let suffix = format!("-sys01-child-{nonce}");
        let listener =
            Listener::for_lineage(Lineage::REHEARSAL, deployment().frames, &suffix).unwrap();
        gateway
            .serve_listener(listener, async {
                loop {
                    if root.join("finish").exists() {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .unwrap();
    });
    assert_eq!(phase, "recovery");
    let (store, _) = gateway.into_parts();
    store.verify().unwrap();
    let expected = if case == "before-sql" { 1 } else { 2 };
    assert_eq!(
        (store.head().unwrap().1, store.head().unwrap().2),
        (expected, 1)
    );
    assert_eq!(store.reader().unwrap().facts(10).unwrap().len(), 1);
    assert_eq!(
        store
            .reader()
            .unwrap()
            .ledger_at(&store.head().unwrap().0.unwrap(), 10)
            .unwrap()
            .len(),
        1
    );
    println!(
        "SYS01 recovered: H=1 ledger=1 anchor=1; original source key charged once; process-local temporary root verified"
    );
}

#[tokio::test]
async fn sys01_abrupt_exit_before_sql_after_sql_and_after_durable_ack_recover() {
    for case in ["before-sql", "after-sql", "uncommitted"] {
        let root = tempfile::Builder::new()
            .prefix("w18-crash-")
            .tempdir()
            .unwrap();
        let mut bytes = [0; 16];
        getrandom::fill(&mut bytes).unwrap();
        let nonce = core_types::scalar::hex(&bytes);
        let provisioned =
            TerminalToken::create_protected(&root.path().join("test-token.dpapi")).unwrap();
        let token = HexBytes(provisioned.test_bytes().to_vec());
        drop(provisioned);
        let name = format!(
            r"\\.\pipe\nemu-core-{}-sys01-child-{nonce}",
            crate::windows::current_sid().unwrap()
        );
        let mut child = spawn(root.path(), case, "initial", &nonce);
        let mut peer = Peer::connect(&name, &token).await;
        let p = text();
        let (e, source) = event(&p, 0);
        peer.upload(
            "p",
            match &p {
                TerminalPayload::Text(v) => v,
                _ => unreachable!(),
            },
        )
        .await;
        peer.upload("s", &source).await;
        let request = peer.send_event("initial", input(e.clone())).await;
        if case == "uncommitted" {
            let receipt = peer.reply(&request).await;
            assert_eq!(receipt.stage, ReceiptStage::RECEIVED_DURABLE);
            child.0.kill().unwrap(); // TerminateProcess: no Store/SQLite destructors.
        } else {
            assert!(peer.wire.read::<TerminalResponse>().await.is_err());
        }
        let status = finished(&mut child, &format!("{case}-initial")).await;
        if case != "uncommitted" {
            assert_eq!(status.code(), Some(86));
        } else {
            assert!(!status.success());
        }
        drop(peer);
        drop(child);
        let mut recovered = spawn(root.path(), case, "recovery", &nonce);
        let mut peer = Peer::connect(&name, &token).await;
        // The new connection starts uploads at zero and retries identical IDs.
        peer.upload(
            "p-new",
            match &p {
                TerminalPayload::Text(v) => v,
                _ => unreachable!(),
            },
        )
        .await;
        peer.upload("s-new", &source).await;
        let retry = peer.send_event("retry-original", input(e)).await;
        let durable = peer.reply(&retry).await;
        assert_eq!(
            durable.raw_log_position,
            Some(DecimalU64(if case == "before-sql" { 0 } else { 1 }))
        );
        assert_eq!(
            peer.reply(&retry).await.stage,
            if case == "before-sql" {
                ReceiptStage::CORE_COMMITTED
            } else {
                ReceiptStage::FILTERED
            }
        );
        std::fs::write(root.path().join("finish"), b"owned test shutdown").unwrap();
        drop(peer);
        assert!(
            finished(&mut recovered, &format!("{case}-recovery"))
                .await
                .success()
        );
    }
    println!(
        "SYS01 actual process crash/kill: pre-SQL rollback, post-SQL receipt-witness recovery, acknowledged-but-uncommitted D recovery all verified; no physical power cut claimed"
    );
}
