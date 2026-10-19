#![forbid(unsafe_code)]
//! Echo Core policy: frozen scope rules and their interpretation (S 3, 12;
//! O 3). Pure functions only: no storage, network, clock or model access.

pub mod canonical;
pub mod catalog;
pub mod endpoint;
pub mod error;
pub mod overlap;
pub mod scope;

pub use catalog::{Catalog, FieldId, FieldKind, FieldSpec, ScopeLimits};
pub use endpoint::Endpoint;
pub use error::PolicyError;
pub use overlap::{Overlap, overlap};
pub use scope::{Atom, FieldValue, Scope, ScopeExpr, ScopeMatch};
