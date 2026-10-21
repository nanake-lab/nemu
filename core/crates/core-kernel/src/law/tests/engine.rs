//! G's stages that do not need the R fields of V2-48: AC2-05 (R part),
//! ADR 0014 §4 aliases, candidate and relation creation, the capacity fuse,
//! input validation, and the I10 signature check.

use std::collections::BTreeMap;

use core_types::generated::{
    EvidenceAllocation, EvidenceGroup, ObjectRef, PersonRelation, RSnapshot, RelationKind, Tendency,
};
use core_types::scalar::{DecimalU64, HexBytes};
use core_types::semantic::tendency_order;

use crate::law::{
    Candidate, Caretaker, GroupContent, GroupItem, Law, LawContext, LawError, LawResult, Match,
    MatchTarget, Output, RState, RelationConcepts, Step, ThetaView, g, initial_r, record,
};

use super::fixture::{concept, law, tendency, text, vector};
use super::pyrandom::PyRandom;

fn law_ref() -> ObjectRef {
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(vec![7; 32]),
        object_type: "echo.law-version".to_owned(),
        resource_version: "test-law-1".to_owned(),
        byte_length: DecimalU64(64),
    }
}

fn concepts() -> RelationConcepts {
    RelationConcepts {
        closeness: concept("nemu.relation.closeness"),
        trust: concept("nemu.relation.trust"),
    }
}

struct Fixture {
    law_ref: ObjectRef,
    concepts: RelationConcepts,
    law: Law,
    capacity: u32,
}

impl Fixture {
    fn new(capacity: u32) -> Self {
        Self {
            law_ref: law_ref(),
            concepts: concepts(),
            law: law(3.0, 1.0, 0.1, 10.0),
            capacity,
        }
    }

    fn ctx(&self) -> LawContext<'_> {
        LawContext {
            law_version: &self.law_ref,
            law: self.law.clone(),
            caretaker: Caretaker {
                person_id: "u-star",
                attachment_prior: 60.0,
            },
            r_capacity: self.capacity,
            relation_concepts: &self.concepts,
        }
    }

    fn snapshot(&self, mut tendencies: Vec<Tendency>) -> RSnapshot {
        tendencies.sort_by(|a, b| tendency_order(&a.tendency_id, &b.tendency_id));
        RSnapshot {
            schema_version: 1,
            revision: "r0".to_owned(),
            law_version: self.law_ref.clone(),
            tendencies,
        }
    }

    fn run(&self, r: &RSnapshot, groups: &[EvidenceGroup], step: &Step<'_>) -> LawResult<Output> {
        g(
            RState {
                snapshot: r,
                groups,
            },
            &theta(),
            step,
            &self.ctx(),
        )
    }
}

fn theta() -> ThetaView {
    ThetaView::new(
        &vector([0.1, -0.2, 0.0, 0.3]),
        &core_types::generated::NatureScale {
            schema_version: 1,
            connection: 0.2,
            empathy: 0.3,
            curiosity: 0.25,
            achievement: 0.15,
        },
    )
    .unwrap()
}

fn step<'a>(
    candidates: &'a [Candidate],
    groups: &'a [GroupItem],
    aliases: &'a BTreeMap<String, String>,
) -> Step<'a> {
    Step {
        revision: "r1",
        candidates,
        groups,
        aliases,
    }
}

fn candidate(record_id: &str, concept_id: &str) -> Candidate {
    Candidate {
        record_id: record_id.to_owned(),
        concept: concept(concept_id),
        predicate: vec![text("when this happens")],
        direction: vector([0.0, 1.0, 0.0, 0.0]),
        relation: None,
    }
}

/// u*'s R₀ plus `n` ordinary tendencies with random histories.
fn random_r(f: &Fixture, rng: &mut PyRandom, n: usize) -> RSnapshot {
    let mut ts = initial_r(
        &f.law,
        &f.concepts,
        Caretaker {
            person_id: "u-star",
            attachment_prior: 60.0,
        },
    )
    .unwrap();
    for i in 0..n {
        let mut t = tendency(&f.law, &format!("k{i}"), None);
        for _ in 0..rng.randint(0, 30) {
            let (w, s) = (rng.uniform(0.0, 3.0), rng.uniform(-1.0, 1.0));
            record(&mut t, &f.law, w, s).unwrap();
        }
        ts.push(t);
    }
    f.snapshot(ts)
}

