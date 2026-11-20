//! Linked from the W02 runtime integration suite to reuse its real temporary-store bootstrap.
use super::*;
use core_runtime::cognition::{RoundEffects, finish_round};
use core_store::{ActivityOutcome, ActivityTicket};
use std::time::Instant;

#[path = "tests/fixtures.rs"]
pub(crate) mod fixtures;
use fixtures::*;

#[test]
fn local_and_remote_replay_identically_across_both_publication_boundaries() {
    for cloud in [false, true] {
        let mut oracle = None;
        for point in [
            None,
            Some(FaultPoint::BeforePublish),
            Some(FaultPoint::SqlCommitted),
        ] {
            let root = TemporaryRoot::new().unwrap();
            let mut store = open(&root);
            let cap = store.take_commit_capability().unwrap();
            seed_with_q(&mut store, &cap, initial_q);
            let first_head = store.base_commit().unwrap();
            let ticket = store.begin_activity().unwrap();
            let record = activity(&store, &ticket, cloud);
            let update = effects(&store, &ticket, &record);
            if let Some(p) = point {
                store.set_fault(Fault::DiskFull(p));
            }
            let result = finish_round(&mut store, &cap, &ticket, record.clone(), review, || {
                Ok(update)
            });
            assert_eq!(result.is_err(), point.is_some());
            drop(ticket);
            drop(store);
            let started = Instant::now();
            let mut store = Store::<Rehearsal>::open_temporary(&root, "runtime", config()).unwrap();
            let cap = store.take_commit_capability().unwrap();
            if point == Some(FaultPoint::BeforePublish) {
                assert_eq!(store.base_commit().unwrap(), first_head);
                let ticket = store.begin_activity().unwrap();
                let update = effects(&store, &ticket, &record);
                assert!(matches!(
                    finish_round(&mut store, &cap, &ticket, record.clone(), review, || Ok(
                        update
                    ))
                    .unwrap(),
                    ActivityOutcome::Published(_)
                ));
            }
            let reader = store.reader().unwrap();
            let snapshot = reader.current_snapshot().unwrap().unwrap();
            let manifest: CoreTransactionV4 = reader.read(&snapshot).unwrap();
            assert!(reader.event_snapshot(&snapshot).is_err());
            assert_eq!(manifest.processed_position.0, 1);
            let h: InternalActivityRecordV2 = reader.read(&manifest.staged_h).unwrap();
            assert_eq!(h, record);
            let q: CognitionState = reader.read(&manifest.staged_q).unwrap();
            assert_eq!(q.inflight.len(), usize::from(!cloud));
            assert_eq!(
                q.activities[0].latest_step.as_ref(),
                Some(&manifest.staged_h)
            );
            let next = store.begin_activity().unwrap();
            assert_eq!(next.input().binding.round_index.0, 1);
            let bytes = (
                manifest.canonical_bytes(limits()).unwrap(),
                q.canonical_bytes(limits()).unwrap(),
                next.input().canonical_bytes(limits()).unwrap(),
            );
            assert_eq!(oracle.get_or_insert_with(|| bytes.clone()), &bytes);
            assert_eq!(
                reader
                    .transactions_at(&store.base_commit().unwrap(), 4)
                    .unwrap()
                    .len(),
                1
            );
            println!(
                "synthetic cloud={cloud} fault={point:?} reopen/read/continue_ms={}",
                started.elapsed().as_millis()
            );
        }
    }
}

