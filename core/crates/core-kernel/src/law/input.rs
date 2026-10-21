//! What G reads, as in-process values (D2). Cross-process and stored values
//! are the IDL types from `core-types`; the structs here only bundle them for
//! one call and are never serialized.
//!
//! LP8, I10: nothing here can carry x, m, the gains g or the recovery rates λ.
//! `ThetaView` holds b and σ̂ and nothing else of θ.

use std::collections::BTreeMap;

use core_types::generated::{
    ConceptRef, EvidenceGroup, FuseConfig, GenesisManifestV2, NatureScale, NatureVector, ObjectRef,
    PersonRelation, RSnapshot, RelationKind, ThetaSnapshot, TrilingualText,
};
use core_types::semantic::{nature, scale};

use super::error::{LawError, LawResult};
use super::num::Nature4;
use super::params::Law;
use super::relation::{Caretaker, RelationConcepts};

/// b and σ̂ of θ before the event (v2.0 §2.4, §8): the only parts of θ G reads.
#[derive(Clone, Debug, PartialEq)]
pub struct ThetaView {
    baseline: Nature4,
    sigma: Nature4,
}

impl ThetaView {
    pub fn new(baseline: &NatureVector, sigma: &NatureScale) -> LawResult<Self> {
        let (b, s) = (nature(baseline), scale(sigma));
        if b.iter()
            .any(|v| !v.is_finite() || !(-1.0..=1.0).contains(v))
        {
            return Err(LawError::InvalidInput("baseline outside [-1, 1]".into()));
        }
        if s.iter().any(|v| !v.is_finite() || *v <= 0.0) {
            return Err(LawError::InvalidInput("sigma must be positive".into()));
        }
        Ok(Self {
            baseline: b,
            sigma: s,
        })
    }

    /// Takes b (`values.baseline`) and σ̂ (`sigma`) from a θ snapshot.
    pub fn from_snapshot(theta: &ThetaSnapshot) -> LawResult<Self> {
        Self::new(&theta.values.baseline, &theta.sigma)
    }

    pub fn baseline(&self) -> &Nature4 {
        &self.baseline
    }

    pub fn sigma(&self) -> &Nature4 {
        &self.sigma
    }
}

/// 𝔏_v and the genesis items the count reads (V2-29).
#[derive(Clone, Debug)]
pub struct LawContext<'a> {
    /// The object reference of the law version; R′ is computed under it.
    pub law_version: &'a ObjectRef,
    pub law: Law,
    pub caretaker: Caretaker<'a>,
    /// R's capacity: a caretaker boundary from the fuse configuration.
    pub r_capacity: u32,
    pub relation_concepts: &'a RelationConcepts,
}

impl<'a> LawContext<'a> {
    pub fn new(
        genesis: &'a GenesisManifestV2,
        law_version: &'a ObjectRef,
        fuses: &'a FuseConfig,
        relation_concepts: &'a RelationConcepts,
    ) -> LawResult<Self> {
        Ok(Self {
            law_version,
            law: Law::new(&genesis.law_version.parameters)?,
            caretaker: Caretaker {
                person_id: &genesis.caretaker_id,
                attachment_prior: genesis.attachment_prior,
            },
            r_capacity: fuses.r_capacity,
            relation_concepts,
        })
    }
}

/// R as G reads it: the snapshot of tendencies and the stored summaries of
/// the evidence groups (both part of R, v2.0 §7; P5-21).
#[derive(Clone, Copy, Debug)]
pub struct RState<'a> {
    pub snapshot: &'a RSnapshot,
    pub groups: &'a [EvidenceGroup],
}

/// A candidate tendency proposed with this event (TendencyCandidateRecord),
/// with the concept registered by the semantic core (P5-06).
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    /// The candidate record's `record_id`; it becomes the tendency ID (V2-46).
    pub record_id: String,
    pub concept: ConceptRef,
    pub predicate: Vec<TrilingualText>,
    pub direction: NatureVector,
    pub relation: Option<PersonRelation>,
}

/// Who a situation match is about.
#[derive(Clone, Debug, PartialEq)]
pub enum MatchTarget {
    Tendency(String),
    /// A structural tendency about a person, created on first involvement (D6).
    Relation {
        person_id: String,
        kind: RelationKind,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Match {
    pub target: MatchTarget,
    /// μ_{c,k} ∈ (0, 1].
    pub mu: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GroupContent {
    /// The members' recorded appraisals a(e), in member order, and μ per tendency.
    Experience {
        appraisals: Vec<NatureVector>,
        matches: Vec<Match>,
    },
    /// EV7: a speaker teaches for (supports) or against a tendency.
    Teaching {
        speaker: String,
        tendency_id: String,
        supports: bool,
    },
}

/// One evidence group this event creates or changes (D2), from P5-24.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupItem {
    pub group_id: String,
    /// Commit position of the first member (D1).
    pub position: u64,
    pub source_key: String,
    /// Member events in H order.
    pub members: Vec<String>,
    /// False when the group's source cannot be verified (EV9).
    pub verifiable: bool,
    pub content: GroupContent,
}

/// Everything one committed event brings to G.
#[derive(Clone, Copy, Debug)]
pub struct Step<'a> {
    /// The revision R′ gets if anything changes.
    pub revision: &'a str,
    pub candidates: &'a [Candidate],
    /// Ordered by position, then group_id (D2).
    pub groups: &'a [GroupItem],
    /// Concept → canonical representative (P5-06's alias graph); absent means itself.
    pub aliases: &'a BTreeMap<String, String>,
}
