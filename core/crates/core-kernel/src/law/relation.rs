//! Tendencies about a person: closeness and trust (v2.0 §5.2 可信, §6, §7,
//! §10; corollary 7; D6, V2-42).
//!
//! Each person has at most one of each. Both are structural: their direction
//! and predicate are fixed by the law, not proposed by a model. The
//! caretaker u*'s closeness is attachment, with the innate prior A_att that
//! never decays; u*'s trust starts from initial evidence A_att, which gives way
//! to experience like any other evidence. Everyone else starts neutral (π = 0)
//! the first time they are involved.

use std::f64::consts::FRAC_1_SQRT_2;

use core_types::generated::{
    ConceptRef, NatureVector, PersonRelation, RelationKind, Tendency, TrilingualText,
};

use super::error::LawResult;
use super::num::{clip, finite};
use super::params::Law;
use super::recurrence::{Origin, initial};

/// Version of the structural relation definitions below (directions, predicates, IDs).
pub const RELATION_FORM: &str = "p5-23-relations-1";

/// Closeness predicts that being with this person raises connection above baseline.
pub const CLOSENESS_DIRECTION: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
/// Trust predicts that relying on what this person says or does lets her learn
/// and get things done: curiosity and achievement, equally.
pub const TRUST_DIRECTION: [f64; 4] = [0.0, 0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2];

/// The registry entries of the two structural concepts (P5-06 registers them).
#[derive(Clone, Debug, PartialEq)]
pub struct RelationConcepts {
    pub closeness: ConceptRef,
    pub trust: ConceptRef,
}

/// u* and the attachment prior, from genesis (v2.0 §13).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Caretaker<'a> {
    pub person_id: &'a str,
    pub attachment_prior: f64,
}

pub fn kind_name(kind: RelationKind) -> &'static str {
    match kind {
        RelationKind::CLOSENESS => "closeness",
        RelationKind::TRUST => "trust",
    }
}

/// `rel:closeness:<person_id>` or `rel:trust:<person_id>`.
pub fn relation_tendency_id(kind: RelationKind, person_id: &str) -> String {
    format!("rel:{}:{person_id}", kind_name(kind))
}

pub fn direction(kind: RelationKind) -> NatureVector {
    let d = match kind {
        RelationKind::CLOSENESS => CLOSENESS_DIRECTION,
        RelationKind::TRUST => TRUST_DIRECTION,
    };
    NatureVector {
        schema_version: 1,
        connection: d[0],
        empathy: d[1],
        curiosity: d[2],
        achievement: d[3],
    }
}

/// One fixed condition each, so κ = κ₁. The text only describes the
/// condition; matching is given by the participants (P5-24).
pub fn predicate(kind: RelationKind) -> Vec<TrilingualText> {
    let (zh, en, ja) = match kind {
        RelationKind::CLOSENESS => (
            "这件事涉及此人：此人说话、被说到或被提到",
            "The event involves this person as speaker, addressee or someone mentioned",
            "この出来事にこの人が話し手、聞き手、または話題の人として関わっている",
        ),
        RelationKind::TRUST => (
            "她依靠此人说的话或做的事",
            "She relies on what this person says or does",
            "彼女がこの人の言葉や行動に頼っている",
        ),
    };
    vec![TrilingualText {
        schema_version: 1,
        zh: zh.to_owned(),
        en: en.to_owned(),
        ja: ja.to_owned(),
    }]
}

/// The structural tendency of `kind` about `person_id`, at its starting value.
pub fn relation_tendency(
    law: &Law,
    concepts: &RelationConcepts,
    caretaker: Caretaker<'_>,
    person_id: &str,
    kind: RelationKind,
) -> LawResult<Tendency> {
    let is_caretaker = person_id == caretaker.person_id;
    let a_att = finite(caretaker.attachment_prior, "attachment prior")?;
    let (prior, initial_evidence) = match (is_caretaker, kind) {
        (true, RelationKind::CLOSENESS) => (Some(a_att), 0.0),
        (true, RelationKind::TRUST) => (None, a_att),
        (false, _) => (None, 0.0),
    };
    let id = relation_tendency_id(kind, person_id);
    let concept = match kind {
        RelationKind::CLOSENESS => &concepts.closeness,
        RelationKind::TRUST => &concepts.trust,
    };
    let relation = PersonRelation {
        schema_version: 1,
        person_id: person_id.to_owned(),
        kind,
    };
    initial(
        law,
        &Origin {
            tendency_id: &id,
            concept,
            predicate: &predicate(kind),
            direction: &direction(kind),
            relation: Some(&relation),
            prior,
            initial_evidence,
        },
    )
}

/// R₀ at birth: u*'s closeness (attachment) and trust, in canonical order.
pub fn initial_r(
    law: &Law,
    concepts: &RelationConcepts,
    caretaker: Caretaker<'_>,
) -> LawResult<Vec<Tendency>> {
    let mut r = vec![
        relation_tendency(
            law,
            concepts,
            caretaker,
            caretaker.person_id,
            RelationKind::CLOSENESS,
        )?,
        relation_tendency(
            law,
            concepts,
            caretaker,
            caretaker.person_id,
            RelationKind::TRUST,
        )?,
    ];
    r.sort_by(|a, b| core_types::semantic::tendency_order(&a.tendency_id, &b.tendency_id));
    Ok(r)
}

fn relation_pi(r: &[Tendency], person_id: &str, kind: RelationKind) -> f64 {
    r.iter()
        .find(|t| {
            t.relation
                .as_ref()
                .is_some_and(|rel| rel.person_id == person_id && rel.kind == kind)
        })
        .map_or(0.0, |t| t.pi)
}

/// Closeness weight for F's empathy channel: clip((1 + π)/2, w_min, 1) (v2.0 §6).
pub fn closeness_weight(r: &[Tendency], person_id: &str, law: &Law) -> f64 {
    clip(
        (1.0 + relation_pi(r, person_id, RelationKind::CLOSENESS)) / 2.0,
        law.w_min,
        1.0,
    )
}

/// Trust in a speaker, the credibility of their teaching: (1 + π)/2 (v2.0 §5.2, §10).
pub fn trust(r: &[Tendency], person_id: &str) -> f64 {
    (1.0 + relation_pi(r, person_id, RelationKind::TRUST)) / 2.0
}
