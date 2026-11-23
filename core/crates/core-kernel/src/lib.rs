#![forbid(unsafe_code)]
//! Echo Core kernel: pure transitions only. No storage, network, clock or
//! model access.

pub mod dynamics;
pub mod effects;
pub mod fast;
pub mod law;
pub mod modulation;

// P5-24 evidence remains test-only. P5-30 opts into pure P5-26 staging;
// this grants no storage/publication capability or production authentication.
#[cfg(test)]
pub mod evidence;
#[cfg(any(test, feature = "synthetic-recompute"))]
pub mod recompute;