#[test]
fn ac2_05_without_new_evidence_r_is_bitwise_unchanged() {
    let f = Fixture::new(64);
    let mut rng = PyRandom::new(2305);
    let none = BTreeMap::new();
    for _ in 0..300 {
        let n = usize::try_from(rng.randint(0, 12)).unwrap();
        let r = random_r(&f, &mut rng, n);
        // Any b and σ̂, any revision label: no evidence, no change (LP5).
        let b: [f64; 4] = std::array::from_fn(|_| rng.uniform(-1.0, 1.0));
        let s: [f64; 4] = std::array::from_fn(|_| rng.uniform(0.01, 1.0));
        let view = ThetaView::new(
            &vector(b),
            &core_types::generated::NatureScale {
                schema_version: 1,
                connection: s[0],
                empathy: s[1],
                curiosity: s[2],
                achievement: s[3],
            },
        )
        .unwrap();
        let out = g(
            RState {
                snapshot: &r,
                groups: &[],
            },
            &view,
            &step(&[], &[], &none),
            &f.ctx(),
        )
        .unwrap();
        assert_eq!(out.r, r);
        assert!(out.groups.is_empty() && out.changes.is_empty());
    }
}

#[test]
fn candidate_enters_r_at_pi_zero_with_an_l_entry() {
    let f = Fixture::new(64);
    let r = f.snapshot(vec![]);
    let none = BTreeMap::new();
    let out = f
        .run(&r, &[], &step(&[candidate("rec-1", "c-tidy")], &[], &none))
        .unwrap();
    assert_eq!(out.r.revision, "r1");
    assert_eq!(out.r.tendencies.len(), 1);
    let t = &out.r.tendencies[0];
    assert_eq!(
        (t.tendency_id.as_str(), t.pi, t.group_count.0),
        ("rec-1", 0.0, 0)
    );
    assert_eq!(out.changes.len(), 1);
    assert_eq!(
        (out.changes[0].pi_before, out.changes[0].pi_after),
        (None, Some(0.0))
    );
}

#[test]
fn alias_adr_0014_any_number_of_synonyms_leaves_pi_and_r_unchanged() {
    let f = Fixture::new(64);
    let mut rng = PyRandom::new(2314);
    for _ in 0..200 {
        let mut k = tendency(&f.law, "k-tidy", None);
        k.concept = concept("c-tidy");
        for _ in 0..rng.randint(0, 20) {
            let (w, s) = (rng.uniform(0.0, 3.0), rng.uniform(-1.0, 1.0));
            record(&mut k, &f.law, w, s).unwrap();
        }
        let r = f.snapshot(vec![k]);
        let n = usize::try_from(rng.randint(1, 20)).unwrap();
        let aliases: BTreeMap<String, String> = (0..n)
            .map(|i| (format!("c-tidy-alias-{i}"), "c-tidy".to_owned()))
            .collect();
        let cands: Vec<Candidate> = (0..n)
            .map(|i| candidate(&format!("rec-{i}"), &format!("c-tidy-alias-{i}")))
            .collect();
        let out = f.run(&r, &[], &step(&cands, &[], &aliases)).unwrap();
        assert_eq!(out.r, r, "no new tendency, the same pi");
        assert!(out.changes.is_empty());
    }
    // Two synonyms proposed in the same step: only the first in canonical order is created.
    let aliases = BTreeMap::from([("c-b".to_owned(), "c-a".to_owned())]);
    let out = f
        .run(
            &f.snapshot(vec![]),
            &[],
            &step(
                &[candidate("rec-2", "c-b"), candidate("rec-1", "c-a")],
                &[],
                &aliases,
            ),
        )
        .unwrap();
    let ids: Vec<&str> = out
        .r
        .tendencies
        .iter()
        .map(|t| t.tendency_id.as_str())
        .collect();
    assert_eq!(ids, ["rec-1"]);
}

