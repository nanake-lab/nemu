"""Whole-event checks, including an old-allocations-only counterfactual oracle."""
from __future__ import annotations

import copy
import random

from law_v2_events import Engine, Event
from law_v2_model import NumericFault, Tendency, nonnegative_product


def oracle_counterfactual(before: Engine, after: Engine) -> dict[str, float]:
    """Use actual revised shares but ONLY pre-existing allocation identities.

    Does not call prepare/revise/apply, so putting a first allocation in the
    implementation's counterfactual cannot make both paths agree by accident.
    A newly initialized tendency has zero evidence irrespective of after rows.
    """
    result: dict[str, float] = {}
    for name in sorted(after.tendencies):
        folded = Tendency(3., 1., prior=before.prior)
        actual = {row.group: row for row in after.allocations[name]}
        for old in before.allocations.get(name, []):
            w = nonnegative_product(old.weight, actual[old.group].share)
            folded.a = old.q * folded.a + w * old.support
            folded.b = old.q * folded.b + nonnegative_product(w, abs(old.support))
        result[name] = folded.pi()
    return result


def event_checks() -> dict[str, object]:
    # Same original 250 x 24 random tape; extend targets instead of replacing it.
    rng = random.Random(20261121)
    append_checks = marginal_checks = withdrawal_checks = prefixes = backfill_checks = 0
    append_bad = marginal_bad = withdrawal_bad = replay_bad = share_bad = faults = oracle_bad = 0
    failures: list[dict[str, object]] = []
    for world in range(250):
        engine = Engine(prior=rng.choice((0., 60., 600.)))
        history: list[Event] = []
        for index in range(24):
            revision = f"g{rng.randrange(index)}" if index and index % 2 else None
            event = Event(revision_group=revision, competitor=f"r{index}",
                          competitor_support=rng.choice((-1., 1.)), new_group=f"g{index}",
                          new_weight=rng.uniform(.01, 5.),
                          new_supports=tuple((k, rng.choice((-1., 1.)) * rng.uniform(.1, 1.))
                                             for k in ("a", "b", "c")))
            before = copy.deepcopy(engine)
            try:
                cf, changes = engine.apply(event)
            except NumericFault as exc:
                faults += 1
                failures.append(dict(world=world, event=index, fault=str(exc), before=before.snapshot()))
                break
            oracle = oracle_counterfactual(before, engine)
            oracle_bad += cf != oracle
            targets = list(event.new_supports)
            if revision is not None:
                targets.append((event.competitor, event.competitor_support))
                backfill_checks += 1
            for name, support in targets:
                sign = (support > 0) - (support < 0)
                initial = before.tendencies[name].pi() if name in before.tendencies else engine.prior/(engine.prior+4.)
                if revision is None:
                    append_checks += 1
                    append_bad += sign * (engine.tendencies[name].pi() - initial) <= 0
                else:
                    marginal_checks += 1
                    marginal_bad += sign * (engine.tendencies[name].pi() - cf[name]) <= 0
            for name, old_share, share, support in changes:
                if share < old_share:
                    withdrawal_checks += 1
                    sign = (support > 0) - (support < 0)
                    withdrawal_bad += sign * (cf[name] - before.tendencies[name].pi()) > 0
            history.append(event)
            recomputed = Engine(prior=engine.prior)
            for saved in history:
                recomputed.apply(saved)
            replay_bad += engine.snapshot() != recomputed.snapshot()
            prefixes += 1
            share_bad += not engine.exact_conservation()
    example = Engine()
    example.append("old", 10., (("a", 1.),))
    before_pi = example.tendencies["a"].pi()
    cf, _ = example.apply(Event("old", "b", 1., "new", .01, (("a", 1.),)))
    return dict(check="LP1-three-clauses-I23-I25", seed=20261121, worlds=250,
                append_checks=append_checks, append_violations=append_bad,
                marginal_checks=marginal_checks, marginal_violations=marginal_bad,
                backfill_candidate_checks=backfill_checks,
                oracle_mismatches=oracle_bad,
                withdrawal_checks=withdrawal_checks, withdrawal_violations=withdrawal_bad,
                history_prefixes=prefixes, replay_mismatches=replay_bad,
                conservation_failures=share_bad, faults=faults, failures=failures,
                example=dict(before=before_pi, counterfactual=cf["a"], after=example.tendencies["a"].pi(),
                             candidate_counterfactual=cf["b"], candidate_after=example.tendencies["b"].pi()),
                scope="original random event tape; all first-allocation targets; independent cf oracle")
