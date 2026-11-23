use super::{super::*, fixture::*, integration::context};
use crate::law::{self, *};
use core_types::generated::*;

#[test]
fn r1_ev3_narratives_cannot_bridge_two_independent_experiences() {
    let mut h = base();
    h[0].semantics.as_mut().unwrap().conversation = Some("left".into());
    let mut f = frame(1, "right-source", Some(-0.8));
    f.semantics.as_mut().unwrap().conversation = Some("right".into());
    matched(&mut f, "k", 1.0, MatchBasis::Reaction);
    h.push(f);
    let before = run(&h).unwrap();
    assert_eq!(before.groups.len(), 2);
    for (i, conversation) in [(2, "left"), (3, "right")] {
        let mut f = frame(i, "her-source", Some(0.8));
        f.attributes.as_mut().unwrap().self_narrative = true;
        f.attributes.as_mut().unwrap().exogenous = false;
        f.semantics.as_mut().unwrap().kind = EventUse::OwnNarrative;
        f.semantics.as_mut().unwrap().conversation = Some(conversation.into());
        h.push(f);
    }
    let after = run(&h).unwrap();
    assert_eq!(after.groups, before.groups);
    assert!(after.delta(&before).unwrap().groups.is_empty());
}

fn relation_frame(i: usize, value: f64) -> Frame {
    let mut f = frame(i, &format!("independent-{i}"), Some(value));
    f.semantics.as_mut().unwrap().participants = vec!["person".into()];
    f
}

#[test]
fn r2_rejected_relation_reentry_matches_full_history_g() {
    let resource: ObjectRef = golden();
    let relations = RelationConcepts {
        closeness: golden(),
        trust: golden(),
    };
    let mut ctx = context(&resource, &relations);
    ctx.r_capacity = 0;
    let empty = law::state::Snapshot {
        schema_version: 1,
        revision: "genesis".into(),
        law_version: resource.clone(),
        tendencies: vec![],
    };
    let theta = ThetaView::new(
        &vector(0.0),
        &NatureScale {
            schema_version: 1,
            connection: 0.2,
            empathy: 0.2,
            curiosity: 0.2,
            achievement: 0.2,
        },
    )
    .unwrap();
    let mut h = vec![relation_frame(0, -0.8)];
    let before = run(&h).unwrap();
    let rejected = g(
        RState {
            snapshot: &empty,
            groups: &[],
        },
        &theta,
        &before.step("reject"),
        &ctx,
    )
    .unwrap();
    assert!(rejected.r.tendencies.is_empty());
    h.push(relation_frame(1, 0.8));
    let all = run(&h).unwrap();
    assert!(matches!(
        all.delta(&before),
        Err(Fault::MissingSemantics(_))
    ));
    h[1].structural_admissions = Some(vec!["rel:closeness:person".into()]);
    let all = run(&h).unwrap();
    let delta = all.delta(&before).unwrap();
    ctx.r_capacity = 100;
    let state = RState {
        snapshot: &rejected.r,
        groups: &rejected.groups,
    };
    let actual = g(state, &theta, &delta.step("reenter"), &ctx).unwrap();
    let expected = g(state, &theta, &all.step("reenter"), &ctx).unwrap();
    println!(
        "R2 actual N={} pi={}; full N={} pi={}",
        actual.r.tendencies[0].group_count.0,
        actual.r.tendencies[0].pi,
        expected.r.tendencies[0].group_count.0,
        expected.r.tendencies[0].pi
    );
    assert_eq!(actual.r, expected.r);
    assert_eq!(actual.groups, expected.groups);
}
