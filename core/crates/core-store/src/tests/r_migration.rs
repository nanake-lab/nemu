use super::*;

fn replay_proposal(
    store: &Store<Rehearsal>,
    e: &EventEnvelope,
) -> (
    ReadTicket<Rehearsal>,
    CoreTransactionV2,
    Vec<PreparedObject<Rehearsal>>,
    ObjectRef,
) {
    let (_ticket, mut value, mut pins) = proposal(store, e);
    let g = store
        .prepare(&r_fixtures::genesis(&value.staged_q), "1")
        .unwrap();
    let genesis = g.reference().clone();
    value.binding.read_set.push(genesis.clone());
    let ticket = store.bind(value.binding.clone()).unwrap();
    pins.push(g);
    (ticket, value, pins, genesis)
}

// Test double only: checks the storage seam, not production G or AC2-19.
fn rebuilt(input: &RReplayInput, remove: bool) -> Result<RebuiltR> {
    let mut r = r_fixtures::snapshot(&input.law_reference);
    let mut group = r_fixtures::group("legacy-g", "legacy-t");
    if remove {
        group.allocations.clear();
    } else {
        let mut t = r_fixtures::tendency("legacy-t");
        // This V is lost in B's rounding. Storage must preserve it independently.
        t.evidence_absolute = 2f64.powi(-53);
        t.evidence_a = t.evidence_absolute;
        t.evidence_b = 4.0;
        t.pi = t.evidence_a / (4.0 + t.evidence_absolute);
        t.group_count = DecimalU64(1);
        group.allocations[0].support = t.evidence_absolute;
        group.allocations[0].mu = 0.0;
        group.allocations[0].projection = None;
        r.tendencies.push(t);
    }
    Ok(RebuiltR {
        snapshot: r,
        groups: vec![group],
    })
}

#[test]
fn bound_law_and_genesis_are_required_for_replay() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "bound-resources");
    let e = event(&mut store, 0);
    let (ticket, mut value, _pins) = proposal(&store, &e);
    let mut other_law = r_fixtures::law();
    other_law.version = 2;
    let lp = store.prepare(&other_law, "2").unwrap();
    let rp = store
        .prepare_r(&r_fixtures::snapshot(lp.reference()))
        .unwrap();
    value.staged_r = rp.reference().clone();
    assert!(store.stage(ticket, value).is_err());
    let (ticket, value, _pins) = proposal(&store, &e);
    let gp = store
        .prepare(&r_fixtures::genesis(&value.staged_q), "1")
        .unwrap();
    assert!(
        store
            .stage_r_replay(ticket, value, gp.reference(), 1, |_| panic!(
                "unbound genesis reached kernel"
            ))
            .is_err()
    );
    assert_eq!(store.head().unwrap(), (None, 0, 0));
}

#[test]
fn canonical_v2_and_v3_evidence_indexes_are_checked_on_recovery() {
    for legacy in [true, false] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "bad-index");
        let cap = store.take_commit_capability().unwrap();
        r_legacy_fixture::seed(&mut store);
        if !legacy {
            let e = event(&mut store, 1);
            let (ticket, value, _pins, genesis) = replay_proposal(&store, &e);
            let staged = store
                .stage_r_replay(ticket, value, &genesis, 2, |input| rebuilt(input, false))
                .unwrap();
            store.commit(&cap, staged, normal()).unwrap();
        }
        let head = store.head().unwrap().0.unwrap();
        // Deliberate corruption of a synthetic DB, bypassing its append-only trigger.
        store
            .db
            .execute_batch("DROP TRIGGER immutable_evidence_index_UPDATE")
            .unwrap();
        store
            .db
            .execute(
                "UPDATE evidence_index SET tendency_id='wrong' WHERE commit_id=?1",
                [head],
            )
            .unwrap();
        assert!(store.verify().is_err());
        drop(store);
        assert!(Store::<Rehearsal>::open_temporary(&root, "bad-index", config()).is_err());
    }
}