#[test]
fn relation_candidate_becomes_the_structural_tendency() {
    let f = Fixture::new(64);
    let mut c = candidate("rec-9", "c-friend");
    c.relation = Some(PersonRelation {
        schema_version: 1,
        person_id: "p7".to_owned(),
        kind: RelationKind::TRUST,
    });
    let none = BTreeMap::new();
    let out = f
        .run(
            &f.snapshot(vec![]),
            &[],
            &step(std::slice::from_ref(&c), &[], &none),
        )
        .unwrap();
    let t = &out.r.tendencies[0];
    assert_eq!(t.tendency_id, "rel:trust:p7");
    assert_eq!(
        t.concept, f.concepts.trust,
        "structure is fixed by the law, not the proposal"
    );
    // Proposed again: no second tendency.
    let again = f
        .run(&out.r, &[], &step(std::slice::from_ref(&c), &[], &none))
        .unwrap();
    assert_eq!(again.r, out.r);
}

#[test]
fn evidence_groups_are_an_explicit_contract_gap() {
    let f = Fixture::new(64);
    let item = GroupItem {
        group_id: "g1".to_owned(),
        position: 10,
        source_key: "s1".to_owned(),
        members: vec!["e10".to_owned()],
        verifiable: true,
        content: GroupContent::Experience {
            appraisals: vec![vector([0.5, 0.0, 0.0, 0.0])],
            matches: vec![Match {
                target: MatchTarget::Relation {
                    person_id: "p1".to_owned(),
                    kind: RelationKind::CLOSENESS,
                },
                mu: 1.0,
            }],
        },
    };
    let none = BTreeMap::new();
    let err = f
        .run(
            &f.snapshot(vec![]),
            &[],
            &step(&[], std::slice::from_ref(&item), &none),
        )
        .unwrap_err();
    assert!(matches!(err, LawError::ContractGap(_)), "{err}");
}

#[test]
fn ac2_21_capacity_removes_the_weakest_candidate_and_stages_it_in_l() {
    let f = Fixture::new(3);
    let mut weak = tendency(&f.law, "weak", None);
    record(&mut weak, &f.law, 0.5, 1.0).unwrap();
    let mut strong = tendency(&f.law, "strong", None);
    record(&mut strong, &f.law, 2.0, 1.0).unwrap();
    let mut persona = tendency(&f.law, "persona", None);
    record(&mut persona, &f.law, 6.0, 1.0).unwrap();
    let r = f.snapshot(vec![weak.clone(), strong, persona.clone()]);
    let none = BTreeMap::new();
    // A fresh candidate has no evidence: it is the weakest and leaves at once.
    let out = f
        .run(&r, &[], &step(&[candidate("rec-1", "c-new")], &[], &none))
        .unwrap();
    assert_eq!(out.r, r);
    assert!(
        out.changes.is_empty(),
        "entered and left in the same step: no net change"
    );
    // The weakest stored candidate leaves; its group allocation goes with it.
    let group = EvidenceGroup {
        schema_version: 1,
        group_id: "g-weak".to_owned(),
        members: vec!["e1".to_owned()],
        source_key: "s".to_owned(),
        credibility: 1.0,
        verifiability: 1.0,
        allocations: vec![EvidenceAllocation {
            schema_version: 1,
            tendency_id: "weak".to_owned(),
            support: 1.0,
            intensity: 0.5,
            share: 1.0,
        }],
    };
    let mut fresh = tendency(&f.law, "fresh", None);
    record(&mut fresh, &f.law, 1.0, 1.0).unwrap();
    let r4 = f.snapshot(vec![weak, fresh, persona]);
    let f2 = Fixture::new(2);
    let out = f2
        .run(&r4, std::slice::from_ref(&group), &step(&[], &[], &none))
        .unwrap();
    let ids: Vec<&str> = out
        .r
        .tendencies
        .iter()
        .map(|t| t.tendency_id.as_str())
        .collect();
    assert_eq!(ids, ["fresh", "persona"]);
    assert_eq!(out.changes.len(), 1);
    assert_eq!(
        (out.changes[0].tendency_id.as_str(), out.changes[0].pi_after),
        ("weak", None)
    );
    assert!(out.groups[0].allocations.is_empty());
    // Only personality left: an explicit fault, nothing is removed.
    let f1 = Fixture::new(0);
    let err = f1
        .run(
            &f.snapshot(vec![persona_only(&f.law)]),
            &[],
            &step(&[], &[], &none),
        )
        .unwrap_err();
    assert!(
        matches!(
            err,
            LawError::CapacityExhausted {
                capacity: 0,
                personality: 1
            }
        ),
        "{err}"
    );
}

