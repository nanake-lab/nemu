use super::*;
use crate::effects::{SituationMatch, StoredTheta, TendencyEffect, evaluate_effects};
use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::*,
    scalar::{DecimalU64, HexBytes, unhex},
};
mod boundaries;
mod histories;
mod oracle;

// Synthetic algebra fixture only, never a deployment default (§9 / I29).
fn limits() -> Limits {
    Limits {
        max_bytes: 1 << 20,
        max_items: 4096,
        max_depth: 32,
    }
}
fn reference(kind: &str, id: u8) -> ObjectRef {
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(vec![id; 32]),
        object_type: kind.into(),
        resource_version: "synthetic-v1".into(),
        byte_length: DecimalU64(1),
    }
}
fn law_ref() -> ObjectRef {
    reference("echo.law-version", 1)
}
fn law(r: &ObjectRef) -> Law<'_> {
    Law {
        version: r,
        g_max: 1.0,
        r_max: 1.0,
        w_min: 0.1,
    }
}
fn effects() -> EffectiveEffects {
    EffectiveEffects {
        effects: vec![],
        theta: StoredTheta {
            b: [0.0; 4],
            g: [1.0; 4],
            lambda: [std::f64::consts::LN_2; 4],
        },
    }
}
fn state(x: f64) -> State {
    State::from_snapshot(
        FastStateSnapshotV3 {
            schema_version: 1,
            revision: "fixture".into(),
            representation_version: REPRESENTATION_VERSION.into(),
            anchor_time_ns: DecimalU64(0),
            nature: NatureVector {
                schema_version: 1,
                connection: x,
                empathy: x,
                curiosity: x,
                achievement: x,
            },
            learned: vec![],
            slots: vec![],
            law_version: law_ref(),
        },
        &law_ref(),
        limits(),
    )
    .unwrap()
}
fn make_effect(id: &str, v: f64, role: FastEffectRole, prestate: u8) -> Effect {
    let e = effects();
    let evaluation = Evaluation {
        drives: vec![],
        f_sum: [0.0; 4],
    };
    let target = compute_target(&e, &evaluation, Some([v; 4]), 1.0, &law(&law_ref())).unwrap();
    let internal = id.is_empty();
    let r = FastEffectRecord {
        schema_version: 1,
        basis: FastBasisIdentity {
            schema_version: 1,
            kind: if internal {
                FastBasisKind::FAST_BASIS_INTERNAL_BRANCH
            } else {
                FastBasisKind::FAST_BASIS_RECORD
            },
            record_id: if internal { None } else { Some(id.into()) },
        },
        role,
        target: codec::vector(target.drive()),
        theta_before: reference("echo.theta-snapshot-v3", prestate),
        effects_before: reference("fixture.effects", prestate),
        interpretation_refs: vec![reference("fixture.interpretation", prestate)],
        governance_refs: vec![reference("fixture.governance", 1)],
        law_version: law_ref(),
    };
    Effect::new(r, &target, limits()).unwrap()
}
fn new(id: &str, v: f64) -> Effect {
    make_effect(id, v, FastEffectRole::FAST_EFFECT_NEW_BASIS, 1)
}
fn history(effect: &Effect, position: u64) -> HistoryBasis<'_> {
    HistoryBasis {
        first_position: position,
        target: effect,
        reinterpreted: None,
        active: true,
        revisit_governance: &effect.record().governance_refs,
    }
}
fn x(s: &State) -> f64 {
    s.snapshot.nature.connection
}
fn h(s: &State, id: &str) -> f64 {
    s.snapshot
        .slots
        .iter()
        .find(|s| s.basis.record_id.as_deref().unwrap_or("") == id)
        .map_or(0.0, |s| s.current.nature.connection)
}
fn bits(a: f64, b: f64) {
    assert_eq!(a.to_bits(), b.to_bits(), "{a:?} != {b:?}");
}
fn same(a: &State, b: &State) {
    assert_eq!(
        a.canonical_bytes(limits()).unwrap(),
        b.canonical_bytes(limits()).unwrap()
    );
}

