//! Bounded sparse effect parameters and their canonical identity.
//!
//! O 12 allows R/θ direct effects to be bounded sparse vectors. This first
//! version supports only constant sparse vectors; bounded functions of x need
//! their own interpreter version and are not accepted here.

use std::collections::BTreeMap;

use core_policy::PolicyError;
use core_policy::canonical::{Writer, digest};

use super::bound::sum_up;

/// Frozen type tag of an effect parameter object.
pub const PARAMETERS_TAG: &str = "echo.effect-parameters";

#[derive(Clone, Debug, PartialEq)]
pub struct EffectParameters {
    pub dimension: u32,
    /// Index -> finite coefficient. Indices are < `dimension`.
    pub entries: BTreeMap<u32, f64>,
}

impl EffectParameters {
    pub fn validate(&self) -> Result<(), String> {
        if self.dimension == 0 {
            return Err("zero dimension".into());
        }
        for (index, value) in &self.entries {
            if *index >= self.dimension {
                return Err(format!("index {index} >= dimension"));
            }
            if !value.is_finite() {
                return Err("non-finite coefficient".into());
            }
        }
        Ok(())
    }

    /// L1 norm summed in index order, rounded to nearest. Used for receipts
    /// (the magnitude actually applied), never for a bound check.
    pub fn l1(&self) -> f64 {
        self.entries.values().fold(0.0, |acc, v| acc + v.abs())
    }

    /// Upper bound of the exact L1 norm. L1 bounds the L2 norm from above,
    /// and rounding never lowers this value below the exact norm, so amplitude
    /// checks made with it cannot be evaded (S 6).
    pub fn l1_upper(&self) -> f64 {
        sum_up(self.entries.values().map(|v| v.abs()))
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, PolicyError> {
        let mut w = Writer::with_header(PARAMETERS_TAG)?;
        w.u32(self.dimension);
        w.count(self.entries.len())?;
        // u32 LE keys sorted by canonical bytes (ADR 0007 rule 6).
        let mut keys: Vec<(u32, f64)> = self.entries.iter().map(|(k, v)| (*k, *v)).collect();
        keys.sort_unstable_by_key(|(k, _)| k.to_le_bytes());
        for (index, value) in keys {
            w.u32(index);
            w.f64(value)?;
        }
        Ok(w.bytes)
    }

    pub fn digest(&self) -> Result<[u8; 32], PolicyError> {
        digest(PARAMETERS_TAG, &self.canonical_bytes()?)
    }
}