fn persona_only(l: &Law) -> Tendency {
    let mut t = tendency(l, "persona", None);
    record(&mut t, l, 6.0, 1.0).unwrap();
    t
}

#[test]
fn stored_state_is_checked_before_use() {
    let f = Fixture::new(64);
    let none = BTreeMap::new();
    let mut t = tendency(&f.law, "k", None);
    t.h_hat = 0.5;
    assert!(matches!(
        f.run(&f.snapshot(vec![t]), &[], &step(&[], &[], &none)),
        Err(LawError::InconsistentState(_))
    ));
    let mut r = f.snapshot(vec![]);
    r.law_version.resource_version = "other-law".to_owned();
    assert!(matches!(
        f.run(&r, &[], &step(&[], &[], &none)),
        Err(LawError::InvalidInput(_))
    ));
    let r = RSnapshot {
        tendencies: vec![tendency(&f.law, "b", None), tendency(&f.law, "a", None)],
        ..f.snapshot(vec![])
    };
    let r2 = RSnapshot {
        tendencies: vec![tendency(&f.law, "bb", None), tendency(&f.law, "a", None)],
        ..f.snapshot(vec![])
    };
    assert!(f.run(&r, &[], &step(&[], &[], &none)).is_err());
    assert!(f.run(&r2, &[], &step(&[], &[], &none)).is_err());
}

#[test]
fn i10_g_signature_has_no_x_m_g_or_lambda() {
    // The parameter list is pinned: R, the θ view, the step, the law context.
    let _pinned: fn(RState<'_>, &ThetaView, &Step<'_>, &LawContext<'_>) -> LawResult<Output> = g;
    // No law source names the fast state, the modulation, the gains or the
    // recovery rates (comments excluded); the θ view holds b and σ̂ only.
    let sources = [
        ("engine.rs", include_str!("../engine.rs")),
        ("input.rs", include_str!("../input.rs")),
        ("fuse.rs", include_str!("../fuse.rs")),
        ("num.rs", include_str!("../num.rs")),
        ("params.rs", include_str!("../params.rs")),
        ("recurrence.rs", include_str!("../recurrence.rs")),
        ("relation.rs", include_str!("../relation.rs")),
        ("shares.rs", include_str!("../shares.rs")),
        ("support.rs", include_str!("../support.rs")),
        ("error.rs", include_str!("../error.rs")),
        ("mod.rs", include_str!("../mod.rs")),
    ];
    let forbidden = [
        "FastStateSnapshot",
        "LearnedComponent",
        "ModulationEnvelope",
        "ThetaValues",
        "gain",
        "recovery",
        "lambda",
        "fast_state",
        "modulation",
    ];
    for (name, src) in sources {
        for (n, line) in src.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for word in forbidden {
                assert!(
                    !code.contains(word),
                    "{name}:{}: `{word}` in `{line}`",
                    n + 1
                );
            }
        }
    }
    let input = include_str!("../input.rs");
    let start = input.find("pub struct ThetaView {").unwrap();
    let body = &input[start..start + input[start..].find('}').unwrap()];
    let fields: Vec<&str> = body
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(fields, ["baseline: Nature4,", "sigma: Nature4,"]);
}
