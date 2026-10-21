//! Temporary R structures until P5-32 (ADR 0016) is merged.
//!
//! **P5-32 合并后换成 IDL 生成的类型**（`TendencyV2`、`EvidenceAllocationV2`、
//! `EvidenceGroupV2`、`RSnapshotV2`），并删去本文件。
//!
//! Each structure is the current IDL message plus exactly the fields P5-32 adds, under
//! the same names: a tendency's `magnitude_sum` and `magnitude_weight`; an allocation's
//! `ordinal`, `mu` and `projection`. The containers hold the extended values in place of
//! their IDL list (the `allocations` / `tendencies` field of `base` stays empty), as
//! `EvidenceGroupV2` field 8 and `RSnapshotV2` field 5 do.
//!
//! These values are crate-internal and in-process only: never serialized, persisted,
//! sent across a process boundary or exported from `core-kernel` (user decision,
//! 2026-10-22). While they exist, P5-23 stays a draft.

use core_types::generated::{EvidenceAllocation, EvidenceGroup, RSnapshot, Tendency};

/// `TendencyV2` until P5-32: the tendency and the typical-magnitude sums.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PendingTendency {
    pub(crate) base: Tendency,
    /// M_k = Σ ω·μ·p_c over the matching experience groups.
    pub(crate) magnitude_sum: f64,
    /// U_k = Σ ω·μ.
    pub(crate) magnitude_weight: f64,
}

/// `EvidenceAllocationV2` until P5-32.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PendingAllocation {
    pub(crate) base: EvidenceAllocation,
    /// Index of this group among the tendency's groups, from 0; fixed once assigned.
    pub(crate) ordinal: u64,
    /// μ_{c,k}.
    pub(crate) mu: f64,
    /// p_c = ⟨Δ_c, d_k⟩; `None` for teaching groups.
    pub(crate) projection: Option<f64>,
}

/// `EvidenceGroupV2` until P5-32: `base.allocations` stays empty.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PendingGroup {
    pub(crate) base: EvidenceGroup,
    pub(crate) allocations: Vec<PendingAllocation>,
}

/// `RSnapshotV2` until P5-32: `base.tendencies` stays empty.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PendingSnapshot {
    pub(crate) base: RSnapshot,
    pub(crate) tendencies: Vec<PendingTendency>,
}

impl PendingTendency {
    /// A tendency with no matching experience group yet: M = U = 0.
    pub(crate) fn fresh(base: Tendency) -> Self {
        Self {
            base,
            magnitude_sum: 0.0,
            magnitude_weight: 0.0,
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.base.tendency_id
    }
}
