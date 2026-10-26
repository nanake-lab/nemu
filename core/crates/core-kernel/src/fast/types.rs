//! Values of the fast state and the inputs of F.

use std::fmt;

/// x ∈ X = [−1, 1]^{4 + |R⁺|}. `learned` holds one component per tendency of
/// R⁺, in the CanonicalV1 set order of tendency IDs (the order of E_eff).
#[derive(Clone, Debug, PartialEq)]
pub struct FastState {
    /// Connection, empathy, curiosity, achievement (v2.0 §3).
    pub nature: [f64; 4],
    pub learned: Vec<LearnedValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LearnedValue {
    pub tendency_id: String,
    pub value: f64,
}

/// D(e): the drive of one event, aligned with E_eff.
#[derive(Clone, Debug, PartialEq)]
pub struct Drive {
    pub nature: [f64; 4],
    pub learned: Vec<LearnedValue>,
}

/// Where an event comes from, which decides its weight in F
/// (ADR 0026 §6.3). System records (pause, review, alarm, restore) never go
/// through F (V2-87), and an action result committed after its deadline is
/// recorded in H without driving x; the caller does not call F for them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventClass {
    /// A new exogenous event: weight 1; resets the internal count n to 0.
    NewExogenous,
    /// Other external events (desktop observations about the user only,
    /// follow-up reports of the same dwell or absence): weight 1; n unchanged.
    OtherExternal,
    /// An internal record committed while the count is `n`: weight κ^n;
    /// the count then becomes n + 1.
    InternalRecord { n: u64 },
    /// A result of her own action, weighted by the count at the round that
    /// issued it: κ^{n_issued}; n unchanged.
    ActionResult { n_issued: u64 },
}

/// The event-dependent inputs of D(e). Nothing here is x̄.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventInput {
    /// The nature appraisal a(e) ∈ [−1, 1]⁴; `None` when the appraiser is
    /// degraded, in which case the nature-channel term is 0 (card P5-10).
    pub appraisal: Option<[f64; 4]>,
    /// w for the empathy channel, in [w_min, 1]; see [`crate::fast::empathy_weight`].
    pub empathy_weight: f64,
    pub class: EventClass,
}

/// Law parameters F needs (v2.0 §16; ADR 0026 §6.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DriveLaw {
    pub g_max: f64,
    pub r_max: f64,
    pub w_min: f64,
    /// κ ∈ (0, 1).
    pub kappa: f64,
}

/// Why F refused its input. A refusal never yields a substitute state; the
/// caller keeps the transaction uncommitted (O §5.4).
#[derive(Clone, Debug, PartialEq)]
pub enum FastError {
    NonFinite(&'static str),
    OutOfRange(&'static str),
    /// The learned components are not in canonical order, repeat an ID, or
    /// do not match the tendencies of R⁺ they must align with.
    Misaligned,
    /// ‖D(e)‖∞ exceeded the conservative bound η_x (O §5.2; v2.0 §6).
    DriveOverBound {
        channel: usize,
    },
    Codec(String),
    RepresentationVersion(String),
}

impl fmt::Display for FastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for FastError {}

/// Record of a deliberate break in causal continuity during recovery
/// (O §5.3). It is written by the runtime; F only returns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinuityBreak {
    /// x was reset to the fixed x_init.
    Reset,
}
