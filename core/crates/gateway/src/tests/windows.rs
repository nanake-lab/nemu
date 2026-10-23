use super::*;
use crate::{framing::FrameConfig, pipe::Listener};
use std::os::windows::io::AsRawHandle;
use tokio::net::windows::named_pipe::ClientOptions;

fn frame_config() -> FrameConfig {
    FrameConfig {
        encoding: limits(),
        deadline: Duration::from_secs(2),
    }
}
#[test]
fn dpapi_create_reload_reject_corruption_and_protect_actual_file_acl() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("terminal-token.dpapi");
    let first = TerminalToken::create_protected(&path).unwrap();
    let encrypted = std::fs::read(&path).unwrap();
    assert!(!encrypted.windows(32).any(|w| w == first.test_bytes()));
    let second = TerminalToken::load_protected(&path).unwrap();
    second
        .authenticate(first.test_bytes(), "device", "session")
        .unwrap();
    assert!(TerminalToken::create_protected(&path).is_err());
    assert_eq!(encrypted, std::fs::read(&path).unwrap());
    let file = std::fs::File::open(&path).unwrap();
    let sid = crate::windows::current_sid().unwrap();
    assert_eq!(
        crate::windows::dacl(file.as_raw_handle()).unwrap(),
        format!("D:P(A;;FA;;;{sid})")
    );
    drop(file);
    std::fs::write(&path, [0; 64]).unwrap();
    assert!(TerminalToken::load_protected(&path).is_err());
    for size in [0, 31, 33] {
        let encrypted = crate::windows::protect(&vec![0; size], false).unwrap();
        std::fs::write(&path, encrypted.as_slice()).unwrap();
        assert!(TerminalToken::load_protected(&path).is_err());
    }
    std::fs::write(&path, vec![0; 16385]).unwrap();
    assert!(TerminalToken::load_protected(&path).is_err());
}
#[tokio::test]
async fn actual_pipe_acl_current_user_connects_squatting_and_experiment_refused() {
    let suffix = format!("-w18-test-{}", std::process::id());
    let name = format!(
        r"\\.\pipe\nemu-core-{}{suffix}",
        crate::windows::current_sid().unwrap()
    );
    assert!(Listener::for_lineage(Lineage::EXPERIMENT, frame_config(), &suffix).is_err());
    assert!(ClientOptions::new().open(&name).is_err());
    let mut listener = Listener::for_lineage(Lineage::REHEARSAL, frame_config(), &suffix).unwrap();
    let sid = crate::windows::current_sid().unwrap();
    assert_eq!(
        crate::windows::dacl(listener.raw_handle()).unwrap(),
        format!("D:P(A;;FA;;;{sid})")
    );
    assert!(Listener::for_lineage(Lineage::REHEARSAL, frame_config(), &suffix).is_err());
    {
        let client = ClientOptions::new().open(&name).unwrap();
        let mut server = listener.accept().await.unwrap();
        let mut client = crate::framing::Framed::new(client, frame_config()).unwrap();
        let value = UserText {
            schema_version: 1,
            text: "synthetic pipe payload".into(),
        };
        client.write(&value).await.unwrap();
        assert_eq!(server.read::<UserText>().await.unwrap(), value);
    }
    // Exercise actual reads AND writes on every replacement, including after
    // abandoned unread bytes. A successful connect alone missed stale I/O.
    for attempt in 0..20 {
        listener.disconnect().unwrap();
        assert_eq!(
            crate::windows::dacl(listener.raw_handle()).unwrap(),
            format!("D:P(A;;FA;;;{sid})")
        );
        assert!(Listener::for_lineage(Lineage::REHEARSAL, frame_config(), &suffix).is_err());
        let client = ClientOptions::new().open(&name).unwrap();
        let mut client = crate::framing::Framed::new(client, frame_config()).unwrap();
        let mut server = listener.accept().await.unwrap();
        let value = UserText {
            schema_version: 1,
            text: format!("synthetic session {attempt}"),
        };
        client.write(&value).await.unwrap();
        assert_eq!(server.read::<UserText>().await.unwrap(), value);
        server.write(&value).await.unwrap();
        assert_eq!(client.read::<UserText>().await.unwrap(), value);
        client.write(&value).await.unwrap();
    }
}
