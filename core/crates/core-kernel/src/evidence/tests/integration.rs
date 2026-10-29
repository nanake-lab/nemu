use super::{super::*, fixture::*};
use crate::law::{self, *};
use core_types::generated::*;

fn context<'a>(law_version: &'a ObjectRef, relations: &'a RelationConcepts) -> LawContext<'a> {
    // Same valid domain as P5-23's fixture, explicitly synthetic.
    let params = LawParameters {
        schema_version: 1,
        n0: 1.0,
        kappa1: 0.1,
        w_max: 8.0,
        w_teach: 1.0,
        v_u: 0.25,
        r_min: 0.1,
        r_max: 1.0,
        h0: 0.01,
        n_h: 10.0,
        h_hat_min: None,
        h0_theta: 0.1,
        h_hat_theta_min: 0.01,
        n_b: 10.0,
        n_sigma: 10.0,
        g_min: 0.2,
        g_max: 3.0,
        recovery_min_per_s: 0.01,
        recovery_max_per_s: 10.0,
        w_min: 0.1,
        t_x_s: 60.0,
        epsilon_x: 0.01,
        k_max: 0.5,
    };
    LawContext {
        law_version,
        law: Law::new(&params).unwrap(),
        caretaker: Caretaker {
            person_id: "caretaker",
            attachment_prior: 0.0,
        },
        r_capacity: 100,
        protected_candidates: 1,
        relation_concepts: relations,
    }
}

fn consume(h: &[Frame]) -> (law::state::Snapshot, Vec<law::state::EvidenceGroup>) {
    consume_mode(h, false)
}

fn consume_mode(
    h: &[Frame],
    batch: bool,
) -> (law::state::Snapshot, Vec<law::state::EvidenceGroup>) {
    let resource: ObjectRef = golden();
    let relations = RelationConcepts {
        closeness: golden(),
        trust: golden(),
    };
    let ctx = context(&resource, &relations);
    let mut snapshot = law::state::Snapshot {
        schema_version: 1,
        revision: "genesis".into(),
        law_version: resource.clone(),
        tendencies: vec![],
    };
    let mut groups = vec![];
    let sigma = NatureScale {
        schema_version: 1,
        connection: 0.2,
        empathy: 0.2,
        curiosity: 0.2,
        achievement: 0.2,
    };
    let theta = ThetaView::new(&vector(0.0), &sigma).unwrap();
    let mut previous = None;
    let ends: Vec<_> = if batch {
        vec![h.len()]
    } else {
        (1..=h.len()).collect()
    };
    for end in ends {
        let mut all = run(&h[..end]).unwrap();
        if batch {
            // This fixture proposes once at genesis; replay the same proposal
            // with the complete evidence sequence in a single G invocation.
            all.candidates = run(&h[..1]).unwrap().candidates;
        }
        let delta = previous
            .as_ref()
            .map_or_else(|| all.clone(), |p| all.delta(p).unwrap());
        let out = g(
            RState {
                snapshot: &snapshot,
                groups: &groups,
            },
            &theta,
            &delta.step(&format!("commit-{end}")),
            &ctx,
        )
        .unwrap();
        snapshot = out.r;
        // P5-23 returns changed summaries, not a replacement for the whole store.
        for updated in out.groups {
            if let Some(old) = groups
                .iter_mut()
                .find(|old| old.group_id == updated.group_id)
            {
                *old = updated;
            } else {
                groups.push(updated);
            }
        }
        groups.sort_by(|a, b| core_types::semantic::tendency_order(&a.group_id, &b.group_id));
        previous = Some(all);
    }
    (snapshot, groups)
}

#[test]
fn ac2_03_assembled_repeated_experience_does_not_add_g_weight() {
    let h = base();
    let (one, _) = consume(&h);
    let mut many = h;
    for i in 1..17 {
        let mut f = frame(i, "one", Some(0.8));
        matched(&mut f, "k", 1.0, MatchBasis::Reaction);
        many.push(f);
    }
    let (repeated, _) = consume(&many);
    assert_eq!(one.tendencies, repeated.tendencies);
    assert_eq!(repeated.tendencies[0].group_count.0, 1);
}

#[test]
fn ev9_supplied_verification_reaches_g_discount_without_local_guessing() {
    let h = base();
    let (verified, _) = consume(&h);
    let mut h = h;
    h[0].attributes.as_mut().unwrap().verifiable = false;
    let (unverified, _) = consume(&h);
    assert!(unverified.tendencies[0].pi < verified.tendencies[0].pi);
    assert!(unverified.tendencies[0].pi > 0.0);
}

