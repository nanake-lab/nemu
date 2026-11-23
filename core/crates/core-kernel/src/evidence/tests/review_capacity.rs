use super::{super::*, fixture::*, integration::context};
use crate::law::{self, *};
use core_types::generated::*;

fn ids(person: &str, trust: bool) -> Vec<String> {
    let mut ids = vec![relation_tendency_id(RelationKind::CLOSENESS, person)];
    if trust {
        ids.push(relation_tendency_id(RelationKind::TRUST, person));
    }
    ids
}

fn event(i: usize, person: &str, value: f64, trust: bool) -> Frame {
    let mut f = frame(i, &format!("source-{i}"), Some(value));
    f.semantics.as_mut().unwrap().participants = vec![person.into()];
    if trust {
        f.semantics.as_mut().unwrap().relied_on = vec![person.into()];
        f.appraisals[0].appraisal.curiosity = value;
        f.appraisals[0].appraisal.achievement = value;
    }
    let entries = ids(person, trust);
    f.structural_admissions = Some(entries.clone());
    if entries.len() > 1 {
        f.simultaneous_proposals = Some(entries);
    }
    f
}

fn theta() -> ThetaView {
    ThetaView::new(
        &vector(0.0),
        &NatureScale {
            schema_version: 1,
            connection: 0.2,
            empathy: 0.2,
            curiosity: 0.2,
            achievement: 0.2,
        },
    )
    .unwrap()
}

fn apply(
    r: &mut law::state::Snapshot,
    groups: &mut Vec<law::state::EvidenceGroup>,
    a: &Assembly,
    ctx: &LawContext<'_>,
) -> Output {
    let out = g(
        RState {
            snapshot: r,
            groups,
        },
        &theta(),
        &a.step("synthetic-commit"),
        ctx,
    )
    .unwrap();
    *r = out.r.clone();
    for group in &out.groups {
        if let Some(old) = groups.iter_mut().find(|old| old.group_id == group.group_id) {
            *old = group.clone();
        } else {
            groups.push(group.clone());
        }
    }
    out
}

#[test]
fn r2_closeness_and_trust_eviction_reentry_backfill_and_k_order() {
    for trust in [false, true] {
        let resource: ObjectRef = golden();
        let concepts = RelationConcepts {
            closeness: golden(),
            trust: golden(),
        };
        let mut ctx = context(&resource, &concepts);
        let mut r = law::state::Snapshot {
            schema_version: 1,
            revision: "genesis".into(),
            law_version: resource.clone(),
            tendencies: vec![],
        };
        let mut groups = vec![];
        let mut h = vec![event(0, "person", -0.8, trust)];
        let first = run(&h).unwrap();
        apply(&mut r, &mut groups, &first, &ctx);
        assert_eq!(r.tendencies.len(), ids("person", trust).len());
        let mut rival = frame(1, "rival", None);
        propose(&mut rival, "j", "rival-concept");
        rival.structural_admissions = Some(vec![]);
        h.push(rival);
        let second = run(&h).unwrap();
        ctx.r_capacity = 1;
        let evicted = apply(&mut r, &mut groups, &second.delta(&first).unwrap(), &ctx);
        assert_eq!(
            r.tendencies
                .iter()
                .map(|t| t.tendency_id.as_str())
                .collect::<Vec<_>>(),
            ["j"]
        );
        assert_eq!(evicted.capacity_audits.len(), ids("person", trust).len());
        h.push(event(2, "person", 0.8, trust));
        let all = run(&h).unwrap();
        let delta = all.delta(&second).unwrap();
        let mut expected_order = vec!["j".into()];
        expected_order.extend(ids("person", trust));
        assert_eq!(delta.proposal_order, expected_order);
        let entries = ids("person", trust);
        ctx.r_capacity = entries.len() as u32;
        ctx.protected_candidates = entries.len() as u32;
        let expected = g(
            RState {
                snapshot: &r,
                groups: &groups,
            },
            &theta(),
            &all.step("synthetic-commit"),
            &ctx,
        )
        .unwrap();
        let actual = apply(&mut r, &mut groups, &delta, &ctx);
        assert_eq!(actual, expected);
        assert!(r.tendencies.iter().all(|t| t.group_count.0 == 2));
        assert_eq!(
            protected_candidates(
                &r.tendencies,
                &delta.proposal_order,
                ctx.protected_candidates
            )
            .unwrap(),
            entries.into_iter().collect()
        );
        // Entry order is reconstructed from H on the next ordinary commit too.
        h.push(frame(3, "unrelated", None));
        assert_eq!(run(&h).unwrap().proposal_order, expected_order);
    }
}

