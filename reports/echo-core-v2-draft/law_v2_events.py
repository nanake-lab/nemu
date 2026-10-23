"""Research event fixture: immutable historical q, explicit first-allocation plan.

Not the Rust G, support evaluator, alias registry, storage or capacity controller.
A prepared counterfactual has revised old shares but no first allocations; it is
an audit view, not a publishable R. Only the complete staged event is committed.
"""
from __future__ import annotations

import copy
import json
from dataclasses import asdict, dataclass, field

from law_v2_model import NumericFault, Tendency, exact_shares, finite, nonnegative_product

Change = tuple[str, float, float, float]


@dataclass(frozen=True)
class Event:
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
    q: float  # immutable coefficient from FIRST insertion


@dataclass(frozen=True)
class FirstAllocation:
    group: str
    name: str
    weight: float
    support: float
    share: float


@dataclass
class Plan:
    changes: list[Change]
    first: list[FirstAllocation]
    counterfactual: dict[str, float]


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
            if not (0 <= finite(row.share) <= 1 and 0 < finite(row.q) <= 1
                    and -1 <= finite(row.support) <= 1):
                raise NumericFault("InvalidAllocation")
            w = nonnegative_product(row.weight, row.share)
            signal = nonnegative_product(w, abs(row.support), "UnderflowEvidence")
            t.a = finite(finite(row.q * t.a) + finite(w * row.support))
            t.b = finite(finite(row.q * t.b) + signal)
            sign = (row.support > 0) - (row.support < 0)
            if sign:
                if t.last_sign and sign != t.last_sign:
                    t.flips += 1
                t.last_sign = sign
            t.groups += 1
        t.validate()
        self.tendencies[name] = t

    def _insert(self, row: FirstAllocation) -> None:
        if any(old.group == row.group for old in self.allocations[row.name]):
            raise NumericFault("DuplicateGroupFixtureInput")
        w = nonnegative_product(row.weight, row.share)
        q = self.tendencies[row.name].add(w, row.support)
        self.allocations[row.name].append(Allocation(row.group, row.weight, row.support, row.share, q))

    def _append(self, group: str, weight: float, supports: tuple[tuple[str, float], ...]) -> None:
        for name, _ in supports:
            self.ensure(name)
        ordered = sorted(supports)
        shares = exact_shares([self.tendencies[k].pi() for k, _ in ordered], [s for _, s in ordered])
        for (name, support), share in zip(ordered, shares, strict=True):
            self._insert(FirstAllocation(group, name, weight, support, share))

    def _publish(self, staged: Engine) -> None:
        self.tendencies, self.allocations = staged.tendencies, staged.allocations

    def append(self, group: str, weight: float, supports: tuple[tuple[str, float], ...]) -> None:
        staged = copy.deepcopy(self)
        staged._append(group, weight, supports)
        self._publish(staged)

    def _revise(self, event: Event) -> tuple[list[Change], list[FirstAllocation]]:
        group = event.revision_group
        if group is None:
            return [], []
        self.ensure(event.competitor)
        found = {name: next(row for row in rows if row.group == group)
                 for name, rows in self.allocations.items() if any(row.group == group for row in rows)}
        if not found or event.competitor in found:
            raise NumericFault("InvalidRevisionFixtureInput")
        first = next(iter(found.values()))
        candidate_support = finite(event.competitor_support)
        candidate_side = (candidate_support > 0) - (candidate_support < 0)
        # Zero has no side. Decide membership before invoking the quantizer;
        # even unused recalculated shares can fault on an unchanged side.
        affected = sorted(name for name, row in found.items() if candidate_side != 0
                          and (row.support > 0) - (row.support < 0) == candidate_side)
        if candidate_side == 0:
            return [], [FirstAllocation(group, event.competitor, first.weight, candidate_support, 0.)]
        before = {name: self.tendencies[name].pi() for name in affected}
        participants = sorted([*affected, event.competitor])
        supports = [found[k].support if k in found else event.competitor_support for k in participants]
        shares = exact_shares([self.tendencies[k].pi() for k in participants], supports)
        changed: list[Change] = []
        pending: list[FirstAllocation] = []
        for name, support, share in zip(participants, supports, shares, strict=True):
            if name in found:
                row = found[name]
                changed.append((name, row.share, share, support))
                row.share = share  # q, ordinal, S and base weight stay fixed
            else:
                # NEW evidence, even though its source group is old.
                pending.append(FirstAllocation(group, name, first.weight, support, share))
        for name in affected:
            self.replay_tendency(name)
        for name, old_share, share, support in changed:
            delta = self.tendencies[name].pi() - before[name]
            sign = (support > 0) - (support < 0)
            if (share < old_share and sign * delta > 0) or (share > old_share and sign * delta < 0):
                raise NumericFault("PrecisionExhaustedWithdrawal")
        return changed, pending

    def prepare(self, event: Event) -> tuple[Engine, Plan]:
        staged = copy.deepcopy(self)
        changes, first = staged._revise(event)
        for name, _ in event.new_supports:
            staged.ensure(name)
        cf = {name: t.pi() for name, t in staged.tendencies.items()}
        return staged, Plan(changes, first, cf)

    def apply(self, event: Event) -> tuple[dict[str, float], list[Change]]:
        staged, plan = self.prepare(event)
        for row in plan.first:
            staged._insert(row)
        if event.new_group:
            staged._append(event.new_group, event.new_weight, event.new_supports)
        if not staged.exact_conservation():
            raise NumericFault("InvalidConservation")
        self._publish(staged)
        return plan.counterfactual, plan.changes

    def exact_conservation(self) -> bool:
        sides: dict[tuple[str, int], float] = {}
        for name in sorted(self.allocations):
            for row in self.allocations[name]:
                sign = (row.support > 0) - (row.support < 0)
                if sign:
                    key = row.group, sign
                    sides[key] = sides.get(key, 0.) + row.share
        return all(v == 1. for v in sides.values())
