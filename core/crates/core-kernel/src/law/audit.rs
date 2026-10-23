//! ADR 0019 §2/§6: audit every revised component using immutable historical q.
//! Removal and addition are separate, so opposite signs cannot hide each other.
use super::engine::Work;
use super::input::LawContext;
use super::pending::Tendency;
use super::replay::{fold_group, start};
use super::support::weight;
use super::{LawError, LawResult};

#[derive(Clone)]
struct Row {
    group: String,
    ordinal: u64,
    q: f64,
    w: f64,
    s: f64,
    mu: f64,
    p: Option<f64>,
}

fn rows(work: &Work, id: &str) -> LawResult<Vec<Row>> {
    let mut out = Vec::new();
    for g in work.groups.values() {
        for a in g.allocations.iter().filter(|a| a.tendency_id == id) {
            out.push(Row {
                group: g.group_id.clone(),
                ordinal: a.ordinal.0,
                q: a.actual_retention,
                w: weight(a.intensity, g.credibility, g.verifiability, a.share)?,
                s: a.support,
                mu: a.mu,
                p: a.projection,
            });
        }
    }
    out.sort_by_key(|r| r.ordinal);
    Ok(out)
}

fn value(t: &Tendency, rows: &[Row], ctx: &LawContext<'_>) -> LawResult<f64> {
    let mut state = start(t, &ctx.law, ctx.caretaker)?;
    for row in rows {
        fold_group(
            &mut state, &ctx.law, row.q, row.w, row.s, row.mu, row.p, false,
        )?;
    }
    Ok(state.pi)
}

fn direction(before: f64, after: f64, signed_change: f64) -> LawResult<()> {
    if (signed_change > 0.0 && after < before) || (signed_change < 0.0 && after > before) {
        return Err(LawError::NumericFault("PrecisionExhaustedWithdrawal"));
    }
    Ok(())
}

pub(super) fn revisions(before: &Work, after: &Work, ctx: &LawContext<'_>) -> LawResult<()> {
    for id in after.ordered_ids() {
        let definition = &after.now[&id];
        let mut staged = rows(before, &id)?;
        let target = rows(after, &id)?;
        // Ordinary revision cannot delete a surviving tendency's allocation or
        // create one. First allocations are withheld until after this audit.
        if staged.len() != target.len() {
            return Err(LawError::InconsistentState(
                "revision changed allocation identities".into(),
            ));
        }
        for i in 0..staged.len() {
            let old = staged[i].clone();
            let new = &target[i];
            if old.ordinal != new.ordinal
                || old.group != new.group
                || old.q.to_bits() != new.q.to_bits()
            {
                return Err(LawError::InconsistentState(
                    "revision changed historical q or ordinal".into(),
                ));
            }
            if old.w.to_bits() == new.w.to_bits()
                && old.s.to_bits() == new.s.to_bits()
                && old.mu.to_bits() == new.mu.to_bits()
                && old.p.map(f64::to_bits) == new.p.map(f64::to_bits)
            {
                continue;
            }
            let old_signal =
                super::num::nonnegative_product(old.w, old.s.abs(), "UnderflowEvidence")?;
            let new_signal =
                super::num::nonnegative_product(new.w, new.s.abs(), "UnderflowEvidence")?;
            let prev = value(definition, &staged, ctx)?;
            staged[i].w = 0.0;
            let removed = value(definition, &staged, ctx)?;
            direction(prev, removed, -old.s.signum() * old_signal)?;
            staged[i] = new.clone();
            let next = value(definition, &staged, ctx)?;
            direction(removed, next, new.s.signum() * new_signal)?;
            if old.s.signum() == new.s.signum() {
                direction(prev, next, new.s.signum() * (new_signal - old_signal))?;
            }
        }
    }
    Ok(())
}
