//! Synthetic truth adapter. Stage all results locally; errors publish nothing.
use crate::{resources::*, world::*};
use core_kernel::{dynamics, effects::*, fast, law, recompute};
use core_types::{
    WireMessage,
    semantic::{nature, scale as values},
};
use std::collections::BTreeMap;
pub type Result<T> = std::result::Result<T, String>;
pub fn err(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
#[derive(Clone, Debug)]
pub struct State {
    pub r: law::state::Snapshot,
    pub groups: Vec<law::state::EvidenceGroup>,
    pub theta: dynamics::DynamicsState,
    pub x: fast::FastState,
    pub time: u64,
}
pub struct Adapter;
pub fn scope() -> recompute::Scope {
    recompute::Scope {
        law: VERSION.into(),
        interpretation: "synthetic-oracle-1".into(),
        sources: "independent-opportunities-1".into(),
    }
}
pub fn effects(s: &State) -> Result<EffectiveEffects> {
    let t = s.theta.values();
    compile_authorized_effects(
        &StoredR {
            tendencies: s
                .r
                .tendencies
                .iter()
                .map(|r| StoredTendency {
                    tendency_id: r.tendency_id.clone(),
                    concept: r.concept.concept_id.clone(),
                    pi: r.pi,
                    r: r.typical_magnitude,
                    d: nature(&r.direction),
                    direction_validated: false,
                })
                .collect(),
        },
        &StoredTheta {
            b: nature(&t.baseline),
            g: values(&t.gain),
            lambda: values(&t.recovery_per_s),
        },
        ProcessTime(s.time),
        &Validity,
    )
    .map_err(err)
}
pub fn genesis() -> Result<State> {
    Ok(State {
        r: law::state::Snapshot {
            schema_version: 1,
            revision: "genesis".into(),
            law_version: reference(),
            tendencies: vec![],
        },
        groups: vec![],
        theta: dynamics::DynamicsState::genesis(dynamics()).map_err(err)?,
        x: fast::FastState::new([0.; 4], vec![]).map_err(err)?,
        time: 0,
    })
}
pub fn candidates() -> Vec<law::Candidate> {
    all_ids()
        .into_iter()
        .map(|id| {
            let parts: Vec<_> = id.split(':').collect();
            let mut d = [0.; 4];
            d[parts[2].parse::<usize>().unwrap()] = if !parts[3].ends_with("negative") {
                1.
            } else {
                -1.
            };
            law::Candidate {
                record_id: id.clone(),
                concept: concept(&id),
                predicate: vec![
                    text(parts[0]),
                    text(if parts[1] == "feedback" {
                        if parts[3].starts_with("available") {
                            "has-time"
                        } else {
                            "evening-contact"
                        }
                    } else {
                        parts[1]
                    }),
                ],
                direction: vector(d),
                relation: None,
            }
        })
        .collect()
}
pub fn situation(person: &str, context: &str) -> SituationMatch {
    SituationMatch {
        mu: if person.is_empty() {
            vec![]
        } else {
            ids(person, context)
                .into_iter()
                .map(|id| (id, 1.))
                .collect()
        },
    }
}
pub fn reaction(
    s: &State,
    p: &Probe,
    time: u64,
) -> Result<(fast::FastState, fast::Drive, Evaluation, fast::FastState)> {
    let eff = effects(s)?;
    let before = fast::project_time(
        &s.x,
        &eff,
        time.checked_sub(s.time).ok_or("backwards probe")?,
    )
    .map_err(err)?;
    let ev = evaluate_effects(&eff, &situation(&p.person, &p.context)).map_err(err)?;
    let drive = fast::compute_drive(
        &eff,
        &ev,
        &fast::EventInput {
            appraisal: Some(p.a),
            empathy_weight: if p.person.is_empty() {
                1.
            } else {
                law::closeness_weight(
                    &s.r.tendencies,
                    &p.person,
                    &law::Law::new(&parameters()).map_err(err)?,
                )
            },
            class: fast::EventClass::NewExogenous,
        },
        &drive_law(),
    )
    .map_err(err)?;
    let after = fast::apply_drive(&before, &drive).map_err(err)?;
    Ok((before, drive, ev, after))
}
pub fn advance(s: &State, e: &Event) -> Result<State> {
    advance_named(s, e, false)
}
pub fn advance_named(s: &State, e: &Event, renamed: bool) -> Result<State> {
    if e.time < s.time || e.ordinal == 0 {
        return Err("event order".into());
    }
    let p = Probe {
        id: format!("{}{}", if renamed { "z" } else { "e" }, e.ordinal),
        person: e.person.clone(),
        context: e.context.clone(),
        a: e.a,
    };
    let (_, _, _, x) = reaction(s, &p, e.time)?;
    let mut proposals = if e.ordinal == 1 { candidates() } else { vec![] };
    let mut order = all_ids();
    if renamed {
        for c in &mut proposals {
            c.record_id = rename_id(&c.record_id, false);
            c.concept.concept_id = rename_id(&c.concept.concept_id, false);
            c.predicate[0] = text(if c.predicate[0].en == "U" { "X" } else { "Y" });
        }
        order = order.iter().map(|id| rename_id(id, false)).collect();
    }
    let group = law::GroupItem {
        group_id: format!(
            "{}-{:08}",
            if renamed { "renamed-cause" } else { "cause" },
            e.ordinal
        ),
        position: e.ordinal,
        source_key: e.person.clone(),
        members: vec![p.id.clone()],
        verifiable: true,
        content: law::GroupContent::Experience {
            appraisals: vec![vector(e.a)],
            matches: ids(&e.person, &e.context)
                .into_iter()
                .map(|id| law::Match {
                    target: law::MatchTarget::Tendency(id),
                    mu: 1.,
                })
                .collect(),
        },
    };
    let reference = reference();
    let relations = relations();
    let ctx = law::LawContext {
        law_version: &reference,
        law: law::Law::new(&parameters()).map_err(err)?,
        caretaker: law::Caretaker {
            person_id: if renamed { "X" } else { "U" },
            attachment_prior: 0.,
        },
        r_capacity: 100,
        protected_candidates: 1,
        relation_concepts: &relations,
    };
    let output = law::g(
        law::RState {
            snapshot: &s.r,
            groups: &s.groups,
        },
        &law::ThetaView::new(&s.theta.values().baseline, &scale(s.theta.sigma())).map_err(err)?,
        &law::Step {
            revision: &p.id,
            candidates: &proposals,
            groups: &[group],
            aliases: &BTreeMap::new(),
            proposal_order: &order,
        },
        &ctx,
    )
    .map_err(err)?;
    if !output.capacity_audits.is_empty() || output.capacity_notice.is_some() {
        return Err(format!("capacity boundary: {:?}", output.capacity_notice));
    }
    let stops: Vec<_> = (0..e.day - 1)
        .filter(|_| e.hours < 24)
        .map(|day| dynamics::Downtime {
            start_ns: u64::from(day) * DAY + u64::from(e.hours) * 3_600_000_000_000,
            end_ns: u64::from(day + 1) * DAY,
        })
        .collect();
    let c = dynamics::ReactionEvent {
        id: p.id,
        ordinal: e.ordinal,
        time_ns: e.time,
        running_time: dynamics::RunningTime::from_history(0, e.time, &stops).map_err(err)?,
        source: dynamics::ReactionSource::ExternalWorld,
        evaluation_unmodulated: true,
        reaction: vector(e.a),
    };
    let theta = dynamics::stage(&s.theta, Some(&c), &dynamics(), &fuses())
        .map_err(err)?
        .into_state();
    let mut groups = s.groups.clone();
    for g in output.groups {
        groups.retain(|old| old.group_id != g.group_id);
        groups.push(g);
    }
    groups.sort_by(|a, b| core_types::semantic::tendency_order(&a.group_id, &b.group_id));
    let mut next = State {
        r: output.r,
        groups,
        theta,
        x,
        time: e.time,
    };
    next.x = fast::f_out(&next.x, &effects(&next)?).map_err(err)?;
    Ok(next)
}
pub fn state_json(s: &State) -> Result<serde_json::Value> {
    Ok(
        serde_json::json!({"synthetic_schema":VERSION,"r":s.r,"groups":s.groups,"theta":dynamics::export_snapshot(&s.theta,"snapshot",&reference(),&fuses(),LIMITS).map_err(err)?,"x":serde_json::json!({"nature":s.x.nature,"learned":s.x.learned.iter().map(|v|serde_json::json!([v.tendency_id,v.value])).collect::<Vec<_>>()}),"anchor":s.time}),
    )
}
pub fn same(a: &State, b: &State) -> Result<bool> {
    Ok(
        a.r.canonical_bytes(LIMITS).map_err(err)? == b.r.canonical_bytes(LIMITS).map_err(err)?
            && a.groups.len() == b.groups.len()
            && a.groups.iter().zip(&b.groups).all(|(x, y)| {
                match (x.canonical_bytes(LIMITS), y.canonical_bytes(LIMITS)) {
                    (Ok(a), Ok(b)) => a == b,
                    _ => false,
                }
            })
            && dynamics::export_snapshot(&a.theta, "snapshot", &reference(), &fuses(), LIMITS)
                .map_err(err)?
                .canonical_bytes(LIMITS)
                .map_err(err)?
                == dynamics::export_snapshot(&b.theta, "snapshot", &reference(), &fuses(), LIMITS)
                    .map_err(err)?
                    .canonical_bytes(LIMITS)
                    .map_err(err)?
            && fast::serialize(&a.x, "snapshot", a.time).map_err(err)?
                == fast::serialize(&b.x, "snapshot", b.time).map_err(err)?,
    )
}
impl recompute::Kernel<Event> for Adapter {
    type State = State;
    type Input = Event;
    fn same_event(&self, a: &Event, b: &Event) -> bool {
        a == b && a.a.map(f64::to_bits) == b.a.map(f64::to_bits)
    }
    fn same_input(&self, a: &Event, b: &Event) -> bool {
        self.same_event(a, b)
    }
    fn genesis(&self, s: &recompute::Scope) -> recompute::Result<State> {
        if s != &scope() {
            return Err(recompute::Fault::Binding("unknown resource"));
        }
        genesis().map_err(recompute::Fault::Computation)
    }
    fn project(
        &self,
        h: &[recompute::Commit<Event>],
        s: &recompute::Scope,
    ) -> recompute::Result<Vec<Event>> {
        if s != &scope() {
            return Err(recompute::Fault::Binding("unknown resource"));
        }
        h.iter()
            .map(|c| match &c.command {
                recompute::Command::Event(e) => Ok(e.clone()),
                _ => Err(recompute::Fault::History("unsupported synthetic control")),
            })
            .collect()
    }
    fn advance(&self, s: &State, e: &Event, _: &recompute::Scope) -> recompute::Result<State> {
        advance(s, e).map_err(recompute::Fault::Computation)
    }
    fn same_bits(&self, a: &State, b: &State) -> bool {
        same(a, b).unwrap_or(false)
    }
}

// Fixed bijection preserving ID ordering; no altered numerical inputs.
pub fn rename_id(id: &str, inverse: bool) -> String {
    let pairs = if inverse {
        [("X", "U"), ("Y", "V")]
    } else {
        [("U", "X"), ("V", "Y")]
    };
    for (a, b) in pairs {
        if id == a {
            return b.into();
        }
        if let Some(rest) = id.strip_prefix(&format!("{a}:")) {
            return format!("{b}:{rest}");
        }
    }
    if inverse {
        if let Some(rest) = id.strip_prefix("renamed-cause-") {
            return format!("cause-{rest}");
        }
        if let Some(rest) = id.strip_prefix('z')
            && rest.parse::<u64>().is_ok()
        {
            return format!("e{rest}");
        }
    }
    id.into()
}
pub fn inverse_names(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::String(s) => *s = rename_id(s, true),
        serde_json::Value::Array(a) => {
            for x in a {
                inverse_names(x);
            }
        }
        serde_json::Value::Object(o) => {
            for x in o.values_mut() {
                inverse_names(x);
            }
        }
        _ => {}
    }
}