#[test]
fn r2_trust_rejection_then_reentry_uses_both_signed_groups() {
    let resource: ObjectRef = golden();
    let concepts = RelationConcepts {
        closeness: golden(),
        trust: golden(),
    };
    let mut ctx = context(&resource, &concepts);
    ctx.r_capacity = 0;
    let mut r = law::state::Snapshot {
        schema_version: 1,
        revision: "genesis".into(),
        law_version: resource.clone(),
        tendencies: vec![],
    };
    let mut groups = vec![];
    let mut h = vec![event(0, "person", -0.8, true)];
    let old = run(&h).unwrap();
    apply(&mut r, &mut groups, &old, &ctx);
    assert!(r.tendencies.is_empty());
    h.push(event(1, "person", 0.8, true));
    let all = run(&h).unwrap();
    ctx.r_capacity = 100;
    let expected = g(
        RState {
            snapshot: &r,
            groups: &groups,
        },
        &theta(),
        &all.step("synthetic-commit"),
        &ctx,
    )
    .unwrap();
    let actual = apply(&mut r, &mut groups, &all.delta(&old).unwrap(), &ctx);
    assert_eq!(actual, expected);
    assert!(r.tendencies.iter().all(|t| t.group_count.0 == 2));
}

#[test]
fn r2_history_does_not_readmit_unrequested_relations_and_order_cannot_be_guessed() {
    let mut h = vec![event(0, "person", -0.8, true)];
    let old = run(&h).unwrap();
    let mut again = event(1, "person", 0.8, true);
    again.simultaneous_proposals = None;
    h.push(again);
    assert!(matches!(run(&h), Err(Fault::Ambiguous(_))));
    h[1].structural_admissions = Some(vec!["rel:closeness:person".into()]);
    let all = run(&h).unwrap();
    let delta = all.delta(&old).unwrap();
    assert_eq!(delta.groups.len(), 2);
    for group in &delta.groups {
        let GroupContent::Experience { matches, .. } = &group.content else {
            panic!()
        };
        assert!(
            matches.iter().any(
                |m| matches!(&m.target, MatchTarget::Tendency(id) if id == "rel:trust:person")
            )
        );
        assert!(!matches.iter().any(|m| matches!(
            m.target,
            MatchTarget::Relation {
                kind: RelationKind::TRUST,
                ..
            }
        )));
    }
    let resource: ObjectRef = golden();
    let concepts = RelationConcepts {
        closeness: golden(),
        trust: golden(),
    };
    let ctx = context(&resource, &concepts);
    let mut r = law::state::Snapshot {
        schema_version: 1,
        revision: "empty".into(),
        law_version: resource.clone(),
        tendencies: vec![],
    };
    apply(&mut r, &mut vec![], &delta, &ctx);
    assert_eq!(r.tendencies.len(), 1);
    assert_eq!(r.tendencies[0].group_count.0, 2);
    h[1].structural_admissions = Some(vec!["not-a-relation".into()]);
    assert!(matches!(run(&h), Err(Fault::InvalidRecord(_))));
    h[1].structural_admissions = Some(vec!["rel:closeness:person".into(); 2]);
    assert!(matches!(run(&h), Err(Fault::InvalidRecord(_))));
}

#[test]
fn r2_retained_relation_ordinary_update_does_not_refresh_k_order() {
    let resource: ObjectRef = golden();
    let concepts = RelationConcepts {
        closeness: golden(),
        trust: golden(),
    };
    let ctx = context(&resource, &concepts);
    let mut r = law::state::Snapshot {
        schema_version: 1,
        revision: "empty".into(),
        law_version: resource.clone(),
        tendencies: vec![],
    };
    let mut groups = vec![];
    let mut h = vec![event(0, "person", -0.8, false)];
    let first = run(&h).unwrap();
    apply(&mut r, &mut groups, &first, &ctx);
    let mut rival = frame(1, "rival", None);
    propose(&mut rival, "j", "rival-concept");
    rival.structural_admissions = Some(vec![]);
    h.push(rival);
    let before = run(&h).unwrap();
    apply(&mut r, &mut groups, &before.delta(&first).unwrap(), &ctx);
    let mut ordinary = event(2, "person", 0.8, false);
    ordinary.structural_admissions = Some(vec![]);
    h.push(ordinary);
    let all = run(&h).unwrap();
    let delta = all.delta(&before).unwrap();
    assert_eq!(delta.proposal_order, before.proposal_order);
    assert_eq!(delta.groups.len(), 1);
    let expected = g(
        RState {
            snapshot: &r,
            groups: &groups,
        },
        &theta(),
        &all.step("synthetic-commit"),
        &ctx,
    )
    .unwrap();
    assert_eq!(apply(&mut r, &mut groups, &delta, &ctx), expected);
}

#[test]
fn r2_future_relation_cannot_justify_an_earlier_admission() {
    let mut h = vec![frame(0, "empty", None), event(1, "person", -0.8, false)];
    h[0].structural_admissions = Some(vec!["rel:closeness:person".into()]);
    assert!(matches!(run(&h), Err(Fault::InvalidRecord(_))));
}
