//! P5-09 acceptance: DSL intersection cases (card requires >= 50).
//! Each case is checked in both directions; overlap must be symmetric.

mod support;

use core_policy::{Endpoint, Overlap, Scope, ScopeExpr, overlap};
use support::*;

use Overlap::{Disjoint as D, Overlapping as O};

fn cases() -> Vec<(&'static str, Scope, Scope, Overlap)> {
    let ev2 = |a: ScopeExpr, b: ScopeExpr| (ev(a), ev(b));
    let mut out = Vec::new();
    let mut push = |name, (a, b): (Scope, Scope), want| out.push((name, a, b, want));

    // Enumerations.
    push(
        "enum shared code",
        ev2(e("event_kind", &[1, 2]), e("event_kind", &[2, 3])),
        O,
    );
    push(
        "enum distinct codes",
        ev2(e("event_kind", &[1]), e("event_kind", &[2])),
        D,
    );
    push(
        "enum distinct sets",
        ev2(e("event_kind", &[1, 2, 3]), e("event_kind", &[4, 5])),
        D,
    );
    push(
        "enum identical",
        ev2(e("event_kind", &[1]), e("event_kind", &[1])),
        O,
    );
    push(
        "true vs enum",
        ev2(ScopeExpr::True, e("event_kind", &[1])),
        O,
    );
    push("true vs true", ev2(ScopeExpr::True, ScopeExpr::True), O);
    push(
        "enum subset",
        ev2(e("event_kind", &[1, 2, 3]), e("event_kind", &[2])),
        O,
    );
    // Concepts.
    push("concept shared", ev2(c(&["a", "b"]), c(&["b"])), O);
    push("concept distinct", ev2(c(&["a"]), c(&["b"])), D);
    push(
        "concept vs enum field",
        ev2(c(&["a"]), e("event_kind", &[1])),
        O,
    );
    push(
        "concept distinct sets",
        ev2(c(&["a", "b"]), c(&["c", "d"])),
        D,
    );
    // Closed numeric intervals.
    push(
        "interval touching end",
        ev2(n("x0", 0, 10), n("x0", 10, 20)),
        O,
    );
    push("interval gap", ev2(n("x0", 0, 10), n("x0", 11, 20)), D);
    push(
        "interval touching start",
        ev2(n("x0", 0, 10), n("x0", -5, 0)),
        O,
    );
    push("interval below", ev2(n("x0", 0, 10), n("x0", -5, -1)), D);
    push("point inside", ev2(n("x0", 5, 5), n("x0", 0, 10)), O);
    push("points apart", ev2(n("x0", 5, 5), n("x0", 6, 6)), D);
    push("points equal", ev2(n("x0", 5, 5), n("x0", 5, 5)), O);
    push(
        "different coordinates",
        ev2(n("x0", 0, 1), n("x1", 5, 6)),
        O,
    );
    push(
        "nested intervals",
        ev2(n("x2", -100, 100), n("x2", -1, 1)),
        O,
    );
    // Conjunctions.
    push(
        "and interval apart",
        ev2(
            and(vec![e("event_kind", &[1]), n("x0", 0, 10)]),
            and(vec![e("event_kind", &[1]), n("x0", 20, 30)]),
        ),
        D,
    );
    push(
        "and enum apart",
        ev2(
            and(vec![e("event_kind", &[1]), n("x0", 0, 10)]),
            and(vec![e("event_kind", &[2]), n("x0", 0, 10)]),
        ),
        D,
    );
    push(
        "and both shared",
        ev2(
            and(vec![e("event_kind", &[1, 2]), n("x0", 0, 10)]),
            and(vec![e("event_kind", &[2, 3]), n("x0", 5, 15)]),
        ),
        O,
    );
    push(
        "unsatisfiable enum and",
        ev2(
            and(vec![e("event_kind", &[1]), e("event_kind", &[2])]),
            ScopeExpr::True,
        ),
        D,
    );
    push(
        "unsatisfiable interval and",
        ev2(and(vec![n("x0", 0, 5), n("x0", 6, 9)]), ScopeExpr::True),
        D,
    );
    push(
        "narrowed interval meets",
        ev2(and(vec![n("x0", 0, 5), n("x0", 3, 9)]), n("x0", 5, 7)),
        O,
    );
    push(
        "narrowed interval misses",
        ev2(and(vec![n("x0", 0, 5), n("x0", 3, 9)]), n("x0", 6, 7)),
        D,
    );
    push(
        "two coordinates one apart",
        ev2(
            and(vec![n("x0", 0, 1), n("x1", 0, 1)]),
            and(vec![n("x0", 0, 1), n("x1", 2, 3)]),
        ),
        D,
    );
    push(
        "two coordinates corner",
        ev2(
            and(vec![n("x0", 0, 1), n("x1", 0, 1)]),
            and(vec![n("x0", 1, 2), n("x1", 1, 2)]),
        ),
        O,
    );
    push(
        "independent coordinates",
        ev2(and(vec![n("x0", 0, 1)]), and(vec![n("x1", 2, 3)])),
        O,
    );
    push(
        "concept and enum apart",
        ev2(
            and(vec![c(&["a"]), e("event_kind", &[1])]),
            and(vec![c(&["a"]), e("event_kind", &[2])]),
        ),
        D,
    );
    // Disjunctions.
    push(
        "or member shared",
        ev2(
            or(vec![e("event_kind", &[1]), e("event_kind", &[2])]),
            e("event_kind", &[2]),
        ),
        O,
    );
    push(
        "or no member",
        ev2(
            or(vec![e("event_kind", &[1]), e("event_kind", &[2])]),
            e("event_kind", &[3]),
        ),
        D,
    );
    push(
        "or intervals around gap",
        ev2(or(vec![n("x0", 0, 1), n("x0", 10, 11)]), n("x0", 5, 6)),
        D,
    );
    push(
        "or interval edge",
        ev2(or(vec![n("x0", 0, 1), n("x0", 10, 11)]), n("x0", 1, 2)),
        O,
    );
    let mixed = || {
        or(vec![
            and(vec![e("event_kind", &[1]), n("x0", 0, 1)]),
            and(vec![e("event_kind", &[2]), n("x0", 10, 11)]),
        ])
    };
    push(
        "or of ands crossed",
        ev2(mixed(), and(vec![e("event_kind", &[1]), n("x0", 10, 11)])),
        D,
    );
    push(
        "or of ands aligned",
        ev2(mixed(), and(vec![e("event_kind", &[2]), n("x0", 10, 11)])),
        O,
    );
    push(
        "or vs or apart",
        ev2(
            or(vec![e("event_kind", &[1]), e("event_kind", &[2])]),
            or(vec![e("event_kind", &[3]), e("event_kind", &[4])]),
        ),
        D,
    );
    push(
        "or vs or shared",
        ev2(
            or(vec![e("event_kind", &[1]), e("event_kind", &[2])]),
            or(vec![e("event_kind", &[3]), e("event_kind", &[2])]),
        ),
        O,
    );
    // Nested combinations.
    let nested = || {
        and(vec![
            or(vec![e("event_kind", &[1]), e("event_kind", &[2])]),
            or(vec![n("x0", 0, 1), n("x0", 5, 6)]),
        ])
    };
    push(
        "nested miss",
        ev2(nested(), and(vec![e("event_kind", &[2]), n("x0", 3, 4)])),
        D,
    );
    push(
        "nested hit",
        ev2(nested(), and(vec![e("event_kind", &[2]), n("x0", 6, 7)])),
        O,
    );
    let concepts = || and(vec![or(vec![c(&["a"]), c(&["b"])]), e("event_kind", &[1])]);
    push(
        "nested concept hit",
        ev2(concepts(), and(vec![c(&["b"]), e("event_kind", &[1])])),
        O,
    );
    push(
        "nested concept miss",
        ev2(concepts(), and(vec![c(&["z"]), e("event_kind", &[1])])),
        D,
    );
    push(
        "nested three fields",
        ev2(
            and(vec![
                e("event_kind", &[3]),
                or(vec![n("x1", 0, 2), n("x2", 0, 2)]),
            ]),
            and(vec![e("event_kind", &[3]), n("x1", 5, 9), n("x2", 1, 1)]),
        ),
        O,
    );
    // Endpoints.
    push(
        "different endpoints",
        (
            at(Endpoint::FEvent, e("event_kind", &[1])),
            at(Endpoint::GMeta, e("event_kind", &[1])),
        ),
        D,
    );
    push(
        "j_direct true vs coordinate",
        (
            at(Endpoint::JDirect, ScopeExpr::True),
            at(Endpoint::JDirect, n("x3", 0, 1)),
        ),
        O,
    );
    push(
        "j_direct coordinates apart",
        (
            at(Endpoint::JDirect, n("x3", 0, 1)),
            at(Endpoint::JDirect, n("x3", 2, 3)),
        ),
        D,
    );
    out
}

