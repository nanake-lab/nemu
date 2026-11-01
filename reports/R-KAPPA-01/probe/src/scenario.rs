use crate::{kernel, verifier::*};
use core_kernel::fast::{self, EventClass, EventInput};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const ROUNDS: u64 = 256;
pub const SWITCH: u64 = 120;
pub const NS: u64 = 1_000_000_000;
pub const EPSILON: f64 = 0.05;
pub const TAU_S: u64 = 2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub round: u64,
    pub time_ns: u64,
    pub text: String,
    pub appraisal: [f64; 4],
    pub mu: f64,
    pub result: Option<(Request, Receipt)>,
    // Scoring labels are deliberately outside Request/Receipt and verifier API.
    pub oracle_new_fact: bool,
    pub oracle_divisible: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fixture {
    pub scenario: String,
    pub known_at_start: Vec<Question>,
    pub events: Vec<Event>,
}

pub fn fixtures() -> Vec<Fixture> {
    ["s1_recall", "s1_changing_content", "s2_new_results"]
        .iter()
        .map(|name| {
            let mut events = Vec::new();
            for round in 0..ROUNDS {
                let discoveries = *name == "s2_new_results";
                let changed = discoveries && round >= SWITCH;
                let question = Question {
                    dividend: if discoveries {
                        7 * (200 + round) + u64::from(changed)
                    } else {
                        700
                    },
                    divisor: 7,
                };
                let issued_ns = (round + 1) * NS;
                let text = match *name {
                    "s1_recall" => "回忆并取回已经知道的 700 可以被 7 整除".into(),
                    "s1_changing_content" => format!(
                        "设想在棋盘上摆 {} 个棋子，构造第 {} 种布局；这只是新设想，没有新依据",
                        round + 1,
                        round * 3 + 2
                    ),
                    _ => format!("请求核验问题 {} / 7", question.dividend),
                };
                events.push(Event {
                    round,
                    time_ns: issued_ns,
                    text,
                    appraisal: [0.0, 0.0, 0.0, if discoveries { 0.0 } else { 0.2 }],
                    mu: if discoveries { 0.0 } else { 0.2 },
                    result: None,
                    oracle_new_fact: false,
                    oracle_divisible: !changed,
                });
                let request = Request {
                    action_id: format!("{name}-{round}"),
                    question: question.clone(),
                    issued_round: round,
                    issued_ns,
                    deadline_ns: issued_ns + TAU_S * NS,
                };
                let receipt = Receipt {
                    action_id: request.action_id.clone(),
                    question_digest: question.digest(),
                    quotient: question.dividend / question.divisor,
                    remainder: question.dividend % question.divisor,
                    verifier_version: VERSION.into(),
                };
                events.push(Event {
                    round,
                    time_ns: issued_ns + NS / 5,
                    text: format!(
                        "工具返回：{} = 7 * {} + {}",
                        question.dividend, receipt.quotient, receipt.remainder
                    ),
                    appraisal: [0.0, 0.0, 0.0, if changed { -0.2 } else { 0.2 }],
                    mu: if discoveries { 0.0 } else { 0.2 },
                    result: Some((request, receipt)),
                    oracle_new_fact: discoveries,
                    oracle_divisible: !changed,
                });
            }
            Fixture {
                scenario: (*name).into(),
                known_at_start: vec![Question {
                    dividend: 700,
                    divisor: 7,
                }],
                events,
            }
        })
        .collect()
}

pub fn envelope() -> Value {
    // Conservative eta includes the one permitted R+ effect, even when mu=0.
    let eta = 3.0;
    let kappa = kernel::law().kappa;
    let mut n = 0;
    while eta * 2.0 * fast::kappa_power(kappa, n) / (1.0 - kappa) > EPSILON / 2.0 {
        n += 1;
    }
    let b_n = (n + 1) as f64; // conservative off-by-one: internal index n commits at n+1 s
    let t_ret = b_n + TAU_S as f64 + libm::log(4.0 / EPSILON);
    json!({"eta":eta,"kappa":kappa,"epsilon":EPSILON,"n":n,"b_n_s":b_n,
        "tau_s":TAU_S,"r":1,"m":1,"lambda_min_per_s":1.0,"t_ret_s":t_ret,
        "t_x_s":17.0,"observation_end_s":256.2,"rounds":ROUNDS,"switch_round":SWITCH,
        "simulation_only":true,"hardware_B_N_calibrated":false})
}

