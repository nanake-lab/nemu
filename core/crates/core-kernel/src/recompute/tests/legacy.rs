//! CONDITIONAL TEST ADAPTER ONLY. Executes the actual old P5-24/G/C modules.
//! Explicit projection variants model upstream inputs; they do NOT authenticate
//! real H, implement new ADR 0032 rules or constitute a second implementation.
use super::{fixture::*, *};
use crate::{dynamics, evidence, law};
use core_types::{WireMessage, generated::*};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Experience(Box<Experience>),
    Capacity { capacity: u32, protected: u32 },
}
#[derive(Clone, Debug, PartialEq)]
pub struct Experience {
    pub original: evidence::Frame,
    pub interpretations: BTreeMap<String, evidence::Frame>,
    pub excluded_from: BTreeSet<String>,
    pub running: dynamics::RunningTime,
}
pub fn experience(i: u64, a: [f64; 4], candidate: bool) -> Event {
    let original = frame(i, a, candidate.then_some("k"));
    Event::Experience(Box::new(Experience {
        interpretations: BTreeMap::from([("i1".into(), original.clone())]),
        original,
        excluded_from: BTreeSet::new(),
        running: dynamics::RunningTime::from_history(0, i * 1_000_000_000, &[]).unwrap(),
    }))
}
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub r: law::state::Snapshot,
    pub groups: Vec<law::state::EvidenceGroup>,
    pub theta: dynamics::DynamicsState,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    delta: evidence::Assembly,
    revision: String,
    capacity: u32,
    protected: u32,
    reactions: Vec<dynamics::ReactionEvent>,
}
#[derive(Clone)]
pub struct Resource {
    pub reference: ObjectRef,
    pub g: law::Law,
    pub c: dynamics::DynamicsLaw,
}
#[derive(Clone)]
pub struct Legacy {
    pub versions: BTreeMap<String, Resource>,
    pub source_aliases: BTreeMap<String, BTreeMap<String, String>>,
}
impl Legacy {
    pub fn new() -> Self {
        let versions = [("old-v1", 1.0), ("old-v2-test", 2.0)]
            .into_iter()
            .map(|(id, n0)| {
                let mut reference: ObjectRef = golden();
                reference.resource_version = id.into();
                (
                    id.into(),
                    Resource {
                        reference,
                        g: law::Law::new(&parameters(n0)).unwrap(),
                        c: dynamics(id),
                    },
                )
            })
            .collect();
        Self {
            versions,
            source_aliases: BTreeMap::from([
                ("s1".into(), BTreeMap::new()),
                ("s2".into(), BTreeMap::new()),
            ]),
        }
    }
    fn resource(&self, scope: &Scope) -> Result<&Resource> {
        self.versions
            .get(&scope.law)
            .ok_or_else(|| Fault::MissingInput("law resource".into()))
    }
}
fn empty() -> evidence::Assembly {
    evidence::Assembly {
        structural_admissions: Some(vec![]),
        groups: vec![],
        external: vec![],
        candidates: vec![],
        proposal_order: vec![],
        aliases: BTreeMap::new(),
    }
}
impl Kernel<Event> for Legacy {
    type State = State;
    type Input = Input;
    fn same_event(&self, a: &Event, b: &Event) -> bool {
        a == b && format!("{a:?}") == format!("{b:?}")
    }
    fn same_input(&self, a: &Input, b: &Input) -> bool {
        a == b && format!("{a:?}") == format!("{b:?}")
    }
    fn genesis(&self, scope: &Scope) -> Result<State> {
        let resource = self.resource(scope)?;
        Ok(State {
            r: law::state::Snapshot {
                schema_version: 1,
                revision: "genesis".into(),
                law_version: resource.reference.clone(),
                tendencies: vec![],
            },
            groups: vec![],
            theta: dynamics::DynamicsState::genesis(resource.c.clone())
                .map_err(|e| Fault::InvalidInput(format!("{e:?}")))?,
        })
    }
    fn project(&self, history: &[Commit<Event>], scope: &Scope) -> Result<Vec<Input>> {
        self.resource(scope)?;
        let aliases = self
            .source_aliases
            .get(&scope.sources)
            .ok_or_else(|| Fault::MissingInput("source view".into()))?;
        let mut frames = vec![];
        let mut previous = empty();
        let mut times = BTreeMap::new();
        let mut inputs = vec![];
        let (mut capacity, mut protected) = (100, 1);
        for commit in history {
            let Command::Event(event) = &commit.command else {
                continue;
            };
            match event {
                Event::Capacity {
                    capacity: cap,
                    protected: k,
                } => {
                    capacity = *cap;
                    protected = *k;
                    let mut delta = empty();
                    delta.proposal_order = previous.proposal_order.clone();
                    inputs.push(Input {
                        delta,
                        revision: commit.id.clone(),
                        capacity,
                        protected,
                        reactions: vec![],
                    });
                }
                Event::Experience(e) => {
                    if e.excluded_from.contains(&scope.sources) {
                        continue;
                    }
                    let frame = e
                        .interpretations
                        .get(&scope.interpretation)
                        .ok_or_else(|| Fault::MissingInput("accepted interpretation".into()))?;
                    if frame.event != e.original.event
                        || frame.event.raw_log_position.0 != commit.position
                    {
                        return Err(Fault::Binding("effective frame rewrote raw event"));
                    }
                    times.insert(frame.event.id.clone(), e.running.clone());
                    frames.push(frame.clone());
                    let all = evidence::assemble(evidence::Prefix {
                        frames: &frames,
                        attribute_version: "attrs",
                        semantic_version: "sem",
                        concept_aliases: &BTreeMap::new(),
                        person_aliases: aliases,
                        genesis_order: &[],
                        limits: LIMITS,
                    })
                    .map_err(|e| Fault::MissingInput(format!("effective evidence: {e:?}")))?;
                    let delta = all
                        .delta(&previous)
                        .map_err(|e| Fault::MissingInput(format!("delta: {e:?}")))?;
                    let reactions = delta
                        .external
                        .iter()
                        .map(|e| {
                            let running = times[&e.event_id].clone();
                            dynamics::ReactionEvent {
                                id: e.event_id.clone(),
                                ordinal: e.position,
                                time_ns: e.position * 1_000_000_000,
                                running_time: running,
                                source: dynamics::ReactionSource::ExternalWorld,
                                evaluation_unmodulated: true,
                                reaction: e.appraisal.clone(),
                            }
                        })
                        .collect();
                    inputs.push(Input {
                        delta,
                        revision: frame.event.id.clone(),
                        capacity,
                        protected,
                        reactions,
                    });
                    previous = all;
                }
            }
        }
        Ok(inputs)
    }
    fn advance(&self, state: &State, input: &Input, scope: &Scope) -> Result<State> {
        let resource = self.resource(scope)?;
        let relations = law::RelationConcepts {
            closeness: golden(),
            trust: golden(),
        };
        let context = law::LawContext {
            law_version: &resource.reference,
            law: resource.g.clone(),
            caretaker: law::Caretaker {
                person_id: "caretaker",
                attachment_prior: 0.0,
            },
            r_capacity: input.capacity,
            protected_candidates: input.protected,
            relation_concepts: &relations,
        };
        let theta =
            law::ThetaView::new(&state.theta.values().baseline, &scale(state.theta.sigma()))
                .map_err(|e| Fault::Computation(format!("{e:?}")))?;
        let g = law::g(
            law::RState {
                snapshot: &state.r,
                groups: &state.groups,
            },
            &theta,
            &input.delta.step(&input.revision),
            &context,
        )
        .map_err(|e| Fault::Computation(format!("{e:?}")))?;
        let mut next = state.clone();
        next.r = g.r;
        for group in g.groups {
            next.groups.retain(|old| old.group_id != group.group_id);
            next.groups.push(group);
        }
        next.groups
            .sort_by(|a, b| core_types::semantic::tendency_order(&a.group_id, &b.group_id));
        for event in &input.reactions {
            next.theta = dynamics::stage(
                &next.theta,
                Some(event),
                &resource.c,
                &dynamics::DynamicsFuses {
                    jump_notice: 100.0,
                    drift_run: 1_000_000,
                    drift_margin: 0.01,
                },
            )
            .map_err(|e| Fault::Computation(format!("{e:?}")))?
            .into_state();
        }
        Ok(next)
    }
    fn same_bits(&self, a: &State, b: &State) -> bool {
        // Native canonical bytes include every V3 field, q and V. The private C
        // state has all-field PartialEq; Debug additionally distinguishes signed
        // zero in TESTS only, as P5-25 does. This is not a production serializer.
        a.r.canonical_bytes(LIMITS).unwrap() == b.r.canonical_bytes(LIMITS).unwrap()
            && a.groups.len() == b.groups.len()
            && a.groups.iter().zip(&b.groups).all(|(x, y)| {
                x.canonical_bytes(LIMITS).unwrap() == y.canonical_bytes(LIMITS).unwrap()
            })
            && a.theta == b.theta
            && format!("{:?}", a.theta) == format!("{:?}", b.theta)
    }
}
