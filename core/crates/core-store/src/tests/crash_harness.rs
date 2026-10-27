//! Only a freshly owned TemporaryRoot can issue a one-use child permit.
//! The old arbitrary-path environment variable is deliberately never read.
use super::*;
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

pub(super) fn run(
    root: &TemporaryRoot,
    operation: &str,
    point: FaultPoint,
    nth: usize,
    tag: Option<&str>,
) {
    let root_path = root.path().canonicalize().unwrap();
    let mut permit = tempfile::Builder::new()
        .prefix("w02-child-permit-")
        .rand_bytes(24)
        .tempfile_in(root.path())
        .unwrap();
    let permit_name = permit.path().file_name().unwrap().to_str().unwrap();
    let input = serde_json::json!({ "protocol": "W02-owned-child-v1", "root": root_path,
        "permit": permit_name, "operation": operation, "point": format!("{point:?}"), "nth": nth, "tag": tag });
    let bytes = serde_json::to_vec(&input).unwrap();
    permit.write_all(&bytes).unwrap();
    permit.as_file().sync_all().unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "tests::crash_harness::crash_worker",
            "--nocapture",
        ])
        .env("NEMU_W02_OWNED_CHILD", "1")
        .env_remove("NEMU_W02_CHILD_TEMP")
        .env_remove("NEMU_W02_CRASH")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(86),
        "{operation}/{point:?}/{nth}: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let trace = String::from_utf8_lossy(&output.stdout);
    assert!(trace.contains(&format!("W02_INJECT {point:?}")), "{trace}");
    assert!(!permit.path().exists(), "child did not consume its permit");
    println!("{}", trace.trim());
}

fn receive() -> Option<(PathBuf, serde_json::Value)> {
    if std::env::var("NEMU_W02_OWNED_CHILD").as_deref() != Ok("1") {
        return None;
    }
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(16385)
        .read_to_end(&mut bytes)
        .unwrap();
    assert!(bytes.len() <= 16384, "oversized child context");
    let input: serde_json::Value =
        serde_json::from_slice(&bytes).expect("missing owned child context");
    assert_eq!(input["protocol"], "W02-owned-child-v1");
    let root = PathBuf::from(input["root"].as_str().unwrap());
    let base = std::env::temp_dir().canonicalize().unwrap();
    // Check lexical containment before inspecting the designated directory.
    assert_eq!(root.parent(), Some(base.as_path()));
    assert!(
        root.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("nemu-w02-")
    );
    let name = input["permit"].as_str().unwrap();
    assert!(
        name.starts_with("w02-child-permit-")
            && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    );
    crate::platform::no_links(&root).unwrap();
    assert_eq!(root.canonicalize().unwrap(), root);
    let permit = root.join(name);
    crate::platform::no_links(&permit).unwrap();
    assert_eq!(
        std::fs::read(&permit).unwrap(),
        bytes,
        "invalid sandbox ownership permit"
    );
    std::fs::remove_file(permit).unwrap(); // One use; before any writer lock or SQL.
    Some((root.join("crash"), input))
}

#[test]
fn crash_worker() {
    let Some((path, input)) = receive() else {
        return;
    };
    let point = super::crash::points()
        .into_iter()
        .find(|p| format!("{p:?}") == input["point"].as_str().unwrap())
        .expect("unknown fault point");
    let fault = Fault::crash_nth(
        point,
        input["nth"].as_u64().unwrap() as usize,
        input["tag"].as_str(),
    );
    let operation = input["operation"].as_str().unwrap();
    if operation == "init" {
        Store::<Rehearsal>::create_with_fault(
            path,
            header::<Rehearsal>(),
            "test-branch",
            config(),
            true,
            fault,
        )
        .unwrap();
        panic!("init fault did not fire");
    }
    if operation == "repair" {
        Store::<Rehearsal>::open_with_fault(path, config(), true, fault).unwrap();
        panic!("repair fault did not fire");
    }
    let mut store = Store::<Rehearsal>::open(path, config(), true).unwrap();
    let cap = store.take_commit_capability().unwrap();
    match operation {
        "ingress" => {
            let (e, _payload) = unpersisted_event(&mut store, 1);
            store.set_fault(fault);
            store.ingest(&e).unwrap();
        }
        "reuse" => {
            let old = store.current().unwrap().unwrap();
            let x = FastStateSnapshotV2::from_canonical(
                &store.objects.read(&old.staged_x).unwrap(),
                config().encoding,
            )
            .unwrap();
            store.set_fault(fault);
            let _first = store.prepare(&x, "1").unwrap();
            let _second = store.prepare(&x, "1").unwrap();
        }
        "gc" => {
            store.set_fault(fault);
            store.collect_garbage().unwrap();
        }
        "r" | "manifest-second" | "commit" => {
            let e = event(&mut store, 1);
            let (ticket, mut value, _pins) = proposal(&store, &e);
            if operation == "r" {
                let mut r = RSnapshotV3::from_canonical(
                    &store.objects.read(&value.staged_r).unwrap(),
                    config().encoding,
                )
                .unwrap();
                r.revision = "r-targeted-fault".into();
                let mut tendency: TendencyV3 = with_refs(&value.staged_q);
                tendency.tendency_id = "target-a".into();
                r.tendencies = vec![tendency.clone()];
                tendency.tendency_id = "target-b".into();
                r.tendencies.push(tendency);
                store.set_fault(fault);
                store.prepare_r(&r).unwrap();
            } else {
                store.set_fault(fault);
                if operation == "manifest-second" {
                    let first = store
                        .stage(store.bind(ticket.binding().clone()).unwrap(), value.clone())
                        .unwrap();
                    value.audit_refs.push(value.staged_q.clone());
                    let _second = store.stage(ticket, value).unwrap();
                    drop(first);
                } else {
                    let staged = store.stage(ticket, value).unwrap();
                    store.commit(&cap, staged, normal()).unwrap();
                }
            }
        }
        _ => panic!("unknown operation"),
    }
    panic!("fault did not fire");
}

#[test]
fn stray_environment_or_missing_permit_never_touches_target() {
    let root = TemporaryRoot::new().unwrap();
    let target = root.store_path("untouched").unwrap();
    std::fs::create_dir(&target).unwrap();
    for requested_handshake in [false, true] {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "tests::crash_harness::crash_worker",
                "--nocapture",
            ])
            .env("NEMU_W02_CHILD_TEMP", &target)
            .env("NEMU_W02_CRASH", "write")
            .stdin(Stdio::null());
        if requested_handshake {
            command.env("NEMU_W02_OWNED_CHILD", "1");
        } else {
            command.env_remove("NEMU_W02_OWNED_CHILD");
        }
        let output = command.output().unwrap();
        assert_eq!(output.status.success(), !requested_handshake);
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 0);
    }
}
