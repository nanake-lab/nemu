//! A synthetic world for G: R as G keeps it, and a seeded generator of
//! committed events (candidates, experience groups, late members, EV5
//! back-collection, teaching and its repeats). No store is opened (AGENTS.md
//! golden rule 14). R is the P5-32 `Snapshot` with its `EvidenceGroup`
//! summaries.

use crate::law::pending::{EvidenceGroup, Snapshot, Tendency};
use std::collections::BTreeMap;

use core_types::generated::{NatureScale, NatureVector, ObjectRef, RelationKind};
use core_types::scalar::{DecimalU64, HexBytes};
use core_types::semantic::tendency_order;

use crate::law::engine::{Output, g};
use crate::law::input::RState;
use crate::law::{
    Candidate, Caretaker, GroupContent, GroupItem, Law, LawContext, LawResult, Match, MatchTarget,
    RelationConcepts, Step, ThetaView, initial_r,
};

use super::fixture::{concept, law, text, vector};
use super::pyrandom::PyRandom;

pub const CARETAKER: &str = "u-star";
pub const A_ATT: f64 = 60.0;

pub fn law_ref() -> ObjectRef {
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(vec![7; 32]),
        object_type: "echo.law-version".to_owned(),
        resource_version: "test-law-1".to_owned(),
        byte_length: DecimalU64(64),
    }
}

pub fn concepts() -> RelationConcepts {
    RelationConcepts {
        closeness: concept("nemu.relation.closeness"),
        trust: concept("nemu.relation.trust"),
    }
}

pub struct Fixture {
    pub law_ref: ObjectRef,
    pub concepts: RelationConcepts,
    pub law: Law,
    pub capacity: u32,
}

impl Fixture {
    pub fn new(capacity: u32) -> Self {
        Self {
            law_ref: law_ref(),
            concepts: concepts(),
            law: law(3.0, 1.0, 0.1, 10.0),
            capacity,
        }
    }

    pub fn ctx(&self) -> LawContext<'_> {
        LawContext {
            law_version: &self.law_ref,
            law: self.law.clone(),
            caretaker: Caretaker {
                person_id: CARETAKER,
                attachment_prior: A_ATT,
            },
            r_capacity: self.capacity,
            relation_concepts: &self.concepts,
        }
    }

    pub fn snapshot(&self, mut tendencies: Vec<Tendency>) -> Snapshot {
        tendencies.sort_by(|a, b| tendency_order(a.tendency_id.as_str(), b.tendency_id.as_str()));
        Snapshot {
            schema_version: 1,
            revision: "r0".to_owned(),
            law_version: self.law_ref.clone(),
            tendencies,
        }
    }

    /// R₀: u*'s attachment and trust.
    pub fn r0(&self) -> Snapshot {
        let ts = initial_r(
            &self.law,
            &self.concepts,
            Caretaker {
                person_id: CARETAKER,
                attachment_prior: A_ATT,
            },
        )
        .unwrap();
        self.snapshot(ts)
    }

    pub fn run(
        &self,
        r: &Snapshot,
        groups: &[EvidenceGroup],
        theta: &ThetaView,
        step: &Step<'_>,
    ) -> LawResult<Output> {
        g(
            RState {
                snapshot: r,
                groups,
            },
            theta,
            step,
            &self.ctx(),
        )
    }
}

pub fn theta(b: [f64; 4], s: [f64; 4]) -> ThetaView {
    ThetaView::new(
        &vector(b),
        &NatureScale {
            schema_version: 1,
            connection: s[0],
            empathy: s[1],
            curiosity: s[2],
            achievement: s[3],
        },
    )
    .unwrap()
}

pub fn default_theta() -> ThetaView {
    theta([0.1, -0.2, 0.0, 0.3], [0.2, 0.3, 0.25, 0.15])
}

/// One committed event's input to G, owned.
#[derive(Clone, Debug)]
pub struct Event {
    pub revision: String,
    pub candidates: Vec<Candidate>,
    pub groups: Vec<GroupItem>,
    pub aliases: BTreeMap<String, String>,
    pub theta: ThetaView,
}

