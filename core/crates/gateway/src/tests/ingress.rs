use super::*;
use core_store::{Experiment, Rehearsal, Store, TemporaryRoot};

#[test]
fn received_durable_survives_reopen_without_claiming_core_commit() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = Store::<Rehearsal>::create_temporary(
        &root,
        "ingress",
        header::<Rehearsal>(),
        "test",
        config(),
    )
    .unwrap();
    let payload = text();
    let (e, source) = event(&payload, 0);
    let validated = validate_event(
        &session(Grants::default()),
        e.clone(),
        payload,
        source,
        limits(),
    )
    .unwrap();
    let receipt = ingest_validated(&mut store, validated).unwrap();
    assert_eq!(receipt.stage, ReceiptStage::RECEIVED_DURABLE);
    assert!(store.current().unwrap().is_none());
    assert!(store.reader().unwrap().facts(10).unwrap().is_empty());
    drop(store);
    let store = Store::<Rehearsal>::open_temporary(&root, "ingress", config()).unwrap();
    assert_eq!(store.next_raw().unwrap().unwrap(), e);
    assert_eq!(store.head().unwrap().1, 0);
    assert_eq!(store.head().unwrap().2, 0);
    store.verify().unwrap();
}
#[test]
fn queue_and_byte_limits_reject_before_success() {
    for bytes in [false, true] {
        let root = TemporaryRoot::new().unwrap();
        let mut cfg = config();
        if bytes {
            cfg.max_pending_bytes = 1;
        } else {
            cfg.max_pending_records = 1;
        }
        let mut store = Store::<Rehearsal>::create_temporary(
            &root,
            "limits",
            header::<Rehearsal>(),
            "test",
            cfg,
        )
        .unwrap();
        let attempts = if bytes { 1 } else { 2 };
        for n in 0..attempts {
            let payload = text();
            let (e, source) = event(&payload, n);
            let value =
                validate_event(&session(Grants::default()), e, payload, source, limits()).unwrap();
            let result = ingest_validated(&mut store, value);
            if bytes || n == 1 {
                assert!(result.is_err());
            } else {
                result.unwrap();
            }
        }
        assert_eq!(store.next_raw().unwrap().is_some(), !bytes);
        assert!(store.current().unwrap().is_none());
        store.verify().unwrap();
    }
}
#[test]
fn experiment_cannot_ingest_terminal_data() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = Store::<Experiment>::create_temporary(
        &root,
        "experiment",
        header::<Experiment>(),
        "test",
        config(),
    )
    .unwrap();
    let payload = text();
    let (e, source) = event(&payload, 0);
    let value = validate_event(&session(Grants::default()), e, payload, source, limits()).unwrap();
    assert!(matches!(
        ingest_validated(&mut store, value),
        Err(Error::Rejected(
            "experiment lineage has no terminal ingress"
        ))
    ));
    assert!(store.next_raw().unwrap().is_none());
}
