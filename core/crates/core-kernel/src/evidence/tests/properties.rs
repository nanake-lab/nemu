use super::{super::*, fixture::*};
use crate::law::GroupContent;
use core_types::generated::*;
use std::collections::BTreeMap;

#[test]
fn ev5_sign_inversion_preserves_group_set_and_order_for_128_histories() {
    let mut seed = 0x5245_434f_5244_u64;
    for run_id in 0..128 {
        let mut h = base();
        for i in 1..12 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = ((seed >> 32) as u32 as f64 / u32::MAX as f64) * 2.0 - 1.0;
            let mut f = frame(i, &format!("source-{}", i % 4), Some(value));
            matched(
                &mut f,
                "k",
                1.0,
                if i % 3 == 0 {
                    MatchBasis::Outcome
                } else {
                    MatchBasis::Reaction
                },
            );
            if i % 5 == 0 {
                f.semantics.as_mut().unwrap().outcome = Outcome::default();
            }
            h.push(f);
        }
        let before = run(&h).unwrap();
        for f in &mut h {
            for a in &mut f.appraisals {
                a.appraisal.connection = -a.appraisal.connection;
            }
        }
        let after = run(&h).unwrap();
        let signature = |a: &Assembly| {
            a.groups
                .iter()
                .map(|g| (g.group_id.clone(), g.position, g.members.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(signature(&before), signature(&after), "EV5 stream {run_id}");
        assert_eq!(before.proposal_order, after.proposal_order);
    }
    println!("EV5: 128 sign-inverted synthetic histories; group identity/order/membership equal");
}

#[test]
fn adr0014_arbitrary_synonym_labels_do_not_change_groups_or_proposal_order() {
    let original = run(&base()).unwrap();
    for count in [1, 2, 7, 31] {
        let mut h = base();
        let mut aliases = BTreeMap::new();
        for i in 0..count {
            let c = format!("alias-{i}");
            let k = format!("synonym-{i}");
            aliases.insert(c.clone(), "concept".into());
            propose(&mut h[0], &k, &c);
            matched(&mut h[0], &k, 0.25, MatchBasis::Reaction);
        }
        let a = assemble(prefix(&h, &aliases, &BTreeMap::new())).unwrap();
        assert_eq!(a.groups, original.groups);
        assert_eq!(a.proposal_order, original.proposal_order);
        assert_eq!(a.candidates, original.candidates);
    }
}

#[test]
fn ac2_03_multiple_accounts_map_to_one_source_and_one_teaching_group() {
    let mut h = base();
    h[0].appraisals.clear();
    teach(&mut h[0], "k", "account-1", true);
    let mut f = frame(1, "account-2", None);
    teach(&mut f, "k", "account-2", true);
    h.push(f);
    let persons = BTreeMap::from([
        ("account-1".into(), "person".into()),
        ("account-2".into(), "person".into()),
    ]);
    let a = assemble(prefix(&h, &BTreeMap::new(), &persons)).unwrap();
    assert_eq!(a.groups.len(), 1);
    assert!(
        matches!(&a.groups[0].content, GroupContent::Teaching { speaker, .. } if speaker == "person")
    );
    for f in &mut h {
        f.appraisals.push(NatureAppraisalRecord {
            schema_version: 1,
            record_id: format!("a-{}", f.event.id),
            event_id: f.event.id.clone(),
            appraisal: vector(0.5),
            persons: vec![],
            binding: binding(f),
            producer: "A".into(),
            producer_version: "1".into(),
        });
    }
    h[0].attributes.as_mut().unwrap().source_key = "account-1".into();
    matched(&mut h[1], "k", 1.0, MatchBasis::Reaction);
    h[0].simultaneous_proposals = Some(vec!["k".into(), "rel:closeness:person".into()]);
    let a = assemble(prefix(&h, &BTreeMap::new(), &persons)).unwrap();
    assert_eq!(a.groups.len(), 2); // one experience group and one teaching group
}

#[test]
fn ac2_11_comparable_action_variation_required_and_missing_result_cannot_supply_it() {
    let mut h = base();
    h[0].appraisals.clear();
    h[0].situations.clear();
    h[0].event.kind = EventKind::ACTION_REQUEST;
    h[0].attributes.as_mut().unwrap().exogenous = false;
    let mut f = frame(1, "reply-a", Some(0.8));
    f.event.parents.push("event-0".into());
    matched(
        &mut f,
        "k",
        1.0,
        MatchBasis::OwnActionOutcome {
            context: "same-situation".into(),
            variant: Some("A".into()),
            action_event: Some("event-0".into()),
        },
    );
    h.push(f);
    assert!(run(&h).unwrap().groups.is_empty());
    let mut f = frame(2, "no-action", Some(-0.8));
    matched(
        &mut f,
        "k",
        1.0,
        MatchBasis::OwnActionOutcome {
            context: "same-situation".into(),
            variant: None,
            action_event: None,
        },
    );
    h.push(f);
    assert_eq!(run(&h).unwrap().groups.len(), 2);
    h[2].semantics.as_mut().unwrap().outcome = Outcome::default();
    assert!(run(&h).unwrap().groups.is_empty());
    h[2].semantics.as_mut().unwrap().outcome.observed = true;
    h[2].semantics.as_mut().unwrap().match_basis.insert(
        "k".into(),
        MatchBasis::OwnActionOutcome {
            context: "different-situation".into(),
            variant: None,
            action_event: None,
        },
    );
    assert!(run(&h).unwrap().groups.is_empty());
}

#[test]
fn i27_lies_exclude_self_same_conversation_and_cross_conversation_descendants() {
    let mut h = base(); // unrelated pre-lie observation remains evidence
    let mut action = frame(1, "action", None);
    action.event.kind = EventKind::ACTION_REQUEST;
    action.attributes.as_mut().unwrap().exogenous = false;
    action.semantics.as_mut().unwrap().conversation = Some("c".into());
    h.push(action);
    for i in 2..5 {
        let mut f = frame(i, &format!("reply-{i}"), Some(0.8));
        matched(&mut f, "k", 1.0, MatchBasis::Reaction);
        if i == 2 {
            f.semantics.as_mut().unwrap().conversation = Some("c".into());
        } else {
            f.event.parents.push(format!("event-{}", i - 1));
        }
        h.push(f);
    }
    let old = run(&h).unwrap();
    let mut audit = frame(5, "audit", None);
    audit.attributes.as_mut().unwrap().exogenous = false;
    audit.semantics.as_mut().unwrap().kind = EventUse::Maintenance;
    audit.lies.push(LieAnnotation {
        schema_version: 1,
        record_id: "lie".into(),
        event_id: "event-1".into(),
        statement: "event-1".into(),
        contradicts: "event-0".into(),
        binding: binding(&audit),
        producer: "M".into(),
        producer_version: "1".into(),
    });
    h.push(audit);
    let a = run(&h).unwrap();
    assert_eq!(a.groups.len(), 1);
    assert_eq!(a.external.len(), 1);
    assert!(matches!(a.delta(&old), Err(Fault::RecomputeRequired(_))));
}

#[test]
fn ev5_late_match_and_new_candidate_backfill_old_event_without_rejudging() {
    let mut h = vec![frame(0, "old", Some(-0.8))];
    let old = h[0].appraisals[0].clone();
    let mut f = frame(1, "proposal", None);
    propose(&mut f, "k", "concept");
    matched(&mut f, "k", 1.0, MatchBasis::Reaction);
    f.situations[0].event_id = "event-0".into();
    h[0].semantics
        .as_mut()
        .unwrap()
        .match_basis
        .insert("k".into(), MatchBasis::Reaction);
    h.push(f);
    let a = run(&h).unwrap();
    assert_eq!(a.groups[0].position, 1);
    assert_eq!(a.groups[0].members, ["event-0"]);
    assert_eq!(h[0].appraisals[0], old);
}

#[test]
fn capacity_reproposal_uses_same_identity_and_all_history_alias_does_not_refresh() {
    let mut h = base();
    let mut f = frame(1, "s1", None);
    propose(&mut f, "j", "other");
    h.push(f);
    let mut f = frame(2, "s2", None);
    propose(&mut f, "new-k-record", "concept");
    h.push(f);
    let a = run(&h).unwrap();
    assert_eq!(a.proposal_order, ["j", "k"]);
    assert_eq!(a.candidates[0].record_id, "k");
    assert_eq!(a.groups[0].members, ["event-0"]);
    let mut f = frame(3, "s3", None);
    propose(&mut f, "alias-record", "synonym");
    h.push(f);
    let aliases = BTreeMap::from([("synonym".into(), "concept".into())]);
    let a = assemble(prefix(&h, &aliases, &BTreeMap::new())).unwrap();
    assert_eq!(a.proposal_order, ["j", "k"]);
    assert!(a.candidates.is_empty());
}

#[test]
fn external_delta_appends_once_and_requires_recompute_for_late_appraisal() {
    let mut h = base();
    let first = run(&h).unwrap();
    assert!(first.delta(&first).unwrap().external.is_empty());
    h.push(frame(1, "another", Some(0.2)));
    let second = run(&h).unwrap();
    assert_eq!(second.delta(&first).unwrap().external, second.external[1..]);
    let mut changed = second.clone();
    changed.external[0].appraisal = vector(-0.2);
    assert!(matches!(
        changed.delta(&first),
        Err(Fault::RecomputeRequired(_))
    ));
}

#[test]
fn relation_proposal_order_uses_first_actual_reliance() {
    let mut h = base();
    h[0].semantics.as_mut().unwrap().participants = vec!["person".into()];
    h[0].simultaneous_proposals = Some(vec!["k".into(), "rel:closeness:person".into()]);
    let mut middle = frame(1, "middle", None);
    propose(&mut middle, "j", "other");
    h.push(middle);
    let mut later = frame(2, "later", Some(0.5));
    later.semantics.as_mut().unwrap().participants = vec!["person".into()];
    later.semantics.as_mut().unwrap().relied_on = vec!["person".into()];
    h.push(later);
    let a = run(&h).unwrap();
    let trust = crate::law::relation_tendency_id(RelationKind::TRUST, "person");
    let j = a.proposal_order.iter().position(|id| id == "j").unwrap();
    assert!(a.proposal_order.iter().position(|id| id == &trust).unwrap() > j);
}