#[test]
fn intersection_cases_hold_in_both_directions() {
    let catalog = catalog();
    let all = cases();
    assert!(all.len() >= 45, "case table shrank: {}", all.len());
    let mut checks = 0;
    for (name, a, b, want) in &all {
        assert_eq!(overlap(a, b, &catalog), *want, "{name}");
        assert_eq!(overlap(b, a, &catalog), *want, "{name} (reversed)");
        checks += 2;
    }
    assert!(
        checks >= 50,
        "card requires at least 50 intersection checks, got {checks}"
    );
}

#[test]
fn dnf_limit_reports_unknown_overlap() {
    let tight = core_policy::ScopeLimits {
        max_dnf_terms: 3,
        ..LIMITS
    };
    let catalog = catalog_with(tight, "fixture-catalog-1");
    let wide = or(vec![
        e("event_kind", &[1]),
        e("event_kind", &[2]),
        e("event_kind", &[3]),
        e("event_kind", &[4]),
    ]);
    let a = Scope::new(Endpoint::FEvent, wide, &catalog).expect("valid");
    let b = Scope::new(Endpoint::FEvent, e("event_kind", &[8]), &catalog).expect("valid");
    assert_eq!(overlap(&a, &b, &catalog), Overlap::Unknown);
    assert!(overlap(&a, &b, &catalog).is_conflict());
}

