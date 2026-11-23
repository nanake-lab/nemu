//! ADR 0019 §2/§6: audit every revised component using immutable historical q.
//! Opposite-sign replacements are split so their effects cannot hide each other.
use super::engine::Work;
use super::input::LawContext;
use super::replay::{fold_group, start};
use super::state::Tendency;
use super::support::weight;
use super::{LawError, LawResult};
use std::collections::BTreeSet;

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

fn revise(
    definition: &Tendency,
    staged: &mut [Row],
    i: usize,
    new: &Row,
    ctx: &LawContext<'_>,
) -> LawResult<()> {
    let old = staged[i].clone();
    let old_signal = super::num::nonnegative_product(old.w, old.s.abs(), "UnderflowEvidence")?;
    let new_signal = super::num::nonnegative_product(new.w, new.s.abs(), "UnderflowEvidence")?;
    let prev = value(definition, staged, ctx)?;
    if (old.s > 0.0 && new.s > 0.0) || (old.s < 0.0 && new.s < 0.0) {
        // An ordinary same-side share revision changes one component directly.
        // Removing ALL its evidence first would invent an intermediate state
        // that can exhaust precision even when the actual revision is valid.
        staged[i] = new.clone();
        let next = value(definition, staged, ctx)?;
        direction(prev, next, new.s.signum() * (new_signal - old_signal))?;
    } else {
        staged[i].w = 0.0;
        let removed = value(definition, staged, ctx)?;
        direction(prev, removed, -old.s.signum() * old_signal)?;
        staged[i] = new.clone();
        let next = value(definition, staged, ctx)?;
        direction(removed, next, new.s.signum() * new_signal)?;
    }
    Ok(())
}

pub(super) fn revisions(
    before: &Work,
    after: &Work,
    affected: &BTreeSet<String>,
    ctx: &LawContext<'_>,
) -> LawResult<()> {
    for id in after.ordered_ids() {
        if !affected.contains(&id) {
            continue;
        }
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
            revise(definition, &mut staged, i, new, ctx)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::law::tests::{fixture, world::Fixture};

    #[test]
    fn partial_withdrawal_does_not_require_a_fictitious_full_withdrawal() {
        let mut f = Fixture::new(8);
        f.law = fixture::law(3.0, 1.0, 0.0, 10.0);
        let definition = fixture::tendency(&f.law, "k", None);
        for sign in [-1.0, 1.0] {
            let mut rows = vec![
                Row {
                    group: "old".into(),
                    ordinal: 0,
                    q: 1.0,
                    w: 1e16,
                    s: sign,
                    mu: 0.0,
                    p: None,
                },
                Row {
                    group: "later".into(),
                    ordinal: 1,
                    q: 1.0,
                    w: 4e16,
                    s: -sign,
                    mu: 0.0,
                    p: None,
                },
            ];
            let before = value(&definition, &rows, &f.ctx()).unwrap();
            let mut removed = rows.clone();
            removed[0].w = 0.0;
            assert!(matches!(
                value(&definition, &removed, &f.ctx()),
                Err(LawError::PrecisionExhausted(_))
            ));
            let mut revised = rows[0].clone();
            revised.w *= 0.5;
            revise(&definition, &mut rows, 0, &revised, &f.ctx()).unwrap();
            let after = value(&definition, &rows, &f.ctx()).unwrap();
            assert!(if sign > 0.0 {
                after < before
            } else {
                after > before
            });
            assert!(after > -1.0 && after < 1.0);
        }
    }
}