#[test]
fn rejected_empty_and_delayed_cloud_returns_only_add_audit() {
    for scenario in [
        "D8",
        "rejected",
        "empty",
        "late",
        "invalid-range",
        "invalid-utf8",
        "ingress",
    ] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = open(&root);
        let cap = store.take_commit_capability().unwrap();
        seed_with_q(&mut store, &cap, initial_q);
        let ticket = store.begin_activity().unwrap();
        let mut record = activity(&store, &ticket, true);
        if scenario == "late" {
            let update = effects(&store, &ticket, &record);
            finish_round(&mut store, &cap, &ticket, record.clone(), review, || {
                Ok(update)
            })
            .unwrap();
            let second = store.begin_activity().unwrap();
            let later = activity(&store, &second, true);
            let update = effects(&store, &second, &later);
            finish_round(&mut store, &cap, &second, later, review, || Ok(update)).unwrap();
        } else if scenario == "invalid-range" || scenario == "invalid-utf8" {
            let r = record.remote.as_mut().unwrap();
            if scenario == "invalid-range" {
                let product = store.prepare_raw(b"abc").unwrap();
                r.adoption = TextAdoption::TEXT_ADOPTION_RANGE;
                r.accepted_range = Some(Utf8Range {
                    schema_version: 1,
                    start: DecimalU64(1),
                    end: DecimalU64(4),
                });
                r.product.as_mut().unwrap().content = product.reference().clone();
            } else {
                let raw = store.prepare_raw(&[0xff]).unwrap();
                r.returned_text = Some(raw.reference().clone());
                r.product.as_mut().unwrap().content = raw.reference().clone();
                r.accepted_range.as_mut().unwrap().end = DecimalU64(1);
            }
        } else if scenario == "ingress" {
            ingress(&mut store, 1);
        } else if scenario != "D8" {
            let r = record.remote.as_mut().unwrap();
            r.product = None;
            r.accepted_range = None;
            r.adoption = if scenario == "empty" {
                r.returned_text = None;
                r.review = None;
                TextAdoption::TEXT_ADOPTION_NO_TEXT
            } else {
                TextAdoption::TEXT_ADOPTION_REJECTED
            };
        }
        let head = store.base_commit().unwrap();
        let before = store.current_manifest().unwrap();
        for _ in 0..2 {
            assert_eq!(
                finish_round(
                    &mut store,
                    &cap,
                    &ticket,
                    record.clone(),
                    |_, _, _| Ok(false),
                    rejected_effects
                )
                .unwrap(),
                ActivityOutcome::AuditOnly
            );
        }
        drop(ticket);
        drop(store);
        let mut store = Store::<Rehearsal>::open_temporary(&root, "runtime", config()).unwrap();
        if scenario == "late" {
            let cap = store.take_commit_capability().unwrap();
            store.audit_late_activity(&cap, &record).unwrap();
        }
        assert_eq!(store.base_commit().unwrap(), head);
        assert_eq!(before, store.current_manifest().unwrap());
        let audit = store
            .reader()
            .unwrap()
            .activity_attempts(&record.remote.unwrap().binding.request_id, 4)
            .unwrap();
        assert_eq!(audit.len(), 1);
    }
}

#[test]
fn cloud_cannot_change_focus_or_register_an_action() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = open(&root);
    let cap = store.take_commit_capability().unwrap();
    seed_with_q(&mut store, &cap, initial_q);
    let ticket = store.begin_activity().unwrap();
    let record = activity(&store, &ticket, true);
    let head = store.base_commit().unwrap();
    for attack in [
        "focus",
        "action",
        "query",
        "protocol",
        "interpretation",
        "read-set",
    ] {
        let mut update = effects(&store, &ticket, &record);
        match attack {
            "focus" => update.next_input.focus = None,
            "read-set" => update.next_input.binding.read_set.clear(),
            "action" => update.action = Some(("untrusted".into(), ticket.reference().clone())),
            "query" => update
                .next_input
                .retrievals
                .push(fixture::<FactProjection>()),
            "interpretation" => update.interpretations = vec![ticket.reference().clone()],
            _ => update.next_input.binding.protocol = ticket.reference().clone(),
        }
        assert!(
            finish_round(&mut store, &cap, &ticket, record.clone(), review, || Ok(
                update
            ))
            .is_err(),
            "{attack}"
        );
        assert_eq!(store.base_commit().unwrap(), head);
    }
}

#[test]
fn external_event_after_internal_round_keeps_the_two_cursors_and_sources_separate() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = open(&root);
    let cap = store.take_commit_capability().unwrap();
    let leaf = seed_with_q(&mut store, &cap, initial_q);
    let ticket = store.begin_activity().unwrap();
    let record = activity(&store, &ticket, true);
    let update = effects(&store, &ticket, &record);
    finish_round(&mut store, &cap, &ticket, record, review, || Ok(update)).unwrap();
    let inner = store.base_commit().unwrap();
    let e = ingress(&mut store, 1);
    let c = case(&store, e.clone(), leaf.reference());
    let tx = runtime::prepare_existing_head(
        &store,
        store.bind(c.binding.clone()).unwrap(),
        staged(&c),
        limits(),
    )
    .unwrap();
    let out = store.commit(&cap, tx.transaction, tx.status).unwrap();
    drop(ticket);
    drop(store);
    let store = Store::<Rehearsal>::open_temporary(&root, "runtime", config()).unwrap();
    let reader = store.reader().unwrap();
    assert_eq!(reader.transactions_at(&inner, 2).unwrap().len(), 1);
    let events = reader.transactions_at(&out.commit_id, 2).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].binding.event_id, e.id);
    assert_eq!(events[1].processed_position.0, 2);
    assert!(
        reader
            .event_snapshot(&reader.snapshot_at(&inner).unwrap().unwrap())
            .is_err()
    );
    assert_eq!(reader.event_snapshot(&out.snapshot).unwrap(), events[1]);
    store.verify().unwrap();
}

