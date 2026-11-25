//! Synthetic transaction/prefix tests for D I §4.4, SYS04 and F §16.16.
use super::*;
use crate::{
    objects::object_ref,
    registry::{
        ApprovedRegistry, CommittedRegistryPrefix, RegistryTransaction, StagedRegistry,
        canonical_concept, committed_concept, provisional_tendency,
    },
};
use ConceptLifecycle::*;

impl RegistryTransaction for World {
    fn approved_registry(&self, stage: &StageAdoption) -> contract::Result<ApprovedRegistry<'_>> {
        let (authorized, successor, new_ids) =
            self.registry_approval.as_ref().ok_or("no approval")?;
        if authorized != stage {
            return Err("different adoption".into());
        }
        Ok(ApprovedRegistry { successor, new_ids })
    }
}

impl CommittedRegistryPrefix for World {
    fn published_registry(&self, pinned: &ObjectRef) -> contract::Result<Option<&[u8]>> {
        Ok(self
            .published_registries
            .iter()
            .find(|(r, _)| r == pinned)
            .map(|(_, b)| b.as_slice()))
    }
}

fn successor(w: &World, version: &str) -> ConceptRegistry {
    ConceptRegistry {
        version: version.into(),
        previous: Some(w.binding.environment.registry.clone()),
        ..w.registry.clone()
    }
}

fn id(version: &str, name: &str) -> ConceptRef {
    ConceptRef {
        schema_version: 1,
        concept_id: name.into(),
        registry_version: version.into(),
    }
}

fn provisional(w: &World, name: &str) -> ConceptRevision {
    ConceptRevision {
        concept_id: name.into(),
        lifecycle: CONCEPT_LIFE_PROVISIONAL,
        label: None,
        definition: None,
        grounding: vec![],
        roles: vec![],
        allows_reported_text: false,
        ..w.registry.entries[0].clone()
    }
}

fn stage(w: &mut World, after: &ConceptRegistry, names: &[&str]) -> crate::Result<StagedRegistry> {
    let staged = w.run(&w.candidate())?;
    w.registry_approval = Some((
        staged.adoption.clone(),
        object_ref(after, &after.version, limits())?,
        names.iter().map(|n| n.to_string()).collect(),
    ));
    staged.stage_registry(w, &w.registry, after, limits())
}

// Advance the synthetic committed prefix and pin its new resources for the next dispatch.
fn pin(w: &mut World, registry: ConceptRegistry) {
    let pinned = object_ref(&registry, &registry.version, limits()).unwrap();
    w.graph.statements[0]
        .predicate
        .registry_version
        .clone_from(&registry.version);
    for e in &mut w.adapter.entries {
        e.predicate.registry_version.clone_from(&registry.version);
    }
    w.registry = registry;
    w.adapter.registry = pinned.clone();
    w.binding.environment.registry = pinned.clone();
    w.graph.registry = pinned;
    w.mapping = reference(&w.adapter);
    w.binding.read_set = vec![w.mapping.clone(), w.context.clone()];
}

