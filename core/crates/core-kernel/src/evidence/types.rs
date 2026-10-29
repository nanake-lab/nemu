use crate::law::{Candidate, GroupItem, Step};
use core_types::generated::{
    ActionStatus, ConceptRef, DeliveryStatus, EpistemicType, EventAttributesRecord, EventEnvelope,
    LieAnnotation, NatureAppraisalRecord, NatureVector, SameMatterAnnotation, SituationMatchRecord,
    TeachingAnnotation, TendencyCandidateRecord,
};
use std::collections::BTreeMap;

pub type Result<T> = std::result::Result<T, Fault>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    InvalidRecord(String),
    HistoryOrder,
    MissingReference(String),
    MissingAttributes(String),
    MissingSemantics(String),
    AliasCycle,
    Ambiguous(String),
    RecomputeRequired(String),
}

/// Non-persistable projection from the semantic owner, not a new wire contract.
/// In particular, `session` in EventEnvelope is NOT assumed to be a conversation.
#[derive(Clone, Debug, PartialEq)]
pub struct Semantics {
    pub version: String,
    pub conversation: Option<String>,
    pub kind: EventUse,
    pub utterance: Utterance,
    pub epistemic: EpistemicType,
    pub speaker: Option<String>,
    /// Canonical people identified in z as speaker, addressee or mentioned person.
    pub participants: Vec<String>,
    /// The subset whose words/actions she relies on (P5-23 trust predicate).
    pub relied_on: Vec<String>,
    /// Recorded outcome observation; absence is unknown, not failure.
    pub outcome: Outcome,
    /// Per-tendency provenance of the match. Every recorded μ needs an entry.
    pub match_basis: BTreeMap<String, MatchBasis>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventUse {
    Experience,
    /// EV8: the document's propositions, not the separate reading reaction.
    Knowledge,
    OwnNarrative,
    /// V2-87: maintenance about her, not an experience happening to her.
    Maintenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Utterance {
    TeachingAboutHer,
    SelfReport,
    Request,
    Fact,
    Other,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    pub action: Option<ActionStatus>,
    pub delivery: Option<DeliveryStatus>,
    /// True only for a recorded observation of the relevant result (e.g. read receipt).
    /// It must not be inferred from the absence of a reply or from a(e) == 0.
    pub observed: bool,
}

impl Outcome {
    pub(super) fn known(&self) -> bool {
        self.action != Some(ActionStatus::UNKNOWN_ACTION)
            && self.delivery != Some(DeliveryStatus::UNKNOWN_DELIVERY)
            && (self.observed
                || matches!(
                    self.action,
                    Some(ActionStatus::SUCCEEDED | ActionStatus::FAILED)
                ))
    }
}

/// Semantic/action mapping is an upstream prerequisite, never parsed from the
/// candidate's human-readable predicate by this module. No numerical threshold
/// or similarity metric is invented for the comparison context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchBasis {
    Reaction,
    ReadContent,
    Outcome,
    OwnActionOutcome {
        /// Exact context identity supplied by the versioned semantic mapping.
        context: String,
        /// None means a recorded comparable case without that action.
        variant: Option<String>,
        /// H event of the action, required exactly when variant is present.
        action_event: Option<String>,
    },
}

/// P5-06 registration of an existing generated candidate record. This view
/// cannot register a concept or attest that the registration was committed.
#[derive(Clone, Debug, PartialEq)]
pub struct RegisteredCandidate {
    pub record: TendencyCandidateRecord,
    pub concept: ConceptRef,
}

/// One H commit's event and the governed records appended at that position.
/// A late record's target may be older; binding.event_id names THIS event.
/// The caller supplies the complete immutable accepted prefix, not proposals.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub event: EventEnvelope,
    pub attributes: Option<EventAttributesRecord>,
    pub semantics: Option<Semantics>,
    pub appraisals: Vec<NatureAppraisalRecord>,
    pub situations: Vec<SituationMatchRecord>,
    pub teachings: Vec<TeachingAnnotation>,
    pub same_matters: Vec<SameMatterAnnotation>,
    pub lies: Vec<LieAnnotation>,
    pub candidates: Vec<RegisteredCandidate>,
    /// Actual committed order of simultaneous canonical candidate identities,
    /// including newly involved structural relations. Required when H position
    /// alone cannot order them. No ID-based tie-break substitutes for this input.
    pub simultaneous_proposals: Option<Vec<String>>,
    /// Accepted structural entry/re-entry proposals at this commit, canonical
    /// identities. Some(empty) means no entry; None means the adapter has not
    /// supplied intent. The originating decision belongs in H, not a side queue.
    pub structural_admissions: Option<Vec<String>>,
}

#[derive(Clone, Copy)]
pub struct Prefix<'a> {
    pub frames: &'a [Frame],
    /// Fixed versions of the supplied recorded projections; no implicit defaults.
    pub attribute_version: &'a str,
    pub semantic_version: &'a str,
    /// Complete effective alias maps for this prefix; correction/reinterpretation
    /// is replayed by P5-26, never patched into an already counted G state.
    pub concept_aliases: &'a BTreeMap<String, String>,
    pub person_aliases: &'a BTreeMap<String, String>,
    /// Genesis candidate IDs in their recorded order (including u* relations).
    pub genesis_order: &'a [String],
    pub limits: core_types::contract::Limits,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExternalEvent {
    pub event_id: String,
    pub position: u64,
    pub appraisal_record: String,
    pub appraisal: NatureVector,
}

/// A complete projection, not a serialized authority or a cache.
#[derive(Clone, Debug, PartialEq)]
pub struct Assembly {
    /// Last commit's entry intent, copied from the governed H projection.
    pub structural_admissions: Option<Vec<String>>,
    pub groups: Vec<GroupItem>,
    pub external: Vec<ExternalEvent>,
    /// Proposals at the last commit, with canonical identity preserved on re-entry.
    pub candidates: Vec<Candidate>,
    pub proposal_order: Vec<String>,
    pub aliases: BTreeMap<String, String>,
}

impl Assembly {
    /// Borrow a prepared input for G. For commits (including re-proposals),
    /// call `delta` first: a raw full assembly has not routed structural entry.
    /// Initial/replay callers must also supply the recorded entry intent.
    /// A source correction,
    /// newly discovered lie, merge or reinterpretation needs P5-26 recomputation.
    pub fn step<'a>(&'a self, revision: &'a str) -> Step<'a> {
        Step {
            revision,
            candidates: &self.candidates,
            groups: &self.groups,
            aliases: &self.aliases,
            proposal_order: &self.proposal_order,
        }
    }

    pub fn delta(&self, previous: &Self) -> Result<Self> {
        let out = self.checked_delta(previous)?;
        super::delta::route(self, out)
    }

    pub(super) fn checked_delta(&self, previous: &Self) -> Result<Self> {
        if !self.external.starts_with(&previous.external) {
            return Err(Fault::RecomputeRequired("external history changed".into()));
        }
        if self.aliases != previous.aliases {
            return Err(Fault::RecomputeRequired("concept aliases changed".into()));
        }
        for old in &previous.groups {
            let Some(new) = self.groups.iter().find(|g| g.group_id == old.group_id) else {
                return Err(Fault::RecomputeRequired(old.group_id.clone()));
            };
            if !super::groups::incremental_extension(old, new) {
                return Err(Fault::RecomputeRequired(old.group_id.clone()));
            }
        }
        let mut out = self.clone();
        out.external.drain(..previous.external.len());
        if out.candidates.is_empty() {
            out.groups.retain(|g| !previous.groups.contains(g));
        }
        Ok(out)
    }
}
