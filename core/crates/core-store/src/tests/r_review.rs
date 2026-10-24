//! PR #72 regressions. Synthetic storage seams, not production G acceptance.
use super::*;

#[test]
fn retired_group_history_exceeds_message_budget_without_blocking_commits() {
    let root = TemporaryRoot::new().unwrap();
    let mut cfg = config();
    cfg.encoding.max_items = 32;
    let mut store = Store::<Rehearsal>::create_temporary(
        &root,
        "group-history",
        header::<Rehearsal>(),
        "test-branch",
        cfg,
    )
    .unwrap();
    let cap = store.take_commit_capability().unwrap();
    let mut previous = Vec::<EvidenceGroupV3>::new();
    let mut first_commit = String::new();
    // Four live groups at a time; nine transactions leave 36 historical IDs.
    for n in 0..9 {
        let e = event(&mut store, n);
        let (ticket, mut value, mut pins) = proposal(&store, &e);
        let law = value.binding.environment.law_version.as_ref().unwrap();
        let mut r = r_fixtures::snapshot(law);
        let mut t = r_fixtures::tendency("current");
        t.group_count = DecimalU64(4);
        r.tendencies.push(t);
        let object = store.prepare_r(&r).unwrap();
        value.staged_r = object.reference().clone();
        pins.push(object);
        for mut group in std::mem::take(&mut previous) {
            group.allocations.clear();
            let object = store.prepare(&group, "1").unwrap();
            value.accepted_results.push(object.reference().clone());
            pins.push(object);
        }
        for ordinal in 0..4 {
            let mut group = r_fixtures::group(&format!("g-{n}-{ordinal}"), "current");
            group.allocations[0].ordinal = DecimalU64(ordinal);
            group.allocations[0].support = 0.0;
            group.allocations[0].mu = 0.0;
            group.allocations[0].projection = None;
            group.allocations[0].actual_retention = 1.0;
            let object = store.prepare(&group, "1").unwrap();
            value.accepted_results.push(object.reference().clone());
            pins.push(object);
            previous.push(group);
        }
        value.canonical_bytes(cfg.encoding).unwrap();
        let staged = store.stage(ticket, value).unwrap();
        let commit = store.commit(&cap, staged, normal()).unwrap().commit_id;
        if n == 0 {
            first_commit = commit.clone();
        }
        assert_eq!(
            store
                .reader()
                .unwrap()
                .evidence_for("current", &commit, 8)
                .unwrap()
                .len(),
            4
        );
    }
    let count: i64 = store
        .db
        .query_row(
            "SELECT count(DISTINCT group_id) FROM evidence_groups",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(count > cfg.encoding.max_items as i64);
    // Retire the final live collection after crossing the old lifetime cap.
    let e = event(&mut store, 9);
    let (ticket, mut value, mut pins) = proposal(&store, &e);
    for mut group in previous {
        group.allocations.clear();
        let object = store.prepare(&group, "1").unwrap();
        value.accepted_results.push(object.reference().clone());
        pins.push(object);
    }
    let staged = store.stage(ticket, value).unwrap();
    store.commit(&cap, staged, normal()).unwrap();

    // The actual pending collection still has its original message limit.
    let e = event(&mut store, 10);
    let (ticket, mut value, mut pins) = proposal(&store, &e);
    let before = store.head().unwrap();
    for n in 0..cfg.encoding.max_items {
        let object = store
            .prepare(
                &UserText {
                    schema_version: 1,
                    text: format!("oversized-result-{n}"),
                },
                "1",
            )
            .unwrap();
        value.accepted_results.push(object.reference().clone());
        pins.push(object);
    }
    assert!(value.canonical_bytes(cfg.encoding).is_err());
    assert!(store.stage(ticket, value).is_err());
    assert_eq!(store.head().unwrap(), before);
    let (ticket, value, _pins) = proposal(&store, &e);
    let staged = store.stage(ticket, value).unwrap();
    let last = store.commit(&cap, staged, normal()).unwrap().commit_id;
    store.verify().unwrap();
    drop(store);
    let store = Store::<Rehearsal>::open_temporary(&root, "group-history", cfg).unwrap();
    let reader = store.reader().unwrap();
    assert!(reader.evidence_for("current", &last, 8).unwrap().is_empty());
    assert_eq!(
        reader
            .evidence_for("current", &first_commit, 8)
            .unwrap()
            .len(),
        4
    );
    assert_eq!(reader.facts(16).unwrap().len(), 11);
}

fn concept_only(
    store: &Store<Rehearsal>,
    value: &mut CoreTransactionV2,
    n: u64,
    pins: &mut Vec<PreparedObject<Rehearsal>>,
) -> ObjectRef {
    let mut resources = Vec::new();
    for label in ["signature", "grounding", "relations"] {
        let object = store
            .prepare(
                &UserText {
                    schema_version: 1,
                    text: format!("{label}-{n}"),
                },
                "1",
            )
            .unwrap();
        resources.push(object.reference().clone());
        pins.push(object);
    }
    let mut concept: ConceptProposalV2 = fixture();
    concept.mention = format!("staged-only-{n}");
    concept.binding = value.binding.clone();
    concept.signature = resources[0].clone();
    concept.grounding = resources[1].clone();
    concept.relations = resources[2].clone();
    let object = store.prepare(&concept, "1").unwrap();
    let reference = object.reference().clone();
    assert!(!value.accepted_results.contains(&reference));
    assert!(!value.binding.read_set.contains(&reference));
    for resource in &resources {
        assert!(!value.accepted_results.contains(resource));
        assert!(!value.binding.read_set.contains(resource));
    }
    value.staged_concepts = vec![reference.clone()];
    pins.push(object);
    reference
}

fn replay_input_concepts(input: &RReplayInput, expected: &[ObjectRef], old_r: &ObjectRef) {
    assert_eq!(input.events.len(), expected.len());
    assert!(input.read(old_r).is_err());
    for (n, (event, reference)) in input.events.iter().zip(expected).enumerate() {
        assert_eq!(event.event.id, format!("event-{n}"));
        assert_eq!(event.staged_concepts, vec![reference.clone()]);
        let concept =
            ConceptProposalV2::from_canonical(input.read(reference).unwrap(), config().encoding)
                .unwrap();
        assert_eq!(concept.mention, format!("staged-only-{n}"));
        for (label, resource) in [
            ("signature", concept.signature),
            ("grounding", concept.grounding),
            ("relations", concept.relations),
        ] {
            assert_eq!(
                UserText::from_canonical(input.read(&resource).unwrap(), config().encoding)
                    .unwrap()
                    .text,
                format!("{label}-{n}")
            );
        }
    }
}

fn empty_rebuilt(input: &RReplayInput) -> RebuiltR {
    RebuiltR {
        snapshot: r_fixtures::snapshot(&input.law_reference),
        groups: vec![],
    }
}

#[test]
fn replay_keeps_each_events_staged_concepts_and_atomic_publication() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "replay-concepts");
    let cap = store.take_commit_capability().unwrap();
    let mut expected = Vec::new();
    for n in 0..2 {
        let e = event(&mut store, n);
        let (ticket, mut value, mut pins) = proposal(&store, &e);
        expected.push(concept_only(&store, &mut value, n, &mut pins));
        let staged = store.stage(ticket, value).unwrap();
        store.commit(&cap, staged, normal()).unwrap();
    }
    let before = store.head().unwrap();
    let old_r = store.current().unwrap().unwrap().staged_r;
    let e = event(&mut store, 2);
    let (_ticket, mut value, mut pins) = proposal(&store, &e);
    let genesis = store
        .prepare(&r_fixtures::genesis(&value.staged_q), "1")
        .unwrap();
    value.binding.read_set.push(genesis.reference().clone());
    expected.push(concept_only(&store, &mut value, 2, &mut pins));
    let pending = expected.last().unwrap();
    let ticket = store.bind(value.binding.clone()).unwrap();
    assert!(
        store
            .stage_r_replay(ticket, value.clone(), genesis.reference(), 3, |input| {
                replay_input_concepts(input, &expected, &old_r);
                Err(Error::Rejected("synthetic kernel failure"))
            })
            .is_err()
    );
    assert_eq!(store.head().unwrap(), before);
    assert!(
        store
            .reader()
            .unwrap()
            .read::<ConceptProposalV2>(pending)
            .is_err()
    );

    let ticket = store.bind(value.binding.clone()).unwrap();
    let staged = store
        .stage_r_replay(ticket, value.clone(), genesis.reference(), 3, |input| {
            replay_input_concepts(input, &expected, &old_r);
            Ok(empty_rebuilt(input))
        })
        .unwrap();
    store.set_fault(Fault::DiskFull(FaultPoint::BeforePublish));
    assert!(store.commit(&cap, staged, normal()).is_err());
    assert_eq!(store.head().unwrap(), before);
    assert!(
        store
            .reader()
            .unwrap()
            .read::<ConceptProposalV2>(pending)
            .is_err()
    );
    store.set_fault(Fault::None);

    let ticket = store.bind(value.binding.clone()).unwrap();
    let staged = store
        .stage_r_replay(ticket, value, genesis.reference(), 3, |input| {
            replay_input_concepts(input, &expected, &old_r);
            Ok(empty_rebuilt(input))
        })
        .unwrap();
    store.commit(&cap, staged, normal()).unwrap();
    store.verify().unwrap();
    drop(store);
    let store = Store::<Rehearsal>::open_temporary(&root, "replay-concepts", config()).unwrap();
    assert_eq!(
        store
            .reader()
            .unwrap()
            .read::<ConceptProposalV2>(pending)
            .unwrap()
            .mention,
        "staged-only-2"
    );
}