#[test]
fn ac2_12_teaching_uses_g_trust_and_repetition_does_not_refresh() {
    let mut h = base();
    h[0].appraisals.clear();
    h[0].situations.clear();
    teach(&mut h[0], "k", "person", true);
    let (one, groups) = consume(&h);
    assert_eq!(groups[0].credibility, 0.5); // no relationship starts neutral
    for i in 1..10 {
        let mut f = frame(i, "person", None);
        teach(&mut f, "k", "person", true);
        h.push(f);
    }
    assert_eq!(consume(&h).0.tendencies, one.tendencies);
}

#[test]
fn ac2_12_prior_reliance_changes_teaching_weight_and_later_repeats_keep_it() {
    let mut h = base();
    h[0].appraisals[0].appraisal = NatureVector {
        schema_version: 1,
        connection: 0.0,
        empathy: 0.0,
        curiosity: 0.8,
        achievement: 0.8,
    };
    h[0].semantics.as_mut().unwrap().participants = vec!["person".into()];
    h[0].semantics.as_mut().unwrap().relied_on = vec!["person".into()];
    h[0].simultaneous_proposals = Some(vec![
        "k".into(),
        "rel:closeness:person".into(),
        "rel:trust:person".into(),
    ]);
    let before = consume(&h).0;
    let expected = law::trust(&before.tendencies, "person");
    assert!(expected > 0.5);
    let mut teaching = frame(1, "person", None);
    teach(&mut teaching, "k", "person", true);
    h.push(teaching);
    let (first, groups) = consume(&h);
    let taught = groups
        .iter()
        .find(|g| g.group_id.contains("teaching"))
        .unwrap();
    assert_eq!(taught.credibility.to_bits(), expected.to_bits());
    let mut repeat = frame(2, "person", None);
    teach(&mut repeat, "k", "person", true);
    h.push(repeat);
    assert_eq!(consume(&h).0, first);
}

#[test]
fn i23_incremental_matches_single_g_call_on_complete_synthetic_prefix() {
    let mut h = base();
    for i in 1..16 {
        let mut f = frame(
            i,
            &format!("s{i}"),
            Some(if i % 3 == 0 { -0.8 } else { 0.8 }),
        );
        matched(&mut f, "k", 1.0, MatchBasis::Reaction);
        h.push(f);
    }
    for end in 1..=h.len() {
        let a = consume(&h[..end]);
        let b = consume_mode(&h[..end], true);
        assert_eq!(a, b, "prefix {end}");
    }
    println!("I23 incremental vs full-sequence G: 16 synthetic prefixes (not production H replay)");
}

#[test]
fn capacity_deferred_candidate_reenters_g_with_original_historical_evidence() {
    let resource: ObjectRef = golden();
    let relations = RelationConcepts {
        closeness: golden(),
        trust: golden(),
    };
    let mut ctx = context(&resource, &relations);
    ctx.r_capacity = 0;
    let snapshot = law::state::Snapshot {
        schema_version: 1,
        revision: "genesis".into(),
        law_version: resource.clone(),
        tendencies: vec![],
    };
    let sigma = NatureScale {
        schema_version: 1,
        connection: 0.2,
        empathy: 0.2,
        curiosity: 0.2,
        achievement: 0.2,
    };
    let theta = ThetaView::new(&vector(0.0), &sigma).unwrap();
    let mut h = base();
    let old = run(&h).unwrap();
    let rejected = g(
        RState {
            snapshot: &snapshot,
            groups: &[],
        },
        &theta,
        &old.step("reject"),
        &ctx,
    )
    .unwrap();
    assert_eq!(rejected.capacity_notice.unwrap().deferred_candidates, ["k"]);
    assert!(rejected.r.tendencies.is_empty());
    let mut again = frame(1, "capacity-raised", None);
    propose(&mut again, "reproposal-record", "concept");
    h.push(again);
    let next = run(&h).unwrap();
    let delta = next.delta(&old).unwrap();
    assert_eq!(delta.candidates[0].record_id, "k");
    assert_eq!(delta.groups, old.groups);
    ctx.r_capacity = 100;
    let admitted = g(
        RState {
            snapshot: &rejected.r,
            groups: &rejected.groups,
        },
        &theta,
        &delta.step("admit"),
        &ctx,
    )
    .unwrap();
    assert!(admitted.capacity_notice.is_none());
    assert_eq!(admitted.r.tendencies[0].group_count.0, 1);
    assert!(admitted.r.tendencies[0].evidence_absolute > 0.0);
    assert_eq!(admitted.groups[0].members, ["event-0"]);
}