#[test]
fn product_limit_reports_unknown_overlap() {
    let tight = core_policy::ScopeLimits {
        max_dnf_terms: 8,
        ..LIMITS
    };
    let catalog = catalog_with(tight, "fixture-catalog-1");
    let pair = |f: &str| or(vec![n(f, 0, 1), n(f, 5, 6)]);
    let expr = and(vec![pair("x0"), pair("x1"), pair("x2"), pair("x3")]);
    let a = Scope::new(Endpoint::FEvent, expr, &catalog).expect("valid");
    assert_eq!(overlap(&a, &a, &catalog), Overlap::Unknown);
}

#[test]
fn pair_limit_reports_unknown_overlap() {
    let tight = core_policy::ScopeLimits {
        max_overlap_pairs: 3,
        ..LIMITS
    };
    let catalog = catalog_with(tight, "fixture-catalog-1");
    let two = or(vec![e("event_kind", &[1]), e("event_kind", &[2])]);
    let a = Scope::new(Endpoint::FEvent, two.clone(), &catalog).expect("valid");
    let b = Scope::new(Endpoint::FEvent, two, &catalog).expect("valid");
    assert_eq!(overlap(&a, &b, &catalog), Overlap::Unknown);
}

#[test]
fn foreign_catalog_version_is_unknown_not_disjoint() {
    let other = catalog_with(LIMITS, "fixture-catalog-2");
    let a = Scope::new(Endpoint::FEvent, e("event_kind", &[1]), &other).expect("valid");
    let b = ev(e("event_kind", &[2]));
    assert_eq!(overlap(&a, &b, &catalog()), Overlap::Unknown);
}
