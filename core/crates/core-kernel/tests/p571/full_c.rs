//! v2 §4 public C adapter. Recurrence and snapshot validation stay in the DUT.
use super::{c, c_law, limits, nature, number, reference};
use core_types::WireMessage;
use serde_json::{Value, json};

pub(super) struct Machine {
    pub state: c::DynamicsState,
    genesis: c::DynamicsState,
    law: c::DynamicsLaw,
    fuses: c::DynamicsFuses,
    binding: core_types::generated::ObjectRef,
    common: bool,
    history: Vec<Value>,
}

impl Machine {
    pub fn new(q: &Value) -> Self {
        let r = &q["resource"];
        let case = &q["case"];
        let law = c_law(r, number(case, "b0"), number(case, "p0"));
        let fuses = c::DynamicsFuses {
            jump_notice: r["jump_notice_bits"]
                .as_str()
                .map(|s| f64::from_bits(u64::from_str_radix(s, 16).unwrap()))
                .unwrap_or_else(|| number(r, "jump_notice")),
            drift_run: r["drift_run"].as_u64().unwrap(),
            drift_margin: number(r, "drift_margin"),
        };
        let common = case["version"] == "A";
        let genesis = if common {
            c::DynamicsState::genesis_common_discount(law.clone())
        } else {
            c::DynamicsState::genesis(law.clone())
        }
        .unwrap();
        let mut binding = reference(&q["resource_ref"], "echo.law-version");
        binding.resource_version = law.version.clone();
        Self {
            state: genesis.clone(),
            genesis,
            law,
            fuses,
            binding,
            common,
            history: vec![],
        }
    }

    pub fn snapshot(&self) -> (Value, Vec<u8>) {
        if self.common {
            let s =
                c::export_snapshot_v3(&self.state, "matrix", &self.binding, &self.fuses, limits())
                    .unwrap();
            (
                serde_json::to_value(&s).unwrap(),
                s.canonical_bytes(limits()).unwrap(),
            )
        } else {
            let s = c::export_snapshot(&self.state, "matrix", &self.binding, &self.fuses, limits())
                .unwrap();
            (
                serde_json::to_value(&s).unwrap(),
                s.canonical_bytes(limits()).unwrap(),
            )
        }
    }

    pub fn advance(&mut self, e: &Value) -> Value {
        // Fixed governance repair replays the admitted prefix; it is not a
        // subtraction approximation or evidence of production H transactions.
        if let Some(invalid) = e["invalidate_prior_ordinals"].as_array() {
            for prior in &mut self.history {
                if invalid.contains(&prior["ordinal"]) {
                    prior["eligible"] = json!(false);
                }
            }
            self.state = self.genesis.clone();
            for prior in self.history.clone() {
                self.one(&prior);
            }
        }
        let result = self.one(e);
        self.history.push(e.clone());
        result
    }

    fn one(&mut self, e: &Value) -> Value {
        let source = match e["source"].as_str().unwrap() {
            "self_action" => c::ReactionSource::SelfAction,
            "program_result" => c::ReactionSource::TriggeredProgramResult,
            "self_narrative" => c::ReactionSource::SelfNarrative,
            "desktop" => c::ReactionSource::DesktopObservation,
            _ => c::ReactionSource::OtherPersonReply,
        };
        let eligible = e["eligible"].as_bool().unwrap();
        let stops: Vec<_> = e["stops"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| c::Downtime {
                start_ns: d["start_ns"].as_u64().unwrap(),
                end_ns: d["end_ns"].as_u64().unwrap(),
            })
            .collect();
        let t = e["time_ns"].as_u64().unwrap();
        let a = if e["a"].is_null() {
            0.0
        } else {
            match e["a"].as_str() {
                Some("nan") => f64::NAN,
                Some("inf") => f64::INFINITY,
                Some("-inf") => f64::NEG_INFINITY,
                _ => number(e, "a"),
            }
        };
        let event = c::ReactionEvent {
            id: e["id"].as_str().unwrap().into(),
            ordinal: e["ordinal"].as_u64().unwrap(),
            time_ns: t,
            running_time: c::RunningTime::from_history(0, t, &stops).unwrap(),
            source,
            evaluation_unmodulated: true,
            reaction: nature([a; 4]),
        };
        // Excluded public origins are still submitted. Missing observations
        // and fixed duplicate/invalid governance use the documented None path.
        let public_exclusion = matches!(
            source,
            c::ReactionSource::SelfAction
                | c::ReactionSource::TriggeredProgramResult
                | c::ReactionSource::SelfNarrative
                | c::ReactionSource::DesktopObservation
        );
        match c::stage(
            &self.state,
            (eligible || public_exclusion).then_some(&event),
            &self.law,
            &self.fuses,
        ) {
            Ok(staged) => {
                let bounds = staged.bounds.as_ref().map(|b| {
                    json!({"range":b.range,
                    "single_event":b.single_event,"range_only":b.range_only,"limits":b.limits()})
                });
                let result = json!({"fault":null,"changed":staged.change.is_some(),"bounds":bounds,
                    "signals":staged.signals.iter().map(|s|format!("{s:?}")).collect::<Vec<_>>()});
                self.state = staged.into_state();
                result
            }
            Err(fault) => {
                json!({"fault":format!("{fault:?}"),"changed":false,"bounds":null,"signals":[]})
            }
        }
    }
}
