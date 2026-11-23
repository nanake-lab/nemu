//! Pure diagnostic probes; no resulting state is returned to the life runner.
use crate::{adapter::*, resources::*, world::*};
use core_kernel::{effects::*, fast};
use serde_json::{Value, json};
pub fn probe(s: &State, p: &Probe, time: u64) -> Result<Value> {
    let (before, drive, ev, after) = reaction(s, p, time)?;
    let eff = effects(s)?;
    let mut marginal = vec![];
    for item in &ev.drives {
        if item.f == [0.; 4] && item.drive == 0. {
            continue;
        }
        let mut removed = eff.clone();
        removed
            .effects
            .retain(|e| e.tendency_id != item.tendency_id);
        let evaluation =
            evaluate_effects(&removed, &situation(&p.person, &p.context)).map_err(err)?;
        let input = fast::EventInput {
            appraisal: Some(p.a),
            empathy_weight: if p.person.is_empty() {
                1.
            } else {
                core_kernel::law::closeness_weight(
                    &s.r.tendencies,
                    &p.person,
                    &core_kernel::law::Law::new(&parameters()).map_err(err)?,
                )
            },
            class: fast::EventClass::NewExogenous,
        };
        let no_drive =
            fast::compute_drive(&removed, &evaluation, &input, &drive_law()).map_err(err)?;
        let no_before = fast::f_out(&before, &removed).map_err(err)?;
        let no_after = fast::apply_drive(&no_before, &no_drive).map_err(err)?;
        let nature: Vec<f64> = (0..4)
            .map(|i| after.nature[i] - no_after.nature[i])
            .collect();
        let learned = after
            .learned
            .iter()
            .find(|l| l.tendency_id == item.tendency_id)
            .map(|l| {
                l.value
                    - before
                        .learned
                        .iter()
                        .find(|b| b.tendency_id == l.tendency_id)
                        .unwrap()
                        .value
            })
            .unwrap_or(0.);
        marginal.push(json!({"id":item.tendency_id,"f":item.f,"drive":item.drive,"nature":nature,"learned":learned}));
    }
    Ok(
        json!({"id":p.id,"delta":(0..4).map(|i|after.nature[i]-before.nature[i]).collect::<Vec<_>>(),"drive":drive.nature(),"learned_delta":after.learned.iter().zip(&before.learned).map(|(a,b)|json!([a.tendency_id,a.value-b.value])).collect::<Vec<_>>(),"marginal":marginal}),
    )
}
pub fn observed(s: &State) -> Value {
    let t = s.theta.values();
    json!({"r":s.r.tendencies.iter().map(|r|json!({"id":r.tendency_id,"pi":r.pi,"r":r.typical_magnitude,"h":r.h_hat,"groups":r.group_count,"active":r.pi>0.5})).collect::<Vec<_>>(),"theta":t,"x_nature":s.x.nature,"x_learned":s.x.learned.iter().map(|l|json!([l.tendency_id,l.value])).collect::<Vec<_>>()})
}
pub fn daily(s: &State, day: u32, probes: &[Probe]) -> Result<Value> {
    let mut view = s.clone();
    view.x = fast::project_time(&s.x, &effects(s)?, u64::from(day) * DAY - s.time).map_err(err)?;
    view.time = u64::from(day) * DAY;
    Ok(
        json!({"day":day,"state":observed(&view),"probes":probes.iter().map(|p|probe(s,p,u64::from(day)*DAY)).collect::<Result<Vec<_>>>()?}),
    )
}
