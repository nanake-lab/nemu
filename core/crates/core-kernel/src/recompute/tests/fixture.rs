use crate::{dynamics::*, evidence::*};
use core_types::{WireMessage, generated::*, scalar::DecimalU64};
use std::{collections::BTreeMap, sync::OnceLock};

pub const LIMITS: core_types::contract::Limits = core_types::contract::Limits {
    max_bytes: 1 << 20,
    max_items: 4096,
    max_depth: 32,
};

// Schema-valid synthetic fields, as in P5-24's fixture. Never reads a user store.
pub fn golden<T: WireMessage>() -> T {
    static ROWS: OnceLock<Vec<serde_json::Value>> = OnceLock::new();
    let rows = ROWS.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../../contracts/golden/legal.json")).unwrap()
    });
    let row = rows
        .iter()
        .find(|r| r["request"]["name"] == T::NAME)
        .unwrap();
    serde_json::from_value(row["request"]["value"].clone()).unwrap()
}
pub fn vector(a: [f64; 4]) -> NatureVector {
    NatureVector {
        schema_version: 1,
        connection: a[0],
        empathy: a[1],
        curiosity: a[2],
        achievement: a[3],
    }
}
pub fn scale(a: [f64; 4]) -> NatureScale {
    NatureScale {
        schema_version: 1,
        connection: a[0],
        empathy: a[1],
        curiosity: a[2],
        achievement: a[3],
    }
}
pub fn dynamics(version: &str) -> DynamicsLaw {
    DynamicsLaw {
        version: version.into(),
        baseline_prior: [0.0; 4],
        gain_prior: [1.0; 4],
        sigma_prior: [0.3; 4],
        tau_prior_s: [10.0; 4],
        baseline_weight: 100.0,
        variance_weight: 10.0,
        duration_weight: 10.0,
        change_prior: 0.05,
        change_prior_weight: 10.0,
        change_min: 0.01,
        gain_min: 0.2,
        gain_max: 3.0,
        recovery_min: 0.001,
        recovery_max: 2.0,
        detector_shift_sigma: 1.0,
        detector_threshold: 8.0,
    }
}
pub fn parameters(n0: f64) -> LawParameters {
    LawParameters {
        schema_version: 1,
        n0,
        kappa1: 0.1,
        w_max: 8.0,
        w_teach: 1.0,
        v_u: 0.25,
        r_min: 0.1,
        r_max: 1.0,
        h0: 0.01,
        n_h: 10.0,
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
pub fn frame(i: u64, a: [f64; 4], candidate: Option<&str>) -> Frame {
    let mut event: EventEnvelope = golden();
    event.id = format!("experience-{i}");
    event.raw_log_position = DecimalU64(i);
    event.parents.clear();
    event.correction_of.clear();
    event.kind = EventKind::OBSERVATION;
    let mut binding: ProposalBindingV2 = golden();
    binding.event_id = event.id.clone();
    binding.raw_log_position = event.raw_log_position;
    binding.branch_id = "synthetic-p5-26".into();
    let mut frame = Frame {
        attributes: Some(EventAttributesRecord {
            schema_version: 1,
            event_id: event.id.clone(),
            rule_table_version: "attrs".into(),
            exogenous: true,
            self_narrative: false,
            verifiable: true,
            source_key: format!("person-{i}"),
        }),
        semantics: Some(Semantics {
            version: "sem".into(),
            conversation: None,
            kind: EventUse::Experience,
            utterance: Utterance::Other,
            epistemic: EpistemicType::USER_ASSERTION,
            speaker: None,
            participants: vec![],
            relied_on: vec![],
            outcome: Outcome {
                observed: true,
                ..Outcome::default()
            },
            match_basis: BTreeMap::from([("k".into(), MatchBasis::Reaction)]),
        }),
        appraisals: vec![NatureAppraisalRecord {
            schema_version: 1,
            record_id: format!("a-{i}"),
            event_id: event.id.clone(),
            appraisal: vector(a),
            persons: vec![],
            binding: binding.clone(),
            producer: "fixture-A".into(),
            producer_version: "1".into(),
        }],
        situations: vec![SituationMatchRecord {
            schema_version: 1,
            record_id: format!("m-{i}"),
            event_id: event.id.clone(),
            matches: vec![TendencyMatch {
                schema_version: 1,
                tendency_id: "k".into(),
                mu: 1.0,
            }],
            binding: binding.clone(),
            producer: "fixture-M".into(),
            producer_version: "1".into(),
        }],
        event,
        teachings: vec![],
        same_matters: vec![],
        lies: vec![],
        candidates: vec![],
        simultaneous_proposals: None,
        structural_admissions: Some(vec![]),
    };
    if let Some(id) = candidate {
        let mut record: TendencyCandidateRecord = golden();
        record.record_id = id.into();
        record.event_id = frame.event.id.clone();
        record.binding = binding;
        record.direction = vector([1.0, 0.0, 0.0, 0.0]);
        record.relation = None;
        record.predicate = vec![TrilingualText {
            schema_version: 1,
            zh: "合成".into(),
            en: "synthetic".into(),
            ja: "合成".into(),
        }];
        frame.candidates.push(RegisteredCandidate {
            record,
            concept: ConceptRef {
                schema_version: 1,
                concept_id: format!("concept-{id}"),
                registry_version: "fixture".into(),
            },
        });
    }
    frame
}
pub fn scope() -> super::Scope {
    super::Scope {
        law: "old-v1".into(),
        interpretation: "i1".into(),
        sources: "s1".into(),
    }
}
pub fn commit<E>(position: u64, command: super::Command<E>) -> super::Commit<E> {
    super::Commit {
        id: format!("commit-{position}"),
        position,
        prefix: format!("prefix-{position}"),
        command,
        check: None,
    }
}
