//! Deterministic pure cause-projection packager implementing ADR 0042.
//!
//! Corresponds to ADR 0042:
//! - §1.2 (`rank`): candidate slot ranking by `max_i |h_i|`, `first_f_pos`, and `BasisId`.
//! - §1.3 (`closure`): source-pack and base-material closure over `BodySource`, `Correction`,
//!   `Continuation` (direct reference only), `Unknown`, and `LocalOnly` edges.
//! - §2.1–§2.4 (`assemble`): fixed `P -> D -> E -> o` order, whole-pack packing (`U2`),
//!   deduplication (`o > D > E`), and deterministic `Binding` / `Outcome` receipts.
//! - §3.2, §3.3, §5.1, §5.3: unavailability/failure receipts and strict exclusion of state/scores.

mod assemble;
mod closure;
mod rank;
mod types;
mod validate;

pub use types::*;

/// Deterministic request serializer and meter abstraction (ADR 0042 §2.1, §2.2, §2.4).
pub trait RequestCodec {
    /// Serializes a request plan into request bytes using `records` to fetch record bodies.
    fn encode(&self, plan: &RequestPlan, records: &RecordTable) -> Vec<u8>;

    /// Measures the complete serialized request size (including framing, sources, corrections).
    fn measure(&self, bytes: &[u8]) -> u64;
}

/// Assembles a deterministic four-section cause-projection request plan or receipt
/// (ADR 0042 §§1.2, 1.3, 2.1–2.4, 3.2, 3.3, 5.1, 5.3).
pub fn assemble<C: RequestCodec>(
    input: &AssemblyInput,
    codec: &C,
) -> Result<Outcome, InvalidInput> {
    assemble::assemble_impl(input, codec)
}

#[cfg(test)]
mod tests;
