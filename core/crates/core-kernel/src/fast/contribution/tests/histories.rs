use super::*;

#[test]
fn e09_first_recall_shares_basis_surface_ids_and_internal_updates() {
    let a = new("governed-root", 0.25);
    let s = state(0.0).apply(&a, limits()).unwrap();
    for _surface in [
        "original",
        "paraphrase",
        "中文",
        "日本語",
        "other-action-id",
    ] {
        same(&s, &s.revisit(&[history(&a, 0)], limits()).unwrap().0);
    }
    // The faulty original->background / first recall->new slot issues twice.
    let mutant = state(x(&s)).apply(&a, limits()).unwrap();
    assert_ne!(x(&s).to_bits(), x(&mutant).to_bits());
    let internal = make_effect("", 0.25, FastEffectRole::FAST_EFFECT_INTERNAL_BRANCH, 1);
    let updated = make_effect("", 0.5, FastEffectRole::FAST_EFFECT_TARGET_UPDATE, 2);
    let s = state(0.0)
        .apply(&internal, limits())
        .unwrap()
        .apply(&updated, limits())
        .unwrap();
    assert_eq!(s.snapshot.slots.len(), 1);
    bits(x(&s), 0.5);
    assert!(s.revisit(&[history(&internal, 0)], limits()).is_err());
    // Contract mutation: internal + REVISIT rejected before consumption.
    let mut invalid = internal.record().clone();
    invalid.role = FastEffectRole::FAST_EFFECT_REVISIT;
    assert!(codec::check(&invalid, limits()).is_err());
}

#[test]
fn revisit_order_is_first_effect_order_not_input_or_canonical_order() {
    let positive = new("z", 1.0);
    let negative = new("a", -1.0);
    let s = state(0.5);
    let ordered = s
        .revisit(&[history(&negative, 1), history(&positive, 0)], limits())
        .unwrap()
        .0;
    bits(x(&ordered), 0.0);
    let reversed = s
        .revisit(&[history(&negative, 0), history(&positive, 1)], limits())
        .unwrap()
        .0;
    bits(x(&reversed), 0.5);
    assert!(
        s.revisit(&[history(&positive, 0), history(&positive, 0)], limits())
            .is_err()
    );
}

#[test]
fn invalidation_alias_merge_zero_removal_and_reentry() {
    let a = new("a", 0.1);
    let b = new("b", 0.2);
    let s = state(0.0)
        .apply(&a, limits())
        .unwrap()
        .apply(&b, limits())
        .unwrap();
    let mut invalid = history(&a, 0);
    invalid.active = false;
    same(&s, &s.revisit(&[invalid], limits()).unwrap().0);
    let merged = s
        .merge_aliases(&[history(&b, 1), history(&a, 0)], limits())
        .unwrap();
    bits(x(&merged), x(&s));
    bits(h(&merged, "a"), 0.30000000000000004);
    bits(merged.snapshot.slots[0].target.nature.connection, 0.1);
    assert_eq!(merged.snapshot.anchor_time_ns, s.snapshot.anchor_time_ns);
    let merged_again = State::from_snapshot(merged.snapshot.clone(), &law_ref(), limits()).unwrap();
    same(&merged, &merged_again);
    let one = new("a", 1.0);
    let neg = new("b", -1.0);
    let other = new("c", 1.0);
    let s = state(0.0)
        .apply(&one, limits())
        .unwrap()
        .apply(&neg, limits())
        .unwrap()
        .apply(&other, limits())
        .unwrap();
    let merged = s
        .merge_aliases(&[history(&one, 0), history(&other, 2)], limits())
        .unwrap();
    bits(h(&merged, "a"), 2.0);
    bits(x(&merged), 1.0);
    let s = state(0.0)
        .apply(&one, limits())
        .unwrap()
        .apply(&new("zero", 0.0), limits())
        .unwrap();
    let cleaned = s.remove_zero_slots();
    assert_eq!(cleaned.snapshot.slots.len(), 1);
    let reentry = make_effect("zero", 0.25, FastEffectRole::FAST_EFFECT_TARGET_UPDATE, 2);
    for t in [0, 1, 1_000_000_000, 1_000_000_000_000] {
        let a = s
            .project_time(&effects(), t)
            .unwrap()
            .apply(&reentry, limits())
            .unwrap();
        let b = cleaned
            .project_time(&effects(), t)
            .unwrap()
            .apply(&reentry, limits())
            .unwrap();
        same(&a, &b);
    }
    let tiny = state(0.0)
        .apply(&new("tiny", f64::from_bits(1)), limits())
        .unwrap();
    assert_eq!(tiny.remove_zero_slots().snapshot.slots.len(), 1);
}

#[test]
fn contact_report_cadence_fixed_and_changed_targets_respect_historical_interval() {
    for times in [vec![0, 1, 2, 3], vec![0, 1_000_000_000, 5_000_000_000]] {
        let mut s = state(0.0);
        let initial = new("contact-1", 0.75);
        s = s.apply(&initial, limits()).unwrap();
        let report = make_effect(
            "contact-1",
            0.75,
            FastEffectRole::FAST_EFFECT_TARGET_UPDATE,
            1,
        );
        // a, mu, people/w, theta and R+ are identical on every report.
        for t in times {
            s = s
                .project_time(&effects(), t)
                .unwrap()
                .apply(&report, limits())
                .unwrap();
            assert!((0.0..=0.75).contains(&h(&s, "contact-1")));
        }
        assert_eq!(s.snapshot.slots.len(), 1);
        for value in [-0.5, 0.25, 0.0] {
            s = s
                .apply(
                    &make_effect(
                        "contact-1",
                        value,
                        FastEffectRole::FAST_EFFECT_TARGET_UPDATE,
                        2,
                    ),
                    limits(),
                )
                .unwrap();
            assert!((-0.5..=0.75).contains(&h(&s, "contact-1")));
        }
        s = s.apply(&new("contact-2", 0.5), limits()).unwrap();
        assert_eq!(s.snapshot.slots.len(), 2);
        // Absence identity supplied by governance uses the very same operation.
        s = s.apply(&new("absence-1", -0.5), limits()).unwrap();
        s = s
            .apply(
                &make_effect(
                    "absence-1",
                    -0.25,
                    FastEffectRole::FAST_EFFECT_TARGET_UPDATE,
                    3,
                ),
                limits(),
            )
            .unwrap();
        assert_eq!(s.snapshot.slots.len(), 3);
    }
}

#[test]
fn i23_incremental_serialized_restart_and_full_record_fold_match_bits() {
    let records = [
        new("a", 0.75),
        new("b", -0.25),
        make_effect("a", 0.125, FastEffectRole::FAST_EFFECT_TARGET_UPDATE, 2),
        new("c", 1.0),
    ];
    let mut incremental = state(0.0);
    for (i, r) in records.iter().enumerate() {
        incremental = incremental
            .project_time(&effects(), i as u64 * 1_000_000_000)
            .unwrap()
            .apply(r, limits())
            .unwrap();
        incremental = State::from_canonical(
            &incremental.canonical_bytes(limits()).unwrap(),
            &law_ref(),
            limits(),
        )
        .unwrap();
    }
    let full = records.iter().enumerate().fold(state(0.0), |s, (i, r)| {
        s.project_time(&effects(), i as u64 * 1_000_000_000)
            .unwrap()
            .apply(r, limits())
            .unwrap()
    });
    same(&incremental, &full);
}
