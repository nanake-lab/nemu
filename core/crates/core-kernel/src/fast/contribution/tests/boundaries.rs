use super::*;

fn content_ref<T: WireMessage>(m: &T, tag: &str) -> ObjectRef {
    let schema = Schema::builtin().unwrap();
    let bytes = m.canonical_bytes(limits()).unwrap();
    let value = schema
        .decode_canonical(T::NAME, &bytes, limits(), 0)
        .unwrap();
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(unhex(&schema.digest(T::NAME, &value, limits()).unwrap()).unwrap()),
        object_type: tag.into(),
        resource_version: "fixture".into(),
        byte_length: DecimalU64(bytes.len() as u64),
    }
}

#[test]
fn migration_preserves_x_anchor_empty_slots_and_binds_record_to_content() {
    let old = FastStateSnapshotV2 {
        schema_version: 1,
        revision: "fixture".into(),
        representation_version: crate::fast::REPRESENTATION_VERSION.into(),
        anchor_time_ns: DecimalU64(47),
        nature: state(0.375).snapshot.nature.clone(),
        learned: vec![],
    };
    let mut expected = state(0.375).snapshot;
    expected.anchor_time_ns = old.anchor_time_ns;
    let record = FastStateMigrationRecord {
        schema_version: 1,
        previous_snapshot: content_ref(&old, "echo.fast-state-v2"),
        migrated_snapshot: content_ref(&expected, "echo.fast-state-v3"),
        prior_decomposition_unknown: true,
    };
    let (s, r) = State::migrate(&old, &law_ref(), record.clone(), limits()).unwrap();
    bits(x(&s), 0.375);
    assert_eq!(s.snapshot.anchor_time_ns, old.anchor_time_ns);
    assert!(s.snapshot.slots.is_empty());
    assert_eq!(r, record);
    let bytes = s.canonical_bytes(limits()).unwrap();
    assert!(FastStateSnapshotV2::from_canonical(&bytes, limits()).is_err());
    assert!(
        FastStateSnapshotV2::from_protobuf(&s.snapshot.protobuf_bytes(limits()).unwrap(), limits())
            .is_err()
    );
    let mut false_claim = record.clone();
    false_claim.prior_decomposition_unknown = false;
    assert!(State::migrate(&old, &law_ref(), false_claim, limits()).is_err());
    let mut wrong = record;
    wrong.migrated_snapshot.digest.0[0] ^= 1;
    assert!(State::migrate(&old, &law_ref(), wrong, limits()).is_err());
    // Old version still decodes/replays; no in-place mutation or storage access.
    let old_bytes = old.canonical_bytes(limits()).unwrap();
    assert_eq!(
        old,
        FastStateSnapshotV2::from_canonical(&old_bytes, limits()).unwrap()
    );
}

#[test]
fn f_out_aligns_current_and_target_but_revisit_reads_recorded_coordinates() {
    let mut e = effects();
    e.effects = vec![TendencyEffect {
        tendency_id: "k".into(),
        concept: "c".into(),
        s: 0.5,
        r: 0.5,
        d: [1.0, 0.0, 0.0, 0.0],
        direction_validated: true,
    }];
    let evaluation = evaluate_effects(
        &e,
        &SituationMatch {
            mu: vec![("k".into(), 1.0)],
        },
    )
    .unwrap();
    let target = compute_target(&e, &evaluation, Some([0.0; 4]), 1.0, &law(&law_ref())).unwrap();
    let mut record = new("a", 0.0).record().clone();
    record.target = codec::vector(target.drive());
    let effect = Effect::new(record, &target, limits()).unwrap();
    let s = state(0.0)
        .f_out(&e)
        .unwrap()
        .apply(&effect, limits())
        .unwrap();
    bits(s.snapshot.learned[0].value, 0.5);
    bits(s.snapshot.slots[0].current.learned[0].value, 0.5);
    let t = s.project_time(&e, 1_000_000_000).unwrap();
    bits(t.snapshot.learned[0].value, 0.25);
    bits(t.snapshot.slots[0].current.learned[0].value, 0.25);
    let removed = s.f_out(&effects()).unwrap();
    assert!(removed.snapshot.learned.is_empty());
    let back = removed.f_out(&e).unwrap();
    bits(back.snapshot.learned[0].value, 0.0);
    bits(back.snapshot.slots[0].target.learned[0].value, 0.0);
    let recalled = back.revisit(&[history(&effect, 0)], limits()).unwrap().0;
    bits(recalled.snapshot.learned[0].value, 0.5);
    // Current θ/R never recomputes the historical target on ordinary revisit.
    e.theta.g = [0.125; 4];
    e.theta.b = [0.25; 4];
    same(&s, &s.f_out(&e).unwrap());
    same(&s, &s.revisit(&[history(&effect, 0)], limits()).unwrap().0);
}