pub fn run(fixture: &Fixture, group: &str) -> (Vec<Value>, Value) {
    assert!(["A", "B", "C"].contains(&group));
    let effects = kernel::effects();
    let mut x = kernel::initial();
    let mut time_ns = 0;
    let mut n = 0;
    let mut gate = Verifier::new(&fixture.known_at_start);
    let mut traces = Vec::new();
    let t_ret = envelope()["t_ret_s"].as_f64().unwrap();
    let t_ret_ns = (t_ret * NS as f64).ceil() as u64;
    let mut at_t_ret: Option<Vec<f64>> = None;
    for event in &fixture.events {
        if at_t_ret.is_none() && event.time_ns >= t_ret_ns {
            at_t_ret = Some(kernel::vector(
                &fast::project_time(&x, &effects, t_ret_ns - time_ns).unwrap(),
            ));
        }
        let (class, verdict) = match &event.result {
            Some((request, receipt)) => (
                EventClass::ActionResult {
                    n_issued: request.issued_round,
                },
                Some(gate.accept(request, receipt, event.time_ns)),
            ),
            None => {
                assert_eq!(n, event.round);
                (EventClass::InternalRecord { n }, None)
            }
        };
        let release = group == "B" || (group == "C" && verdict.as_ref().is_some_and(|v| v.release));
        let allowed = verdict.as_ref().is_none_or(|v| v.pulse_allowed);
        let before = kernel::vector(&x);
        let (after, drive, projected) = if allowed {
            kernel::pulse(
                &x,
                &effects,
                event.time_ns - time_ns,
                EventInput {
                    appraisal: Some(event.appraisal),
                    empathy_weight: 1.0,
                    class,
                },
                event.mu,
                release,
            )
        } else {
            let projected = fast::project_time(&x, &effects, event.time_ns - time_ns).unwrap();
            (projected.clone(), vec![0.0; 5], kernel::vector(&projected))
        };
        let next_n = fast::next_internal_count(class, n);
        let v = kernel::vector(&after);
        let delta: Vec<_> = v.iter().zip(&projected).map(|(a, b)| a - b).collect();
        let class_name = if event.result.is_some() {
            "ActionResult"
        } else {
            "InternalRecord"
        };
        traces.push(json!({"scenario":fixture.scenario,"group":group,"round":event.round,
            "time_ns":event.time_ns,"class":class_name,"n_before":n,"n_after":next_n,
            "n_issued":event.result.as_ref().map(|(r,_)|r.issued_round),
            "appraisal":event.appraisal,"mu":event.mu,"before":before,"projected":projected,
            "drive":drive,"delta_x":delta,"after":v,"release":release && allowed,
            "class_weight":if !allowed {0.0} else if release {1.0} else {fast::kappa_power(kernel::law().kappa,event.round)},
            "verdict":verdict,"known_fact_count":gate.known.len(),
            "oracle_new_fact":event.oracle_new_fact,"oracle_divisible":event.oracle_divisible}));
        x = after;
        time_ns = event.time_ns;
        n = next_n;
    }
    let max_all = traces
        .iter()
        .flat_map(|t| t["after"].as_array().unwrap())
        .map(|v| v.as_f64().unwrap().abs())
        .fold(0.0, f64::max);
    let mut max_after_return = at_t_ret
        .as_ref()
        .unwrap()
        .iter()
        .map(|v| v.abs())
        .fold(0.0, f64::max);
    for trace in &traces {
        if trace["time_ns"].as_u64().unwrap() >= t_ret_ns {
            for value in trace["after"].as_array().unwrap() {
                max_after_return = max_after_return.max(value.as_f64().unwrap().abs());
            }
        }
    }
    let select = |round: u64| {
        traces
            .iter()
            .find(|t| t["round"] == round && t["class"] == "ActionResult")
            .unwrap()
            .clone()
    };
    let final_state = kernel::vector(&x);
    let silent_after_10s = kernel::vector(&fast::project_time(&x, &effects, 10 * NS).unwrap());
    let first_negative_after_switch = traces
        .iter()
        .find(|t| t["round"].as_u64().unwrap() >= SWITCH && t["after"][3].as_f64().unwrap() < 0.0)
        .map(|t| t["time_ns"].clone());
    let summary = json!({"scenario":fixture.scenario,"group":group,"events":fixture.events.len(),
        "internal_records":n,"action_results":gate.completed.len(),"final_gate":gate,
        "max_abs_state":max_all,"at_t_ret":at_t_ret,"max_abs_after_t_ret":max_after_return,
        "old_uniform_return_holds_in_observed_window":max_after_return <= EPSILON,
        "early_result":select(0),"before_change":select(SWITCH-1),"first_counterexample":select(SWITCH),
        "first_negative_after_switch_ns":first_negative_after_switch,
        "final_state":final_state,"silent_after_10s":silent_after_10s});
    (traces, summary)
}