impl Event {
    pub fn step(&self) -> Step<'_> {
        Step {
            revision: &self.revision,
            candidates: &self.candidates,
            groups: &self.groups,
            aliases: &self.aliases,
        }
    }
}

/// R and its evidence groups, advanced one event at a time by G.
#[derive(Clone)]
pub struct World {
    pub r: Snapshot,
    pub groups: BTreeMap<String, EvidenceGroup>,
    /// The last version of every group item sent, to send late members and repeats.
    pub items: BTreeMap<String, GroupItem>,
}

impl World {
    pub fn new(f: &Fixture) -> Self {
        Self {
            r: f.r0(),
            groups: BTreeMap::new(),
            items: BTreeMap::new(),
        }
    }

    pub fn group_list(&self) -> Vec<EvidenceGroup> {
        self.groups.values().cloned().collect()
    }

    pub fn apply(&mut self, f: &Fixture, e: &Event) -> LawResult<Output> {
        let out = f.run(&self.r, &self.group_list(), &e.theta, &e.step())?;
        self.r = out.r.clone();
        for g in &out.groups {
            self.groups.insert(g.group_id.clone(), g.clone());
        }
        for item in &e.groups {
            self.items.insert(item.group_id.clone(), item.clone());
        }
        Ok(out)
    }

    pub fn ids(&self) -> Vec<String> {
        self.r
            .tendencies
            .iter()
            .map(|t| t.tendency_id.as_str().to_owned())
            .collect()
    }
}

pub fn unit(rng: &mut PyRandom) -> [f64; 4] {
    loop {
        let raw: [f64; 4] = std::array::from_fn(|_| rng.uniform(-1.0, 1.0));
        let n = raw.iter().fold(0.0, |s, v| s + v * v).sqrt();
        if n > 0.1 {
            return std::array::from_fn(|i| raw[i] / n);
        }
    }
}

const PEOPLE: [&str; 4] = [CARETAKER, "p1", "p2", "p3"];
const CONCEPTS: [&str; 6] = [
    "c-tidy",
    "c-music",
    "c-walks",
    "c-tidy-syn",
    "c-jokes",
    "c-music-syn",
];

/// A seeded generator of events.
pub struct Generator {
    pub rng: PyRandom,
    counter: u64,
    position: u64,
}

impl Generator {
    pub fn new(seed: u32) -> Self {
        Self {
            rng: PyRandom::new(seed),
            counter: 0,
            position: 0,
        }
    }

    fn next(&mut self, prefix: &str) -> String {
        self.counter += 1;
        format!("{prefix}{}", self.counter)
    }

    fn appraisal(&mut self) -> NatureVector {
        vector(std::array::from_fn(|_| self.rng.uniform(-1.0, 1.0)))
    }

    fn target(&mut self, world: &World, fresh: &[String]) -> MatchTarget {
        let ids = world.ids();
        let pick = self.rng.random();
        if pick < 0.3 || (ids.is_empty() && fresh.is_empty()) {
            let person = PEOPLE[usize::try_from(self.rng.randint(0, 3)).unwrap()];
            let kind = if self.rng.random() < 0.5 {
                RelationKind::CLOSENESS
            } else {
                RelationKind::TRUST
            };
            MatchTarget::Relation {
                person_id: person.to_owned(),
                kind,
            }
        } else if pick < 0.5 && !fresh.is_empty() {
            let i = usize::try_from(self.rng.randint(0, fresh.len() as i64 - 1)).unwrap();
            MatchTarget::Tendency(fresh[i].clone())
        } else if !ids.is_empty() {
            let i = usize::try_from(self.rng.randint(0, ids.len() as i64 - 1)).unwrap();
            MatchTarget::Tendency(ids[i].clone())
        } else {
            MatchTarget::Tendency(fresh[0].clone())
        }
    }

    fn matches(&mut self, world: &World, fresh: &[String]) -> Vec<Match> {
        let mut out: Vec<Match> = Vec::new();
        for _ in 0..self.rng.randint(1, 3) {
            let target = self.target(world, fresh);
            if out.iter().any(|m| m.target == target) {
                continue;
            }
            out.push(Match {
                target,
                mu: self.rng.uniform(0.05, 1.0),
            });
        }
        out
    }

