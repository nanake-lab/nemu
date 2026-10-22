#![deny(unsafe_code)]
//! W02. Only `Store` owns the writer; readers expose committed objects only.
//! All persisted messages and hashes use the generated core-types contract.
//! The sole unsafe boundary is the documented Win32 volume query in platform.
mod config;
mod error;
mod ingress;
mod lineage;
mod objects;
#[allow(unsafe_code)]
mod platform;
mod publication;
mod queries;
mod reader;
mod recovery;
mod schema;
mod store;
mod transaction;
mod witness;

pub use config::{Fault, FaultPoint, StoreConfig};
pub use error::{Error, Result};
pub use lineage::{Experiment, Life, LineageKind, Rehearsal, TemporaryRoot};
pub use objects::PreparedObject;
pub use reader::Reader;
pub use store::Store;
pub use transaction::{CommitCapability, PreparedTransaction, ReadTicket};
#[cfg(test)]
mod tests;
