//! Q2 continuation: restore only synthetic checkpoints produced by this binary.
use crate::{adapter::*, probes, resources::*, world::*};
use core_kernel::{dynamics, fast};
use serde_json::{Value, json};
pub fn restore(v: Value) -> Result<State> {
    if v["synthetic_schema"] != VERSION {
        return Err("not a synthetic checkpoint".into());
    }
    let theta = serde_json::from_value(v["theta"].clone()).map_err(err)?;
    let learned = v["x"]["learned"]
        .as_array()
        .ok_or("learned")?
        .iter()
        .map(|r| {
            Ok(fast::LearnedValue {
                tendency_id: r[0].as_str().ok_or("id")?.into(),
                value: r[1].as_f64().ok_or("value")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(State {
        r: serde_json::from_value(v["r"].clone()).map_err(err)?,
        groups: serde_json::from_value(v["groups"].clone()).map_err(err)?,
        theta: dynamics::restore_snapshot(&theta, &reference(), &dynamics(), &fuses(), LIMITS)
            .map_err(err)?,
        x: fast::FastState::new(
            serde_json::from_value(v["x"]["nature"].clone()).map_err(err)?,
            learned,
        )
        .map_err(err)?,
        time: v["anchor"].as_u64().ok_or("anchor")?,
    })
}
pub fn continuation(s: &State, day: u32, ordinal: u64, hours: u32, seed: u64) -> Result<Value> {
    let run = Run {
        id: "Q2-common".into(),
        world: "W0".into(),
        seed,
        density: 4,
        hours,
        days: 30,
        status: "ready".into(),
        reason: String::new(),
    };
    let common = events(&run);
    let mut branches = vec![];
    for sign in [-1., 1.] {
        let mut with = s.clone();
        let mut without = s.clone();
        let new = Event {
            ordinal: ordinal + 1,
            day: day + 1,
            time: u64::from(day) * DAY + 1,
            hours,
            person: "U".into(),
            context: "punctual".into(),
            a: [sign * 0.25, 0., 0., 0.],
        };
        with = advance(&with, &new)?;
        let mut rows = vec![
            json!({"step":0,"with":probes::observed(&with),"without":probes::observed(&without)}),
        ];
        for e in &common {
            let mut e = e.clone();
            e.ordinal += ordinal + 1;
            e.day += day;
            e.time += u64::from(day) * DAY;
            with = advance(&with, &e)?;
            without = advance(&without, &e)?;
            rows.push(json!({"step":e.ordinal-ordinal-1,"with":probes::observed(&with),"without":probes::observed(&without)}));
        }
        branches.push(json!({"sign":sign,"new_event":new,"rows":rows}));
    }
    Ok(
        json!({"status":"passed","branches":branches,"causal_history_path":"not_identified: report increments only; near matches remain confounded"}),
    )
}
