//! G at its boundaries: candidates, ADR 0014 aliases, relation candidates,
//! the capacity fuse with re-sharing, input validation. The I10 signature
//! check and the type checks are in `types.rs`.

use std::collections::BTreeMap;

use core_types::generated::{EvidenceGroupV2, PersonRelation, RSnapshotV2, RelationKind};

use crate::law::{Candidate, LawError, record};

use super::cases::{event, experience, find, on};
use super::engine::{candidate, check_world, run_world, step};
use super::fixture::tendency;
use super::world::{Fixture, World, default_theta};

#[test]
fn candidate_enters_r_at_pi_zero_with_an_l_entry() {
    let f = Fixture::new(64);
    let r = f.snapshot(vec![]);
    let out = f
        .run(
            &r,
            &[],
            &default_theta(),
            &step(&[candidate("rec-1", "c-tidy")], &[], &BTreeMap::new()),
        )
        .unwrap();
    assert_eq!(out.r.revision, "r1");
    let t = &out.r.tendencies[0];
    assert_eq!(
        (t.tendency_id.as_str(), t.pi, t.group_count.0),
        ("rec-1", 0.0, 0)
    );
    assert_eq!(
        (out.changes[0].pi_before, out.changes[0].pi_after),
        (None, Some(0.0))
    );
}

