"""PR61 blocking regressions: candidate backfill and incoming-weight underflow."""
from __future__ import annotations

import copy
import math
import random
from collections.abc import Callable
from dataclasses import asdict

from law_v2_event_checks import oracle_counterfactual
from law_v2_events import Allocation, Engine, Event
from law_v2_model import NumericFault


def backfill_random() -> dict[str, object]:
    rng = random.Random(20261023)
    only = mixed = existing = marginal_bad = oracle_bad = replay_bad = share_bad = faults = 0
    first_bad = first_checks = prefixes = 0
    failures: list[dict[str, object]] = []
    examples: list[dict[str, object]] = []
    for world in range(150):
        engine = Engine(prior=rng.choice((0., 60., 600.)))
        root = Event(new_group="root", new_weight=rng.uniform(1., 5.),
                     new_supports=(("a", .8), ("c", -.9)))
        engine.apply(root)
        history = [root]
        for index in range(24):
            name, sign = f"r{index}", rng.choice((-1., 1.))
            if index % 3 == 0:
                preload = Event(new_group=f"pre{index}", new_weight=rng.uniform(.1, 2.),
                                new_supports=((name, rng.choice((-1., 1.))),))
                engine.apply(preload)
                history.append(preload)
                existing += 1
            event = Event(revision_group="root", competitor=name, competitor_support=sign,
                          new_group=f"new{index}" if index % 2 else "",
                          new_weight=rng.uniform(.01, 3.),
                          new_supports=((name, sign), ("a", rng.choice((-1., 1.)))) if index % 2 else ())
            before = copy.deepcopy(engine)
            try:
                cf, _ = engine.apply(event)
                # The plan must defer exactly this new (old-group, tendency) pair.
                prepared, plan = before.prepare(event)
                first_checks += 1
                first_bad += (len(plan.first) != 1 or plan.first[0].name != name
                              or any(r.group == "root" for r in prepared.allocations[name]))
                first_state = prepared.tendencies[name].pi()
                prepared._insert(plan.first[0])
                first_bad += sign * (prepared.tendencies[name].pi() - first_state) <= 0
            except NumericFault as exc:
                faults += 1
                failures.append(dict(world=world, index=index, fault=str(exc),
                                     event=asdict(event), before=before.snapshot()))
                break
            oracle_bad += cf != oracle_counterfactual(before, engine)
            marginal_bad += sign * (engine.tendencies[name].pi() - cf[name]) <= 0
            if event.new_group:
                mixed += 1
            else:
                only += 1
            if world == 0 and index < 2:
                examples.append(dict(mode="mixed" if event.new_group else "only-backfill",
                                     candidate=name, cf=cf[name], after=engine.tendencies[name].pi()))
            history.append(event)
            replay = Engine(prior=engine.prior)
            for saved in history:
                replay.apply(saved)
            replay_bad += engine.snapshot() != replay.snapshot()
            share_bad += not engine.exact_conservation()
            prefixes += 1
    return dict(seed=20261023, worlds=150, only_backfill=only, mixed=mixed,
                existing_tendency_first_allocation=existing, candidate_marginal_violations=marginal_bad,
                first_allocation_checks=first_checks, first_allocation_violations=first_bad,
                oracle_mismatches=oracle_bad, history_prefixes=prefixes,
                replay_mismatches=replay_bad, conservation_failures=share_bad, faults=faults,
                failures=failures, examples=examples,
                ok=not any((marginal_bad, first_bad, oracle_bad, replay_bad, share_bad, faults)))


def weight_boundaries() -> dict[str, object]:
    rows: list[dict[str, object]] = []

    def expect_fault(name: str, engine: Engine, action: Callable[[], object], expected: str) -> None:
        before = engine.snapshot()
        fault = "not_raised"
        try:
            action()
        except NumericFault as exc:
            fault = str(exc)
        rows.append(dict(case=name, fault=fault, expected=expected, unchanged=engine.snapshot() == before,
                         ok=fault == expected and engine.snapshot() == before))

    for sign in (-1., 1.):
        e = Engine()
        event = Event(new_group="tiny", new_weight=math.ulp(0.),
                      new_supports=(("a", sign), ("b", sign)))
        expect_fault(f"new_group_half_share_{sign:+.0f}", e, lambda: e.apply(event), "UnderflowEvidenceWeight")

    # Valid earlier events; the NEW backfill pair has a one-unit positive share.
    e = Engine()
    e.append("old", 1e-310, (("a", 1.),))
    e.append("pre", float(2**54-4), (("b", -1.),))
    event = Event(revision_group="old", competitor="b", competitor_support=1.)
    expect_fault("backfill_first_allocation", e, lambda: e.apply(event), "UnderflowEvidenceWeight")

    # Valid earlier events; reducing an OLD positive share makes its W underflow.
    e = Engine()
    e.append("pre", 1e14, (("a", -1.),))
    e.append("old", 1e-311, (("a", 1.),))
    event = Event(revision_group="old", competitor="b", competitor_support=1.)
    expect_fault("revised_old_allocation", e, lambda: e.apply(event), "UnderflowEvidenceWeight")

    # Late failure after both revision and candidate insertion must publish nothing.
    e = Engine()
    e.append("old", 10., (("z", 1.),))
    event = Event("old", "a", 1., "tiny", math.ulp(0.), (("a", 1.), ("z", 1.)))
    expect_fault("mixed_event_atomicity", e, lambda: e.apply(event), "UnderflowEvidenceWeight")

    # Stored-row fault injection exercises fold validation independently of entry.
    for label, weight, share, support, fault in (
        ("replay_share", math.ulp(0.), .5, 1., "UnderflowEvidenceWeight"),
        ("replay_signal", math.ulp(0.), 1., .25, "UnderflowEvidence"),
    ):
        e = Engine()
        e.ensure("a")
        e.allocations["a"] = [Allocation("injected", weight, support, share, 1.)]
        expect_fault(label, e, lambda: e.replay_tendency("a"), fault)

    legal: list[dict[str, object]] = []
    for label, weight, support in (("zero_weight", 0., 1.), ("zero_support", 1., 0.)):
        e = Engine()
        e.apply(Event(new_group="zero", new_weight=weight, new_supports=(("a", support), ("b", support))))
        legal.append(dict(case=label, pi=[t.pi() for t in e.tendencies.values()],
                          groups=[t.groups for t in e.tendencies.values()]))
    zeros_ok = all(r["pi"] == [0., 0.] and r["groups"] == [1, 1] for r in legal)
    return dict(cases=rows, genuine_zeros=legal, ok=all(r["ok"] for r in rows) and zeros_ok)


def review_checks() -> dict[str, object]:
    e = Engine()
    e.append("old", 10., (("a", 1.),))
    before = copy.deepcopy(e)
    cf, _ = e.apply(Event(revision_group="old", competitor="b", competitor_support=1.))
    regression = dict(counterfactual=cf, after={k: t.pi() for k, t in e.tendencies.items()},
                      ok=cf["b"] == 0. and e.tendencies["b"].pi() > cf["b"]
                      and cf == oracle_counterfactual(before, e))
    random_cases, boundaries = backfill_random(), weight_boundaries()
    return dict(check="PR61-blocking-regressions", regression=regression,
                random_backfill=random_cases, incoming_weight_boundaries=boundaries,
                ok=regression["ok"] is True and random_cases["ok"] is True and boundaries["ok"] is True)
