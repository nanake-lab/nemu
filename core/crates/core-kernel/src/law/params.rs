//! The law parameters G reads (v2.0 §5.2, §16), validated once.
//!
//! Values come from the law version 𝔏_v (`LawParameters`); the attachment prior
//! and the caretaker come from genesis and the capacity from the fuse
//! configuration (V2-29). Nothing here reads x, m, g or λ (LP8, I10).

use core_types::generated::LawParameters;
use core_types::semantic;

use super::error::{LawError, LawResult};

/// Law parameters used by the tendency count.
#[derive(Clone, Debug, PartialEq)]
pub struct Law {
    pub n0: f64,
    pub kappa1: f64,
    pub w_max: f64,
    pub w_teach: f64,
    pub v_u: f64,
    pub r_min: f64,
    pub r_max: f64,
    pub h0: f64,
    pub n_h: f64,
    pub h_hat_min: Option<f64>,
    pub w_min: f64,
}

impl Law {
    /// Validate the parameters G needs. Beyond the cross-field checks of
    /// `core-types` (V2-32), G needs n_h > 0 (ĥ is defined before the first
    /// group) and r_min > 0 (the predicted magnitude r̃ divides the support).
    pub fn new(p: &LawParameters) -> LawResult<Self> {
        semantic::check_law_parameters(p).map_err(|e| LawError::InvalidInput(e.to_string()))?;
        let all = [
            p.n0, p.kappa1, p.w_max, p.w_teach, p.v_u, p.r_min, p.r_max, p.h0, p.n_h, p.w_min,
        ];
        if all.iter().any(|v| !v.is_finite() || *v < 0.0) {
            return Err(LawError::InvalidParameter(
                "law parameters must be finite and nonnegative",
            ));
        }
        if p.n_h <= 0.0 {
            return Err(LawError::InvalidParameter("n_h must be positive"));
        }
        if p.r_min <= 0.0 {
            return Err(LawError::InvalidParameter("r_min must be positive"));
        }
        if p.h0 > 1.0 || p.v_u > 1.0 || p.w_min > 1.0 {
            return Err(LawError::InvalidParameter(
                "h0, v_u and w_min lie in [0, 1]",
            ));
        }
        if let Some(m) = p.h_hat_min
            && (!m.is_finite() || !(0.0..=1.0).contains(&m))
        {
            return Err(LawError::InvalidParameter("h_hat_min lies in [0, 1]"));
        }
        Ok(Self {
            n0: p.n0,
            kappa1: p.kappa1,
            w_max: p.w_max,
            w_teach: p.w_teach,
            v_u: p.v_u,
            r_min: p.r_min,
            r_max: p.r_max,
            h0: p.h0,
            n_h: p.n_h,
            h_hat_min: p.h_hat_min,
            w_min: p.w_min,
        })
    }

    /// κ_k = κ₁ × (number of conditions in the situation predicate).
    pub fn kappa(&self, conditions: usize) -> LawResult<f64> {
        let count = u32::try_from(conditions).map_err(|_| LawError::Overflow("predicate"))?;
        super::num::finite(self.kappa1 * f64::from(count), "kappa")
    }

    /// c₀ = n₀ + κ_k: the part of B_k that never decays.
    pub fn base(&self, conditions: usize) -> LawResult<f64> {
        super::num::finite(self.n0 + self.kappa(conditions)?, "n0 + kappa")
    }

    /// ĥ_k = (h₀·n_h + F_k)/(n_h + N_k), floored at ĥ_min when the law sets one.
    pub fn h_hat(&self, groups: u64, flips: u64) -> LawResult<f64> {
        let h = (self.h0 * self.n_h + flips as f64) / (self.n_h + groups as f64);
        let h = match self.h_hat_min {
            Some(m) => h.max(m),
            None => h,
        };
        super::num::finite(h, "h_hat")
    }
}