#[test]
fn provisional_ids_and_tendency_text_are_staged_with_the_exact_first_adoption() {
    let mut w = World::new();
    let mut next = successor(&w, "r2");
    let candidate: TendencyCandidateRecord = fixture();
    let tendency = provisional_tendency(
        "opaque-2".into(),
        candidate.concept.clone(),
        w.binding.environment.config.clone(),
        vec![w.graph.statements[0].reported_text.clone()],
    );
    assert_eq!(tendency.label.as_ref(), Some(&candidate.concept.name));
    assert_eq!(
        tendency.definition.as_ref(),
        Some(&candidate.concept.definition)
    );
    assert_eq!(
        tendency.positive_examples,
        candidate.concept.positive_examples
    );
    assert_eq!(
        tendency.negative_examples,
        candidate.concept.negative_examples
    );
    next.entries.extend([tendency, provisional(&w, "opaque-1")]);
    let old = w.registry.clone();
    let staged = stage(&mut w, &next, &["opaque-1", "opaque-2"]).unwrap();
    let adoption = &staged.irl.adoption;
    assert_eq!(adoption.registry_before, w.binding.environment.registry);
    assert_eq!(
        adoption.registry_after,
        Some(object_ref(&next, "r2", limits()).unwrap())
    );
    assert_eq!(adoption.staged_revisions.len(), 2);
    for e in staged
        .registry
        .entries
        .iter()
        .filter(|e| e.concept_id != "report")
    {
        assert!(
            adoption
                .staged_revisions
                .contains(&object_ref(e, "r2", limits()).unwrap())
        );
    }
    assert_eq!(old, w.registry);
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Err(Fault::Binding)
    );
    let pinned = adoption.registry_after.clone().unwrap();
    let query = id("r2", "opaque-1");
    assert_eq!(
        committed_concept(&w, &pinned, &query, limits()),
        Err(Fault::Binding)
    );
    w.transaction = None;
    assert_eq!(committed_concept(&w, &pinned, &query, limits()), Ok(None));
    // rollback: no manifest reaches the committed prefix
    assert_eq!(committed_irl(&w, &w.turn, "tx", limits()), Ok(None));
    let IrlProduct::Graph(graph) = staged.irl.product else {
        panic!()
    };
    assert_eq!(graph.registry, reference(&old));
    w.publish(staged.irl.adoption, &graph);
    w.published_registries.push((
        pinned.clone(),
        staged.registry.canonical_bytes(limits()).unwrap(),
    ));
    assert_eq!(
        committed_concept(&w, &pinned, &query, limits()),
        Ok(Some(query.clone()))
    );
    w.published_registries[0].1 = old.canonical_bytes(limits()).unwrap();
    assert!(committed_concept(&w, &pinned, &query, limits()).is_err());
    assert_eq!(
        committed_irl(&w, &w.turn, "tx", limits()),
        Ok(Some(CommittedIrl::Graph(graph)))
    );
}

#[test]
fn new_ids_require_core_allocation_and_the_actual_transaction_and_adoption() {
    let mut w = World::new();
    let mut next = successor(&w, "r2");
    next.entries.push(provisional(&w, "allocated"));
    assert!(stage(&mut w, &next, &["worker-picked"]).is_err());
    assert!(stage(&mut w, &next, &["allocated", "allocated"]).is_err());
    let staged = w.run(&w.candidate()).unwrap();
    w.registry_approval = Some((
        staged.adoption.clone(),
        object_ref(&next, "r2", limits()).unwrap(),
        vec!["allocated".into()],
    ));
    w.transaction = Some("different transaction");
    assert!(
        staged
            .stage_registry(&w, &w.registry, &next, limits())
            .is_err()
    );
    w.transaction = Some("tx");
    let staged = w.run(&w.candidate()).unwrap();
    let mut substituted = next.clone();
    substituted.entries[1]
        .label
        .clone_from(&w.registry.entries[0].label);
    assert!(
        staged
            .stage_registry(&w, &w.registry, &substituted, limits())
            .is_err()
    );
    w.graph.references[0].source.epistemic_type = EpistemicType::USER_SELF_REPORT;
    assert!(stage(&mut w, &next, &["allocated"]).is_err());
}

