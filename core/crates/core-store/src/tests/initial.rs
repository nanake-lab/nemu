use super::*;
use std::{fs, path::Path};

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
fn independent_creations_with_the_same_header_and_branch_share_a_store_id() {
    let temp = TemporaryRoot::new().unwrap();
    let create = |name, header, branch| {
        let mut store =
            Store::<Rehearsal>::create_temporary(&temp, name, header, branch, config()).unwrap();
        let cap = store.take_commit_capability().unwrap();
        let reference = publish(&mut store, &cap, Fault::None).unwrap();
        let p: InitialPublication = store.reader().unwrap().read(&reference).unwrap();
        p.store_id
    };
    let header = header::<Rehearsal>();
    let id = create("first", header.clone(), "test-branch");
    assert_eq!(id, create("second", header.clone(), "test-branch"));
    assert_ne!(id, create("fork", header.clone(), "another-branch"));
    let mut later = header;
    later.created_at_ns.0 += 1;
    assert_ne!(id, create("later", later, "test-branch"));
}

fn copy_directory(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn initial_recovery_verifies_copied_bytes_but_ordinary_open_rejects_the_copy() {
    let temp = TemporaryRoot::new().unwrap();
    let source_name = "source-path-canary";
    let mut store = rehearsal(&temp, source_name);
    let cap = store.take_commit_capability().unwrap();
    let reference = publish(&mut store, &cap, Fault::None).unwrap();
    let source = store.path().to_owned();
    let header = store.header().clone();
    let branch = store.branch.clone();
    drop(store);

    let copy = temp.store_path("copy").unwrap();
    copy_directory(&source, &copy);
    copy_directory(
        &crate::witness::path(&source).unwrap(),
        &crate::witness::path(&copy).unwrap(),
    );
    let objects = crate::objects::Objects::new(copy.join("objects"), config());
    let bytes = objects.read(&reference).unwrap();
    // The full source path contains this component with either path separator.
    assert!(
        !bytes
            .windows(source_name.len())
            .any(|w| w == source_name.as_bytes())
    );

    // echoctl has no relocation entry yet. Verify the copied prefix and its receipts
    // read-only, with the original identity and without rebinding W02 metadata.
    let db = rusqlite::Connection::open_with_flags(
        copy.join("store.sqlite"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    crate::recovery::verify(&db, &objects, &header, &branch).unwrap();
    crate::witness::reconcile(&db, &copy, &objects, false, &Fault::None).unwrap();
    drop(db);
    let location_error =
        "copied or moved store requires explicit recovery; cannot resume a cloned branch";
    assert!(
        matches!(Store::<Rehearsal>::open_temporary(&temp, "copy", config()), Err(Error::Rejected(reason)) if reason == location_error)
    );
    assert!(
        matches!(Reader::<Rehearsal>::open_temporary(&temp, "copy", config()), Err(Error::Rejected(reason)) if reason == location_error)
    );
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
