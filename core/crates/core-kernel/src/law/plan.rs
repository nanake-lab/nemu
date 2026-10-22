//! D3, first half: what one evidence group contributes before shares are
//! applied, for an experience group (S, strength, μ, p_c per tendency) and a
//! teaching group (EV7, V2-45). See `group.rs` for what is recomputed when.

use std::collections::BTreeMap;

use core_types::semantic::nature;

use super::engine::Work;
use super::error::{LawError, LawResult};
use super::input::{GroupContent, GroupItem, LawContext, MatchTarget, ThetaView};
use super::num::Nature4;
use super::relation::{relation_tendency_id, trust};
use super::support::{deviation, experience, teaching};
use core_types::generated::{EvidenceAllocationV2, EvidenceGroupV2};

/// An allocation's values before its share is applied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Values {
    pub(super) support: f64,
    pub(super) intensity: f64,
    pub(super) mu: f64,
    pub(super) projection: Option<f64>,
}

pub(super) fn of(a: &EvidenceAllocationV2) -> Values {
    Values {
        support: a.support,
        intensity: a.intensity,
        mu: a.mu,
        projection: a.projection,
    }
}

pub(super) struct Plan {
    pub(super) credibility: f64,
    pub(super) verifiability: f64,
    /// The members changed: every side is re-shared (V2-39).
    pub(super) reshare_all: bool,
    pub(super) values: BTreeMap<String, Values>,
}

pub(super) fn find<'a>(
    g: Option<&'a EvidenceGroupV2>,
    id: &str,
) -> Option<&'a EvidenceAllocationV2> {
    g.and_then(|g| g.allocations.iter().find(|a| a.tendency_id == id))
}

pub(super) fn experience_plan(
    work: &Work,
    item: &GroupItem,
    stored: Option<&EvidenceGroupV2>,
    members: &[String],
    theta: &ThetaView,
    remap: &BTreeMap<String, String>,
    ctx: &LawContext<'_>,
) -> LawResult<Plan> {
    let law = &ctx.law;
    let GroupContent::Experience {
        appraisals,
        matches,
    } = &item.content
    else {
        unreachable!("experience plan for a teaching group")
    };
    if appraisals.len() != item.members.len() {
        return Err(LawError::InvalidInput(format!(
            "group {}: one appraisal per member",
            item.group_id
        )));
    }
    let mut deviations: Vec<Nature4> = Vec::with_capacity(appraisals.len());
    for a in appraisals {
        let a = nature(a);
        if a.iter()
            .any(|v| !v.is_finite() || !(-1.0..=1.0).contains(v))
        {
            return Err(LawError::InvalidInput("a(e) outside [-1, 1]".into()));
        }
        deviations.push(deviation(&a, theta.baseline())?);
    }
    let verifiability = if item.verifiable { 1.0 } else { law.v_u };
    let changed = stored.is_none_or(|g| g.members != members || g.verifiability != verifiability);
    // μ per tendency: a direct match wins over an alias mapped onto the same tendency (D10).
    // Priority of a μ source for one tendency: a direct match, then the stored
    // allocation, then a synonym mapped onto it (D10: synonyms add nothing).
    const ALIAS: u8 = 0;
    const STORED: u8 = 1;
    const DIRECT: u8 = 2;
    let mut mus: BTreeMap<String, (f64, u8)> = BTreeMap::new();
    for a in stored.into_iter().flat_map(|g| &g.allocations) {
        mus.insert(a.tendency_id.clone(), (a.mu, STORED));
    }
    for m in matches {
        if !(m.mu.is_finite() && m.mu > 0.0 && m.mu <= 1.0) {
            return Err(LawError::InvalidInput(format!(
                "mu {} outside (0, 1]",
                m.mu
            )));
        }
        let (id, direct) = match &m.target {
            MatchTarget::Tendency(t) => {
                let r = remap.get(t).cloned().unwrap_or_else(|| t.clone());
                let direct = r == *t;
                (r, direct)
            }
            MatchTarget::Relation { person_id, kind } => {
                (relation_tendency_id(*kind, person_id), true)
            }
        };
        if !work.now.contains_key(&id) {
            continue; // not in R, e.g. removed by the capacity fuse
        }
        let priority = if direct { DIRECT } else { ALIAS };
        if mus.get(&id).is_none_or(|(_, p)| priority > *p) {
            mus.insert(id, (m.mu, priority));
        }
    }
    let mut values = BTreeMap::new();
    for (id, (mu, _)) in mus {
        let v = match find(stored, &id) {
            Some(a) if !changed => {
                if a.mu.to_bits() != mu.to_bits() {
                    return Err(LawError::InconsistentState(format!(
                        "group {}: mu of {id} changed without new members (reinterpretation is P5-26)",
                        item.group_id
                    )));
                }
                of(a)
            }
            _ => {
                let t = &work.now[&id];
                let r_tilde = t.typical_magnitude.max(law.r_min);
                let g = experience(
                    &deviations,
                    &nature(&t.direction),
                    theta.sigma(),
                    r_tilde,
                    mu,
                    law.w_max,
                )?;
                Values {
                    support: g.support,
                    intensity: g.intensity,
                    mu,
                    projection: g.projection,
                }
            }
        };
        values.insert(id, v);
    }
    Ok(Plan {
        credibility: 1.0,
        verifiability,
        reshare_all: changed,
        values,
    })
}

pub(super) fn teaching_plan(
    work: &Work,
    item: &GroupItem,
    stored: Option<&EvidenceGroupV2>,
    remap: &BTreeMap<String, String>,
    ctx: &LawContext<'_>,
) -> LawResult<Plan> {
    let GroupContent::Teaching {
        speaker,
        tendency_id,
        supports,
    } = &item.content
    else {
        unreachable!("teaching plan for an experience group")
    };
    let id = remap
        .get(tendency_id)
        .cloned()
        .unwrap_or_else(|| tendency_id.clone());
    let fresh = teaching(*supports, ctx.law.w_teach);
    // EV7: the group's credibility is fixed when it is first formed; repeats never refresh.
    let (credibility, verifiability) = match stored {
        Some(g) => (g.credibility, g.verifiability),
        None => (
            trust(&work.bases(), speaker),
            if item.verifiable { 1.0 } else { ctx.law.v_u },
        ),
    };
    let mut values: BTreeMap<String, Values> = stored
        .into_iter()
        .flat_map(|g| &g.allocations)
        .map(|a| (a.tendency_id.clone(), of(a)))
        .collect();
    if values.keys().any(|k| *k != id) {
        return Err(LawError::InconsistentState(format!(
            "teaching group {} changed its tendency",
            item.group_id
        )));
    }
    match values.get(&id) {
        Some(v) if v.support != fresh.support => {
            return Err(LawError::InconsistentState(format!(
                "teaching group {} changed its sign",
                item.group_id
            )));
        }
        Some(_) => {}
        None if work.now.contains_key(&id) => {
            values.insert(
                id,
                Values {
                    support: fresh.support,
                    intensity: fresh.intensity,
                    mu: 1.0,
                    projection: None,
                },
            );
        }
        None => {}
    }
    Ok(Plan {
        credibility,
        verifiability,
        reshare_all: false,
        values,
    })
}
