use crate::{Error, Result, ValidatedEvent};
use core_store::{LineageKind, Store};
use core_types::generated::{Lineage, Receipt};

/// Reuses W02's object durability, D quotas and RECEIVED_DURABLE receipt.
/// The caller holds the single writer. No receipt is invented on an I/O error;
/// publication may have happened, so recovery/retry uses the original stable key.
/// CORE_COMMITTED can only come from the runtime after its full transaction.
pub fn ingest_validated<L: LineageKind>(
    store: &mut Store<L>,
    value: ValidatedEvent,
) -> Result<Receipt> {
    if L::KIND == Lineage::EXPERIMENT {
        return Err(Error::Rejected(
            "experiment lineage has no terminal ingress",
        ));
    }
    let payload = value.payload.prepare(store, "1")?;
    let provenance = store.prepare(&value.provenance, "1")?;
    if payload.reference() != &value.event.payload_ref
        || provenance.reference() != &value.event.provenance
    {
        return Err(Error::Rejected("prepared object binding mismatch"));
    }
    Ok(store.ingest(&value.event)?)
}
