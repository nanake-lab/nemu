use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    History(&'static str),
    Binding(&'static str),
    MissingInput(String),
    InvalidInput(String),
    Computation(String),
}
pub type Result<T> = std::result::Result<T, Fault>;

/// Resolved, immutable resource identities, never implicit process defaults.
/// `sources` selects the effective-source projection, NOT modified raw H.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scope {
    pub law: String,
    pub interpretation: String,
    pub sources: String,
}
impl Scope {
    pub(super) fn validate(&self) -> Result<()> {
        if [&self.law, &self.interpretation, &self.sources]
            .iter()
            .any(|s| s.is_empty())
        {
            return Err(Fault::Binding("empty resource identity"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub lineage: String,
    /// Existing authoritative prefix identity supplied by the storage owner.
    /// No hash of a locally invented serialization is substituted for it.
    pub prefix: String,
    pub scope: Scope,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebuildReason {
    SourceCorrection,
    Reinterpretation,
    LawVersion,
    Repair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewVerdict {
    LawCorrect,
    LawIncorrect,
}

/// No "deny this result" operation exists. A law error is recorded; applying
/// a successor requires a separate versioned Rebuild event, never a verdict.
#[derive(Clone, Debug, PartialEq)]
pub enum Command<E> {
    Event(E),
    Pause,
    Resume,
    Rebuild {
        scope: Scope,
        reason: RebuildReason,
    },
    Review {
        alarm_id: String,
        binding: Binding,
        verdict: ReviewVerdict,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unavailable {
    Missing,
    Failed,
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckOutcome {
    Consistent,
    Inconsistent,
    Unavailable(Unavailable),
    /// Proven impossible candidate (e.g. nonfinite or V2-96 violation).
    ComputationFault(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trigger {
    ReadoutJump,
    ThetaJump,
    SlowDrift {
        /// Component identity belongs to the selected law, not this mechanism.
        component: usize,
        direction: i8,
    },
    ComputationFault,
}

/// An already recorded check. Replay must never reinterpret its timeout using
/// the current clock. Authentication/actual independence are upstream duties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckRecord {
    pub alarm_id: String,
    pub binding: Binding,
    pub triggers: Vec<Trigger>,
    pub outcome: CheckOutcome,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Commit<E> {
    pub id: String,
    pub position: u64,
    pub prefix: String,
    pub command: Command<E>,
    /// System metadata in the same committed H prefix; never evidence for G/C.
    pub check: Option<CheckRecord>,
}

/// Law implementations own mathematics and effective-H projection. Projection
/// must include *all* relevant records in canonical order, including explicit
/// capacity changes/re-proposals; no sign-based selection or model calls.
/// Different versions may use entirely different state/estimator formulas.
pub trait Kernel<E> {
    type State: Clone + Debug;
    type Input: Clone;
    /// Exact identity of the complete projected event/input, including numeric
    /// bits. Rust's ordinary f64 PartialEq equates signed zeros, and is not a
    /// sufficient cache or immutable-history comparison for arbitrary laws.
    fn same_event(&self, a: &E, b: &E) -> bool;
    fn same_input(&self, a: &Self::Input, b: &Self::Input) -> bool;
    fn genesis(&self, scope: &Scope) -> Result<Self::State>;
    fn project(&self, history: &[Commit<E>], scope: &Scope) -> Result<Vec<Self::Input>>;
    fn advance(
        &self,
        state: &Self::State,
        input: &Self::Input,
        scope: &Scope,
    ) -> Result<Self::State>;
    /// Must compare every future-affecting field, including estimator state,
    /// q/V, allocations and versions; visible b/g/lambda alone are insufficient.
    fn same_bits(&self, a: &Self::State, b: &Self::State) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditKind {
    Advance,
    Pause,
    Resume,
    Rebuild(RebuildReason),
    Deferred,
    Review {
        alarm_id: String,
        verdict: ReviewVerdict,
    },
    Rejected(CheckOutcome),
}

/// Staged H/L material. The outer transaction encodes and commits both records
/// with the state; notification delivery follows successful commit only.
#[derive(Clone, Debug)]
pub struct Audit<S> {
    pub binding: Binding,
    pub kind: AuditKind,
    pub before_scope: Scope,
    pub after_scope: Scope,
    /// Exact states allow the authoritative encoder to make a lossless diff.
    pub before: S,
    pub after: S,
    pub changed: bool,
    pub check: Option<CheckRecord>,
    /// Both recipients, as required by V2-97; this is not network delivery.
    pub notify_developer_and_caretaker: bool,
}