#[test]
fn alias_adr_0014_any_number_of_synonyms_leaves_pi_and_r_unchanged() {
    for seed in 0..60 {
        let (f, w, mut events, _) = run_world(2_800 + seed, 30, 64);
        // Any candidate whose concept is a synonym of a concept in R: no new tendency.
        let present: Vec<String> =
            w.r.tendencies
                .iter()
                .map(|t| t.concept.concept_id.clone())
                .collect();
        let Some(c) = present.iter().find(|c| c.starts_with("c-")) else {
            continue;
        };
        let n = usize::try_from(events.rng.randint(1, 20)).unwrap();
        let aliases: BTreeMap<String, String> = (0..n)
            .map(|i| (format!("{c}-alias-{i}"), c.clone()))
            .collect();
        let cands: Vec<Candidate> = (0..n)
            .map(|i| candidate(&format!("syn-{i}"), &format!("{c}-alias-{i}")))
            .collect();
        let out = f
            .run(
                &w.r,
                &w.group_list(),
                &default_theta(),
                &step(&cands, &[], &aliases),
            )
            .unwrap();
        assert_eq!(out.r, w.r, "no new tendency, the same pi");
        assert!(out.changes.is_empty());
    }
    // Synonyms proposed in the same step: the canonical concept is created first,
    // and a group matching both counts once for it.
    let f = Fixture::new(64);
    let aliases = BTreeMap::from([("c-b".to_owned(), "c-a".to_owned())]);
    let g1 = experience(
        "g1",
        1,
        &["e1"],
        &[[0.0, 0.6, 0.0, 0.0]],
        vec![on("rec-2", 1.0), on("rec-1", 0.5)],
    );
    let out = f
        .run(
            &f.snapshot(vec![]),
            &[],
            &default_theta(),
            &step(
                &[candidate("rec-2", "c-b"), candidate("rec-1", "c-a")],
                std::slice::from_ref(&g1),
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
    assert_eq!(out.groups[0].allocations.len(), 1);
    assert_eq!(
        out.groups[0].allocations[0].mu, 0.5,
        "the direct match wins over the alias"
    );
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
            &default_theta(),
            &step(std::slice::from_ref(&c), &[], &none),
        )
        .unwrap();
    let t = &out.r.tendencies[0];
    assert_eq!(t.tendency_id.as_str(), "rel:trust:p7");
    assert_eq!(
        t.concept, f.concepts.trust,
        "structure is fixed by the law, not the proposal"
    );
    let again = f
        .run(
            &out.r,
            &[],
            &default_theta(),
            &step(std::slice::from_ref(&c), &[], &none),
        )
        .unwrap();
    assert_eq!(again.r, out.r);
}

#[test]
fn ac2_21_capacity_removes_the_weakest_candidate_and_reshares_its_groups() {
    let f = Fixture::new(4);
    let mut w = World::new(&f);
    // rec-1 and rec-2 share g1; rec-1 also gets strong evidence of its own.
    let g1 = experience(
        "g1",
        1,
        &["e1"],
        &[[0.0, 0.6, 0.0, 0.0]],
        vec![on("rec-1", 1.0), on("rec-2", 1.0)],
    );
    let g2 = experience(
        "g2",
        2,
        &["e2"],
        &[[0.0, 0.6, 0.0, 0.0]],
        vec![on("rec-1", 1.0)],
    );
    w.apply(
        &f,
        &event(
            vec![candidate("rec-1", "c-a"), candidate("rec-2", "c-b")],
            vec![g1, g2],
        ),
    )
    .unwrap();
    assert_eq!(w.r.tendencies.len(), 4);
    let shared = w.groups["g1"]
        .allocations
        .iter()
        .find(|a| a.tendency_id == "rec-1")
        .unwrap()
        .share;
    assert!(shared < 1.0);
    let before = find(&w, "rec-1").clone();
    // A fifth tendency arrives; the weakest candidate (fresh rec-3, no evidence) leaves at once.
    let out = w
        .apply(&f, &event(vec![candidate("rec-3", "c-c")], vec![]))
        .unwrap();
    assert!(out.changes.is_empty(), "entered and left in the same step");
    // Now rec-2 is the weakest: give rec-3 evidence so that rec-2 must leave.
    let g3 = experience(
        "g3",
        3,
        &["e3"],
        &[[0.0, 0.6, 0.0, 0.0]],
        vec![on("rec-4", 1.0)],
    );
    let g4 = experience(
        "g4",
        4,
        &["e4"],
        &[[0.0, 0.6, 0.0, 0.0]],
        vec![on("rec-4", 1.0)],
    );
    let out = w
        .apply(&f, &event(vec![candidate("rec-4", "c-d")], vec![g3, g4]))
        .unwrap();
    assert!(
        w.r.tendencies
            .iter()
            .all(|t| t.tendency_id.as_str() != "rec-2")
    );
    let g = &w.groups["g1"];
    assert_eq!(g.allocations.len(), 1);
    assert_eq!(
        g.allocations[0].share, 1.0,
        "rec-1 takes back the whole side"
    );
    assert!(find(&w, "rec-1").pi > before.pi);
    let removed = out
        .changes
        .iter()
        .find(|c| c.tendency_id == "rec-2")
        .unwrap();
    assert_eq!(removed.pi_after, None);
    assert_eq!(removed.evidence_groups, ["g1"]);
    assert_eq!(
        out.changes
            .iter()
            .find(|c| c.tendency_id == "rec-1")
            .unwrap()
            .evidence_groups,
        ["g1"]
    );
    check_world(&f, &w);
    // Only personality left: an explicit fault, nothing is removed.
    let f0 = Fixture::new(0);
    let mut persona = tendency(&f0.law, "persona", None);
    record(&mut persona, &f0.law, 6.0, 1.0).unwrap();
    let err = f0.run(
        &f0.snapshot(vec![persona]),
        &[],
        &default_theta(),
        &step(&[], &[], &BTreeMap::new()),
    );
    assert!(
        matches!(err, Err(LawError::InconsistentState(_))),
        "a tendency with groups needs its allocations"
    );
    let mut fresh_persona = tendency(&f0.law, "persona", Some(60.0));
    fresh_persona.relation.as_mut().unwrap().person_id = "someone".to_owned();
    let err = f0
        .run(
            &f0.snapshot(vec![fresh_persona]),
            &[],
            &default_theta(),
            &step(&[], &[], &BTreeMap::new()),
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

#[test]
fn stored_state_is_checked_before_use() {
    let (f, w, _, _) = run_world(2_900, 40, 10);
    let groups = w.group_list();
    let run = |r: &RSnapshotV2, gs: &[EvidenceGroupV2]| {
        f.run(r, gs, &default_theta(), &step(&[], &[], &BTreeMap::new()))
    };
    assert!(run(&w.r, &groups).is_ok());
    let k =
        w.r.tendencies
            .iter()
            .position(|t| t.group_count.0 > 0)
            .unwrap();
    // ĥ differs from the law.
    let mut r = w.r.clone();
    r.tendencies[k].h_hat = 0.5;
    assert!(matches!(
        run(&r, &groups),
        Err(LawError::InconsistentState(_))
    ));
    // r_k differs from M/U.
    let mut r = w.r.clone();
    r.tendencies[k].magnitude_sum += 0.125;
    assert!(run(&r, &groups).is_err());
    // An ordinal gap.
    let id = w.r.tendencies[k].tendency_id.as_str().to_owned();
    let mut gs = groups.clone();
    let a = gs
        .iter_mut()
        .flat_map(|g| g.allocations.iter_mut())
        .find(|a| a.tendency_id == id)
        .unwrap();
    a.ordinal.0 += 1000;
    assert!(matches!(
        run(&w.r, &gs),
        Err(LawError::InconsistentState(_))
    ));
    // A tendency repeats.
    let mut r = w.r.clone();
    r.tendencies.push(r.tendencies[0].clone());
    assert!(run(&r, &groups).is_err());
    // Another law version.
    let mut r = w.r.clone();
    r.law_version.resource_version = "other-law".to_owned();
    assert!(matches!(run(&r, &groups), Err(LawError::InvalidInput(_))));
    // Out of canonical order.
    let mut r = w.r.clone();
    r.tendencies.reverse();
    assert!(run(&r, &groups).is_err());
}
