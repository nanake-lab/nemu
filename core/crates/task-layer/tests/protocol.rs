use core_types::generated::*;
use std::{cell::Cell, fs};
use task_layer::task::{protocol::*, speech::*};

#[test]
fn candidate_status_retries_failed_push_and_uses_versioned_performable_slots() {
    let tmp = tempfile::tempdir().unwrap();
    let folder = tmp.path().join("_candidates/core/protocol");
    fs::create_dir_all(&folder).unwrap();
    fs::write(
        folder.join("manifest.json"),
        r#"{"version":"v0","status":"candidate","files":["ja.json"]}"#,
    )
    .unwrap();
    fs::write(
        folder.join("ja.json"),
        r#"{"facts":"AI","boundaries":"b","format":"{acts}","language":"ja"}"#,
    )
    .unwrap();
    let protocol =
        Protocol::load(tmp.path(), Source::Candidate, Lang::JA, &["wave", "nod"]).unwrap();
    assert_ne!(
        protocol.version(),
        Protocol::load(tmp.path(), Source::Candidate, Lang::JA, &["nod"])
            .unwrap()
            .version()
    );
    assert!(Protocol::load(tmp.path(), Source::Final, Lang::JA, &[]).is_err());
    assert!(
        Protocol::load(
            tmp.path(),
            Source::Candidate,
            Lang::JA,
            &["wave]]\ninstruction"]
        )
        .is_err()
    );
    let run = RunStateV2 {
        schema_version: 1,
        states: vec![RunStatus::NORMAL],
        speech_mode: SpeechMode::B_REMOTE_PROJECTED,
        strict: false,
        born_at: None,
        autonomy: Autonomy::PAUSED,
        reasons: vec![],
        lineage: TerminalLineage::TERMINAL_REHEARSAL,
    };
    let mut last = None;
    assert!(
        protocol
            .push_status(run.clone(), &mut last, |_| Err(()))
            .is_err()
    );
    let pushes = Cell::new(0);
    for _ in 0..2 {
        protocol
            .push_status(run.clone(), &mut last, |state| {
                assert!(state.reasons.iter().any(|s| s == "协议草稿"));
                pushes.set(pushes.get() + 1);
                Ok::<_, ()>(())
            })
            .unwrap();
    }
    assert_eq!(pushes.get(), 1);
}
#[test]
fn settings_read_explicit_budgets_and_override_approved_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("settings.json");
    fs::write(
        &file,
        r#"{"generation":{"tokens":42,"duration_ns":17},"k":2}"#,
    )
    .unwrap();
    let defaults = TaskSettings::load(&file).unwrap();
    assert_eq!(defaults.speech, SpeechSettings::default());
    fs::write(&file,r#"{"generation":{"tokens":18,"duration_ns":103},"k":1,"speech":{"missing":"none","self_quote":"strict"}}"#).unwrap();
    let changed = TaskSettings::load(&file).unwrap();
    assert_eq!(changed.generation.tokens, 18);
    assert_eq!(changed.generation.duration_ns, 103);
    assert_eq!(changed.speech.self_quote, SelfQuote::Strict);
    assert_eq!(changed.speech.missing, MissingHint::None);
    assert_eq!(changed.k, 1);
    for (json, tokens, ns, k) in [
        ("{}", 128, 6_000_000_000, 2),
        (r#"{"generation":{"tokens":42}}"#, 42, 6_000_000_000, 2),
        (r#"{"generation":{"duration_ns":17},"k":3}"#, 128, 17, 3),
    ] {
        fs::write(&file, json).unwrap();
        let s = TaskSettings::load(&file).unwrap();
        assert_eq!(
            (s.generation.tokens, s.generation.duration_ns, s.k),
            (tokens, ns, k)
        );
    }
    fs::write(&file, "{").unwrap();
    assert!(TaskSettings::load(&file).is_err());
    assert!(TaskSettings::load(tmp.path()).is_err());
}
