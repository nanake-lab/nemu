use super::{super::*, fixture::*};
use core_types::generated::*;

#[test]
fn r1_excluded_vertices_cannot_bridge_source_conversation_or_same_matter() {
    for kind in [
        EventUse::OwnNarrative,
        EventUse::Knowledge,
        EventUse::Maintenance,
        EventUse::Experience,
    ] {
        for matter in [false, true] {
            let mut h = base();
            let mut f = frame(1, "independent", Some(-0.8));
            matched(&mut f, "k", 1.0, MatchBasis::Reaction);
            h.push(f);
            let before = run(&h).unwrap();
            for (i, target) in [(2, 0), (3, 1)] {
                let mut f = frame(i, "excluded-source", Some(0.8));
                f.attributes.as_mut().unwrap().exogenous = false;
                f.attributes.as_mut().unwrap().self_narrative = kind == EventUse::OwnNarrative;
                f.semantics.as_mut().unwrap().kind = kind;
                if matter {
                    f.same_matters.push(SameMatterAnnotation {
                        schema_version: 1,
                        record_id: format!("same-{i}"),
                        event_id: f.event.id.clone(),
                        same_as: vec![format!("event-{target}")],
                        binding: binding(&f),
                        producer: "synthetic-M".into(),
                        producer_version: "1".into(),
                    });
                } else {
                    let c = format!("conversation-{target}");
                    h[target].semantics.as_mut().unwrap().conversation = Some(c.clone());
                    f.semantics.as_mut().unwrap().conversation = Some(c);
                }
                if kind == EventUse::Experience {
                    // Explicitly tainted; old independent observations precede
                    // the lie, so the I27 exclusion must not taint them backwards.
                    f.lies.push(LieAnnotation {
                        schema_version: 1,
                        record_id: format!("lie-{i}"),
                        event_id: f.event.id.clone(),
                        statement: f.event.id.clone(),
                        contradicts: "event-0".into(),
                        binding: binding(&f),
                        producer: "synthetic-M".into(),
                        producer_version: "1".into(),
                    });
                }
                h.push(f);
            }
            let after = run(&h).unwrap();
            assert_eq!(after.groups, before.groups, "{kind:?}, matter={matter}");
            assert_eq!(after.external, before.external);
            assert!(after.delta(&before).unwrap().groups.is_empty());
        }
    }
}

#[test]
fn r1_real_experience_without_a_still_supplies_conversation_identity() {
    let mut h = base();
    let mut other = frame(1, "second", Some(0.8));
    matched(&mut other, "k", 1.0, MatchBasis::Reaction);
    h.push(other);
    h[0].semantics.as_mut().unwrap().conversation = Some("c0".into());
    h[1].semantics.as_mut().unwrap().conversation = Some("c1".into());
    for (i, c) in [(2, "c0"), (3, "c1")] {
        let mut f = frame(i, "real-shared-source", None);
        f.semantics.as_mut().unwrap().conversation = Some(c.into());
        h.push(f);
    }
    assert_eq!(run(&h).unwrap().groups.len(), 1);
}

#[test]
fn ev5_nonzero_baseline_four_channel_reflection_preserves_matches_and_groups() {
    let b = [0.1, -0.2, 0.3, -0.1];
    let mut seed = 0x455635_u64;
    for _ in 0..32 {
        let mut h = base();
        for i in 1..10 {
            let mut f = frame(i, &format!("source-{}", i % 3), Some(0.0));
            matched(&mut f, "k", 1.0, MatchBasis::Reaction);
            h.push(f);
        }
        for f in &mut h {
            let mut values = [0.0; 4];
            for (i, value) in values.iter_mut().enumerate() {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                *value = b[i] + ((seed >> 32) as u32 as f64 / u32::MAX as f64 - 0.5) * 0.8;
            }
            f.appraisals[0].appraisal = NatureVector {
                schema_version: 1,
                connection: values[0],
                empathy: values[1],
                curiosity: values[2],
                achievement: values[3],
            };
        }
        let before = run(&h).unwrap();
        assert!(!before.groups.is_empty());
        for f in &mut h {
            let a = &mut f.appraisals[0].appraisal;
            a.connection = 2.0 * b[0] - a.connection;
            a.empathy = 2.0 * b[1] - a.empathy;
            a.curiosity = 2.0 * b[2] - a.curiosity;
            a.achievement = 2.0 * b[3] - a.achievement;
        }
        let after = run(&h).unwrap();
        assert_eq!(before.proposal_order, after.proposal_order);
        assert_eq!(before.groups.len(), after.groups.len());
        for (a, b) in before.groups.iter().zip(&after.groups) {
            assert_eq!(
                (
                    &a.group_id,
                    a.position,
                    &a.members,
                    &a.source_key,
                    a.verifiable
                ),
                (
                    &b.group_id,
                    b.position,
                    &b.members,
                    &b.source_key,
                    b.verifiable
                )
            );
            let crate::law::GroupContent::Experience { matches: am, .. } = &a.content else {
                panic!()
            };
            let crate::law::GroupContent::Experience { matches: bm, .. } = &b.content else {
                panic!()
            };
            assert_eq!(am, bm);
        }
    }
}
