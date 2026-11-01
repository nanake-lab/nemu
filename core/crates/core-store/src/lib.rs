#![deny(unsafe_code)]
//! W02. Only `Store` owns the writer; readers expose committed objects only.
//! All persisted messages and hashes use the generated core-types contract.
//! The only unsafe boundary is documented Win32 volume/publication calls in platform.
mod config;
mod error;
mod fault;
mod ingress;
mod lineage;
mod migration;
mod objects;
#[allow(unsafe_code)]
mod platform;
mod publication;
mod queries;
mod reader;
mod recovery;
mod schema;
mod store;
mod terminal_ingress;
mod theta;
mod transaction;
mod witness;

pub use config::StoreConfig;
pub use error::{Error, Result};
pub use fault::{Fault, FaultPoint};
pub use lineage::{Experiment, Life, LineageKind, Rehearsal, TemporaryRoot};
pub use objects::PreparedObject;
pub use queries::EvidenceGroupRecord;
pub use reader::Reader;
pub use store::Store;
pub use transaction::{CommitCapability, PreparedTransaction, ReadTicket};
#[cfg(test)]
mod tests;
