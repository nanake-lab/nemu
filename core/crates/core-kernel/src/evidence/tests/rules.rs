use super::{super::*, fixture::*};
use crate::law::GroupContent;
use core_types::generated::*;
use std::collections::BTreeMap;

#[test]
fn ev1_missing_attributes_semantics_binding_and_future_parents_fail_closed() {
    let original = base();
    let mut h = original.clone();
    h[0].attributes = None;
    assert!(matches!(run(&h), Err(Fault::MissingAttributes(_))));
    let mut h = original.clone();
    h[0].semantics = None;
    assert!(matches!(run(&h), Err(Fault::MissingSemantics(_))));
    let mut h = original.clone();
    h[0].appraisals[0].binding.event_id = "uncommitted".into();
    assert!(matches!(run(&h), Err(Fault::InvalidRecord(_))));
    let mut h = original.clone();
    h[0].event.parents.push("future".into());
    assert!(matches!(run(&h), Err(Fault::MissingReference(_))));
    let mut h = original;
    h.push(h[0].clone());
    assert!(matches!(run(&h), Err(Fault::HistoryOrder)));
}

#[test]
fn ev1_schema_nonfinite_and_unknown_match_are_rejected() {
    let original = base();
    let mut h = original.clone();
    h[0].appraisals[0].appraisal.connection = f64::NAN;
    assert!(run(&h).is_err());
    let mut h = original.clone();
    h[0].attributes.as_mut().unwrap().schema_version = 2;
    assert!(run(&h).is_err());
    let mut h = original;
    h[0].situations[0].matches[0].tendency_id = "unknown".into();
    assert!(matches!(run(&h), Err(Fault::MissingReference(_))));
}

#[test]
fn ac2_10_i24_narratives_and_candidates_alone_create_no_evidence() {
    let mut h = base();
    h[0].attributes.as_mut().unwrap().self_narrative = true;
    h[0].attributes.as_mut().unwrap().exogenous = false;
    h[0].semantics.as_mut().unwrap().kind = EventUse::OwnNarrative;
    let a = run(&h).unwrap();
    assert!(a.groups.is_empty());
    assert!(a.external.is_empty());
    assert_eq!(a.candidates.len(), 1);
    let mut h = base();
    h[0].appraisals.clear();
    let a = run(&h).unwrap();
    assert!(a.groups.is_empty());
    assert!(a.external.is_empty());
    assert_eq!(a.candidates.len(), 1);
}

#[test]
fn ev8_read_content_is_not_evidence_but_reading_reaction_is() {
    let mut h = base();
    h[0].semantics
        .as_mut()
        .unwrap()
        .match_basis
        .insert("k".into(), MatchBasis::ReadContent);
    let a = run(&h).unwrap();
    assert!(a.groups.is_empty());
    assert_eq!(a.external.len(), 1);
    h[0].semantics
        .as_mut()
        .unwrap()
        .match_basis
        .insert("k".into(), MatchBasis::Reaction);
    assert_eq!(run(&h).unwrap().groups.len(), 1);
    h[0].semantics.as_mut().unwrap().kind = EventUse::Knowledge;
    assert!(run(&h).unwrap().groups.is_empty());
    assert!(run(&h).unwrap().external.is_empty());
}

#[test]
fn adr0014_unknown_excludes_only_result_basis_zero_is_not_unknown() {
    let mut h = base();
    h[0].semantics
        .as_mut()
        .unwrap()
        .match_basis
        .insert("k".into(), MatchBasis::Outcome);
    for action in [Some(ActionStatus::UNKNOWN_ACTION), None] {
        h[0].semantics.as_mut().unwrap().outcome = Outcome {
            action,
            ..Outcome::default()
        };
        assert!(run(&h).unwrap().groups.is_empty());
        assert_eq!(run(&h).unwrap().external.len(), 1); // observed a(e) is retained
    }
    h[0].semantics.as_mut().unwrap().outcome = Outcome {
        observed: true,
        delivery: Some(DeliveryStatus::UNKNOWN_DELIVERY),
        action: None,
    };
    assert!(run(&h).unwrap().groups.is_empty());
    h[0].semantics.as_mut().unwrap().outcome = Outcome {
        observed: true,
        ..Outcome::default()
    };
    h[0].appraisals[0].appraisal = vector(0.0);
    assert_eq!(run(&h).unwrap().groups.len(), 1);
    h[0].semantics.as_mut().unwrap().outcome = Outcome::default();
    h[0].semantics
        .as_mut()
        .unwrap()
        .match_basis
        .insert("k".into(), MatchBasis::Reaction);
    assert_eq!(run(&h).unwrap().groups.len(), 1);
}