#[test]
fn historical_v2_is_readable_and_cannot_be_newly_published() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "legacy-read");
    let (id, r, group) = r_legacy_fixture::seed(&mut store);
    let expected = store.objects.read(&r).unwrap();
    let reader = store.reader().unwrap();
    assert_eq!(
        reader
            .read::<RSnapshotV2>(&r)
            .unwrap()
            .canonical_bytes(config().encoding)
            .unwrap(),
        expected
    );
    assert_eq!(
        reader.read::<EvidenceGroupV2>(&group).unwrap().group_id,
        "legacy-g"
    );
    assert!(matches!(
        &reader.evidence_history_for("legacy-t", &id, 10).unwrap()[0],
        StoredEvidenceGroup::V2(_)
    ));
    assert!(reader.evidence_for("legacy-t", &id, 10).is_err());
    assert!(store.prepare(&fixture::<TendencyV2>(), "1").is_err());
    assert!(
        store
            .prepare(&fixture::<EvidenceAllocationV2>(), "1")
            .is_err()
    );
    assert!(store.prepare(&fixture::<EvidenceGroupV2>(), "1").is_err());
    assert!(store.prepare(&fixture::<RSnapshotV2>(), "1").is_err());
    let e = event(&mut store, 1);
    let (ticket, value, _pins) = proposal(&store, &e);
    let before = store.head().unwrap();
    assert!(store.stage(ticket, value).is_err());
    assert_eq!(store.head().unwrap(), before);
    drop(reader);
    drop(store);
    let mut store = Store::<Rehearsal>::open_temporary(&root, "legacy-read", config()).unwrap();
    store.collect_garbage().unwrap();
    assert_eq!(
        store
            .reader()
            .unwrap()
            .read::<RSnapshotV2>(&r)
            .unwrap()
            .canonical_bytes(config().encoding)
            .unwrap(),
        expected
    );
    let path = store.objects.path(&r).unwrap();
    drop(store);
    std::fs::write(path, b"damaged legacy index").unwrap();
    assert!(Store::<Rehearsal>::open_temporary(&root, "legacy-read", config()).is_err());
}

#[test]
fn full_h_replay_preserves_q_v_and_historical_versions_or_removes_association() {
    for remove in [false, true] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&root, "migrate");
        let cap = store.take_commit_capability().unwrap();
        let (old, old_r, old_group) = r_legacy_fixture::seed(&mut store);
        let old_bytes = store.objects.read(&old_r).unwrap();
        let (mut e, _payload) = unpersisted_event(&mut store, 1);
        e.correction_of = vec!["event-0".into()];
        store.ingest(&e).unwrap();
        let (ticket, value, _pins, genesis) = replay_proposal(&store, &e);
        let staged = store
            .stage_r_replay(ticket, value, &genesis, 2, |input| {
                assert_eq!(input.events.len(), 2);
                assert_eq!(input.events[0].event.id, "event-0");
                assert_eq!(input.events[1].event.correction_of, vec!["event-0"]);
                assert!(input.read(&old_r).is_err());
                assert!(input.read(&old_group).is_err());
                let payload = &input.events[0].event.payload_ref;
                assert_eq!(
                    UserText::from_canonical(input.read(payload)?, config().encoding)?.text,
                    "synthetic-0"
                );
                rebuilt(input, remove)
            })
            .unwrap();
        store.collect_garbage().unwrap();
        let new = store.commit(&cap, staged, normal()).unwrap();
        store.verify().unwrap();
        drop(store);
        let store = Store::<Rehearsal>::open_temporary(&root, "migrate", config()).unwrap();
        let reader = store.reader().unwrap();
        let groups = reader.evidence_for("legacy-t", &new.commit_id, 10).unwrap();
        if remove {
            assert!(groups.is_empty());
        } else {
            assert_eq!(
                groups[0].allocations[0].actual_retention.to_bits(),
                0.75f64.to_bits()
            );
            let r: RSnapshotV3 = reader
                .read(&store.current().unwrap().unwrap().staged_r)
                .unwrap();
            assert_eq!(
                r.tendencies[0].evidence_absolute.to_bits(),
                2f64.powi(-53).to_bits()
            );
            assert_eq!(r.tendencies[0].evidence_b - 4.0, 0.0);
        }
        assert!(matches!(
            &reader.evidence_history_for("legacy-t", &old, 10).unwrap()[0],
            StoredEvidenceGroup::V2(_)
        ));
        assert_eq!(store.objects.read(&old_r).unwrap(), old_bytes);
    }
}

