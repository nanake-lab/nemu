//! Pure input and output types for ADR 0042 cause-projection assembly.
//!
//! Covers ADR 0042 §§1.2, 1.3, 2.1–2.4, 3.2, 3.3, 5.1, and 5.3.

use std::collections::BTreeMap;

/// Canonical record identity bytes. Ordering uses canonical byte order (ADR 0042 §1.3).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordId(pub Vec<u8>);

/// Canonical basis identity bytes. Ordering uses canonical byte order (ADR 0042 §1.2).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BasisId(pub Vec<u8>);

/// Commit position in H. Smaller means earlier (ADR 0042 §1.2, §1.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HPos(pub u64);

/// Governance eligibility of a historical record (ADR 0042 §1.3, §3.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Eligibility {
    /// Record has a valid external root and is permitted for projection.
    Eligible,
    /// Record is known to be ineligible (no real external root, revoked/prohibited, etc.).
    Ineligible { reason: String },
    /// Record eligibility has not yet been decided.
    Undetermined,
}

/// Immutable historical record visible within the bound fact frontier (ADR 0042 §1.3, §5.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub id: RecordId,
    pub content_version: Vec<u8>,
    pub h_pos: HPos,
    /// Raw payload bytes, preserved verbatim without editing or annotation.
    pub body: Vec<u8>,
    pub eligibility: Eligibility,
}

/// Recorded relation kind between historical records (ADR 0042 §1.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationKind {
    /// Original source / body citation / paraphrase (`from` cites `to`).
    BodySource,
    /// Dialogue continuation (`from` replies to `to`).
    Continuation,
    /// Factual correction / falsification / retraction / permission (`from` corrects `to`).
    Correction,
    /// F-application, explanation, prior state, resource link, or merge record; local replay only.
    LocalOnly,
    /// Unmapped or indistinguishable relation kind.
    Unknown,
}

/// Directed relation edge between two records (ADR 0042 §1.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relation {
    pub from: RecordId,
    pub to: RecordId,
    pub kind: RelationKind,
}

/// Resolved member basis of a slot and all its effective source records up to `n` (ADR 0042 §1.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub basis: BasisId,
    /// External source record IDs from first NewBasis and effective TargetUpdates up to `n`.
    pub sources: Vec<RecordId>,
}

/// Residual basis slot with finite signed coordinates `h` (ADR 0042 §1.2, §1.3).
#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    pub basis: BasisId,
    /// All current residual coordinates (nature + learned); must be finite.
    pub h: Vec<f64>,
    /// Earliest commit position of F application in H for this slot.
    pub first_f_pos: HPos,
    /// All member bases belonging to this slot (including transitive aliases/merges).
    pub members: Vec<Member>,
    /// True if an unresolved merge governance relation touches this slot.
    pub merge_undecided: bool,
}

/// Delivered byte slice status of a dialogue entry (ADR 0042 §2.1, §3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivered {
    /// Complete record body was delivered.
    Full,
    /// Only byte interval `[start, end)` of `body` was actually delivered.
    Partial { start: u64, end: u64 },
}

/// Dialogue entry selected by the existing dialogue plan (ADR 0042 §2.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DItem {
    pub id: RecordId,
    pub delivered: Delivered,
}

/// Capacity budget and reserved output budget for the assembled request (ADR 0042 §2.1, §2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    pub capacity: u64,
    pub output_reserve: u64,
}

/// Complete pure input to cause-projection assembly (ADR 0042 §§1.2, 1.3, 2.1–2.4).
#[derive(Clone, Debug, PartialEq)]
pub struct AssemblyInput {
    /// Opaque fixed protocol bytes `P`, used whole.
    pub protocol: Vec<u8>,
    /// Fact source records explicitly cited by `P`.
    pub protocol_cited: Vec<RecordId>,
    /// Retained dialogue `D` in dialogue order.
    pub dialogue: Vec<DItem>,
    /// Latest user/input record `o`.
    pub latest: RecordId,
    /// Mandatory task facts from prior queries.
    pub required_facts: Vec<RecordId>,
    /// All records visible within the bound fact frontier.
    pub records: Vec<Record>,
    /// Recorded relations within the bound fact frontier.
    pub relations: Vec<Relation>,
    /// Whether query completeness of the bound frontier is confirmed.
    pub frontier_complete: bool,
    /// Residual slots in the current state snapshot.
    pub slots: Vec<Slot>,
    /// Whether legacy migration decomposition is unknown (ADR 0042 §1.2, §3.2).
    pub decomposition_unknown: bool,
    /// Token/byte capacity and output reserve budget.
    pub budget: Budget,
}

/// Presented dialogue item in section `D` of the request plan (ADR 0042 §2.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentedD {
    pub id: RecordId,
    pub delivered: Delivered,
    pub continuation_refs: Vec<RecordId>,
}

/// Presented experience/cause item in section `E` of the request plan (ADR 0042 §2.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentedE {
    pub id: RecordId,
    pub continuation_refs: Vec<RecordId>,
}