#[test]
fn adr0014_late_known_result_is_a_new_event() {
    let mut h = base();
    h[0].semantics.as_mut().unwrap().outcome = Outcome::default();
    h[0].semantics
        .as_mut()
        .unwrap()
        .match_basis
        .insert("k".into(), MatchBasis::Outcome);
    let old = h[0].clone();
    let mut late = frame(1, "second", Some(-0.8));
    late.event.parents.push(h[0].event.id.clone());
    matched(&mut late, "k", 1.0, MatchBasis::Outcome);
    h.push(late);
    let a = run(&h).unwrap();
    assert_eq!(a.groups[0].members, ["event-1"]);
    assert_eq!(h[0], old);
}

#[test]
fn ac2_03_same_source_conversation_and_transitive_matter_form_one_group() {
    let mut h = base();
    for i in 1..4 {
        let mut f = frame(i, &format!("s-{i}"), Some(0.8));
        matched(&mut f, "k", 1.0, MatchBasis::Reaction);
        h.push(f);
    }
    h[1].attributes.as_mut().unwrap().source_key = "one".into();
    h[1].semantics.as_mut().unwrap().conversation = Some("conversation".into());
    h[2].semantics.as_mut().unwrap().conversation = Some("conversation".into());
    let same_binding = binding(&h[3]);
    h[3].same_matters.push(SameMatterAnnotation {
        schema_version: 1,
        record_id: "same".into(),
        event_id: "event-3".into(),
        same_as: vec!["event-2".into()],
        binding: same_binding,
        producer: "M".into(),
        producer_version: "1".into(),
    });
    let a = run(&h).unwrap();
    assert_eq!(a.groups.len(), 1);
    assert_eq!(a.groups[0].members.len(), 4);
    assert_eq!(a.groups[0].position, 1);
}

#[test]
fn application_session_does_not_merge_unrelated_conversations() {
    let mut h = base();
    let mut f = frame(1, "other", Some(0.8));
    matched(&mut f, "k", 1.0, MatchBasis::Reaction);
    h.push(f);
    assert_eq!(run(&h).unwrap().groups.len(), 2);
}

#[test]
fn ev7_repetition_keeps_first_position_and_verification() {
    let mut h = base();
    h[0].appraisals.clear();
    teach(&mut h[0], "k", "person", true);
    let first = run(&h).unwrap().groups;
    for i in 1..10 {
        let mut f = frame(i, "person", None);
        teach(&mut f, "k", "person", true);
        f.attributes.as_mut().unwrap().verifiable = false;
        h.push(f);
    }
    assert_eq!(run(&h).unwrap().groups, first);
}

#[test]
fn adr0014_self_report_requests_facts_and_quotation_do_not_teach() {
    for utterance in [
        Utterance::SelfReport,
        Utterance::Request,
        Utterance::Fact,
        Utterance::Other,
    ] {
        let mut h = base();
        h[0].appraisals.clear();
        teach(&mut h[0], "k", "person", true);
        h[0].semantics.as_mut().unwrap().utterance = utterance;
        assert!(run(&h).unwrap().groups.is_empty());
    }
    for epistemic in [
        EpistemicType::USER_SELF_REPORT,
        EpistemicType::QUOTATION,
        EpistemicType::MODEL_INFERENCE,
    ] {
        let mut h = base();
        h[0].appraisals.clear();
        teach(&mut h[0], "k", "person", true);
        h[0].semantics.as_mut().unwrap().epistemic = epistemic;
        assert!(run(&h).unwrap().groups.is_empty());
    }
}

