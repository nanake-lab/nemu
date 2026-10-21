//! Tendencies about people (D6, V2-42), AC2-13's property part, and the
//! capacity fuse's choice (D7, V2-43).

use core_types::generated::{RelationKind, Tendency};
use core_types::semantic::{check_direction, check_tendency};

use crate::law::{
    Caretaker, Law, RelationConcepts, check_stored, check_weight_cap, closeness_weight,
    eviction_choice, initial_r, record, relation_tendency, relation_tendency_id, trust,
};

use super::fixture::{concept, law, tendency};

fn concepts() -> RelationConcepts {
    RelationConcepts {
        closeness: concept("nemu.relation.closeness"),
        trust: concept("nemu.relation.trust"),
    }
}

const U_STAR: Caretaker<'static> = Caretaker {
    person_id: "u-star",
    attachment_prior: 60.0,
};

fn caretaker_r(l: &Law) -> Vec<Tendency> {
    initial_r(l, &concepts(), U_STAR).unwrap()
}

fn closeness(r: &[Tendency]) -> &Tendency {
    r.iter()
        .find(|t| t.tendency_id == "rel:closeness:u-star")
        .unwrap()
}

#[test]
fn relation_structure_ids_directions_and_r0() {
    let l = law(3.0, 1.0, 0.1, 10.0);
    assert_eq!(
        relation_tendency_id(RelationKind::TRUST, "p1"),
        "rel:trust:p1"
    );
    let r = caretaker_r(&l);
    let ids: Vec<&str> = r.iter().map(|t| t.tendency_id.as_str()).collect();
    assert_eq!(
        ids,
        ["rel:trust:u-star", "rel:closeness:u-star"],
        "canonical order"
    );
    for t in &r {
        check_direction(&t.direction).unwrap();
        check_tendency(t).unwrap();
        check_stored(t, &l).unwrap();
        assert_eq!(t.predicate.len(), 1);
        assert_eq!(t.group_count.0, 0);
    }
    let att = closeness(&r);
    let tr = r
        .iter()
        .find(|t| t.tendency_id == "rel:trust:u-star")
        .unwrap();
    // Attachment: prior, no evidence. Trust: the same amount as initial evidence.
    assert_eq!(att.attachment_prior, Some(60.0));
    assert_eq!((att.evidence_a, att.evidence_b), (0.0, 4.0));
    assert_eq!(tr.attachment_prior, None);
    assert_eq!((tr.evidence_a, tr.evidence_b), (60.0, 64.0));
    assert_eq!(att.pi, 60.0 / 64.0);
    assert_eq!(tr.pi, att.pi, "trust starts as high as attachment");
    // Anyone else starts neutral.
    let other = relation_tendency(&l, &concepts(), U_STAR, "p1", RelationKind::CLOSENESS).unwrap();
    assert_eq!((other.pi, other.attachment_prior), (0.0, None));
}

#[test]
fn relation_readouts_closeness_weight_and_trust() {
    let l = law(3.0, 1.0, 0.1, 10.0);
    let r = caretaker_r(&l);
    let high = (1.0 + 60.0 / 64.0) / 2.0;
    assert_eq!(closeness_weight(&r, "u-star", &l), high);
    assert_eq!(trust(&r, "u-star"), high);
    // Unknown people read as π = 0; closeness is floored at w_min.
    assert_eq!(trust(&r, "stranger"), 0.5);
    assert_eq!(closeness_weight(&r, "stranger", &l), 0.5);
    let mut cold =
        relation_tendency(&l, &concepts(), U_STAR, "p2", RelationKind::CLOSENESS).unwrap();
    for _ in 0..50 {
        record(&mut cold, &l, 3.0, -1.0).unwrap();
    }
    assert!(closeness_weight(std::slice::from_ref(&cold), "p2", &l) >= l.w_min);
}

