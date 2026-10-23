use super::*;

#[test]
fn i29_all_six_cross_lineage_opens_rejected() {
    let root = TemporaryRoot::new().unwrap();
    let mut rehearsal = rehearsal(&root, "rehearsal");
    let mut experiment = Store::<Experiment>::create_temporary(
        &root,
        "experiment",
        header::<Experiment>(),
        "experiment",
        config(),
    )
    .unwrap();
    let mut life_header = header::<Life>();
    let genesis: GenesisManifestV2 = fixture();
    life_header.genesis_manifest_digest = Some(HexBytes(
        core_types::scalar::unhex(
            &core_types::contract::Schema::builtin()
                .unwrap()
                .digest(
                    GenesisManifestV2::NAME,
                    &serde_json::to_value(&genesis).unwrap(),
                    config().encoding,
                )
                .unwrap(),
        )
        .unwrap(),
    ));
    let mut life =
        Store::<Life>::create_from_genesis(&root, "life", life_header, &genesis, "life", config())
            .unwrap();
    fn populate<L: LineageKind>(store: &mut Store<L>, label: &str) -> ObjectRef {
        let cap = store.take_commit_capability().unwrap();
        let (mut e, _payload) = unpersisted_event(store, 0);
        e.id = format!("{label}-event");
        e.stable_source_key = format!("{label}-source");
        store.ingest(&e).unwrap();
        let (ticket, value, _pins) = proposal(store, &e);
        let staged = store.stage(ticket, value).unwrap();
        store.commit(&cap, staged, normal()).unwrap();
        let reader = store.reader().unwrap();
        assert_eq!(reader.facts(10).unwrap(), vec![e.clone()]);
        store.objects.reference(&e, "1").unwrap()
    }
    let r = populate(&mut rehearsal, "rehearsal");
    let l = populate(&mut life, "life");
    let e = populate(&mut experiment, "experiment");
    for foreign in [&l, &e] {
        assert!(
            rehearsal
                .reader()
                .unwrap()
                .read::<EventEnvelope>(foreign)
                .is_err()
        );
    }
    for foreign in [&r, &e] {
        assert!(
            life.reader()
                .unwrap()
                .read::<EventEnvelope>(foreign)
                .is_err()
        );
    }
    for foreign in [&r, &l] {
        assert!(
            experiment
                .reader()
                .unwrap()
                .read::<EventEnvelope>(foreign)
                .is_err()
        );
    }
    assert_eq!(
        life.reader()
            .unwrap()
            .facts(10)
            .unwrap()
            .iter()
            .map(|e| e.id.as_str())
            .collect::<Vec<_>>(),
        vec!["life-event"]
    );
    drop((rehearsal, experiment, life));
    for name in ["life", "experiment"] {
        assert!(Store::<Rehearsal>::open_temporary(&root, name, config()).is_err());
        assert!(Reader::<Rehearsal>::open_temporary(&root, name, config()).is_err());
    }
    for name in ["rehearsal", "experiment"] {
        assert!(Store::<Life>::open_temporary(&root, name, config()).is_err());
        assert!(Reader::<Life>::open_temporary(&root, name, config()).is_err());
    }
    for name in ["rehearsal", "life"] {
        assert!(Store::<Experiment>::open_temporary(&root, name, config()).is_err());
        assert!(Reader::<Experiment>::open_temporary(&root, name, config()).is_err());
    }
}

#[test]
fn reader_and_capabilities_cannot_cross_stores_even_same_kind() {
    let root = TemporaryRoot::new().unwrap();
    let mut a = rehearsal(&root, "a");
    let mut b = rehearsal(&root, "b");
    let cap_a = a.take_commit_capability().unwrap();
    let cap_b = b.take_commit_capability().unwrap();
    let published = advance(&mut a, &cap_a, 0);
    assert!(
        b.reader()
            .unwrap()
            .read::<CoreTransactionV2>(&published.snapshot)
            .is_err()
    );
    assert!(b.reader().unwrap().facts(10).unwrap().is_empty());
    let e = event(&mut a, 1);
    let (ticket, value, _pins) = proposal(&a, &e);
    assert!(b.stage(ticket, value).is_err());
    let (ticket, value, _pins) = proposal(&a, &e);
    let staged = a.stage(ticket, value).unwrap();
    assert!(a.commit(&cap_b, staged, normal()).is_err());
}

#[test]
fn at04_single_writer_and_copy_cannot_resume_as_same_branch() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "original");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    assert!(Store::<Rehearsal>::open_temporary(&root, "original", config()).is_err());
    let original = store.path().to_path_buf();
    drop(store);
    let copy = root.store_path("copy").unwrap();
    std::fs::create_dir(&copy).unwrap();
    std::fs::copy(original.join("store.sqlite"), copy.join("store.sqlite")).unwrap();
    assert!(Store::<Rehearsal>::open_temporary(&root, "copy", config()).is_err());
    assert!(Reader::<Rehearsal>::open_temporary(&root, "copy", config()).is_err());
    Store::<Rehearsal>::open_temporary(&root, "original", config())
        .unwrap()
        .verify()
        .unwrap();
}

#[test]
fn malformed_headers_and_existing_directories_never_initialize() {
    let root = TemporaryRoot::new().unwrap();
    assert!(
        Store::<Experiment>::create_temporary(
            &root,
            "bad",
            header::<Rehearsal>(),
            "branch",
            config()
        )
        .is_err()
    );
    let mut h = header::<Experiment>();
    h.world_id = None;
    assert!(
        Store::<Experiment>::create_temporary(&root, "missing-world", h, "branch", config())
            .is_err()
    );
    assert!(
        Store::<Life>::create_from_genesis(
            &root,
            "life",
            header::<Life>(),
            &fixture(),
            "branch",
            config()
        )
        .is_err()
    );
    let store = rehearsal(&root, "exists");
    drop(store);
    assert!(
        Store::<Rehearsal>::create_temporary(
            &root,
            "exists",
            header::<Rehearsal>(),
            "branch",
            config()
        )
        .is_err()
    );
}

#[test]
fn at04_old_database_in_original_location_cannot_refund_ledger() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "rollback");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let path = store.path().to_path_buf();
    drop(store);
    let backup = root.path().join("old.sqlite");
    std::fs::copy(path.join("store.sqlite"), &backup).unwrap();
    let mut store = Store::<Rehearsal>::open_temporary(&root, "rollback", config()).unwrap();
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 1);
    drop(store);
    // Simulate replacing an old backup after all DB handles have closed. The
    // separately retained receipts prove that this would lose acknowledged L.
    std::fs::copy(&backup, path.join("store.sqlite")).unwrap();
    let result = Store::<Rehearsal>::open_temporary(&root, "rollback", config());
    assert!(
        matches!(result, Err(Error::Rejected(message)) if message.contains("predates acknowledged history"))
    );
    assert!(Reader::<Rehearsal>::open_temporary(&root, "rollback", config()).is_err());
}
