//! ADR 0035 §4: A1 pure transitions. The caller resolves trusted law resources,
//! H references, basis governance and consumption eligibility before calling.
//! No H lookup, clock, storage, consumption key or inferred identity lives here.
mod codec;
mod event;
mod maintenance;
#[cfg(test)]
mod tests;

use super::{Drive, FastError};
use crate::effects::{EffectiveEffects, Evaluation};
use core_types::generated::{FastStateSnapshotV3, ObjectRef};
pub use event::{Effect, HistoryBasis};

/// Representation identifier, not a calibrated parameter (ADR 0035 §4.1).
pub const REPRESENTATION_VERSION: &str = "echo-core-ADR0035/fast-state/1";

/// Validated state; only pure transitions can change its private snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    snapshot: FastStateSnapshotV3,
}

/// ADR 0035 §§4.3, 4.9, 7: explicit trusted resource view, without κ or defaults.
pub struct Law<'a> {
    pub version: &'a ObjectRef,
    pub g_max: f64,
    pub r_max: f64,
    pub w_min: f64,
}

#[derive(Clone, Debug)]
pub struct Target {
    drive: Drive,
    law_version: ObjectRef,
}
impl Target {
    pub fn drive(&self) -> &Drive {
        &self.drive
    }
}

/// Compute the target from recorded appraisal/M and the specified prestate.
/// No current x, event class, internal count or transport time is an input.
pub fn compute_target(
    effects: &EffectiveEffects,
    evaluation: &Evaluation,
    appraisal: Option<[f64; 4]>,
    empathy_weight: f64,
    law: &Law<'_>,
) -> Result<Target, FastError> {
    super::drive::check_target_law(law.g_max, law.r_max, law.w_min)?;
    let drive = super::drive::compute_unweighted_or_legacy(
        effects,
        evaluation,
        appraisal,
        empathy_weight,
        law.g_max,
        law.r_max,
        law.w_min,
        1.0,
    )?;
    Ok(Target {
        drive,
        law_version: law.version.clone(),
    })
}

impl State {
    pub fn snapshot(&self) -> &FastStateSnapshotV3 {
        &self.snapshot
    }

    /// ADR 0035 §4.4 / I21: project from the stored shared anchor. This returns
    /// a staged value; repeated reads of self do not accumulate decay.
    pub fn project_time(&self, effects: &EffectiveEffects, at_ns: u64) -> Result<Self, FastError> {
        let delta = at_ns
            .checked_sub(self.snapshot.anchor_time_ns.0)
            .ok_or(FastError::OutOfRange("time before anchor"))?;
        let x = super::project_time(&self.visible(), effects, delta)?;
        let delta_s = delta as f64 / 1e9; // Existing Φ uses nanoseconds -> seconds.
        let mut pulls = Vec::with_capacity(4 + effects.effects.len());
        for rate in effects.theta.lambda {
            pulls.push(super::space::time_pull(rate, delta_s)?);
        }
        for e in &effects.effects {
            pulls.push(super::space::time_pull(
                super::space::learned_rate(&e.d, &effects.theta.lambda)?,
                delta_s,
            )?);
        }
        let mut next = self.clone();
        next.set_visible(&x);
        next.snapshot.anchor_time_ns.0 = at_ns;
        for slot in &mut next.snapshot.slots {
            let h = codec::values(&slot.current);
            let h = h
                .iter()
                .zip(&pulls)
                .map(|(h, pull)| finite(h + pull * (0.0 - h), "slot projection"))
                .collect::<Result<Vec<_>, _>>()?;
            codec::set_values(&mut slot.current, &h);
        }
        Ok(next)
    }
}

fn finite(v: f64, name: &'static str) -> Result<f64, FastError> {
    if !v.is_finite() {
        return Err(FastError::NonFinite(name));
    }
    Ok(if v == 0.0 { 0.0 } else { v })
}

/// ADR 0035 §4.5: every intermediate is rounded binary64; never use FMA.
fn coordinate(x: f64, h: f64, v: f64) -> Result<(f64, f64), FastError> {
    let d = finite(v - h, "replacement difference")?;
    let s = finite(x + d, "replacement sum")?;
    if s > -1.0 && s < 1.0 {
        return Ok((s, v));
    }
    let bound = if s >= 1.0 { 1.0 } else { -1.0 };
    let t = finite(h + finite(bound - x, "clipping loss")?, "slot update")?;
    Ok((bound, finite(t.clamp(h.min(v), h.max(v)), "slot update")?))
}