#[test]
fn ac2_13_attachment_resists_ordinary_contrary_experience_but_is_finite() {
    let l = law(3.0, 1.0, 0.1, 10.0);
    // Ordinary contrary experience: short cold spells between ordinary warm
    // ones, a long random mix of warm and cold, and long alternation all
    // leave attachment in R⁺. (How long a cold spell it withstands depends on
    // A_att and n₀, calibrated in P5-30; with A_att = 60 it is about 30.)
    let mut att = closeness(&caretaker_r(&l)).clone();
    for round in 0..200 {
        for _ in 0..(round % 10) {
            record(&mut att, &l, 1.0, -1.0).unwrap();
            assert!(att.pi > 0.5, "round {round}: {}", att.pi);
        }
        for _ in 0..10 {
            record(&mut att, &l, 1.0, 1.0).unwrap();
        }
    }
    let mut rng = super::pyrandom::PyRandom::new(2313);
    let mut mixed = closeness(&caretaker_r(&l)).clone();
    for _ in 0..5000 {
        let s = rng.choice(&[1.0, -1.0]);
        record(&mut mixed, &l, rng.uniform(0.0, 2.0), s).unwrap();
        assert!(mixed.pi > 0.5, "{}", mixed.pi);
    }
    let mut alternating = closeness(&caretaker_r(&l)).clone();
    for n in 0..5000 {
        record(
            &mut alternating,
            &l,
            1.0,
            if n % 2 == 0 { -1.0 } else { 1.0 },
        )
        .unwrap();
        assert!(alternating.pi > 0.5);
    }
    // Long, consistent contrary experience changes it in finitely many groups,
    // and an ordinary tendency changes sooner.
    let flip = |mut t: Tendency| {
        (1..=100_000u32).find(|_| {
            record(&mut t, &l, 1.0, -1.0).unwrap();
            t.pi <= 0.5
        })
    };
    let n_att = flip(closeness(&caretaker_r(&l)).clone()).unwrap();
    let n_plain = flip(tendency(&l, "plain", None)).unwrap();
    assert!(n_plain < n_att, "{n_plain} {n_att}");
}

#[test]
fn ac2_21_capacity_choice_takes_the_weakest_candidate_never_personality() {
    let l = law(3.0, 1.0, 0.1, 10.0);
    let mut weak = tendency(&l, "weak", None);
    record(&mut weak, &l, 0.5, 1.0).unwrap();
    let mut strong = tendency(&l, "strong", None);
    record(&mut strong, &l, 2.0, 1.0).unwrap();
    let mut person = tendency(&l, "persona", None);
    record(&mut person, &l, 6.0, 1.0).unwrap();
    assert!(person.pi > 0.5);
    let r = vec![strong.clone(), person.clone(), weak.clone()];
    assert_eq!(eviction_choice(&r, &l).unwrap(), Some(2));
    // Ties go to the first in canonical order ("bb" before "aaa": shorter key first).
    let r = vec![tendency(&l, "aaa", None), tendency(&l, "bb", None)];
    assert_eq!(eviction_choice(&r, &l).unwrap(), Some(1));
    // Attachment's prior counts as evidence, so it is never the weakest.
    let r = vec![
        closeness(&caretaker_r(&l)).clone(),
        tendency(&l, "fresh", None),
    ];
    assert_eq!(eviction_choice(&r, &l).unwrap(), Some(1));
    // Only personality left: nothing to remove.
    assert_eq!(eviction_choice(&[person], &l).unwrap(), None);
}

#[test]
fn ac2_21_single_group_weight_never_exceeds_w_max() {
    let l = law(3.0, 1.0, 0.1, 10.0);
    assert!(check_weight_cap(l.w_max, &l, "g", "k").is_ok());
    assert!(check_weight_cap(l.w_max * (1.0 + 1e-15), &l, "g", "k").is_err());
    // The strength formula caps any reaction at w_max.
    let g = crate::law::experience(
        &[[2.0, 0.0, 0.0, 0.0]],
        &[1.0, 0.0, 0.0, 0.0],
        &[1e-6; 4],
        0.05,
        1.0,
        l.w_max,
    )
    .unwrap();
    assert_eq!(g.intensity, l.w_max);
}

#[test]
fn teaching_weight_follows_trust_in_the_speaker_and_verifiability() {
    // EV7, V2-45: S = ±1, strength w_teach, credibility = trust (½ for a stranger).
    let l = law(3.0, 1.0, 0.1, 10.0);
    let r = caretaker_r(&l);
    let for_it = crate::law::teaching(true, l.w_teach);
    let against = crate::law::teaching(false, l.w_teach);
    assert_eq!((for_it.support, against.support), (1.0, -1.0));
    assert_eq!((for_it.intensity, for_it.projection), (l.w_teach, None));
    let w_parent = crate::law::weight(for_it.intensity, trust(&r, "u-star"), 1.0, 1.0).unwrap();
    let w_stranger = crate::law::weight(for_it.intensity, trust(&r, "stranger"), 1.0, 1.0).unwrap();
    let w_unverified =
        crate::law::weight(for_it.intensity, trust(&r, "stranger"), l.v_u, 1.0).unwrap();
    assert!(w_parent > w_stranger && w_stranger > w_unverified);
    assert_eq!(w_stranger, l.w_teach * 0.5);
    assert!(w_parent <= l.w_max);
}