#[test]
fn revision_chain_preserves_identity_through_promotion_rename_and_deprecation() {
    let mut w = World::new();
    let mut first = successor(&w, "r2");
    first.entries.push(provisional(&w, "fixed-id"));
    let first = stage(&mut w, &first, &["fixed-id"]).unwrap();
    pin(&mut w, first.registry);
    let mut stable = successor(&w, "r3");
    let entry = stable
        .entries
        .iter_mut()
        .find(|e| e.concept_id == "fixed-id")
        .unwrap();
    entry.previous = Some(object_ref(entry, "r2", limits()).unwrap());
    entry.lifecycle = CONCEPT_LIFE_STABLE;
    assert!(stage(&mut w, &stable, &[]).is_err()); // no definition or grounding
    let entry = stable
        .entries
        .iter_mut()
        .find(|e| e.concept_id == "fixed-id")
        .unwrap();
    let text = TrilingualText {
        schema_version: 1,
        zh: "名字".into(),
        en: "name".into(),
        ja: "名前".into(),
    };
    entry.label = Some(text.clone());
    entry.definition = Some(text.clone());
    entry.grounding = vec![w.graph.statements[0].reported_text.clone()];
    // A revision cannot point into its own version or exchange predecessors with a different ID.
    let mut same_version = stable.clone();
    let revised = same_version
        .entries
        .iter_mut()
        .find(|e| e.concept_id == "fixed-id")
        .unwrap();
    revised.previous = Some(object_ref(revised, "r3", limits()).unwrap());
    assert!(stage(&mut w, &same_version, &[]).is_err());
    let mut crossed = stable.clone();
    let old_report = w
        .registry
        .entries
        .iter()
        .find(|e| e.concept_id == "report")
        .unwrap();
    let old_fixed = w
        .registry
        .entries
        .iter()
        .find(|e| e.concept_id == "fixed-id")
        .unwrap();
    for e in &mut crossed.entries {
        let other = if e.concept_id == "report" {
            old_fixed
        } else {
            old_report
        };
        e.previous = Some(object_ref(other, "r2", limits()).unwrap());
    }
    assert!(stage(&mut w, &crossed, &[]).is_err());
    let stable = stage(&mut w, &stable, &[]).unwrap();
    let former = stable.irl.adoption.staged_revisions[0].clone();
    pin(&mut w, stable.registry);
    let old_ref = id("r3", "fixed-id");
    let old_registry = w.registry.clone();
    let old_resource = w.binding.environment.registry.clone();
    let mut retired = successor(&w, "r4");
    let current = retired
        .entries
        .iter_mut()
        .find(|e| e.concept_id == "fixed-id")
        .unwrap();
    current.lifecycle = CONCEPT_LIFE_DEPRECATED;
    current.label.as_mut().unwrap().en = "renamed".into();
    assert!(stage(&mut w, &retired, &[]).is_err()); // stale predecessor
    retired
        .entries
        .iter_mut()
        .find(|e| e.concept_id == "fixed-id")
        .unwrap()
        .previous = Some(former);
    let retired = stage(&mut w, &retired, &[]).unwrap();
    pin(&mut w, retired.registry);
    assert_eq!(
        canonical_concept(&old_registry, &old_resource, &old_ref, limits()).unwrap(),
        old_ref
    );
    let mut resurrection = successor(&w, "r5");
    let current = resurrection
        .entries
        .iter_mut()
        .find(|e| e.concept_id == "fixed-id")
        .unwrap();
    current.previous = Some(object_ref(current, "r4", limits()).unwrap());
    current.lifecycle = CONCEPT_LIFE_STABLE;
    assert!(stage(&mut w, &resurrection, &[]).is_err());
}

#[test]
fn malicious_revision_and_fabricated_grounding_are_rejected_as_a_unit() {
    let mut w = World::new();
    let mut good = successor(&w, "r2");
    good.entries.push(provisional(&w, "new"));
    let mut cases = Vec::new();
    for change in [
        |r: &mut ConceptRegistry| {
            r.entries.remove(0);
        },
        |r: &mut ConceptRegistry| r.entries[1].concept_id = r.entries[0].concept_id.clone(),
        |r: &mut ConceptRegistry| r.entries[1].previous = Some(reference(&r.entries[0])),
        |r: &mut ConceptRegistry| r.entries[1].lifecycle = CONCEPT_LIFE_DEPRECATED,
        |r: &mut ConceptRegistry| r.previous.as_mut().unwrap().digest.0[0] ^= 1,
        |r: &mut ConceptRegistry| {
            r.version
                .clone_from(&r.previous.as_ref().unwrap().resource_version)
        },
        |r: &mut ConceptRegistry| {
            let mut loc = r.entries[0].grounding[0].clone();
            loc.source.record_id = "invented".into();
            r.entries[1].grounding.push(loc);
        },
        |r: &mut ConceptRegistry| {
            r.entries[1].label = r.entries[0].label.clone();
            r.entries[1].label.as_mut().unwrap().ja.clear();
        },
    ] {
        let mut broken = good.clone();
        change(&mut broken);
        cases.push(broken);
    }
    for broken in cases {
        assert!(stage(&mut w, &broken, &["new"]).is_err());
    }
    assert_eq!(w.registry.entries.len(), 1);
}