#[test]
fn numerical_and_contract_boundaries_reject_without_mutating_input() {
    let s = state(0.0);
    let before = s.clone();
    assert!(s.project_time(&effects(), 0).is_ok());
    let future = s.project_time(&effects(), 5).unwrap();
    assert!(future.project_time(&effects(), 4).is_err());
    let mut snapshot = s.snapshot.clone();
    snapshot.nature.connection = f64::NAN;
    assert!(State::from_snapshot(snapshot, &law_ref(), limits()).is_err());
    let mut snapshot = s.snapshot.clone();
    snapshot.law_version = reference("echo.law-version", 8);
    assert!(State::from_snapshot(snapshot, &law_ref(), limits()).is_err());
    assert!(coordinate(0.0, -f64::MAX, f64::MAX).is_err());
    let mut e = effects();
    e.theta.g = [2.0; 4];
    let evaluation = Evaluation {
        drives: vec![],
        f_sum: [0.0; 4],
    };
    assert!(compute_target(&e, &evaluation, Some([1.0; 4]), 1.0, &law(&law_ref())).is_err());
    let r = law_ref();
    let mut invalid_law = law(&r);
    invalid_law.g_max = f64::INFINITY;
    assert!(compute_target(&effects(), &evaluation, None, 1.0, &invalid_law).is_err());
    same(&s, &before);
    let first = new("a", 0.25);
    let applied = s.apply(&first, limits()).unwrap();
    assert!(applied.apply(&new("a", 0.5), limits()).is_err());
    let mut duplicate = applied.snapshot.clone();
    duplicate.slots.push(duplicate.slots[0].clone());
    assert!(State::from_snapshot(duplicate, &law_ref(), limits()).is_err());
    let second = new("b", 0.25);
    let mut overflow = applied.apply(&second, limits()).unwrap().snapshot;
    for slot in &mut overflow.slots {
        slot.current.nature.connection = f64::MAX;
    }
    let overflow = State::from_snapshot(overflow, &law_ref(), limits()).unwrap();
    let preserved = overflow.clone();
    assert!(
        overflow
            .merge_aliases(&[history(&first, 0), history(&second, 1)], limits())
            .is_err()
    );
    same(&overflow, &preserved);
}

#[test]
fn target_has_no_kappa_and_preserves_legacy_external_arithmetic() {
    let e = effects();
    let evaluation = Evaluation {
        drives: vec![],
        f_sum: [0.0; 4],
    };
    let r = law_ref();
    let l = law(&r);
    let target = compute_target(&e, &evaluation, Some([0.5; 4]), 1.0, &l).unwrap();
    let old = crate::fast::compute_drive(
        &e,
        &evaluation,
        &crate::fast::EventInput {
            appraisal: Some([0.5; 4]),
            empathy_weight: 1.0,
            class: crate::fast::EventClass::NewExogenous,
        },
        &crate::fast::DriveLaw {
            g_max: 1.0,
            r_max: 1.0,
            w_min: 0.1,
            kappa: 0.5,
        },
    )
    .unwrap();
    for (a, b) in target.drive().nature().iter().zip(old.nature()) {
        bits(*a, *b);
    }
    let missing = compute_target(&e, &evaluation, None, 1.0, &l).unwrap();
    assert_eq!(missing.drive().nature(), &[0.0; 4]);
}
