//! Trusted in-process stage interfaces. No stage receives Store, CommitCapability,
//! a clock, task protocol p, or an all-purpose context. Worker IPC must be adapted
//! through the semantic/permission owners before implementing these interfaces.
use crate::{Result, binding::Bound, check, validate};
use core_store::{LineageKind, Rehearsal};
use core_types::{contract::Limits, generated::*};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub x: FastStateSnapshotV2,
    pub r: RSnapshotV3,
    pub theta: ThetaSnapshot,
}
/// In-process R+ projection. No concepts, predicates, evidence or audit text.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ActiveTendency {
    pub tendency_id: String,
    pub direction: NatureVector,
    pub strength: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_types::{WireMessage, scalar::unhex};

    /// Review R1 / v2.0 §6: r_k is an observable F input even when s_k and d_k agree.
    #[test]
    fn review_r1_active_view_preserves_differences_in_typical_magnitude() {
        let limits = Limits { max_bytes: 4 << 20, max_items: 4096, max_depth: 64 };
        let mut low = RSnapshotV3::from_canonical(
            &unhex(include_str!("../tests/fixtures/RSnapshotV3.hex").trim()).unwrap(), limits,
        ).unwrap();
        let mut tendency = TendencyV3::from_canonical(
            &unhex(include_str!("../tests/fixtures/TendencyV3.hex").trim()).unwrap(), limits,
        ).unwrap();
        tendency.pi = 0.75;
        tendency.typical_magnitude = 0.1;
        low.tendencies = vec![tendency];
        let mut high = low.clone();
        high.tendencies[0].typical_magnitude = 0.8;
        assert_ne!(active(&low), active(&high), "F cannot distinguish r=0.1 from r=0.8");
    }
}
pub(crate) fn active(r: &RSnapshotV3) -> Vec<ActiveTendency> {
    r.tendencies
        .iter()
        .filter(|t| t.pi > 0.5)
        .map(|t| ActiveTendency {
            tendency_id: t.tendency_id.clone(),
            direction: t.direction.clone(),
            strength: 2.0 * t.pi - 1.0,
        })
        .collect()
}
pub struct Projection<'a> {
    pub x: &'a FastStateSnapshotV2,
    pub theta: &'a ThetaValues,
    pub active: &'a [ActiveTendency],
}
pub struct Interpretation<'a> {
    pub event: &'a EventEnvelope,
    pub facts: &'a FactProjection,
    pub binding: &'a ProposalBindingV2,
}
pub struct SemanticOutput {
    pub graph: SemanticGraph,
    pub concepts: Vec<ConceptProposalV2>,
}
pub struct Appraisal<'a> {
    pub graph: &'a SemanticGraph,
    pub binding: &'a ProposalBindingV2,
}
pub struct Matching<'a> {
    pub graph: &'a SemanticGraph,
    /// References for definition-only resolution; no pi, x, m or strengths.
    pub concepts: &'a [(String, ConceptRef)],
    pub binding: &'a ProposalBindingV2,
}
pub struct MatchOutput {
    pub record: SituationMatchRecord,
    pub candidates: Vec<TendencyCandidateRecord>,
}
pub struct Pulse<'a> {
    pub theta: &'a ThetaValues,
    pub active: &'a [ActiveTendency],
    pub projected: &'a FastStateSnapshotV2,
    pub graph: Option<&'a SemanticGraph>,
    pub appraisal: Option<&'a NatureAppraisalRecord>,
    pub matching: Option<&'a SituationMatchRecord>,
}
pub struct Learning<'a> {
    pub r: &'a RSnapshotV3,
    pub baseline: &'a NatureVector,
    pub sigma: &'a NatureScale,
    pub event: &'a EventEnvelope,
    pub attributes: &'a EventAttributesRecord,
    pub graph: &'a SemanticGraph,
    pub appraisal: &'a NatureAppraisalRecord,
    pub matching: &'a MatchOutput,
    pub binding: &'a ProposalBindingV2,
}
pub struct DynamicsInput<'a> {
    pub theta: &'a ThetaSnapshot,
    pub event: &'a EventEnvelope,
    pub appraisal: &'a NatureAppraisalRecord,
    pub binding: &'a ProposalBindingV2,
}
pub struct Change<T> {
    pub value: T,
    pub ledger: Vec<LedgerEntry>,
}
pub type StageResult<T> = std::result::Result<T, FailureCode>;
pub trait Interpret {
    fn interpret(&mut self, input: Interpretation<'_>) -> StageResult<Bound<SemanticOutput>>;
}
pub trait Appraise {
    fn appraise(&mut self, input: Appraisal<'_>) -> StageResult<NatureAppraisalRecord>;
}
pub trait Match {
    fn match_event(&mut self, input: Matching<'_>) -> StageResult<MatchOutput>;
}
pub trait Fast {
    fn project(&self, view: Projection<'_>, process_ns: u64) -> StageResult<FastStateSnapshotV2>;
    fn pulse(&self, input: Pulse<'_>) -> StageResult<FastStateSnapshotV2>;
    /// No event or time input: cannot apply a second event pulse or advance time.
    fn finish(
        &self,
        active: &[ActiveTendency],
        x: &FastStateSnapshotV2,
    ) -> StageResult<FastStateSnapshotV2>;
}
pub trait Learn {
    fn learn(&mut self, input: Learning<'_>) -> StageResult<Bound<Change<RSnapshotV3>>>;
}
pub trait Dynamics {
    fn update(&mut self, input: DynamicsInput<'_>) -> StageResult<Bound<Change<ThetaSnapshot>>>;
}
pub struct JInput<'a> {
    pub snapshot_id: &'a str,
    pub view: Projection<'a>,
    pub generation_time_ns: u64,
}
pub trait Compile {
    fn compile(&self, input: JInput<'_>) -> StageResult<ModulationEnvelopeV2>;
}

#[derive(Default)]
pub struct Components {
    pub irl: Option<Box<dyn Interpret>>,
    pub a: Option<Box<dyn Appraise>>,
    pub m: Option<Box<dyn Match>>,
    pub f: Option<Box<dyn Fast>>,
    pub g: Option<Box<dyn Learn>>,
    pub c: Option<Box<dyn Dynamics>>,
    pub j: Option<Box<dyn Compile>>,
}
pub struct StageSet<L: LineageKind> {
    pub(crate) parts: Components,
    pub(crate) limits: Limits,
    pub(crate) environment: VersionedEnvironmentV2,
    _lineage: PhantomData<L>,
}
impl<L: LineageKind> StageSet<L> {
    /// LIFE/EXPERIMENT cannot construct the incomplete rehearsal capability.
    /// Supplied adapters are trusted code; presence is not a behavioral certificate.
    pub fn complete(
        parts: Components,
        environment: VersionedEnvironmentV2,
        limits: Limits,
    ) -> Result<Self> {
        validate(&environment, limits)?;
        check(
            parts.irl.is_some()
                && parts.a.is_some()
                && parts.m.is_some()
                && parts.f.is_some()
                && parts.g.is_some()
                && parts.c.is_some()
                && parts.j.is_some(),
            "all stages are required",
        )?;
        check(
            environment.nature_appraiser.is_some()
                && environment.situation_matcher.is_some()
                && environment.direction_set.is_some()
                && environment.law_version.is_some(),
            "all stage resources are required",
        )?;
        Ok(Self {
            parts,
            limits,
            environment,
            _lineage: PhantomData,
        })
    }
}
impl StageSet<Rehearsal> {
    /// Missing modules take the explicit E13 degradation path; growth stays off.
    /// ```compile_fail
    /// use core_runtime::stages::{StageSet, Components};
    /// use core_store::Life;
    /// fn incomplete(e: core_types::generated::VersionedEnvironmentV2, l: core_types::contract::Limits) {
    ///     let _ = StageSet::<Life>::rehearsal(Components::default(), e, l);
    /// }
    /// ```
    pub fn rehearsal(
        parts: Components,
        environment: VersionedEnvironmentV2,
        limits: Limits,
    ) -> Result<Self> {
        validate(&environment, limits)?;
        Ok(Self {
            parts,
            limits,
            environment,
            _lineage: PhantomData,
        })
    }
}
