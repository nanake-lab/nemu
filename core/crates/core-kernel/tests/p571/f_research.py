"""Independent v2 §6 / ADR0035 §4 research transitions, including rejected laws.

No DUT imports or traces. Inputs and arithmetic primitives are the only shared
objects. Mutations are named in the frozen ledger, never production resources.
"""

import struct
from copy import deepcopy
from typing import Any, Callable

from arithmetic import Q, rounded as r


def clip(x: Q) -> Q:
    return max(Q(-1), min(Q(1), x))


def scalar_bits(value: str) -> Q:
    return Q(struct.unpack(">d", bytes.fromhex(value))[0])


class FResearch:
    def __init__(
        self,
        resource: dict[str, Any],
        case: dict[str, Any],
        primitive: Callable[[str, Q], Q],
    ) -> None:
        self.p, self.case, self.fn = resource, case, primitive
        self.version = case["version"]
        self.base = self.version if self.version in ("old", "A1", "A2", "B") else "A1"
        self.mutant = "" if self.version == self.base else self.version
        if self.mutant == "kappa-one":
            self.base = "old"
        self.size = 5 if case["id"] == "09-19" else 4
        self.x = [Q(0)] * self.size
        self.h: dict[str, list[Q]] = {}
        self.targets: dict[str, list[Q]] = {}
        self.history: dict[str, dict[str, Any]] = {}
        self.active: dict[str, bool] = {}
        self.anchor, self.internal_count = 0, 0
        self.consumed: dict[str, list[str]] = {}
        self.last: dict[str, Any] = {}
        self.fixture_c_mass = 0
        self.theta = deepcopy(case["theta"])
        self.theta["b"] = [Q(v) for v in self.theta["b"]]
        self.theta["g"] = [Q(v) for v in self.theta["g"]]
        self.theta["lambda"] = [scalar_bits(v) for v in self.theta["lambda_bits"]]
        if "initial" in case:
            initial = case["initial"]
            j = initial["coordinate"]
            self.x[j] = Q(initial["x"])
            self.h["r1"] = [Q(0)] * self.size
            self.h["r1"][j] = Q(initial["h"])
            self.targets["r1"] = [Q(0)] * self.size

    def projection(self, at: int) -> tuple[list[Q], dict[str, list[Q]]]:
        dt = r(Q(at - self.anchor) / 10**9)
        rates = self.theta["lambda"] + (
            [self.theta["lambda"][self.case["coordinate"]]] if self.size == 5 else []
        )
        rests = self.theta["b"] + ([Q(0)] if self.size == 5 else [])
        pulls = [-self.fn("expm1", -r(rate * dt)) for rate in rates]
        x = [clip(r(v + r(p * r(b - v)))) for v, p, b in zip(self.x, pulls, rests)]
        h = {
            root: [r(v + r(p * (-v))) for v, p in zip(values, pulls)]
            for root, values in self.h.items()
        }
        if self.mutant in ("A0", "sum-h-x"):
            x = [
                clip(r(rest + sum((v[j] for v in h.values()), Q(0))))
                for j, rest in enumerate(rests)
            ]
        return x, h

    def project(self, at: int) -> None:
        self.x, self.h = self.projection(at)
        self.anchor = at

    def target(self, event: dict[str, Any]) -> list[Q]:
        if event.get("target_mode") == "evaluate_recorded_inputs":
            ti, ri = event["theta_revision"], event["effects_revision"]
            if self.mutant == "wrong-theta" and event["op"] == "revisit":
                ti = 0
            if self.mutant == "wrong-R" and event["op"] == "revisit":
                ri = 0
            theta = self.case["historical_inputs"][ti]
            effect = self.case["historical_inputs"][ri]
            a = [Q(v) for v in event["a"]]
            if self.mutant == "wrong-interpretation" and event["op"] == "revisit":
                a = [-v for v in a]
            learned = r(Q(effect["effect_s"]) * Q(event["mu"]))
            scale = r(learned * Q(effect["effect_r"]))
            value = [
                r(r(Q(g) * a_i) + r(scale * Q(d)))
                for g, a_i, d in zip(theta["theta_g"], a, effect["effect_direction"])
            ]
            self.last["resolved_inputs"] = dict(
                theta_revision=ti,
                effects_revision=ri,
                interpretation=event["interpretation"],
                a=a,
                mu=Q(event["mu"]),
                w=Q(event["w"]),
                involved_people=event["involved_people"],
                governance_id=event["governance_id"],
            )
            return value + [learned]
        if event.get("target_mode") == "current_theta":
            return [
                r(g * r(Q(a) - b))
                for g, a, b in zip(self.theta["g"], event["a"], self.theta["b"])
            ]
        return [Q(v) for v in event["target"]]

    def step(self, event: dict[str, Any]) -> None:
        if (
            self.mutant == "sort-first-by-name"
            and self.case["id"] == "09-06"
            and event["ordinal"] <= 3
        ):
            selected = sorted(self.case["events"][:3], key=lambda e: e["root"])[
                event["ordinal"] - 1
            ]
            event = dict(selected, ordinal=event["ordinal"], time_ns=event["time_ns"])
        self.last = dict(fault=None, applied=False, op=event["op"])
        op, root = event["op"], event["root"]
        if self.case["id"] == "09-05" and event["ordinal"] == 1:
            self.fixture_c_mass += 1
        if self.mutant == "derived-new-mass" and event["ordinal"] > 1:
            self.fixture_c_mass += 1
        if self.mutant == "result-learns-C" and event.get("source") == "action_result":
            self.fixture_c_mass += 1
        if self.mutant == "candidate-mass" and event.get("admission") in (
            "candidate",
            "repeat",
            "alias",
        ):
            self.fixture_c_mass += 1
        if self.case.get("negative_control") == "F-off":
            return
        if op == "noop":
            if (
                self.mutant == "retransmit-as-revisit"
                and event.get("admission") == "retransmit"
            ):
                op = "revisit"
            elif self.mutant in ("candidate-mass", "unverified-accepted"):
                op = "update" if root in self.history else "new"
            else:
                return
        if op == "invalidate":
            self.active[root] = False
            return
        if op == "reinterpret":
            self.last["interpreter"] = event["interpretation"]
            return
        if op == "roundtrip":
            # The independent state has no external cache. Exact copy roundtrip
            # is compared with the DUT's full canonical encode/decode operation.
            self.x, self.h, self.targets = deepcopy((self.x, self.h, self.targets))
            return
        if op == "migrate":
            self.h, self.targets = {}, {}
            if self.mutant == "guess-decomposition":
                self.h["guessed"] = list(self.x)
                self.targets["guessed"] = list(self.x)
            self.last["migration"] = dict(
                empty_slots=True, decomposition="non_identifiable"
            )
            return
        if op == "remove_zero":
            for key in list(self.h):
                if self.mutant == "evict-nonzero" or all(v == 0 for v in self.h[key]):
                    del self.h[key]
                    self.targets.pop(key, None)
            return
        if op == "merge":
            if self.base in ("old", "B"):
                # These references have no contribution slots. In particular,
                # maintenance must not discard B's already consumed identities.
                return
            members = sorted(event["members"], key=lambda k: self.history[k]["ordinal"])
            values = [Q(0)] * self.size
            for key in members:
                values = [
                    r(a + b)
                    for a, b in zip(values, self.h.get(key, [Q(0)] * self.size))
                ]
            saved = self.targets.get(members[0], self.target(self.history[members[0]]))
            for key in members:
                self.h.pop(key, None)
                self.targets.pop(key, None)
            self.h[members[0]], self.targets[members[0]] = values, saved
            return
        if op == "revisit":
            if not event.get("active", self.active.get(root, True)):
                return
            if event["kind"] == "internal" and self.mutant != "internal-revisit":
                self.last["fault"] = "internal revisit"
                return
            if self.mutant in ("reject-revisit", "ban-revisit"):
                return
        if (
            self.mutant == "reject-internal-update"
            and event["kind"] == "internal"
            and op == "update"
        ):
            return
        if self.mutant == "reject-result" and event.get("source") == "action_result":
            return
        if (
            self.mutant == "reject-backfill"
            and event.get("admission") == "verified_backfill"
        ):
            return
        if (
            self.mutant
            in (
                "text-root",
                "action-id-root",
                "text-new-slot",
                "derived-new-root",
                "double-original-recall",
            )
            and root in self.history
        ):
            root = event["id"]
            op = "new"
        if self.mutant == "unknown-new" and event["kind"] == "internal":
            root = event["id"]
        if self.base == "B":
            # Frozen §3.1 / P09-B: the synthetic truth table names real parts,
            # not targets, request IDs or parameter/interpretation revisions.
            admission = self.case["b_admissions"][str(event["ordinal"])]
            part = admission["real_part"]
            self.last["b_admission"] = admission
            if (
                op not in ("new", "update")
                or event["kind"] != "record"
                or part is None
                or part in self.consumed.get(root, [])
            ):
                # No F pulse also means no new Phi anchor. Future reads still
                # project from the last admitted pulse, not each rejected visit.
                return
        self.project(event["time_ns"])
        if (
            op == "revisit"
            and event.get("target_mode") != "evaluate_recorded_inputs"
            and root in self.history
        ):
            # §6 C-02 / ADR0035 §4.3: current-theta targets were evaluated at
            # their original submission, never re-evaluated on a later visit.
            value = (
                self.target(self.history[root])
                if self.base == "old"
                else list(self.history[root]["recorded_target"])
            )
        else:
            value = self.target(event)
        if op in ("new", "update"):
            self.history[root] = deepcopy(event)
            # H's recorded target survives zero-slot eviction and migration;
            # targets/h below represent the current fast-state slots only.
            self.history[root]["recorded_target"] = list(value)
            self.active[root] = True
        if self.base == "old":
            if op != "revisit" and event.get("source") == "action_result":
                exponent = 1
            elif op == "new" and event["kind"] == "record":
                exponent, self.internal_count = 0, 0
            else:
                # ADR0026 §6.3: n counts already committed internal records.
                exponent = self.internal_count
                self.internal_count += 1
            weight = Q(1)
            for _ in range(exponent):
                weight = r(
                    weight
                    * (Q(1) if self.mutant == "kappa-one" else Q(self.p["kappa"]))
                )
            self.x = [clip(r(x + r(weight * v))) for x, v in zip(self.x, value)]
        elif self.base == "B":
            self.x = [clip(r(x + v)) for x, v in zip(self.x, value)]
            self.consumed.setdefault(root, []).append(part)
        else:
            old = self.h.get(root, [Q(0)] * self.size)
            if self.mutant == "add-not-replace":
                old = [Q(0)] * self.size
            if self.mutant == "target-difference":
                old = self.targets.get(root, [Q(0)] * self.size)
            updated: list[Q] = []
            for j, (x, h, v) in enumerate(zip(self.x, old, value)):
                total = r(x + r(v - h))
                self.x[j] = clip(total)
                if self.base == "A2" or self.mutant in (
                    "A0",
                    "loss-to-background",
                    "force-equal",
                ):
                    updated.append(v)
                elif -1 < total < 1:
                    updated.append(v)
                else:
                    wall = Q(1 if total >= 1 else -1)
                    updated.append(max(min(h, v), min(max(h, v), r(h + r(wall - x)))))
            self.h[root], self.targets[root] = updated, value
            if self.mutant == "proportional":
                self.h[root] = value
                for j in range(self.size):
                    total = sum((h[j] for h in self.h.values()), Q(0))
                    if abs(total) > 1:
                        for component in self.h.values():
                            component[j] = r(component[j] / abs(total))
            if self.mutant == "zero-h" and all(v == 0 for v in value):
                self.h[root] = [Q(0)] * self.size
            if self.mutant == "zero-x" and all(v == 0 for v in value):
                self.x = [Q(0)] * self.size
        if self.case.get("negative_control") == "x-zero":
            self.x = [Q(0)] * self.size
        if op in ("new", "update"):
            self.targets[root] = list(value)
        self.last["applied"] = True

    def state(self) -> dict[str, Any]:
        return dict(
            x=self.x,
            h=self.h,
            targets=self.targets,
            anchor=self.anchor,
            history=self.history,
            consumed=self.consumed,
            active=self.active,
            internal_count=self.internal_count,
            fixture_c_mass=self.fixture_c_mass,
        )
