"""Research event fixture for the three LP1 clauses requested 2026-10-22.

This is not the Rust G, support evaluator, alias registry, storage, or capacity
controller. It implements allocation revision, immutable historical discounts,
and append events, with all state explicit and replayed from synthetic input H.
"""
from __future__ import annotations

import copy
import json
import random
from dataclasses import asdict, dataclass, field

from law_v2_model import NumericFault, Tendency, exact_shares, finite


@dataclass(frozen=True)
class Event:
    # Revision adds this competitor to an existing group's same side.
    revision_group: str | None = None
    competitor: str = ""
    competitor_support: float = 1.
    new_group: str = ""
    new_weight: float = 0.
    new_supports: tuple[tuple[str, float], ...] = ()


@dataclass
class Allocation:
    group: str
    weight: float
    support: float
    share: float
    q: float  # immutable coefficient from the event that FIRST inserted this allocation


@dataclass
class Engine:
    prior: float = 0.
    tendencies: dict[str, Tendency] = field(default_factory=dict)
    allocations: dict[str, list[Allocation]] = field(default_factory=dict)

    def ensure(self, name: str) -> None:
        if name not in self.tendencies:
            self.tendencies[name] = Tendency(3., 1., prior=self.prior)
            self.allocations[name] = []

    def snapshot(self) -> str:
        return json.dumps(asdict(self), sort_keys=True, separators=(",", ":"), allow_nan=False)

    def replay_tendency(self, name: str) -> None:
        t = Tendency(3., 1., prior=self.prior)
        for row in self.allocations[name]:
            w = finite(row.weight * row.share)
            t.a = finite(finite(row.q * t.a) + finite(w * row.support))
            t.b = finite(finite(row.q * t.b) + finite(w * abs(row.support)))
            sign = (row.support > 0) - (row.support < 0)
            if sign:
                if t.last_sign and sign != t.last_sign:
                    t.flips += 1
                t.last_sign = sign
            t.groups += 1
        t.validate()
        self.tendencies[name] = t

    def append(self, group: str, weight: float, supports: tuple[tuple[str, float], ...]) -> None:
        for name, _ in supports:
            self.ensure(name)
        ordered = sorted(supports)
        shares = exact_shares([self.tendencies[k].pi() for k, _ in ordered], [s for _, s in ordered])
        for (name, support), share in zip(ordered, shares, strict=True):
            if any(row.group == group for row in self.allocations[name]):
                raise NumericFault("DuplicateGroupFixtureInput")
            q = self.tendencies[name].add(weight * share, support)
            self.allocations[name].append(Allocation(group, weight, support, share, q))

    def revise(self, event: Event) -> list[tuple[str, float, float, float]]:
        group = event.revision_group
        if group is None:
            return []
        self.ensure(event.competitor)
        found = {name: next(row for row in rows if row.group == group)
                 for name, rows in self.allocations.items() if any(row.group == group for row in rows)}
        if not found or event.competitor in found:
            raise NumericFault("InvalidRevisionFixtureInput")
        first = next(iter(found.values()))
        before = {name: self.tendencies[name].pi() for name in found}
        participants = sorted([*found, event.competitor])
        supports = [found[k].support if k in found else event.competitor_support for k in participants]
        shares = exact_shares([self.tendencies[k].pi() for k in participants], supports)
        changed: list[tuple[str, float, float, float]] = []
        for name, support, share in zip(participants, supports, shares, strict=True):
            if name in found:
                row = found[name]
                if support * event.competitor_support < 0:
                    share = row.share  # unchanged membership on the opposite side
                changed.append((name, row.share, share, support))
                row.share = share  # q, ordinal, S and base weight are NOT changed
            else:
                q = self.tendencies[name].add(first.weight * share, support)
                self.allocations[name].append(Allocation(group, first.weight, support, share, q))
        for name in sorted(found):
            self.replay_tendency(name)
        for name, old_share, share, support in changed:
            delta = self.tendencies[name].pi() - before[name]
            sign = (support > 0) - (support < 0)
            if (share < old_share and sign * delta > 0) or (share > old_share and sign * delta < 0):
                raise NumericFault("PrecisionExhaustedWithdrawal")
        return changed

    def apply(self, event: Event) -> tuple[dict[str, float], list[tuple[str, float, float, float]]]:
        # Fixed order: plan and finish all revisions, THEN append new evidence.
        changed = self.revise(event)
        for name, _ in event.new_supports:
            self.ensure(name)
        counterfactual = {name: t.pi() for name, t in self.tendencies.items()}
        if event.new_group:
            self.append(event.new_group, event.new_weight, event.new_supports)
        return counterfactual, changed

    def exact_conservation(self) -> bool:
        sides: dict[tuple[str, int], float] = {}
        for name in sorted(self.allocations):
            for row in self.allocations[name]:
                sign = (row.support > 0) - (row.support < 0)
                if sign:
                    key = row.group, sign
                    sides[key] = sides.get(key, 0.) + row.share
        return all(v == 1. for v in sides.values())


def event_checks() -> dict[str, object]:
    rng = random.Random(20261121)
    append_checks = marginal_checks = withdrawal_checks = prefixes = 0
    append_bad = marginal_bad = withdrawal_bad = replay_bad = share_bad = faults = 0
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
            # Atomic staging; failures retain the old state and stop this lineage.
            staged = copy.deepcopy(engine)
            try:
                cf, changes = staged.apply(event)
            except NumericFault as exc:
                faults += 1
                failures.append(dict(world=world, event=index, fault=str(exc)))
                break
            without_new = copy.deepcopy(before)
            without_new.apply(Event(revision_group=event.revision_group,
                                    competitor=event.competitor,
                                    competitor_support=event.competitor_support))
            for name, sign in event.new_supports:
                initial = before.tendencies[name].pi() if name in before.tendencies else engine.prior/(engine.prior+4.)
                if revision is None:
                    append_checks += 1
                    append_bad += sign * (staged.tendencies[name].pi() - initial) <= 0
                else:
                    marginal_checks += 1
                    marginal_bad += sign * (staged.tendencies[name].pi() - cf[name]) <= 0
                    marginal_bad += cf[name] != without_new.tendencies[name].pi()
            for name, old_share, share, support in changes:
                if share < old_share:
                    withdrawal_checks += 1
                    withdrawal_bad += support * (cf[name] - before.tendencies[name].pi()) > 0
            engine = staged
            history.append(event)
            recomputed = Engine(prior=engine.prior)
            for saved in history:
                recomputed.apply(saved)
            replay_bad += engine.snapshot() != recomputed.snapshot()
            prefixes += 1
            share_bad += not engine.exact_conservation()
    # The formerly ambiguous example is checked against all three actual states.
    example = Engine()
    example.append("old", 10., (("a", 1.),))
    before_pi = example.tendencies["a"].pi()
    cf, _ = example.apply(Event("old", "b", 1., "new", .01, (("a", 1.),)))
    return dict(check="LP1-three-clauses-I23-I25", seed=20261121, worlds=250,
                append_checks=append_checks, append_violations=append_bad,
                marginal_checks=marginal_checks, marginal_violations=marginal_bad,
                withdrawal_checks=withdrawal_checks, withdrawal_violations=withdrawal_bad,
                history_prefixes=prefixes, replay_mismatches=replay_bad,
                conservation_failures=share_bad, faults=faults, failures=failures,
                example=dict(before=before_pi, counterfactual=cf["a"], after=example.tendencies["a"].pi()),
                scope="synthetic event fixture with frozen historical q, revision-first order")