#[test]
fn replay_hash_checks_staged_concept_resources_before_calling_kernel() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "damaged-concept");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let before = store.head().unwrap();
    let e = event(&mut store, 1);
    let (_ticket, mut value, mut pins) = proposal(&store, &e);
    let genesis = store
        .prepare(&r_fixtures::genesis(&value.staged_q), "1")
        .unwrap();
    value.binding.read_set.push(genesis.reference().clone());
    let reference = concept_only(&store, &mut value, 1, &mut pins);
    let concept = ConceptProposalV2::from_canonical(
        &store.objects.read(&reference).unwrap(),
        config().encoding,
    )
    .unwrap();
    let corrupted = UserText {
        schema_version: 1,
        text: "signature-x".into(),
    }
    .canonical_bytes(config().encoding)
    .unwrap();
    assert_eq!(corrupted.len() as u64, concept.signature.byte_length.0);
    std::fs::write(store.objects.path(&concept.signature).unwrap(), corrupted).unwrap();
    let ticket = store.bind(value.binding.clone()).unwrap();
    let error = store
        .stage_r_replay(ticket, value, genesis.reference(), 2, |_| {
            panic!("corrupt concept reached kernel")
        })
        .err()
        .expect("corrupt concept accepted");
    assert!(
        matches!(error, Error::Rejected("object digest mismatch")),
        "{error}"
    );
    assert_eq!(store.head().unwrap(), before);
    assert_eq!(store.reader().unwrap().facts(8).unwrap().len(), 1);
}