#[test]
fn b1_b2_b3_projection_uses_original_phi_and_shared_anchor() {
    let a = new("a", 1.0);
    let b = new("b", 1.0);
    let saturated = state(0.0)
        .apply(&a, limits())
        .unwrap()
        .apply(&b, limits())
        .unwrap();
    // Arbitrary legal slot configurations, including h outside X (B1).
    for outside in [0.0, 2.0, -2.0, f64::MAX, f64::from_bits(1)] {
        let mut snapshot = saturated.snapshot.clone();
        snapshot.slots[0].current.nature.connection = outside;
        let s = State::from_snapshot(snapshot, &law_ref(), limits()).unwrap();
        for baseline in [-0.75, 0.0, 0.375] {
            let mut e = effects();
            e.theta.b = [baseline; 4];
            e.theta.lambda = [0.01, 0.25, 1.0, 4.0];
            for t in [0, 1, 1_000_000_000, 17_000_000_000, u64::MAX] {
                let projected = s.project_time(&e, t).unwrap();
                let old = crate::fast::project_time(&s.visible(), &e, t).unwrap();
                for (a, b) in projected.visible().nature.iter().zip(old.nature) {
                    bits(*a, b);
                }
                if t == 0 {
                    same(&s, &projected);
                }
                let value = h(&projected, "a");
                assert!(value.abs() <= outside.abs());
                assert!(value == 0.0 || value.is_sign_negative() == outside.is_sign_negative());
                same(&projected, &s.project_time(&e, t).unwrap());
            }
        }
    }
    bits(
        x(&saturated.project_time(&effects(), 1_000_000_000).unwrap()),
        0.5,
    );
    // A0 mutant retains both pre-clip units and yields 1 at the half-life.
    assert_ne!(
        0.5_f64.to_bits(),
        (0.5_f64 * 2.0).clamp(-1.0, 1.0).to_bits()
    );
}

#[test]
fn b4_b5_first_effect_matches_old_f_and_repetition_is_bitwise_idempotent() {
    for start in [-1.0, -0.4, 0.0, 0.7, 1.0] {
        for v in [-1.0, -0.125, 0.0, 0.375, 1.0] {
            let effect = new("a", v);
            let s = state(start);
            let next = s.apply(&effect, limits()).unwrap();
            let drive = Drive {
                nature: [v; 4],
                learned: vec![],
            };
            let old = crate::fast::apply_drive(&s.visible(), &drive).unwrap();
            for (a, b) in next.visible().nature.iter().zip(old.nature) {
                bits(*a, b);
            }
            same(&next, &next.apply(&effect, limits()).unwrap());
            same(
                &next,
                &next.revisit(&[history(&effect, 0)], limits()).unwrap().0,
            );
        }
    }
    let (x, h, v) = (
        0.44103831552214,
        -0.07324038289205337,
        0.0002230830984472614,
    );
    let (first, slot) = coordinate(x, h, v).unwrap();
    let (second, slot2) = coordinate(first, slot, v).unwrap();
    bits(first, second);
    bits(slot, slot2);
    let naive_h = h + (first - x);
    assert_ne!(
        first.to_bits(),
        (first + (v - naive_h)).to_bits(),
        "B5 naive mutant must fail"
    );
}

#[test]
fn e09_r4_saturation_revisit_and_target_revision_witnesses() {
    let a = new("a", 0.75);
    let b = new("b", 0.75);
    let s = state(0.0)
        .apply(&a, limits())
        .unwrap()
        .apply(&b, limits())
        .unwrap();
    bits(h(&s, "a"), 0.75);
    bits(h(&s, "b"), 0.25);
    let decayed = s.project_time(&effects(), 1_000_000_000).unwrap();
    let next = decayed.revisit(&[history(&b, 1)], limits()).unwrap().0;
    bits(x(&next), 1.0);
    bits(h(&next, "b"), 0.625);
    assert!(h(&next, "b") > h(&s, "b")); // accepted cross-time increase
    let a = new("a", 1.0);
    let b = new("b", 1.0);
    let s = state(0.0)
        .apply(&a, limits())
        .unwrap()
        .apply(&b, limits())
        .unwrap()
        .project_time(&effects(), 1_000_000_000)
        .unwrap();
    let update = FastEffectRole::FAST_EFFECT_TARGET_UPDATE;
    let s = s
        .apply(&make_effect("a", 0.0, update, 2), limits())
        .unwrap()
        .apply(&make_effect("b", 0.0, update, 2), limits())
        .unwrap();
    bits(x(&s), 0.0);
    let s = state(0.0)
        .apply(&new("a", 1.0), limits())
        .unwrap()
        .apply(&new("b", -1.0), limits())
        .unwrap()
        .apply(&new("c", 1.0), limits())
        .unwrap();
    let revised = s
        .apply(&make_effect("b", 0.0, update, 2), limits())
        .unwrap();
    bits(h(&revised, "b"), -1.0);
    bits(x(&revised), 1.0);
}
