//! Candidate slot ranking by residual magnitude (ADR 0042 §1.2, U1; Step 5.3).

use super::types::Slot;

/// Ranks non-zero candidate slots deterministically without exposing scores (ADR 0042 §1.2, §5.3).
///
/// Computes temporary `s = max_i |h_i|` (`0.0` when `h` is empty), drops slots with `s == 0.0`
/// (including `-0.0`), and sorts by:
/// `s` descending (`f64::total_cmp`) -> `first_f_pos` ascending -> `basis` byte order ascending.
pub(crate) fn rank_candidate_slots(slots: &[Slot]) -> Vec<&Slot> {
    let mut scored: Vec<(f64, &Slot)> = Vec::new();

    for slot in slots {
        let s = slot.h.iter().copied().map(f64::abs).fold(0.0_f64, f64::max);
        if s == 0.0 {
            continue;
        }
        scored.push((s, slot));
    }

    scored.sort_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then_with(|| a.1.first_f_pos.cmp(&b.1.first_f_pos))
            .then_with(|| a.1.basis.cmp(&b.1.basis))
    });

    scored.into_iter().map(|(_, slot)| slot).collect()
}
