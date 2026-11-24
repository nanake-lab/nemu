use super::*;

// Called in the existing one-use TemporaryRoot child harness, which terminates
// the writer process at the selected W02 barrier without running destructors.
pub(super) fn publish(
    store: &mut Store<Rehearsal>,
    cap: &CommitCapability<Rehearsal>,
    fault: Fault,
) -> Result<ObjectRef> {
    let resource = store.prepare_raw(b"synthetic startup resource")?;
    let mut env: VersionedEnvironmentV2 = with_refs(resource.reference());
    env.implementations.clear();
    env.law_version = None;
    let mut theta: ThetaSnapshotV2 = fixture();
    let law = store.prepare(
        &with_refs::<LawVersion>(resource.reference()),
        &theta.parameters.law_id,
    )?;
    theta.law_version = law.reference().clone();
    let mut r: RSnapshotV3 = fixture();
    r.law_version = law.reference().clone();
    r.tendencies.clear();
    let mut x: FastStateSnapshotV2 = fixture();
    x.learned.clear();
    let x = store.prepare(&x, &x.revision)?;
    let r = store.prepare_r(&r)?;
    let theta = store.prepare(&theta, &theta.revision)?;
    store.set_fault(fault);
    store.publish_rehearsal_initial(cap, &env, &x, &r, &theta)
}

#[test]
fn initial_crash_before_and_after_sql_preserves_pending_ingress() {
    for (fault, committed) in [
        (FaultPoint::BeforePublish, false),
        (FaultPoint::SqlCommitted, true),
        (FaultPoint::AfterPublish, true),
    ] {
        let temp = TemporaryRoot::new().unwrap();
        let mut store = rehearsal(&temp, "crash");
        let pending = event(&mut store, 0);
        drop(store);
        super::crash_harness::run(&temp, "initial", fault, 1, None);
        let mut store = Store::<Rehearsal>::open_temporary(&temp, "crash", config()).unwrap();
        assert_eq!(store.head().unwrap().0.is_some(), committed);
        assert_eq!(store.next_raw().unwrap(), Some(pending.clone()));
        let cap = store.take_commit_capability().unwrap();
        let reference = publish(&mut store, &cap, Fault::None).unwrap();
        assert_eq!(
            store.reader().unwrap().current_snapshot().unwrap(),
            Some(reference.clone())
        );
        assert_eq!(store.next_raw().unwrap(), Some(pending));
        assert_eq!(store.head().unwrap().1, 0);
        assert_eq!(crate::store::next_commit_position(&store.db).unwrap(), 1);
        assert_eq!(
            crate::witness::capture(store.path(), &store.objects).unwrap()["commits"].get(&0),
            Some(&reference)
        );
    }
}

#[test]
fn initial_receipt_rejects_replacement_with_an_older_empty_database() {
    let temp = TemporaryRoot::new().unwrap();
    let store = rehearsal(&temp, "rollback");
    let path = store.path().join("store.sqlite");
    drop(store);
    let prior = std::fs::read(&path).unwrap();
    let mut store = Store::<Rehearsal>::open_temporary(&temp, "rollback", config()).unwrap();
    let cap = store.take_commit_capability().unwrap();
    publish(&mut store, &cap, Fault::None).unwrap();
    drop(store);
    // Simulate restoring a closed temporary DB without its newer commit.
    std::fs::write(&path, prior).unwrap();
    assert!(Reader::<Rehearsal>::open_temporary(&temp, "rollback", config()).is_err());
    assert!(Store::<Rehearsal>::open_temporary(&temp, "rollback", config()).is_err());
}
