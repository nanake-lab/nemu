//! Validate complete published G state; cf is an unpublishable partial audit view.
use super::engine::stored_of;
use super::input::{LawContext, RState};
use super::replay::check_with_allocations;
use super::{LawError, LawResult};
use core_types::semantic::tendency_order;
use std::collections::BTreeSet;

pub(super) fn validate(r: RState<'_>, ctx: &LawContext<'_>) -> LawResult<()> {
    if r.snapshot.law_version != *ctx.law_version {
        return Err(LawError::InvalidInput(
            "R was computed under another law version; recompute it (v2.0 §5.4)".into(),
        ));
    }
    let ts = &r.snapshot.tendencies;
    if ts
        .windows(2)
        .any(|p| !tendency_order(&p[0].tendency_id, &p[1].tendency_id).is_lt())
    {
        return Err(LawError::InconsistentState(
            "R's tendencies are not in canonical order or repeat".into(),
        ));
    }
    let ids: BTreeSet<&str> = ts.iter().map(|t| t.tendency_id.as_str()).collect();
    let mut groups = BTreeSet::new();
    for g in r.groups {
        if !g.credibility.is_finite()
            || !(0.0..=1.0).contains(&g.credibility)
            || !g.verifiability.is_finite()
            || !(0.0..=1.0).contains(&g.verifiability)
        {
            return Err(LawError::InconsistentState(
                "group factors outside [0,1]".into(),
            ));
        }
        if !groups.insert(g.group_id.as_str()) {
            return Err(LawError::InconsistentState(format!(
                "group {} repeats",
                g.group_id
            )));
        }
        let mut seen = BTreeSet::new();
        for a in &g.allocations {
            if !a.support.is_finite()
                || !(-1.0..=1.0).contains(&a.support)
                || !a.intensity.is_finite()
                || !(0.0..=ctx.law.w_max).contains(&a.intensity)
                || !a.share.is_finite()
                || !(0.0..=1.0).contains(&a.share)
                || a.share * (1u64 << 52) as f64 != (a.share * (1u64 << 52) as f64).floor()
                || (a.support == 0.0 && a.share != 0.0)
                || (a.support != 0.0 && a.share == 0.0)
                || !a.mu.is_finite()
                || !(0.0..=1.0).contains(&a.mu)
                || a.projection.is_some_and(|p| !p.is_finite())
            {
                return Err(LawError::InconsistentState(
                    "invalid allocation factors or non-dyadic share".into(),
                ));
            }
            if !ids.contains(a.tendency_id.as_str()) || !seen.insert(&a.tendency_id) {
                return Err(LawError::InconsistentState(format!(
                    "group {} allocates to {} twice or outside R",
                    g.group_id, a.tendency_id
                )));
            }
        }
        for positive in [true, false] {
            let side: Vec<_> = g
                .allocations
                .iter()
                .filter(|a| {
                    if positive {
                        a.support > 0.0
                    } else {
                        a.support < 0.0
                    }
                })
                .collect();
            if !side.is_empty() && side.iter().map(|a| a.share).sum::<f64>() != 1.0 {
                return Err(LawError::InconsistentState(
                    "I25 side does not sum to exactly one".into(),
                ));
            }
        }
    }
    for t in ts {
        // check_with_allocations checks the same count and contiguous-ordinal
        // conditions without cloning every group's legacy projection per t.
        check_with_allocations(t, &ctx.law, stored_of(r.groups.iter(), &t.tendency_id))?;
    }
    Ok(())
}
