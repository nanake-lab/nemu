//! Replaceable, versioned evidence-only estimator (P5-23, 2026-10-23).
//! Its version is part of the law resource; changing it requires a new law.
use super::num::finite;
use super::{LawError, LawResult};

#[derive(Clone, Copy, Debug)]
pub struct EvidenceCounts {
    pub groups: u64,
    pub flips: u64,
}

pub trait ChangeRateEstimator {
    fn version(&self) -> &'static str;
    fn before_group(&self, evidence: EvidenceCounts) -> LawResult<f64>;
}

#[derive(Clone, Copy, Debug)]
pub struct FlipFrequency {
    pub h0: f64,
    pub n_h: f64,
    pub minimum: Option<f64>,
}

impl ChangeRateEstimator for FlipFrequency {
    fn version(&self) -> &'static str {
        "flip-frequency-v1"
    }

    fn before_group(&self, evidence: EvidenceCounts) -> LawResult<f64> {
        if evidence.groups >= 1 << 53 || evidence.flips > evidence.groups {
            return Err(LawError::Overflow("change-rate counts"));
        }
        let numerator = finite(
            finite(self.h0 * self.n_h, "h prior")? + evidence.flips as f64,
            "h numerator",
        )?;
        let denominator = finite(self.n_h + evidence.groups as f64, "h denominator")?;
        let h = finite(numerator / denominator, "h_hat")?;
        let h = self.minimum.map_or(h, |m| h.max(m));
        if !(0.0..1.0).contains(&h) {
            return Err(LawError::PrecisionExhausted(
                "change rate outside [0,1)".into(),
            ));
        }
        Ok(h)
    }
}