#[test]
fn replay_budget_kernel_and_publication_failures_leave_old_head() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "replay-failures");
    let cap = store.take_commit_capability().unwrap();
    r_legacy_fixture::seed(&mut store);
    let before = store.head().unwrap();
    let e = event(&mut store, 1);
    let (ticket, value, _pins, genesis) = replay_proposal(&store, &e);
    assert!(
        store
            .stage_r_replay(ticket, value, &genesis, 1, |_| panic!(
                "truncated input reached kernel"
            ))
            .is_err()
    );
    let (ticket, value, _pins, genesis) = replay_proposal(&store, &e);
    assert!(
        store
            .stage_r_replay(ticket, value, &genesis, 2, |_| Err(Error::Rejected(
                "kernel unavailable"
            )))
            .is_err()
    );
    let (ticket, value, _pins, genesis) = replay_proposal(&store, &e);
    assert!(
        store
            .stage_r_replay(ticket, value, &genesis, 2, |input| {
                let mut result = rebuilt(input, true)?;
                result.groups.clear(); // Cannot resurrect or leave a stale legacy group.
                Ok(result)
            })
            .is_err()
    );
    let (ticket, value, _pins, genesis) = replay_proposal(&store, &e);
    let staged = store
        .stage_r_replay(ticket, value, &genesis, 2, |input| rebuilt(input, false))
        .unwrap();
    store.set_fault(Fault::DiskFull(FaultPoint::BeforePublish));
    assert!(store.commit(&cap, staged, normal()).is_err());
    assert_eq!(store.head().unwrap(), before);
    store.set_fault(Fault::None);
    store.verify().unwrap();
}

#[test]
fn v3_law_semantics_and_ordinals_are_checked_before_publication() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "v3-invalid");
    let e = event(&mut store, 0);
    let (ticket, mut value, _pins) = proposal(&store, &e);
    let law = value.binding.environment.law_version.clone().unwrap();
    let mut r = r_fixtures::snapshot(&law);
    r.tendencies.push(r_fixtures::tendency("legacy-t"));
    assert!(store.prepare(&r, "1").is_err());
    for field in ["B", "pi", "r", "V"] {
        let mut wrong = r.clone();
        match field {
            "B" => wrong.tendencies[0].evidence_b = 5.0,
            "pi" => wrong.tendencies[0].pi = 0.1,
            "r" => wrong.tendencies[0].typical_magnitude = 0.1,
            _ => wrong.tendencies[0].evidence_absolute = -1.0,
        }
        assert!(store.prepare_r(&wrong).is_err(), "{field}");
    }
    r.tendencies[0].group_count = DecimalU64(1);
    let rp = store.prepare_r(&r).unwrap();
    value.staged_r = rp.reference().clone();
    let mut group = r_fixtures::group("g", "legacy-t");
    group.allocations[0].actual_retention = 0.0;
    assert!(store.prepare(&group, "1").is_err());
    group.allocations[0].actual_retention = 0.75;
    group.allocations[0].ordinal = DecimalU64(1);
    let gp = store.prepare(&group, "1").unwrap();
    value.accepted_results.push(gp.reference().clone());
    assert!(store.stage(ticket, value).is_err());
    assert_eq!(store.head().unwrap(), (None, 0, 0));
}
