use super::*;
use std::collections::BTreeMap;

fn key(r: &ObjectRef) -> Vec<u8> {
    r.canonical_bytes(limits()).unwrap()
}

// ADR 0035 §9(12): synthetic H objects, with actual distinct θ/R+ contents.
// This is a fixture resolver, not production storage or identity governance.
struct SyntheticH {
    theta: BTreeMap<Vec<u8>, StoredTheta>,
    views: BTreeMap<Vec<u8>, Vec<TendencyEffect>>,
    interpretations: BTreeMap<Vec<u8>, (f64, f64)>,
    governance: BTreeMap<Vec<u8>, ()>,
}

impl SyntheticH {
    fn new() -> Self {
        let mut h = Self {
            theta: BTreeMap::new(),
            views: BTreeMap::new(),
            interpretations: BTreeMap::new(),
            governance: BTreeMap::new(),
        };
        for (id, gain) in [(10, 0.125), (11, 0.25), (12, 0.5), (13, 0.75)] {
            let mut theta = effects().theta;
            theta.g = [gain; 4];
            h.theta
                .insert(key(&reference("echo.theta-snapshot-v3", id)), theta);
            h.views.insert(
                key(&reference("fixture.effects", id)),
                vec![TendencyEffect {
                    tendency_id: "k".into(),
                    concept: "c".into(),
                    s: gain,
                    r: 0.5,
                    d: [1.0, 0.0, 0.0, 0.0],
                    direction_validated: true,
                }],
            );
        }
        for (id, a, mu) in [
            (20, 0.5, 0.5),
            (21, 0.5, 0.5),
            (22, 0.5, 0.5),
            (23, 0.75, 0.25),
            (24, -0.25, 0.75),
        ] {
            h.interpretations
                .insert(key(&reference("fixture.interpretation", id)), (a, mu));
        }
        for id in [1, 9] {
            h.governance
                .insert(key(&reference("fixture.governance", id)), ());
        }
        h
    }

    fn target(&self, record: &FastEffectRecord) -> Target {
        let effects = EffectiveEffects {
            theta: self.theta[&key(&record.theta_before)],
            effects: self.views[&key(&record.effects_before)].clone(),
        };
        assert_eq!(record.interpretation_refs.len(), 1);
        let (a, mu) = self.interpretations[&key(&record.interpretation_refs[0])];
        let evaluation = evaluate_effects(
            &effects,
            &SituationMatch {
                mu: vec![("k".into(), mu)],
            },
        )
        .unwrap();
        compute_target(&effects, &evaluation, Some([a; 4]), 1.0, &law(&law_ref())).unwrap()
    }

    fn effect(&self, prestate: u8, interpretation: u8, role: FastEffectRole) -> Effect {
        let mut record = new("a", 0.0).record().clone();
        record.role = role;
        record.theta_before = reference("echo.theta-snapshot-v3", prestate);
        record.effects_before = reference("fixture.effects", prestate);
        record.interpretation_refs = vec![reference("fixture.interpretation", interpretation)];
        let target = self.target(&record);
        record.target = codec::vector(target.drive());
        Effect::new(record, &target, limits()).unwrap()
    }

    fn check(&self, record: &FastEffectRecord, prestate: u8, interpretation: u8, governance: u8) {
        assert_eq!(
            record.theta_before,
            reference("echo.theta-snapshot-v3", prestate)
        );
        assert_eq!(
            record.effects_before,
            reference("fixture.effects", prestate)
        );
        assert_eq!(
            record.interpretation_refs,
            vec![reference("fixture.interpretation", interpretation)]
        );
        assert_eq!(
            record.governance_refs,
            vec![reference("fixture.governance", governance)]
        );
        for r in &record.governance_refs {
            assert!(self.governance.contains_key(&key(r)));
        }
        // Resolve both prestate objects and the interpretation at EVERY step.
        assert_eq!(record.target, codec::vector(self.target(record).drive()));
    }
}

#[test]
fn interpreter_revision_uses_latest_target_prestate_and_next_update_uses_current() {
    let h = SyntheticH::new();
    let update = FastEffectRole::FAST_EFFECT_TARGET_UPDATE;
    let initial = h.effect(10, 20, FastEffectRole::FAST_EFFECT_NEW_BASIS);
    let first = h.effect(11, 21, update);
    let second = h.effect(12, 22, update);
    let view = EffectiveEffects {
        theta: h.theta[&key(&initial.record().theta_before)],
        effects: h.views[&key(&initial.record().effects_before)].clone(),
    };
    let mut s = state(0.0).f_out(&view).unwrap();
    for (record, prestate, interpretation, connection, learned) in [
        (&initial, 10, 20, 0.09375, 0.0625),
        (&first, 11, 21, 0.1875, 0.125),
        (&second, 12, 22, 0.375, 0.25),
    ] {
        h.check(record.record(), prestate, interpretation, 1);
        s = s.apply(record, limits()).unwrap();
        bits(x(&s), connection);
        bits(s.snapshot.learned[0].value, learned);
    }
    let before = s.canonical_bytes(limits()).unwrap();
    let revised = h.effect(12, 23, update);
    h.check(revised.record(), 12, 23, 1);
    // Appending the new interpretation alone has no F transition.
    assert_eq!(s.canonical_bytes(limits()).unwrap(), before);
    let (old, old_receipts) = s.revisit(&[history(&second, 0)], limits()).unwrap();
    same(&s, &old);
    h.check(&old_receipts[0], 12, 22, 1);
    let governance = [reference("fixture.governance", 9)];
    let hist = HistoryBasis {
        first_position: 0,
        target: &second,
        reinterpreted: Some(&revised),
        active: true,
        revisit_governance: &governance,
    };
    let (next, receipts) = s.revisit(&[hist], limits()).unwrap();
    h.check(&receipts[0], 12, 23, 9);
    bits(x(&next), 0.4375);
    bits(next.snapshot.learned[0].value, 0.125);

    // Each guard must reject independently; the other reference stays θ2/R2.
    for wrong in [10, 11] {
        for theta_only in [true, false] {
            let mut record = revised.record().clone();
            if theta_only {
                record.theta_before = reference("echo.theta-snapshot-v3", wrong);
            } else {
                record.effects_before = reference("fixture.effects", wrong);
            }
            let target = h.target(revised.record());
            let mismatched = Effect::new(record, &target, limits()).unwrap();
            let hist = HistoryBasis {
                first_position: 0,
                target: &second,
                reinterpreted: Some(&mismatched),
                active: true,
                revisit_governance: &governance,
            };
            assert!(s.revisit(&[hist], limits()).is_err());
            assert_eq!(s.canonical_bytes(limits()).unwrap(), before);
        }
    }
    let current = h.effect(13, 24, update);
    h.check(current.record(), 13, 24, 1);
    let final_state = next.apply(&current, limits()).unwrap();
    bits(x(&final_state), 0.09375);
    bits(final_state.snapshot.learned[0].value, 0.5625);
    let (recalled, receipts) = final_state
        .revisit(&[history(&current, 0)], limits())
        .unwrap();
    h.check(&receipts[0], 13, 24, 1);
    same(&final_state, &recalled);
}
