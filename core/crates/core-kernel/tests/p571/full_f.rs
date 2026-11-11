//! §5 public F adapter and synthetic recorded-input resolver. No oracle math.
use super::{effects, fast, limits, nature, number, reference};
use core_types::{
    WireMessage,
    contract::Schema,
    generated::*,
    scalar::{DecimalU64, HexBytes},
};
use fast::contribution as f;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn array(v: &Value) -> [f64; 4] {
    std::array::from_fn(|i| v[i].as_f64().unwrap())
}
fn vector(n: [f64; 4], learned: &[fast::LearnedValue]) -> LawStateVector {
    LawStateVector {
        schema_version: 1,
        nature: NatureReal {
            schema_version: 1,
            connection: n[0],
            empathy: n[1],
            curiosity: n[2],
            achievement: n[3],
        },
        learned: learned
            .iter()
            .map(|v| LawLearnedCoordinate {
                schema_version: 1,
                tendency_id: v.tendency_id.clone(),
                value: v.value,
            })
            .collect(),
    }
}
fn content_ref<T: WireMessage>(value: &T, tag: &str) -> ObjectRef {
    let schema = Schema::builtin().unwrap();
    let bytes = value.canonical_bytes(limits()).unwrap();
    let decoded = schema
        .decode_canonical(T::NAME, &bytes, limits(), 0)
        .unwrap();
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(
            core_types::scalar::unhex(&schema.digest(T::NAME, &decoded, limits()).unwrap())
                .unwrap(),
        ),
        byte_length: DecimalU64(bytes.len() as u64),
        object_type: tag.into(),
        resource_version: "P5-71-v2".into(),
    }
}

