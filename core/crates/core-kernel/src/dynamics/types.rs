use super::{Fault, Result, numeric as n};
use core_types::generated::{NatureScale, NatureVector, ThetaChange, ThetaValues};

/// Temporary, versioned law-resource view, not an IDL or storage message.
/// The requested estimator-state contract is tracked in tasks/requests/P5-25-dynamics.md.
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicsLaw {
    pub version: String,
    pub baseline_prior: [f64; 4],
    pub gain_prior: [f64; 4],
    pub sigma_prior: [f64; 4],
    pub tau_prior_s: [f64; 4],
    pub baseline_weight: f64,
    pub variance_weight: f64,
    pub duration_weight: f64,
    pub change_prior: f64,
    pub change_prior_weight: f64,
    pub change_min: f64,
    pub gain_min: f64,
    pub gain_max: f64,
    pub recovery_min: f64,
    pub recovery_max: f64,
    /// Gaussian alternative-mean displacement, in sigma_prior units.
    pub detector_shift_sigma: f64,
    /// Evidence threshold on cumulative Gaussian log likelihood ratio.
    pub detector_threshold: f64,
}

/// Mutable caretaker boundaries are a separate input; they are not learned.
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicsFuses {
    pub max_step: f64,
    pub drift_run: u64,
    pub drift_margin: f64,
}