#[test]
fn directed_aliases_resolve_in_the_pinned_version_and_reject_cycles_conflicts_and_dangling_ends() {
    let mut w = World::new();
    let mut r = successor(&w, "r2");
    r.entries
        .extend([provisional(&w, "a"), provisional(&w, "b")]);
    let link = |a, b| ConceptLink {
        schema_version: 1,
        origin: id("r2", a),
        target: id("r2", b),
        kind: "canonical".into(),
        basis: w.context.clone(),
    };
    let invalid = [
        link("report", "a"),
        link("a", "report"),
        link("report", "missing"),
        link("report", "report"),
    ];
    r.aliases = vec![link("a", "b"), link("b", "report")];
    for bad in invalid {
        let mut wrong = r.clone();
        wrong.aliases.push(bad);
        assert!(stage(&mut w, &wrong, &["a", "b"]).is_err());
    }
    let mut cross_version = r.clone();
    cross_version.aliases[0].target.registry_version = "synthetic-v1".into();
    assert!(stage(&mut w, &cross_version, &["a", "b"]).is_err());
    let a = stage(&mut w, &r, &["a", "b"]).unwrap();
    r.aliases.reverse();
    r.entries.reverse();
    let b = stage(&mut w, &r, &["b", "a"]).unwrap();
    assert_eq!(a.irl.adoption, b.irl.adoption);
    let pinned = a.irl.adoption.registry_after.unwrap();
    assert_eq!(
        canonical_concept(&r, &pinned, &id("r2", "a"), limits()).unwrap(),
        id("r2", "report")
    );
    assert!(canonical_concept(&r, &pinned, &id("synthetic-v1", "a"), limits()).is_err());
}

#[test]
fn alias_of_an_existing_concept_is_published_with_its_governed_revision() {
    let mut w = World::new();
    let mut first = successor(&w, "r2");
    first.entries.push(provisional(&w, "old-name"));
    let first = stage(&mut w, &first, &["old-name"]).unwrap();
    pin(&mut w, first.registry);
    let previous = w.binding.environment.registry.clone();
    let mut next = successor(&w, "r3");
    next.aliases.push(ConceptLink {
        schema_version: 1,
        origin: id("r3", "old-name"),
        target: id("r3", "report"),
        kind: "canonical".into(),
        basis: w.context.clone(),
    });
    // StageAdoption requires its actual staged revision list when registry_after is set.
    assert!(stage(&mut w, &next, &[]).is_err());
    let revised = next
        .entries
        .iter_mut()
        .find(|e| e.concept_id == "old-name")
        .unwrap();
    revised.previous = Some(object_ref(revised, "r2", limits()).unwrap());
    revised.governance = w.context.clone();
    let staged = stage(&mut w, &next, &[]).unwrap();
    let current = staged.irl.adoption.registry_after.unwrap();
    let old_name = id("r2", "old-name");
    assert_eq!(
        canonical_concept(&w.registry, &previous, &old_name, limits()).unwrap(),
        old_name
    );
    assert_eq!(
        canonical_concept(&staged.registry, &current, &id("r3", "old-name"), limits()).unwrap(),
        id("r3", "report")
    );
}
