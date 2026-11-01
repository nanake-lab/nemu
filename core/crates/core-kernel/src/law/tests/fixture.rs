//! Test fixtures: law parameters and tendencies. Synthetic values only; no
//! store is opened (AGENTS.md golden rule 14).

use crate::law::state::Tendency;
use core_types::generated::{
    ConceptRef, LawParameters, NatureVector, PersonRelation, RelationKind, TrilingualText,
};

use crate::law::{Law, Origin, initial};

pub fn parameters(n0: f64, kappa1: f64, h0: f64, n_h: f64) -> LawParameters {
    LawParameters {
        schema_version: 1,
        n0,
        kappa1,
        w_max: 8.0,
        w_teach: 1.0,
        v_u: 0.5,
        r_min: 0.05,
        r_max: 1.0,
        h0,
        n_h,
        h_hat_min: None,
        h0_theta: 0.1,
        h_hat_theta_min: 0.01,
        n_b: 10.0,
        n_sigma: 10.0,
        g_min: 0.2,
        g_max: 3.0,
        recovery_min_per_s: 0.01,
        recovery_max_per_s: 10.0,
        w_min: 0.1,
        t_x_s: 60.0,
        epsilon_x: 0.01,
        k_max: 0.5,
    }
}

/// A law whose κ_k equals `kappa` for a one-condition predicate.
pub fn law(n0: f64, kappa: f64, h0: f64, n_h: f64) -> Law {
    Law::new(&parameters(n0, kappa, h0, n_h)).expect("valid test law")
}

pub fn text(s: &str) -> TrilingualText {
    TrilingualText {
        schema_version: 1,
        zh: s.to_owned(),
        en: s.to_owned(),
        ja: s.to_owned(),
    }
}

pub fn concept(id: &str) -> ConceptRef {
    ConceptRef {
        schema_version: 1,
        concept_id: id.to_owned(),
        registry_version: "test-registry-1".to_owned(),
    }
}

pub fn vector(v: [f64; 4]) -> NatureVector {
    NatureVector {
        schema_version: 1,
        connection: v[0],
        empathy: v[1],
        curiosity: v[2],
        achievement: v[3],
    }
}

/// A fresh one-condition tendency along the connection channel.
pub fn tendency(law: &Law, id: &str, prior: Option<f64>) -> Tendency {
    let predicate = [text("situation")];
    let direction = vector([1.0, 0.0, 0.0, 0.0]);
    let concept = concept(id);
    let relation = prior.map(|_| PersonRelation {
        schema_version: 1,
        person_id: "caretaker".to_owned(),
        kind: RelationKind::CLOSENESS,
    });
    initial(
        law,
        &Origin {
            tendency_id: id,
            concept: &concept,
            predicate: &predicate,
            direction: &direction,
            relation: relation.as_ref(),
            prior,
            initial_evidence: 0.0,
        },
    )
    .expect("valid test tendency")
}
