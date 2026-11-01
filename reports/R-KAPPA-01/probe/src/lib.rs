#![forbid(unsafe_code)]
pub use core_kernel::effects;
include!(concat!(env!("OUT_DIR"), "/ablated.rs"));
pub mod kernel;
pub mod scenario;
pub mod verifier;

#[cfg(test)]
mod tests;