/// Four-section request plan `P -> D -> E -> o` (ADR 0042 §§2.1, 5.1, 5.3).
///
/// Never contains `f64`, `h`, ranking scores `s`, or rank indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestPlan {
    pub protocol: Vec<u8>,
    pub dialogue: Vec<PresentedD>,
    /// Experience section `E`, sorted by `(h_pos, id)` ascending.
    pub experience: Vec<PresentedE>,
    pub latest: RecordId,
    pub latest_continuation_refs: Vec<RecordId>,
    /// Always `false` (`TEXT_PROJECTED`, ADR 0042 §5.1).
    pub strict: bool,
}

/// Packing outcome for a single ranked candidate slot (ADR 0042 §2.2, §2.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackFate {
    /// Whole pack fit within budget; `new_records` are newly added to `E` in `(h_pos, id)` order.
    Accepted { new_records: Vec<RecordId> },
    /// First non-excluded candidate pack that overflowed capacity; packing stopped here (U2).
    FirstOverflow,
    /// Non-excluded candidate ranked after `FirstOverflow` and therefore never tried (U2).
    NotTried,
    /// Pack excluded because a member lacked external sources or a closure record was ineligible.
    Excluded {
        record: Option<RecordId>,
        reason: String,
    },
}

/// Local audit entry for one candidate pack in rank order (ADR 0042 §2.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackEntry {
    pub basis: BasisId,
    /// Complete closure of the candidate pack, sorted by `(h_pos, id)` ascending.
    pub records: Vec<RecordId>,
    pub fate: PackFate,
}

/// Local binding record for deterministic replay and auditing (ADR 0042 §2.4, §5.1, §5.3).
///
/// Never contains `f64`, `h`, ranking scores `s`, or rank indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    /// Complete base closure, sorted by `(h_pos, id)` ascending.
    pub base_records: Vec<RecordId>,
    /// All ranked candidate packs in rank order (including `Excluded`).
    pub packs: Vec<PackEntry>,
    /// Number of accepted candidate packs `K`.
    pub accepted: u32,
    /// Measured size of the final encoded request bytes.
    pub measured: u64,
}

/// Sub-status when a valid request is produced (ADR 0042 §3.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SentNote {
    /// Every eligible non-excluded candidate pack fit within budget.
    AllFit,
    /// Packing stopped at a `FirstOverflow` candidate; base (and earlier packs) still sent.
    OptionalOverBudget,
    /// Zero eligible non-excluded candidate packs existed (either none ranked or all excluded).
    NoEligibleExperience,
}

/// Projection-unavailable failure reasons; no request is sent (ADR 0042 §1.3, §3.2, §3.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unavailable {
    FrontierIncomplete,
    DecompositionUnknown,
    MergeUndecided(BasisId),
    UnknownRelation { from: RecordId, to: RecordId },
    MissingRecord(RecordId),
    EligibilityUndetermined(RecordId),
}

/// Base-request failure reasons; no request is sent (ADR 0042 §2.1, §3.2, §3.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BaseFailure {
    Ineligible {
        record: RecordId,
        reason: String,
    },
    OverCapacity {
        measured: u64,
        output_reserve: u64,
        capacity: u64,
    },
}

/// Result outcome matching ADR 0042 §3.3 receipts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Sent {
        plan: RequestPlan,
        bytes: Vec<u8>,
        binding: Binding,
        note: SentNote,
    },
    /// Receipt "投影不可得"; nothing sent.
    ProjectionUnavailable(Unavailable),
    /// Receipt "基础请求失败"; nothing sent.
    BaseRequestFailed(BaseFailure),
}

/// Caller-contract validation error (`Err(InvalidInput)`), distinct from ADR §3.3 `Outcome` receipts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvalidInput {
    /// A slot's `h` coordinate is `NaN` or `±inf` (Step 5.1).
    NonFiniteH { basis: BasisId, index: usize },
    /// Same `RecordId` appears multiple times with differing fields (ADR 0042 §1.3, Step 5.1).
    ConflictingDuplicateRecord(RecordId),
    /// Two slots share the same `BasisId` (Step 5.1).
    DuplicateSlotBasis(BasisId),
    /// A slot has an empty `members` list (Step 0 Q2 Reading A).
    EmptySlotMembers(BasisId),
    /// A locked base-material ID (`latest`, `dialogue`, `required_facts`, `protocol_cited`) is absent.
    MissingBaseRecord(RecordId),
    /// A `Delivered::Partial` slice violates `start <= end <= body.len()`.
    InvalidPartialRange {
        id: RecordId,
        start: u64,
        end: u64,
        body_len: u64,
    },
}

/// Read-only canonical table of records keyed by `RecordId` (ADR 0042 §1.3, §2.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordTable(BTreeMap<RecordId, Record>);

impl RecordTable {
    /// Constructs a canonical `RecordTable` from a validated map (ADR 0042 §1.3).
    pub(crate) fn new(map: BTreeMap<RecordId, Record>) -> Self {
        Self(map)
    }

    /// Returns the immutable record for `id`, if present within the frontier (ADR 0042 §1.3).
    pub fn get(&self, id: &RecordId) -> Option<&Record> {
        self.0.get(id)
    }
}
