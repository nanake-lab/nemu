//! Version dispatch at the storage boundary. Legacy bytes remain read-only;
//! only an explicit successor migration can supply their missing estimator.
use crate::{Error, Result, objects::Objects};
use core_types::{
    WireMessage,
    contract::Schema,
    generated::{ObjectRef, ThetaSnapshot, ThetaSnapshotV2},
};

pub(crate) const LEGACY_READ_ONLY: &str = "legacy theta head is read-only; explicit complete-estimator migration is required before writing";

pub(crate) enum Snapshot {
    Legacy,
    Current(Box<ThetaSnapshotV2>),
}

pub(crate) fn read(objects: &Objects, reference: &ObjectRef) -> Result<Snapshot> {
    let schema = Schema::builtin()?;
    let limits = objects.config.encoding;
    if reference.object_type == schema.message(ThetaSnapshotV2::NAME)?.tag {
        // Objects::read checks both the content digest and full V2 semantics.
        let value = ThetaSnapshotV2::from_canonical(&objects.read(reference)?, limits)?;
        Ok(Snapshot::Current(Box::new(value)))
    } else if reference.object_type == schema.message(ThetaSnapshot::NAME)?.tag {
        ThetaSnapshot::from_canonical(&objects.read(reference)?, limits)?;
        Ok(Snapshot::Legacy)
    } else {
        Err(Error::Rejected("unsupported theta snapshot object type"))
    }
}

pub(crate) fn writable(objects: &Objects, reference: &ObjectRef) -> Result<ThetaSnapshotV2> {
    match read(objects, reference)? {
        Snapshot::Current(value) => Ok(*value),
        Snapshot::Legacy => Err(Error::Rejected(LEGACY_READ_ONLY)),
    }
}