#[test]
fn local_fuses_refuse_a_plan_and_utf8_range_adopts_only_the_reviewed_slice() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = open(&root);
    let cap = store.take_commit_capability().unwrap();
    seed_with_q(&mut store, &cap, initial_q);
    let ticket = store.begin_activity().unwrap();
    let local = activity(&store, &ticket, false);
    let head = store.base_commit().unwrap();
    for (r, m) in [(0, 2), (1, 0)] {
        let mut e = effects(&store, &ticket, &local);
        (e.actions_per_round, e.inflight_capacity) = (r, m);
        assert!(finish_round(&mut store, &cap, &ticket, local.clone(), review, || Ok(e)).is_err());
        assert_eq!(store.base_commit().unwrap(), head);
    }
    let mut cloud = activity(&store, &ticket, true);
    let candidate = TEXT.strip_suffix("。").unwrap();
    let receipt = cloud.remote.as_mut().unwrap();
    receipt.adoption = TextAdoption::TEXT_ADOPTION_RANGE;
    receipt.accepted_range.as_mut().unwrap().end = DecimalU64(candidate.len() as u64);
    let slice = store.prepare_raw(candidate.as_bytes()).unwrap();
    receipt.product.as_mut().unwrap().content = slice.reference().clone();
    let e = effects(&store, &ticket, &cloud);
    let accepted = finish_round(
        &mut store,
        &cap,
        &ticket,
        cloud,
        |_, _, s| Ok(s == candidate),
        || Ok(e),
    )
    .unwrap();
    let ActivityOutcome::Published(_) = accepted else {
        panic!("reviewed range not published")
    };
    let next = store.begin_activity().unwrap();
    let content = &next.input().materials[0].content;
    let reader = store.reader().unwrap();
    assert_eq!(reader.read_raw(content).unwrap(), candidate.as_bytes());
    assert!(reader.read::<UserText>(content).is_err());
}

#[test]
fn attempt_before_and_after_sql_recovers_without_advancing_q() {
    for point in [FaultPoint::BeforePublish, FaultPoint::SqlCommitted] {
        let root = TemporaryRoot::new().unwrap();
        let mut store = open(&root);
        let cap = store.take_commit_capability().unwrap();
        seed_with_q(&mut store, &cap, initial_q);
        let ticket = store.begin_activity().unwrap();
        let record = activity(&store, &ticket, true);
        let request = record.remote.as_ref().unwrap().binding.request_id.clone();
        let head = store.base_commit().unwrap();
        store.set_fault(Fault::DiskFull(point));
        assert!(
            finish_round(
                &mut store,
                &cap,
                &ticket,
                record.clone(),
                |_, _, _| Ok(false),
                rejected_effects
            )
            .is_err()
        );
        drop(ticket);
        drop(store);
        let mut store = Store::<Rehearsal>::open_temporary(&root, "runtime", config()).unwrap();
        assert_eq!(store.base_commit().unwrap(), head);
        assert_eq!(
            store
                .reader()
                .unwrap()
                .activity_attempts(&request, 2)
                .unwrap()
                .len(),
            usize::from(point == FaultPoint::SqlCommitted)
        );
        let cap = store.take_commit_capability().unwrap();
        let ticket = store.begin_activity().unwrap();
        assert_eq!(
            finish_round(
                &mut store,
                &cap,
                &ticket,
                record,
                |_, _, _| Ok(false),
                rejected_effects
            )
            .unwrap(),
            ActivityOutcome::AuditOnly
        );
        assert_eq!(
            store
                .reader()
                .unwrap()
                .activity_attempts(&request, 2)
                .unwrap()
                .len(),
            1
        );
        store.verify().unwrap();
    }
}
