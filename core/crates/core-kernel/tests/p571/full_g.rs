//! Frozen §2.4 public G boundary. All recurrence remains in law::g.
use super::{limits, nature, number, reference};
use core_kernel::law;
use core_types::{WireMessage, generated::*};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) struct Machine {
    resource: Value,
    law: law::Law,
    binding: ObjectRef,
    concepts: law::RelationConcepts,
    theta: law::ThetaView,
    snapshot: law::state::Snapshot,
    groups: Vec<law::state::EvidenceGroup>,
}
fn array(v: &Value) -> [f64; 4] {
    std::array::from_fn(|i| v[i].as_f64().unwrap())
}
impl Machine {
    pub fn new(q: &Value) -> Self {
        let r = &q["resource"];
        let mut fields = serde_json::Map::new();
        for name in [
            "schema_version",
            "n0",
            "kappa1",
            "w_max",
            "w_teach",
            "v_u",
            "r_min",
            "r_max",
            "h0",
            "n_h",
            "h_hat_min",
            "h0_theta",
            "h_hat_theta_min",
            "n_b",
            "n_sigma",
            "g_min",
            "g_max",
            "recovery_min_per_s",
            "recovery_max_per_s",
            "w_min",
            "t_x_s",
            "epsilon_x",
            "k_max",
        ] {
            if !r[name].is_null() {
                fields.insert(name.into(), r[name].clone());
            }
        }
        let parameters: LawParameters = serde_json::from_value(Value::Object(fields)).unwrap();
        let law = law::Law::new(&parameters).unwrap();
        let binding = reference(&q["resource_ref"], "echo.law-version");
        let concept = |id: &str| ConceptRef {
            schema_version: 1,
            concept_id: id.into(),
            registry_version: r["registry_version"].as_str().unwrap().into(),
        };
        let concepts = law::RelationConcepts {
            closeness: concept(r["relation_concepts"][0].as_str().unwrap()),
            trust: concept(r["relation_concepts"][1].as_str().unwrap()),
        };
        let caretaker = law::Caretaker {
            person_id: r["caretaker"]["person_id"].as_str().unwrap(),
            attachment_prior: number(&r["caretaker"], "attachment_prior"),
        };
        let tendencies = law::initial_r(&law, &concepts, caretaker).unwrap();
        let sigma = array(&r["theta_sigma"]);
        let theta = law::ThetaView::new(
            &nature(array(&r["theta_b"])),
            &NatureScale {
                schema_version: 1,
                connection: sigma[0],
                empathy: sigma[1],
                curiosity: sigma[2],
                achievement: sigma[3],
            },
        )
        .unwrap();
        Self {
            resource: r.clone(),
            law,
            binding: binding.clone(),
            concepts,
            theta,
            snapshot: law::state::Snapshot {
                schema_version: 1,
                revision: "r0".into(),
                law_version: binding,
                tendencies,
            },
            groups: vec![],
        }
    }
    pub fn snapshot(&self) -> (Value, Vec<u8>) {
        let bytes = self.snapshot.canonical_bytes(limits()).unwrap();
        (json!({"r":self.snapshot,"groups":self.groups}), bytes)
    }
    pub fn advance(&mut self, e: &Value, freeze: bool) -> Value {
        if freeze {
            return json!({"changes":[],"counterfactual":{},"after_additions":{},"frozen_negative_control":true});
        }
        let candidates: Vec<_> = e["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| law::Candidate {
                record_id: v["id"].as_str().unwrap().into(),
                concept: ConceptRef {
                    schema_version: 1,
                    concept_id: v["concept"].as_str().unwrap().into(),
                    registry_version: v["registry_version"].as_str().unwrap().into(),
                },
                predicate: vec![TrilingualText {
                    schema_version: 1,
                    zh: v["predicate"]["zh"].as_str().unwrap().into(),
                    en: v["predicate"]["en"].as_str().unwrap().into(),
                    ja: v["predicate"]["ja"].as_str().unwrap().into(),
                }],
                direction: nature(array(&v["direction"])),
                relation: None,
            })
            .collect();
        let groups: Vec<_> = e["groups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| law::GroupItem {
                group_id: v["id"].as_str().unwrap().into(),
                position: v["first_position"].as_u64().unwrap(),
                source_key: v["source_keys"][0].as_str().unwrap().into(),
                members: v["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().into())
                    .collect(),
                verifiable: v["verifiable"].as_bool().unwrap(),
                content: law::GroupContent::Experience {
                    appraisals: vec![nature(array(&v["appraisal"]))],
                    matches: v["matches"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|m| law::Match {
                            target: law::MatchTarget::Tendency(
                                m["candidate"].as_str().unwrap().into(),
                            ),
                            mu: number(m, "mu"),
                        })
                        .collect(),
                },
            })
            .collect();
        let aliases: BTreeMap<_, _> = e["aliases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| {
                (
                    a["alias"].as_str().unwrap().into(),
                    a["canonical"].as_str().unwrap().into(),
                )
            })
            .collect();
        let order: Vec<String> = e["proposal_order"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        let ctx = law::LawContext {
            law_version: &self.binding,
            law: self.law.clone(),
            caretaker: law::Caretaker {
                person_id: self.resource["caretaker"]["person_id"].as_str().unwrap(),
                attachment_prior: number(&self.resource["caretaker"], "attachment_prior"),
            },
            r_capacity: self.resource["r_capacity"].as_u64().unwrap() as u32,
            protected_candidates: self.resource["protected_candidates"].as_u64().unwrap() as u32,
            relation_concepts: &self.concepts,
        };
        let out = law::g(
            law::RState {
                snapshot: &self.snapshot,
                groups: &self.groups,
            },
            &self.theta,
            &law::Step {
                revision: e["revision"].as_str().unwrap(),
                candidates: &candidates,
                groups: &groups,
                aliases: &aliases,
                proposal_order: &order,
            },
            &ctx,
        )
        .unwrap();
        let audit = json!({"changes":out.changes,"counterfactual":out.counterfactual,"after_additions":out.after_additions,
            "capacity_audits":out.capacity_audits.iter().map(|a|json!({"removed":a.removed,"before":a.before,"after":a.after})).collect::<Vec<_>>(),
            "capacity_notice":out.capacity_notice.as_ref().map(|a|json!({"capacity":a.capacity,"retained":a.retained,
                "protected_candidates":a.protected_candidates,"deferred_candidates":a.deferred_candidates}))});
        self.snapshot = out.r;
        for group in out.groups {
            self.groups.retain(|old| old.group_id != group.group_id);
            self.groups.push(group);
        }
        self.groups.sort_by(|a, b| a.group_id.cmp(&b.group_id));
        audit
    }
}
