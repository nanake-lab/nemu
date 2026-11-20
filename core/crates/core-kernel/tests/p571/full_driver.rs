//! Sequential full-prefix DUT driver. Each completed prefix is flushed before
//! the next one; wall deadline checks also occur inside replay loops.
use super::{full_c, full_f, full_g};
use serde_json::{Value, json};
use std::io::Write;
use std::time::{Duration, Instant};

enum Machine {
    C(Box<full_c::Machine>),
    F(Box<full_f::Machine>),
    G(Box<full_g::Machine>),
    Cf(Box<(full_c::Machine, full_f::Machine)>),
}
fn theta(values: &core_types::generated::ThetaValues) -> core_kernel::effects::StoredTheta {
    core_kernel::effects::StoredTheta {
        b: core_types::semantic::nature(&values.baseline),
        g: core_types::semantic::scale(&values.gain),
        lambda: core_types::semantic::scale(&values.recovery_per_s),
    }
}
impl Machine {
    fn new(q: &Value) -> Self {
        match q["family"].as_str().unwrap() {
            "C" => Self::C(Box::new(full_c::Machine::new(q))),
            "F" => Self::F(Box::new(full_f::Machine::new(q))),
            "G" => Self::G(Box::new(full_g::Machine::new(q))),
            "CF" => {
                let version = q["case"]["version"].as_str().unwrap();
                let mut cq = q.clone();
                cq["case"]["version"] = json!(if version.starts_with("oldC") {
                    "old"
                } else {
                    "A"
                });
                cq["resource"] = q["c_resource"].clone();
                cq["resource_ref"] = q["c_resource_ref"].clone();
                let c = full_c::Machine::new(&cq);
                let mut fq = q.clone();
                fq["case"]["version"] = json!(if version.ends_with("oldF") {
                    "old"
                } else {
                    "A1"
                });
                fq["case"]["theta"] = json!({"b":vec![0.0;4],"g":vec![1.0;4],"lambda_bits":vec!["3fe62e42fefa39ef";4]});
                Self::Cf(Box::new((c, full_f::Machine::new(&fq))))
            }
            _ => panic!("unregistered DUT family"),
        }
    }
    fn advance(&mut self, e: &Value, control: &str, index: usize) -> Value {
        match self {
            Self::C(m) => {
                if control == "C-frozen" {
                    json!({"fault":null,"changed":false,"signals":[],"bounds":null})
                } else {
                    m.advance(e)
                }
            }
            Self::F(m) => m.advance(e),
            Self::G(m) => m.advance(e, control == "G-frozen" && index > 0),
            Self::Cf(pair) => {
                let (c, f) = pair.as_mut();
                let mut ca = Value::Null;
                if !e["c"].is_null() && control != "C-frozen" {
                    ca = c.advance(&e["c"]);
                }
                if !ca["fault"].is_null() {
                    return json!({"c":ca,"f":null});
                }
                let before = f.snapshot().0;
                if !e["c"].is_null() || !e["f"].is_null() {
                    f.project(e["time_ns"].as_u64().unwrap());
                }
                let after_time = f.snapshot().0;
                let fa = if !e["f"].is_null() {
                    f.advance(&e["f"])
                } else {
                    Value::Null
                };
                if !e["c"].is_null() && control != "C-frozen" {
                    f.effects.theta = theta(c.state.values());
                }
                json!({"c":ca,"f":fa,"fast_before_time":before,"fast_after_time":after_time})
            }
        }
    }
    fn snapshot(&self) -> (Value, Vec<u8>) {
        match self {
            Self::C(m) => m.snapshot(),
            Self::F(m) => m.snapshot(),
            Self::G(m) => m.snapshot(),
            Self::Cf(pair) => {
                let (cv, cb) = pair.0.snapshot();
                let (fv, fb) = pair.1.snapshot();
                let mut bytes = (cb.len() as u64).to_le_bytes().to_vec();
                bytes.extend(cb);
                bytes.extend(fb);
                (json!({"c":cv,"f":fv}), bytes)
            }
        }
    }
    fn reads(&self, case: &Value) -> Value {
        if let Self::F(m) = self {
            json!(
                case["reads_ns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|d| m.read(d.as_u64().unwrap()))
                    .collect::<Vec<_>>()
            )
        } else if let Self::C(m) = self {
            if case["reads_ns"].as_array().is_some_and(|v| !v.is_empty()) {
                let effects = core_kernel::effects::EffectiveEffects {
                    effects: vec![],
                    theta: theta(m.state.values()),
                };
                let x = core_kernel::fast::FastState::new(
                    std::array::from_fn(|i| case["read_initial_x"][i].as_f64().unwrap()),
                    vec![],
                )
                .unwrap();
                return json!(case["reads_ns"].as_array().unwrap().iter().map(|d| {
                    let p=core_kernel::fast::project_time(&x,&effects,d.as_u64().unwrap()).unwrap();
                    json!({"nature":p.nature.map(f64::to_bits),"learned":[],"phi_nature":p.nature.map(f64::to_bits),"phi_learned":[]})
                }).collect::<Vec<_>>());
            }
            Value::Null
        } else {
            Value::Null
        }
    }
}
pub(super) fn run(q: &Value) -> Value {
    let deadline = Instant::now() + Duration::from_millis(q["remaining_ms"].as_u64().unwrap());
    let events = q["case"]["events"].as_array().unwrap();
    let control = q["case"]["negative_control"].as_str().unwrap_or("");
    let mut incremental = Machine::new(q);
    let route_count = if q["family"] == "G" {
        q["case"]["f_route_bindings"].as_array().unwrap().len()
    } else {
        1
    };
    let mut siblings: Vec<_> = (1..route_count).map(|_| Machine::new(q)).collect();
    let mut completed = 0;
    for prefix in 0..=events.len() {
        if Instant::now() >= deadline {
            return json!({"status":"not_run","reason":"16 hour deadline","completed_prefix":completed});
        }
        let start = Instant::now();
        let audit = if prefix == 0 {
            Value::Null
        } else {
            incremental.advance(&events[prefix - 1], control, prefix - 1)
        };
        let incremental_ns = start.elapsed().as_nanos() as u64;
        let start = Instant::now();
        let mut replay = Machine::new(q);
        let mut replay_audit = Value::Null;
        for (index, event) in events[..prefix].iter().enumerate() {
            if Instant::now() >= deadline {
                return json!({"status":"not_run","reason":"16 hour deadline during replay","completed_prefix":completed});
            }
            replay_audit = replay.advance(event, control, index);
        }
        let replay_ns = start.elapsed().as_nanos() as u64;
        let (snapshot, bytes) = incremental.snapshot();
        let (other, other_bytes) = replay.snapshot();
        let mut i23 = bytes == other_bytes && snapshot == other && audit == replay_audit;
        let mut routes = Vec::new();
        for (route, sibling) in siblings.iter_mut().enumerate() {
            if Instant::now() >= deadline {
                return json!({"status":"not_run","reason":"16 hour deadline during G routes","completed_prefix":completed});
            }
            let sa = if prefix == 0 {
                Value::Null
            } else {
                sibling.advance(&events[prefix - 1], control, prefix - 1)
            };
            let mut sr = Machine::new(q);
            let mut ra = Value::Null;
            for (index, event) in events[..prefix].iter().enumerate() {
                ra = sr.advance(event, control, index);
            }
            let (sv, sb) = sibling.snapshot();
            let (rv, rb) = sr.snapshot();
            let same =
                sb == bytes && sv == snapshot && sa == audit && sb == rb && sv == rv && sa == ra;
            i23 &= same;
            routes.push(json!({"route":q["case"]["f_route_bindings"][route+1],"snapshot":sv,"audit":sa,"canonical":core_types::scalar::hex(&sb),"same":same}));
        }
        let reads = if prefix == events.len() {
            incremental.reads(&q["case"])
        } else if prefix > 0 && events[prefix - 1]["readonly_after_ns"].is_array() {
            let before = incremental.snapshot();
            let mut read_case = q["case"].clone();
            read_case["reads_ns"] = events[prefix - 1]["readonly_after_ns"].clone();
            let read = incremental.reads(&read_case);
            i23 &= before == incremental.snapshot();
            read
        } else {
            Value::Null
        };
        let row = json!({"prefix":prefix,"snapshot":snapshot,"canonical":core_types::scalar::hex(&bytes),"audit":audit,
            "i23":i23,"replay_difference":if i23 {Value::Null} else {json!({"snapshot":other,"audit":replay_audit})},
            "incremental_ns":incremental_ns,"replay_ns":replay_ns,"reads":reads,"additional_G_routes":routes});
        println!("P571-PREFIX:{}", serde_json::to_string(&row).unwrap());
        std::io::stdout().flush().unwrap();
        completed = prefix;
    }
    json!({"status":"completed","completed_prefix":completed})
}