#[test]
fn forged_teaching_speaker_and_alias_cycles_are_rejected() {
    let mut h = base();
    teach(&mut h[0], "k", "person", true);
    h[0].teachings[0].speaker = "forged".into();
    assert!(run(&h).is_err());
    let aliases = BTreeMap::from([("a".into(), "b".into()), ("b".into(), "a".into())]);
    assert!(matches!(
        assemble(prefix(&base(), &aliases, &BTreeMap::new())),
        Err(Fault::AliasCycle)
    ));
}

#[test]
fn ev9_mixed_group_properties_and_mu_are_explicit_gaps() {
    let mut h = base();
    let mut f = frame(1, "one", Some(0.8));
    matched(&mut f, "k", 1.0, MatchBasis::Reaction);
    h.push(f);
    h[1].attributes.as_mut().unwrap().verifiable = false;
    assert!(matches!(run(&h), Err(Fault::Ambiguous(_))));
    h[1].attributes.as_mut().unwrap().verifiable = true;
    h[1].situations[0].matches[0].mu = 0.5;
    assert!(matches!(run(&h), Err(Fault::Ambiguous(_))));
}

#[test]
fn unknown_outcome_cannot_leak_via_another_tendencys_group_mean() {
    let mut h = base();
    propose(&mut h[0], "j", "another-concept");
    matched(&mut h[0], "j", 1.0, MatchBasis::Reaction);
    h[0].semantics
        .as_mut()
        .unwrap()
        .match_basis
        .insert("k".into(), MatchBasis::Outcome);
    h[0].simultaneous_proposals = Some(vec!["k".into(), "j".into()]);
    let mut second = frame(1, "one", Some(-0.8));
    matched(&mut second, "k", 1.0, MatchBasis::Outcome);
    matched(&mut second, "j", 1.0, MatchBasis::Reaction);
    second.semantics.as_mut().unwrap().outcome = Outcome::default();
    h.push(second);
    assert!(matches!(run(&h), Err(Fault::Ambiguous(_))));
    h[1].semantics.as_mut().unwrap().outcome.observed = true;
    assert_eq!(run(&h).unwrap().groups.len(), 1);
}

#[test]
fn external_sequence_excludes_self_tool_maintenance_and_a_degraded() {
    let mut h = base();
    for i in 1..5 {
        h.push(frame(i, &format!("s-{i}"), Some(0.5)));
    }
    h[1].attributes.as_mut().unwrap().exogenous = false; // own action
    h[2].attributes.as_mut().unwrap().exogenous = false; // triggered program
    h[3].attributes.as_mut().unwrap().exogenous = false;
    h[3].semantics.as_mut().unwrap().kind = EventUse::Maintenance;
    h[4].appraisals.clear();
    let reply = frame(5, "reply", Some(-0.3));
    h.push(reply);
    assert_eq!(
        run(&h)
            .unwrap()
            .external
            .iter()
            .map(|e| e.event_id.as_str())
            .collect::<Vec<_>>(),
        ["event-0", "event-5"]
    );
}

#[test]
fn relations_use_participants_and_reliance_not_appraisal_sign() {
    let mut h = base();
    h[0].situations.clear();
    let sem = h[0].semantics.as_mut().unwrap();
    sem.participants = vec!["a".into(), "b".into()];
    sem.relied_on = vec!["b".into()];
    assert!(matches!(run(&h), Err(Fault::Ambiguous(_))));
    h[0].simultaneous_proposals = Some(vec![
        "k".into(),
        "rel:closeness:a".into(),
        "rel:closeness:b".into(),
        "rel:trust:b".into(),
    ]);
    let a = run(&h).unwrap();
    let GroupContent::Experience { matches, .. } = &a.groups[0].content else {
        panic!()
    };
    assert_eq!(matches.len(), 3);
    assert_eq!(a.proposal_order.len(), 4);
}