    /// One random event for `world`.
    pub fn event(&mut self, world: &World) -> Event {
        let mut candidates = Vec::new();
        let mut aliases = BTreeMap::new();
        aliases.insert("c-tidy-syn".to_owned(), "c-tidy".to_owned());
        aliases.insert("c-music-syn".to_owned(), "c-music".to_owned());
        if self.rng.random() < 0.35 {
            let concept_id = CONCEPTS[usize::try_from(self.rng.randint(0, 5)).unwrap()];
            let conditions = usize::try_from(self.rng.randint(1, 3)).unwrap();
            let d = unit(&mut self.rng);
            candidates.push(Candidate {
                record_id: self.next("rec-"),
                concept: concept(concept_id),
                predicate: vec![text("condition"); conditions],
                direction: vector(d),
                relation: None,
            });
        }
        let fresh: Vec<String> = candidates.iter().map(|c| c.record_id.clone()).collect();
        let mut groups: Vec<GroupItem> = Vec::new();
        for _ in 0..self.rng.randint(0, 3) {
            let roll = self.rng.random();
            let stored: Vec<&GroupItem> = world
                .items
                .values()
                .filter(|i| !groups.iter().any(|g| g.group_id == i.group_id))
                .collect();
            let pick = |rng: &mut PyRandom, n: usize| {
                usize::try_from(rng.randint(0, n as i64 - 1)).unwrap()
            };
            if roll < 0.15 && !stored.is_empty() {
                // A later member of an existing group (V2-39), or a repeated teaching.
                let mut item = stored[pick(&mut self.rng, stored.len())].clone();
                self.position += 1;
                item.members.push(format!("e{}", self.position));
                if let GroupContent::Experience { appraisals, .. } = &mut item.content {
                    appraisals.push(self.appraisal());
                }
                groups.push(item);
            } else if roll < 0.3 && !stored.is_empty() && !fresh.is_empty() {
                // EV5 back-collection: an old experience group now also matches the new candidate.
                let mut item = stored[pick(&mut self.rng, stored.len())].clone();
                if let GroupContent::Experience { matches, .. } = &mut item.content {
                    let target = MatchTarget::Tendency(fresh[0].clone());
                    if !matches.iter().any(|m| m.target == target) {
                        matches.push(Match {
                            target,
                            mu: self.rng.uniform(0.05, 1.0),
                        });
                    }
                    groups.push(item);
                }
            } else if roll < 0.45 {
                let ids = world.ids();
                if ids.is_empty() {
                    continue;
                }
                let speaker = PEOPLE[usize::try_from(self.rng.randint(0, 3)).unwrap()];
                self.position += 1;
                groups.push(GroupItem {
                    group_id: self.next("t-"),
                    position: self.position,
                    source_key: format!("speaker:{speaker}"),
                    members: vec![format!("e{}", self.position)],
                    verifiable: self.rng.random() < 0.8,
                    content: GroupContent::Teaching {
                        speaker: speaker.to_owned(),
                        tendency_id: ids[pick(&mut self.rng, ids.len())].clone(),
                        supports: self.rng.random() < 0.6,
                    },
                });
            } else {
                let members = usize::try_from(self.rng.randint(1, 3)).unwrap();
                let first = self.position + 1;
                self.position += members as u64;
                let id = self.next("g-");
                groups.push(GroupItem {
                    group_id: id.clone(),
                    position: first,
                    source_key: format!("src:{id}"),
                    members: (0..members)
                        .map(|k| format!("e{}", first + k as u64))
                        .collect(),
                    verifiable: self.rng.random() < 0.8,
                    content: GroupContent::Experience {
                        appraisals: (0..members).map(|_| self.appraisal()).collect(),
                        matches: self.matches(world, &fresh),
                    },
                });
            }
        }
        let b: [f64; 4] = std::array::from_fn(|_| self.rng.uniform(-0.5, 0.5));
        let s: [f64; 4] = std::array::from_fn(|_| self.rng.uniform(0.05, 0.5));
        Event {
            revision: self.next("r"),
            candidates,
            groups,
            aliases,
            theta: theta(b, s),
        }
    }
}
