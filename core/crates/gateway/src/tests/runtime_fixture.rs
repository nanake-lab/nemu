//! Test-only stand-in for the not-yet-connected runtime. Uses generated frozen
//! fixtures and W02's PUBLIC writer/capability APIs, not SQL or growth operators.
//! Never compiled into gateway's production library.
use super::*;
use core_store::{CommitCapability, PreparedObject, Rehearsal, Store};

pub(super) const BRANCH: &str = "synthetic-w18-sys01";

fn with_refs<T: WireMessage>(reference: &ObjectRef) -> T {
    fn replace(v: &mut serde_json::Value, reference: &serde_json::Value) {
        if v.get("object_type").is_some() && v.get("digest").is_some() {
            *v = reference.clone();
        } else if let Some(map) = v.as_object_mut() {
            for value in map.values_mut() {
                replace(value, reference);
            }
        } else if let Some(items) = v.as_array_mut() {
            for value in items {
                replace(value, reference);
            }
        }
    }
    let mut value = serde_json::to_value(fixture::<T>()).unwrap();
    replace(&mut value, &serde_json::to_value(reference).unwrap());
    serde_json::from_value(value).unwrap()
}

pub(super) fn commit_result(
    store: &mut Store<Rehearsal>,
    cap: &CommitCapability<Rehearsal>,
) -> CoreCommitResultV2 {
    let event = store.next_raw().unwrap().unwrap();
    assert!(
        store.settle_duplicate().unwrap().is_none(),
        "new event required"
    );
    let n = event.raw_log_position.0;
    let leaf = store
        .prepare(
            &UserText {
                schema_version: 1,
                text: "synthetic fixture resource".into(),
            },
            "1",
        )
        .unwrap();
    let mut binding: ProposalBindingV2 = with_refs(leaf.reference());
    binding.branch_id = BRANCH.into();
    binding.event_id = event.id.clone();
    binding.raw_log_position = event.raw_log_position;
    binding.base_commit = store.base_commit().unwrap();
    binding.anchor_time_ns = DecimalU64(store.head().unwrap().2);
    binding.process_time_ns = DecimalU64(n + 1);
    binding.read_set = vec![leaf.reference().clone()];
    if let Some(old) = store.current().unwrap() {
        let reader = store.reader().unwrap();
        binding.x_revision = reader
            .read::<FastStateSnapshotV2>(&old.staged_x)
            .unwrap()
            .revision;
        binding.r_revision = reader.read::<RSnapshotV3>(&old.staged_r).unwrap().revision;
        binding.theta_revision = reader
            .read::<ThetaSnapshotV2>(&old.staged_theta)
            .unwrap()
            .revision;
        binding.read_set.extend([
            old.staged_h,
            old.staged_x,
            old.staged_r,
            old.staged_theta,
            old.staged_l,
            old.staged_q,
            old.staged_u,
        ]);
    }
    binding
        .read_set
        .sort_by_key(|r| core_types::scalar::hex(&r.digest.0));
    binding.read_set.dedup();
    let ticket = store.bind(binding.clone()).unwrap();
    let mut theta: ThetaSnapshotV2 = fixture();
    let law: LawVersion = with_refs(leaf.reference());
    let law = store.prepare(&law, &theta.parameters.law_id).unwrap();
    let r = RSnapshotV3 {
        schema_version: 1,
        revision: format!("synthetic-r-{n}"),
        law_version: law.reference().clone(),
        tendencies: vec![],
    };
    let r = store.prepare_r(&r).unwrap();
    theta.revision = format!("synthetic-theta-{n}");
    theta.law_version = law.reference().clone();
    let theta = store.prepare(&theta, "1").unwrap();
    let mut x: FastStateSnapshotV2 = fixture();
    x.revision = format!("synthetic-x-{n}");
    x.anchor_time_ns = DecimalU64(n + 1);
    let x = store.prepare(&x, "1").unwrap();
    let mut ledger: LedgerEntry = with_refs(leaf.reference());
    ledger.entry_id = format!("synthetic-entry-{n}");
    ledger.event_id = event.id.clone();
    let ledger_pin = store.prepare(&ledger, "1").unwrap();
    let event_pin = store.prepare(&event, "1").unwrap();
    let value = CoreTransactionV2 {
        schema_version: 1,
        staged_h: event_pin.reference().clone(),
        staged_x: x.reference().clone(),
        staged_r: r.reference().clone(),
        staged_theta: theta.reference().clone(),
        staged_l: ledger_pin.reference().clone(),
        staged_q: leaf.reference().clone(),
        staged_u: leaf.reference().clone(),
        processed_position: DecimalU64(n + 1),
        anchor_time_ns: DecimalU64(n + 1),
        staged_concepts: vec![],
        accepted_results: vec![],
        audit_refs: vec![],
        binding,
        staged_ledger: vec![ledger],
    };
    // Pins live through staging/publication; no test-only store access is needed.
    let _pins: Vec<PreparedObject<Rehearsal>> = vec![leaf, law, r, theta, x, ledger_pin, event_pin];
    let prepared = store.stage(ticket, value).unwrap();
    let result = store
        .commit(
            cap,
            prepared,
            DegradedStatus {
                schema_version: 1,
                states: vec![RunStatus::NO_GROWTH],
                reasons: vec![FailureReason {
                    schema_version: 1,
                    code: FailureCode::GROWTH_OFF_REHEARSAL,
                    detail_ref: None,
                }],
            },
        )
        .unwrap();
    assert!(result.committed);
    assert_eq!(result.event_id, event.id);
    result
}
pub(super) fn commit_next(
    store: &mut Store<Rehearsal>,
    cap: &CommitCapability<Rehearsal>,
) -> Receipt {
    let event = store.next_raw().unwrap().unwrap();
    commit_result(store, cap);
    Receipt {
        schema_version: 1,
        stable_source_key: event.stable_source_key,
        stage: ReceiptStage::CORE_COMMITTED,
        event_id: Some(event.id),
        reason: None,
    }
}
