//! Shared reference-test diagnostics and child lifetime management.
use crate::law::LawError;
use std::process::Child;

/// Rust carries diagnostic context; compare the reference fault category too.
pub(super) fn fault_code(error: &LawError) -> &str {
    match error {
        LawError::NumericFault(code) => code,
        LawError::NonFinite(_) => "NonFinite",
        LawError::PrecisionExhausted(_) => "PrecisionExhausted",
        LawError::Overflow("group count") => "CounterExhausted",
        other => panic!("unmapped reference fault: {other}"),
    }
}

pub(super) struct ReferenceChild(pub(super) Child);

impl Drop for ReferenceChild {
    fn drop(&mut self) {
        // A failed assertion must not leave the million-vector producer running.
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
