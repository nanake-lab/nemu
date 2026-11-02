//! Narrow inputs and outputs of the two effect phases. The caller builds the
//! inputs from the committed R and θ; nothing here is a general context map.

use std::fmt;

/// Processing time in nanoseconds on the committed time axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessTime(pub u64);

/// Validity snapshot of ADR 0008 §2. Echo Core v2.0 has no validity contracts
/// (§15), so it carries nothing; it stays in the signature so callers keep the
/// shape fixed by ADR 0008.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Validity;

/// One stored tendency as far as its effect is concerned (v2.0 §5.2, §7).
/// Candidates (π ≤ ½) may be passed; they have no effect (I06).
#[derive(Clone, Debug, PartialEq)]
pub struct StoredTendency {
    pub tendency_id: String,
    /// Reference of the tendency's concept (name, definition, examples).
    pub concept: String,
    /// π_k ∈ (−1, 1).
    pub pi: f64,
    /// Typical magnitude r_k ≥ 0, already clipped to [0, r_max] by the law.
    pub r: f64,
    /// Predicted direction d_k ∈ ℝ⁴, a unit vector, in the fixed channel order
    /// connection, empathy, curiosity, achievement.
    pub d: [f64; 4],
    /// Whether the concept's model direction u_k passed validation (v2.0 §9.6).
    /// Passed through for J; it does not change f_k or D_k.
    pub direction_validated: bool,
}

/// The stored R as a list of tendencies. Order does not matter: the compiler
/// sorts by tendency ID in the CanonicalV1 set order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StoredR {
    pub tendencies: Vec<StoredTendency>,
}

/// The stored θ = (b, g, λ), per nature channel in the fixed order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StoredTheta {
    pub b: [f64; 4],
    pub g: [f64; 4],
    pub lambda: [f64; 4],
}

/// One effect in E_eff: a tendency of R⁺ with s_k > 0.
#[derive(Clone, Debug, PartialEq)]
pub struct TendencyEffect {
    pub tendency_id: String,
    pub concept: String,
    /// s_k = max(0, 2π_k − 1) ∈ (0, 1).
    pub s: f64,
    pub r: f64,
    pub d: [f64; 4],
    pub direction_validated: bool,
}

/// E_eff: the effects of R⁺ in canonical tendency order, and θ.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectiveEffects {
    pub effects: Vec<TendencyEffect>,
    pub theta: StoredTheta,
}

/// Situation match of one event: μ_k(e) ∈ [0, 1] per tendency ID. Tendencies
/// not listed have μ = 0. Entries for tendencies outside R⁺ are ignored, since
/// the matcher also scores candidates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SituationMatch {
    pub mu: Vec<(String, f64)>,
}

/// Contribution of one tendency of R⁺ to one event (v2.0 §6).
#[derive(Clone, Debug, PartialEq)]
pub struct TendencyDrive {
    pub tendency_id: String,
    /// μ_k(e) as given (0 when not listed).
    pub mu: f64,
    /// f_k(e) = s_k μ_k(e) r_k d_k, added to the nature channels.
    pub f: [f64; 4],
    /// D_k(e) = s_k μ_k(e), the drive of the learned component x_k.
    pub drive: f64,
}

/// Result of [`crate::effects::evaluate_effects`].
#[derive(Clone, Debug, PartialEq)]
pub struct Evaluation {
    /// One entry per tendency of R⁺, in canonical tendency order.
    pub drives: Vec<TendencyDrive>,
    /// Σ_k f_k(e) per channel, summed in canonical tendency order.
    pub f_sum: [f64; 4],
}

/// Why compilation refused the stored state. A refusal is a structural fault
/// of the committed snapshot: the caller must not substitute partial effects.
#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    NonFinite {
        tendency_id: Option<String>,
        field: &'static str,
    },
    PiOutOfRange {
        tendency_id: String,
    },
    NegativeMagnitude {
        tendency_id: String,
    },
    DirectionNotUnit {
        tendency_id: String,
    },
    DuplicateTendency {
        tendency_id: String,
    },
    EmptyIdentifier,
    ThetaOutOfRange {
        field: &'static str,
    },
}

/// Why evaluation refused the situation match.
#[derive(Clone, Debug, PartialEq)]
pub enum EvaluateError {
    MuOutOfRange { tendency_id: String },
    DuplicateMatch { tendency_id: String },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for EvaluateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CompileError {}
impl std::error::Error for EvaluateError {}