/// A trusted ingress projection. The caller must validate H provenance and
/// the stored m=0 evaluation before constructing this view; this is no authenticator.
#[derive(Clone, Debug, PartialEq)]
pub struct ReactionEvent {
    pub id: String,
    pub ordinal: u64,
    pub time_ns: u64,
    pub source: ReactionSource,
    pub evaluation_unmodulated: bool,
    pub reaction: NatureVector,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactionSource {
    ExternalWorld,
    OtherPersonReply,
    SelfAction,
    TriggeredProgramResult,
    SelfNarrative,
    DeceptionInducedReply,
}
impl ReactionSource {
    pub(super) fn learns(self) -> bool {
        matches!(self, Self::ExternalWorld | Self::OtherPersonReply)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewSignal {
    /// The entire candidate is held. No estimator state or theta value is released.
    StepLimit,
    /// Component order: b[0..4], g[4..8], lambda[8..12].
    SlowDrift { component: usize, direction: i8 },
}

/// C stages a complete state and an existing-IDL ThetaChange. It cannot write H/L/storage.
/// `held` means state() is exactly the input state. P5-26 must record the review trigger.
#[derive(Clone, Debug, PartialEq)]
pub struct StagedDynamics {
    pub(super) state: DynamicsState,
    pub change: Option<ThetaChange>,
    pub signals: Vec<ReviewSignal>,
    pub held: bool,
    pub proposed_values: Option<ThetaValues>,
}
impl StagedDynamics {
    pub fn state(&self) -> &DynamicsState {
        &self.state
    }
    pub fn into_state(self) -> DynamicsState {
        self.state
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Detector {
    pub segment_count: u64,
    pub changes: u64,
    pub mean: [f64; 4],
    pub positive: [f64; 4],
    pub negative: [f64; 4],
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Run {
    pub sign: i8,
    pub start_ns: u64,
}

/// All future-affecting estimator information is explicit here, including
/// change detection, completed-duration sums, open runs and drift counters.
/// Fields are private: constructors and C are the only mutation paths.
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicsState {
    pub(super) law: DynamicsLaw,
    pub(super) values: ThetaValues,
    pub(super) weight: f64,
    pub(super) reaction_sum: [f64; 4],
    pub(super) square_sum: [f64; 4],
    pub(super) sigma: [f64; 4],
    pub(super) duration_sum: [f64; 4],
    pub(super) duration_mass: [f64; 4],
    pub(super) tau: [f64; 4],
    pub(super) runs: [Run; 4],
    pub(super) detector: Detector,
    pub(super) event_count: u64,
    pub(super) h: f64,
    pub(super) last_event: Option<(u64, u64, String)>,
    pub(super) drift_sign: [i8; 12],
    pub(super) drift_count: [u64; 12],
}
impl DynamicsState {
    pub fn genesis(law: DynamicsLaw) -> Result<Self> {
        law.validate()?;
        let mut recovery = [0.0; 4];
        for (i, v) in recovery.iter_mut().enumerate() {
            *v = n::div(1.0, law.tau_prior_s[i])?.clamp(law.recovery_min, law.recovery_max);
        }
        let values = theta(law.baseline_prior, law.gain_prior, recovery);
        Ok(Self {
            values,
            sigma: law.sigma_prior,
            tau: law.tau_prior_s,
            h: law.change_prior.max(law.change_min),
            law,
            weight: 0.0,
            reaction_sum: [0.0; 4],
            square_sum: [0.0; 4],
            duration_sum: [0.0; 4],
            duration_mass: [0.0; 4],
            runs: std::array::from_fn(|_| Run::default()),
            detector: Detector::default(),
            event_count: 0,
            last_event: None,
            drift_sign: [0; 12],
            drift_count: [0; 12],
        })
    }
    pub fn values(&self) -> &ThetaValues {
        &self.values
    }
    pub fn change_rate(&self) -> f64 {
        self.h
    }
    pub fn change_count(&self) -> u64 {
        self.detector.changes
    }
    pub fn event_count(&self) -> u64 {
        self.event_count
    }
    pub fn sigma(&self) -> [f64; 4] {
        self.sigma
    }
    pub fn tau_s(&self) -> [f64; 4] {
        self.tau
    }
    pub fn weight_sum(&self) -> f64 {
        self.weight
    }
}

impl DynamicsLaw {
    pub fn validate(&self) -> Result<()> {
        if self.version.is_empty() {
            return Err(Fault::InvalidLaw);
        }
        for v in self.baseline_prior {
            if !v.is_finite() || !(-1.0..=1.0).contains(&v) || v.is_sign_negative() && v == 0.0 {
                return Err(Fault::InvalidLaw);
            }
        }
        for values in [&self.gain_prior, &self.sigma_prior, &self.tau_prior_s] {
            for &v in values {
                if !v.is_finite() || v <= 0.0 {
                    return Err(Fault::InvalidLaw);
                }
            }
        }
        for v in [
            self.baseline_weight,
            self.variance_weight,
            self.duration_weight,
            self.change_prior_weight,
            self.gain_min,
            self.gain_max,
            self.recovery_min,
            self.recovery_max,
            self.detector_shift_sigma,
            self.detector_threshold,
        ] {
            if !v.is_finite() || v <= 0.0 || v >= (1_u64 << 53) as f64 {
                return Err(Fault::InvalidLaw);
            }
        }
        for h in [self.change_prior, self.change_min] {
            if !h.is_finite() || h <= 0.0 || h >= 1.0 || 1.0 - h == 1.0 {
                return Err(Fault::InvalidLaw);
            }
        }
        if self.gain_min > self.gain_max
            || self.recovery_min > self.recovery_max
            || self
                .gain_prior
                .iter()
                .any(|g| *g < self.gain_min || *g > self.gain_max)
        {
            return Err(Fault::InvalidLaw);
        }
        for s in self.sigma_prior {
            n::mul(s, s)?;
        }
        Ok(())
    }
}
impl DynamicsFuses {
    pub(super) fn validate(&self) -> Result<()> {
        if !self.max_step.is_finite()
            || self.max_step <= 0.0
            || !self.drift_margin.is_finite()
            || self.drift_margin < 0.0
            || self.drift_run == 0
        {
            return Err(Fault::InvalidFuses);
        }
        n::count(self.drift_run)?;
        Ok(())
    }
}
pub(super) fn nature(v: [f64; 4]) -> NatureVector {
    NatureVector {
        schema_version: 1,
        connection: v[0],
        empathy: v[1],
        curiosity: v[2],
        achievement: v[3],
    }
}
pub(super) fn scale(v: [f64; 4]) -> NatureScale {
    NatureScale {
        schema_version: 1,
        connection: v[0],
        empathy: v[1],
        curiosity: v[2],
        achievement: v[3],
    }
}
pub(super) fn theta(b: [f64; 4], g: [f64; 4], l: [f64; 4]) -> ThetaValues {
    ThetaValues {
        schema_version: 1,
        baseline: nature(b),
        gain: scale(g),
        recovery_per_s: scale(l),
    }
}
pub(super) fn components(t: &ThetaValues) -> [f64; 12] {
    let mut v = [0.0; 12];
    v[..4].copy_from_slice(&core_types::semantic::nature(&t.baseline));
    v[4..8].copy_from_slice(&core_types::semantic::scale(&t.gain));
    v[8..].copy_from_slice(&core_types::semantic::scale(&t.recovery_per_s));
    v
}
