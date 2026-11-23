use crate::ablated;
use core_kernel::{effects::*, fast};

pub fn effects() -> EffectiveEffects {
    compile_authorized_effects(
        &StoredR {
            tendencies: vec![StoredTendency {
                tendency_id: "r0".into(),
                concept: "synthetic-success".into(),
                pi: 0.75,
                r: 0.1,
                d: [0.0, 0.0, 0.0, 1.0],
                direction_validated: false,
            }],
        },
        &StoredTheta {
            b: [0.0; 4],
            g: [1.0; 4],
            lambda: [1.0; 4],
        },
        ProcessTime(0),
        &Validity,
    )
    .unwrap()
}

pub fn law() -> fast::DriveLaw {
    fast::DriveLaw {
        g_max: 1.0,
        r_max: 1.0,
        w_min: 0.1,
        kappa: 0.5,
    }
}

pub fn initial() -> fast::FastState {
    fast::FastState::new(
        [0.0, 0.0, 0.0, 0.25],
        vec![fast::LearnedValue {
            tendency_id: "r0".into(),
            value: 0.0,
        }],
    )
    .unwrap()
}

/// Uses the actual production evaluator, decay and F_out. With release=false,
/// compute_drive/apply_drive are production too. With release=true, use the
/// build-time ONE LINE ablation, with the original source class and legal κ.
pub fn pulse(
    state: &fast::FastState,
    effects: &EffectiveEffects,
    delta_ns: u64,
    input: fast::EventInput,
    mu: f64,
    release: bool,
) -> (fast::FastState, Vec<f64>, Vec<f64>) {
    let projected = fast::project_time(state, effects, delta_ns).unwrap();
    let evaluation = evaluate_effects(
        effects,
        &SituationMatch {
            mu: vec![("r0".into(), mu)],
        },
    )
    .unwrap();
    let (after, drive) = if release {
        use ablated::types as t;
        let l = law();
        let class = match input.class {
            fast::EventClass::NewExogenous => t::EventClass::NewExogenous,
            fast::EventClass::OtherExternal => t::EventClass::OtherExternal,
            fast::EventClass::InternalRecord { n } => t::EventClass::InternalRecord { n },
            fast::EventClass::ActionResult { n_issued } => t::EventClass::ActionResult { n_issued },
        };
        let d = ablated::drive::compute_drive(
            effects,
            &evaluation,
            &t::EventInput {
                appraisal: input.appraisal,
                empathy_weight: input.empathy_weight,
                class,
            },
            &t::DriveLaw {
                g_max: l.g_max,
                r_max: l.r_max,
                w_min: l.w_min,
                kappa: l.kappa,
            },
        )
        .unwrap();
        let x = t::FastState::new(
            projected.nature,
            projected
                .learned
                .iter()
                .map(|v| t::LearnedValue {
                    tendency_id: v.tendency_id.clone(),
                    value: v.value,
                })
                .collect(),
        )
        .unwrap();
        let y = ablated::space::apply_drive(&x, &d).unwrap();
        let drive: Vec<_> = d
            .nature()
            .iter()
            .copied()
            .chain(d.learned().iter().map(|v| v.value))
            .collect();
        (
            fast::FastState::new(
                y.nature,
                y.learned
                    .into_iter()
                    .map(|v| fast::LearnedValue {
                        tendency_id: v.tendency_id,
                        value: v.value,
                    })
                    .collect(),
            )
            .unwrap(),
            drive,
        )
    } else {
        let d = fast::compute_drive(effects, &evaluation, &input, &law()).unwrap();
        let drive = d
            .nature()
            .iter()
            .copied()
            .chain(d.learned().iter().map(|v| v.value))
            .collect();
        (fast::apply_drive(&projected, &d).unwrap(), drive)
    };
    (
        fast::f_out(&after, effects).unwrap(),
        drive,
        vector(&projected),
    )
}

pub fn vector(x: &fast::FastState) -> Vec<f64> {
    x.nature
        .iter()
        .copied()
        .chain(x.learned.iter().map(|v| v.value))
        .collect()
}