pub(super) struct Machine {
    pub state: f::State,
    pub old: fast::FastState,
    pub effects: effects::EffectiveEffects,
    old_version: bool,
    old_anchor: u64,
    internal: u64,
    binding: ObjectRef,
    input_ref: ObjectRef,
    resource: Value,
    case: Value,
    history: BTreeMap<String, (Value, f::Effect)>,
    active: BTreeMap<String, bool>,
}
impl Machine {
    pub fn new(q: &Value) -> Self {
        let case = &q["case"];
        let binding = reference(&q["resource_ref"], "echo.law-version");
        let input_ref = reference(&q["input_ref"], "p571.synthetic-history");
        let theta = &case["theta"];
        let stored = effects::StoredTheta {
            b: array(&theta["b"]),
            g: array(&theta["g"]),
            lambda: std::array::from_fn(|i| {
                f64::from_bits(
                    u64::from_str_radix(theta["lambda_bits"][i].as_str().unwrap(), 16).unwrap(),
                )
            }),
        };
        let effect_list = if case["id"] == "09-19" {
            vec![effects::TendencyEffect {
                tendency_id: "k".into(),
                concept: "synthetic-k".into(),
                s: number(&case["historical_inputs"][0], "effect_s"),
                r: number(&case["historical_inputs"][0], "effect_r"),
                d: std::array::from_fn(|j| {
                    if j == case["coordinate"].as_u64().unwrap() as usize {
                        number(case, "sign")
                    } else {
                        0.0
                    }
                }),
                direction_validated: false,
            }]
        } else {
            vec![]
        };
        let learned = effect_list
            .iter()
            .map(|e| LearnedComponent {
                schema_version: 1,
                tendency_id: e.tendency_id.clone(),
                value: 0.0,
            })
            .collect();
        let mut snapshot = FastStateSnapshotV3 {
            schema_version: 1,
            revision: "matrix".into(),
            representation_version: f::REPRESENTATION_VERSION.into(),
            anchor_time_ns: DecimalU64(0),
            nature: nature([0.0; 4]),
            learned,
            slots: vec![],
            law_version: binding.clone(),
        };
        if !case["initial"].is_null() {
            let init = &case["initial"];
            let j = init["coordinate"].as_u64().unwrap() as usize;
            let mut x = [0.0; 4];
            x[j] = number(init, "x");
            snapshot.nature = nature(x);
            let mut h = [0.0; 4];
            h[j] = number(init, "h");
            snapshot.slots.push(FastBasisSlot {
                schema_version: 1,
                basis: FastBasisIdentity {
                    schema_version: 1,
                    kind: FastBasisKind::FAST_BASIS_RECORD,
                    record_id: Some("r1".into()),
                },
                current: vector(h, &[]),
                target: vector([0.0; 4], &[]),
            });
        }
        let state = f::State::from_snapshot(snapshot, &binding, limits()).unwrap();
        Self {
            old: state.visible(),
            state,
            effects: effects::EffectiveEffects {
                effects: effect_list,
                theta: stored,
            },
            old_version: case["version"] == "old",
            old_anchor: 0,
            internal: 0,
            binding,
            input_ref,
            resource: q["resource"].clone(),
            case: case.clone(),
            history: BTreeMap::new(),
            active: BTreeMap::new(),
        }
    }
    pub fn snapshot(&self) -> (Value, Vec<u8>) {
        if self.old_version {
            let s = FastStateSnapshotV2 {
                schema_version: 1,
                revision: "matrix".into(),
                representation_version: fast::REPRESENTATION_VERSION.into(),
                anchor_time_ns: DecimalU64(self.old_anchor),
                nature: nature(self.old.nature),
                learned: self
                    .old
                    .learned
                    .iter()
                    .map(|v| LearnedComponent {
                        schema_version: 1,
                        tendency_id: v.tendency_id.clone(),
                        value: v.value,
                    })
                    .collect(),
            };
            (
                serde_json::to_value(&s).unwrap(),
                s.canonical_bytes(limits()).unwrap(),
            )
        } else {
            (
                serde_json::to_value(self.state.snapshot()).unwrap(),
                self.state.canonical_bytes(limits()).unwrap(),
            )
        }
    }
    pub fn project(&mut self, at: u64) {
        if self.old_version {
            self.old = fast::project_time(&self.old, &self.effects, at - self.old_anchor).unwrap();
            self.old_anchor = at;
        } else {
            self.state = self.state.project_time(&self.effects, at).unwrap();
        }
    }
    pub fn read(&self, delta: u64) -> Value {
        let x = if self.old_version {
            fast::project_time(&self.old, &self.effects, delta).unwrap()
        } else {
            self.state
                .project_time(
                    &self.effects,
                    self.state.snapshot().anchor_time_ns.0 + delta,
                )
                .unwrap()
                .visible()
        };
        let base = if self.old_version {
            self.old.clone()
        } else {
            self.state.visible()
        };
        let phi = fast::project_time(&base, &self.effects, delta).unwrap();
        json!({"nature":x.nature.map(f64::to_bits),"learned":x.learned.iter().map(|v|v.value.to_bits()).collect::<Vec<_>>(),
            "phi_nature":phi.nature.map(f64::to_bits),"phi_learned":phi.learned.iter().map(|v|v.value.to_bits()).collect::<Vec<_>>()})
    }
    fn target_inputs(
        &self,
        event: &Value,
    ) -> (effects::EffectiveEffects, effects::Evaluation, [f64; 4]) {
        let mut eff = self.effects.clone();
        let a = array(&event["target"]);
        let historical = event["target_mode"] == "evaluate_recorded_inputs";
        if historical {
            let t = event["theta_revision"].as_u64().unwrap() as usize;
            let r = event["effects_revision"].as_u64().unwrap() as usize;
            eff.theta.g = array(&self.case["historical_inputs"][t]["theta_g"]);
            eff.effects[0].s = number(&self.case["historical_inputs"][r], "effect_s");
            eff.effects[0].r = number(&self.case["historical_inputs"][r], "effect_r");
            eff.effects[0].d = array(&self.case["historical_inputs"][r]["effect_direction"]);
        } else if event["target_mode"] != "current_theta" {
            // Direct-target algebra fixture: neutral recorded target construction;
            // the live projection theta (e.g. §5 09-14) is a separate input.
            eff.theta.b = [0.0; 4];
            eff.theta.g = [1.0; 4];
        }
        let matches = effects::SituationMatch {
            mu: if historical {
                vec![("k".into(), number(event, "mu"))]
            } else {
                vec![]
            },
        };
        let evaluation = effects::evaluate_effects(&eff, &matches).unwrap();
        (eff, evaluation, a)
    }
    fn effect(&self, event: &Value) -> f::Effect {
        let (eff, evaluation, a) = self.target_inputs(event);
        let target = f::compute_target(
            &eff,
            &evaluation,
            Some(a),
            1.0,
            &f::Law {
                version: &self.binding,
                g_max: number(&self.resource, "g_max"),
                r_max: number(&self.resource, "r_max"),
                w_min: number(&self.resource, "w_min"),
            },
        )
        .unwrap();
        let internal = event["kind"] == "internal";
        let historical = event["target_mode"] == "evaluate_recorded_inputs";
        let theta_ref = if historical {
            reference(
                &self.case["historical_bindings"]
                    [event["theta_revision"].as_u64().unwrap() as usize]["theta"],
                "echo.theta-snapshot-v3",
            )
        } else {
            let mut r = self.input_ref.clone();
            r.object_type = "echo.theta-snapshot-v3".into();
            r
        };
        let effects_ref = if historical {
            reference(
                &self.case["historical_bindings"]
                    [event["effects_revision"].as_u64().unwrap() as usize]["effects"],
                "p571.effects-view",
            )
        } else {
            self.input_ref.clone()
        };
        f::Effect::new(
            FastEffectRecord {
                schema_version: 1,
                basis: FastBasisIdentity {
                    schema_version: 1,
                    kind: if internal {
                        FastBasisKind::FAST_BASIS_INTERNAL_BRANCH
                    } else {
                        FastBasisKind::FAST_BASIS_RECORD
                    },
                    record_id: if internal {
                        None
                    } else {
                        Some(event["root"].as_str().unwrap().into())
                    },
                },
                role: if event["op"] == "update" {
                    FastEffectRole::FAST_EFFECT_TARGET_UPDATE
                } else if internal {
                    FastEffectRole::FAST_EFFECT_INTERNAL_BRANCH
                } else {
                    FastEffectRole::FAST_EFFECT_NEW_BASIS
                },
                target: vector(*target.drive().nature(), target.drive().learned()),
                theta_before: theta_ref,
                effects_before: effects_ref,
                interpretation_refs: vec![if historical {
                    reference(
                        &event["recorded_bindings"]["interpretation"],
                        "p571.interpretation",
                    )
                } else {
                    self.input_ref.clone()
                }],
                governance_refs: vec![if historical {
                    reference(&event["recorded_bindings"]["governance"], "p571.governance")
                } else {
                    self.input_ref.clone()
                }],
                law_version: self.binding.clone(),
            },
            &target,
            limits(),
        )
        .unwrap()
    }
    pub fn advance(&mut self, event: &Value) -> Value {
        let op = event["op"].as_str().unwrap();
        let root = event["root"].as_str().unwrap();
        if self.case["negative_control"] == "F-off" {
            return json!({"applied":false,"fault":null});
        }
        match op {
            "noop" | "reinterpret" => return json!({"applied":false,"fault":null}),
            "invalidate" => {
                self.active.insert(root.into(), false);
                return json!({"applied":false,"fault":null});
            }
            "roundtrip" => {
                if !self.old_version {
                    self.state = f::State::from_canonical(
                        &self.state.canonical_bytes(limits()).unwrap(),
                        &self.binding,
                        limits(),
                    )
                    .unwrap();
                }
                return json!({"applied":false,"fault":null,"roundtrip":true});
            }
            "remove_zero" => {
                if !self.old_version {
                    self.state = self.state.remove_zero_slots();
                }
                return json!({"applied":false,"fault":null});
            }
            "migrate" => {
                if self.old_version {
                    return json!({"applied":false,"fault":null,"migration":"not_applicable_old_law"});
                }
                let s = self.state.snapshot();
                let old = FastStateSnapshotV2 {
                    schema_version: 1,
                    revision: s.revision.clone(),
                    representation_version: fast::REPRESENTATION_VERSION.into(),
                    anchor_time_ns: s.anchor_time_ns,
                    nature: s.nature.clone(),
                    learned: s.learned.clone(),
                };
                let mut next = s.clone();
                next.slots.clear();
                let record = FastStateMigrationRecord {
                    schema_version: 1,
                    previous_snapshot: content_ref(&old, "echo.fast-state-v2"),
                    migrated_snapshot: content_ref(&next, "echo.fast-state-v3"),
                    prior_decomposition_unknown: true,
                };
                let (state, record) =
                    f::State::migrate(&old, &self.binding, record, limits()).unwrap();
                self.state = state;
                return json!({"applied":false,"fault":null,"migration":record});
            }
            "merge" => {
                if !self.old_version {
                    let members: Vec<_> = event["members"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|key| {
                            let (e, t) = &self.history[key.as_str().unwrap()];
                            f::HistoryBasis {
                                first_position: e["ordinal"].as_u64().unwrap(),
                                target: t,
                                reinterpreted: None,
                                active: true,
                                revisit_governance: &t.record().governance_refs,
                            }
                        })
                        .collect();
                    self.state = self.state.merge_aliases(&members, limits()).unwrap();
                }
                return json!({"applied":false,"fault":null});
            }
            _ => {}
        }
        let active = event["active"]
            .as_bool()
            .unwrap_or(*self.active.get(root).unwrap_or(&true));
        if !active {
            return json!({"applied":false,"fault":null});
        }
        // Rejected internal revisits leave both time anchor and state intact.
        if op == "revisit" && event["kind"] == "internal" {
            let (_, original) = &self.history[root];
            let result = self.state.revisit(
                &[f::HistoryBasis {
                    first_position: 1,
                    target: original,
                    reinterpreted: None,
                    active: true,
                    revisit_governance: &original.record().governance_refs,
                }],
                limits(),
            );
            return json!({"applied":false,"fault":result.err().map(|e|format!("{e:?}"))});
        }
        let resolved = if op == "revisit" && event["target_mode"] != "evaluate_recorded_inputs" {
            self.history
                .get(root)
                .map(|(e, _)| e.clone())
                .unwrap_or_else(|| event.clone())
        } else {
            event.clone()
        };
        let effect = self.effect(&resolved);
        self.project(event["time_ns"].as_u64().unwrap());
        let result = if self.old_version {
            let (eff, evaluation, a) = self.target_inputs(&resolved);
            let class = if op != "revisit" && event["source"] == "action_result" {
                fast::EventClass::ActionResult { n_issued: 1 }
            } else if op == "new" && event["kind"] == "record" {
                self.internal = 0;
                fast::EventClass::NewExogenous
            } else {
                self.internal += 1;
                fast::EventClass::InternalRecord { n: self.internal }
            };
            let drive = fast::compute_drive(
                &eff,
                &evaluation,
                &fast::EventInput {
                    appraisal: Some(a),
                    empathy_weight: 1.0,
                    class,
                },
                &fast::DriveLaw {
                    g_max: number(&self.resource, "g_max"),
                    r_max: number(&self.resource, "r_max"),
                    w_min: number(&self.resource, "w_min"),
                    kappa: number(&self.resource, "kappa"),
                },
            )
            .unwrap();
            self.old = fast::apply_drive(&self.old, &drive).unwrap();
            json!({"applied":true,"fault":null,"record":effect.record()})
        } else if op == "revisit" {
            let (prior, original) = &self.history[root];
            let revised = (event["target_mode"] == "evaluate_recorded_inputs").then_some(&effect);
            let (state, receipts) = self
                .state
                .revisit(
                    &[f::HistoryBasis {
                        first_position: prior["ordinal"].as_u64().unwrap(),
                        target: original,
                        reinterpreted: revised,
                        active: true,
                        revisit_governance: &effect.record().governance_refs,
                    }],
                    limits(),
                )
                .unwrap();
            self.state = state;
            json!({"applied":true,"fault":null,"receipts":receipts})
        } else {
            self.state = self.state.apply(&effect, limits()).unwrap();
            json!({"applied":true,"fault":null,"record":effect.record()})
        };
        if op == "new" || op == "update" {
            self.history.insert(root.into(), (event.clone(), effect));
            self.active.insert(root.into(), true);
        }
        if self.case["negative_control"] == "x-zero" {
            self.old.nature = [0.0; 4];
            for v in &mut self.old.learned {
                v.value = 0.0;
            }
            let mut s = self.state.snapshot().clone();
            s.nature = nature([0.0; 4]);
            for v in &mut s.learned {
                v.value = 0.0;
            }
            self.state = f::State::from_snapshot(s, &self.binding, limits()).unwrap();
        }
        result
    }
}
