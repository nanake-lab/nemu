#![deny(unsafe_code)]
//! W19. Read-only authority access through core-store; rebuildable derived indexes.
mod embedding;
mod error;
mod index;
mod query;
mod retrieval;
mod source;
#[allow(unsafe_code)]
mod sqlite_vec;
#[cfg(feature = "cpu-embedding")]
pub use embedding::CpuEmbedder;
pub use embedding::Embedder;
pub use error::{Error, Result};
pub use index::FactView;
pub use query::{Access, Budget, Budgets, FactQuery, INDEX_VERSION, Purpose, QUERY_VERSION};
pub use retrieval::{Execution, RetrievalTrace};
#[cfg(test)]
mod tests;
