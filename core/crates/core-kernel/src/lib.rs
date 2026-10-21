#![forbid(unsafe_code)]
//! Echo Core kernel: pure transitions only. No storage, network, clock or
//! model access.

pub mod effects;
pub mod law;
