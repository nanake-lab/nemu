use super::{
    candidates::Catalog,
    records::{Records, canonical},
    *,
};
use crate::law::{Match, MatchTarget};
use core_types::generated::RelationKind;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn usable(
    i: usize,
    prefix: Prefix<'_>,
    records: &Records<'_>,
    tainted: &BTreeSet<usize>,
) -> bool {
    let f = &prefix.frames[i];
    !tainted.contains(&i)
        && records.appraisals.contains_key(&i)
        && !f.attributes.as_ref().expect("validated").self_narrative
        && f.semantics.as_ref().expect("validated").kind == EventUse::Experience
}

pub(super) fn matches(
    i: usize,
    prefix: Prefix<'_>,
    records: &Records<'_>,
    catalog: &Catalog,
    tainted: &BTreeSet<usize>,
) -> Result<Vec<Match>> {
    let sem = prefix.frames[i].semantics.as_ref().expect("validated");
    let mut selected: BTreeMap<String, (f64, bool)> = BTreeMap::new();
    for (target, mu) in records.matches.get(&i).into_iter().flatten() {
        let id = catalog.target(target, prefix)?;
        let basis = sem.match_basis.get(target).ok_or_else(|| {
            Fault::MissingSemantics(format!("{}:{target}", prefix.frames[i].event.id))
        })?;
        let eligible = match basis {
            MatchBasis::Reaction => true,
            MatchBasis::ReadContent => false,
            MatchBasis::Outcome => sem.outcome.known(),
            MatchBasis::OwnActionOutcome {
                context,
                variant,
                action_event,
            } => {
                sem.outcome.known()
                    && action_event
                        .as_ref()
                        .is_none_or(|e| !tainted.contains(&records.index[e]))
                    && comparable(i, &id, context, variant, prefix, records, catalog, tainted)?
            }
        };
        if !eligible {
            continue;
        }
        let direct = id == *target;
        match selected.get(&id) {
            Some((_, true)) if !direct => {}
            Some((old, p)) if *p == direct && old.to_bits() != mu.to_bits() => {
                return Err(Fault::Ambiguous(format!(
                    "conflicting alias matches for {id}"
                )));
            }
            _ => {
                selected.insert(id, (*mu, direct));
            }
        }
    }
    let mut out: Vec<_> = selected
        .into_iter()
        .map(|(id, (mu, _))| Match {
            target: MatchTarget::Tendency(id),
            mu,
        })
        .collect();
    for (people, kind) in [
        (&sem.participants, RelationKind::CLOSENESS),
        (&sem.relied_on, RelationKind::TRUST),
    ] {
        let mut canonical_people = BTreeSet::new();
        for person in people {
            canonical_people.insert(canonical(prefix.person_aliases, person)?);
        }
        out.extend(canonical_people.into_iter().map(|person_id| Match {
            target: MatchTarget::Relation { person_id, kind },
            mu: 1.0,
        }));
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn comparable(
    i: usize,
    target: &str,
    context: &str,
    variant: &Option<String>,
    prefix: Prefix<'_>,
    records: &Records<'_>,
    catalog: &Catalog,
    tainted: &BTreeSet<usize>,
) -> Result<bool> {
    for (j, frame) in prefix.frames.iter().enumerate() {
        if i == j || !usable(j, prefix, records, tainted) {
            continue;
        }
        let sem = frame.semantics.as_ref().expect("validated");
        if !sem.outcome.known() {
            continue;
        }
        for other in records
            .matches
            .get(&j)
            .into_iter()
            .flatten()
            .map(|(k, _)| k)
        {
            if catalog.target(other, prefix)? != target {
                continue;
            }
            if let Some(MatchBasis::OwnActionOutcome {
                context: c,
                variant: v,
                action_event,
            }) = sem.match_basis.get(other)
                && c == context
                && v != variant
                && action_event
                    .as_ref()
                    .is_none_or(|e| !tainted.contains(&records.index[e]))
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
