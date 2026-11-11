"""Eligibility, atomic rejection and independent C variants for frozen §4."""

from copy import deepcopy
from typing import Any, Callable

from arithmetic import Q, rounded as r
from c_oracle import C, vec
from c_bounds import bounds, Interval
from f_research import scalar_bits


class CResearch:
    def __init__(
        self, p: dict[str, Any], case: dict[str, Any], fn: Callable[[str, Q], Q]
    ) -> None:
        self.p, self.case, self.fn = p, case, fn
        self.version = case["version"]
        self.mutant = "" if self.version in ("old", "A", "B") else self.version
        baseline = (
            self.version
            if not self.mutant
            else "old"
            if self.mutant in ("fixed-nb", "pull-to-birth")
            else "A"
        )
        self.original_case = dict(case, version=baseline)
        self.c = C(p, self.original_case, fn)
        self.old = C(p, dict(case, version="old"), fn)
        self.history: list[dict[str, Any]] = []
        self.last: dict[str, Any] = {}
        self.qualified: list[int] = []
        if "initial" in case and "S" in case["initial"]:
            initial = case["initial"]
            self.c.mass = scalar_bits(initial["P_bits"]) if baseline != "old" else None
            self.c.e["weight"] = Q(initial["S"])
            self.c.e["reaction_sum"] = [Q(initial["U"])] * 4
            mass = Q(case["p0"]) if self.c.mass is None else self.c.mass
            value = r(
                r(r(mass * Q(case["b0"])) + Q(initial["U"])) / r(mass + Q(initial["S"]))
            )
            self.c.values["baseline"] = vec([value] * 4)
            self.c.exact = dict(P=mass, S=Q(initial["S"]), U=Q(initial["U"]))
        mass = Q(case["p0"]) if self.c.mass is None else self.c.mass
        self.enclosure = dict(
            P=Interval.point(mass),
            S=Interval.point(self.c.e["weight"]),
            U=Interval.point(self.c.e["reaction_sum"][0]),
        )

    def step(self, event: dict[str, Any], measure_bounds: bool = True) -> None:
        self.last = dict(fault=None, changed=False, signals=[], bounds=None)
        ev = deepcopy(event)
        if "invalidate_prior_ordinals" in ev:
            invalid = ev["invalidate_prior_ordinals"]
            if self.mutant == "subtract-only":
                for old in self.history:
                    if old["ordinal"] in invalid:
                        self.c.e["reaction_sum"] = [
                            r(v - Q(old["a"])) for v in self.c.e["reaction_sum"]
                        ]
            else:
                rebuilt = CResearch(self.p, self.case, self.fn)
                for old in self.history:
                    old = deepcopy(old)
                    if old["ordinal"] in invalid:
                        old["eligible"] = False
                    rebuilt.step(old, False)
                self.c, self.old, self.qualified = (
                    rebuilt.c,
                    rebuilt.old,
                    rebuilt.qualified,
                )
                self.enclosure = rebuilt.enclosure
        self.history.append(deepcopy(event))
        if self.case.get("negative_control") == "C-frozen":
            return
        if (
            self.mutant in ("missing-as-zero", "unknown-as-zero")
            and ev["source"] == "missing_A"
        ):
            ev.update(a=0, eligible=True)
        if self.mutant == "duplicate-mass" and "contact" in ev:
            ev["eligible"] = True
        if self.mutant == "time-learning" and not ev["eligible"]:
            ev.update(a=0.5, eligible=True)
        if not ev["eligible"]:
            return
        if isinstance(ev["a"], str):
            if self.mutant == "clip-continue":
                ev["a"] = {"nan": 0.0, "inf": 1.0, "-inf": -1.0}[ev["a"]]
            else:
                self.last["fault"] = "NonFinite"
                return
        if self.mutant == "sign-screen" and ev["a"] < 0:
            return
        if self.mutant == "x-as-a":
            ev["a"] = -0.5 if ev["a"] > 0 else 0.5
        if self.mutant == "wall-clock":
            ev["running_ns"] = ev["time_ns"]
        if self.mutant == "old-b-segment":
            ev["duration_baseline"] = self.old.values["baseline"]["connection"]
        # The elementary-function provider is an external read-only service,
        # not recurrence state and not a cloneable subprocess/file handle.
        previous = deepcopy(self.c, {id(self.c.fn): self.c.fn})
        if self.mutant == "time-reset":
            self.c.values["baseline"] = vec([Q(self.case["b0"])] * 4)
        try:
            self.c.step(ev)
            if self.c.mass is not None and self.c.mass < Q(1, 2**1022):
                if self.mutant == "underflow-fault":
                    raise ArithmeticError("Underflow")
                if self.mutant == "flush-subnormal":
                    self.c.mass = Q(0)
        except ArithmeticError as fault:
            self.c = previous
            self.last["fault"] = str(fault)
            return
        if self.mutant == "old-b-segment":
            try:
                self.old.step(ev)
            except ArithmeticError:
                pass
        self.qualified.append(ev["ordinal"])
        rho = Interval.point(self.c.last_rho)
        point = Interval.point
        if self.version == "B":
            self.enclosure["S"] = self.enclosure["S"] + point(1)
            self.enclosure["U"] = self.enclosure["U"] + point(Q(ev["a"]))
        else:
            if self.c.mass is not None:
                self.enclosure["P"] = rho * self.enclosure["P"]
            self.enclosure["S"] = rho * self.enclosure["S"] + point(1)
            self.enclosure["U"] = rho * self.enclosure["U"] + point(Q(ev["a"]))
        self.last["changed"] = True
        before_values = [
            previous.values[k][n]
            for k in ("baseline", "gain", "recovery_per_s")
            for n in ("connection", "empathy", "curiosity", "achievement")
        ]
        after_values = [
            self.c.values[k][n]
            for k in ("baseline", "gain", "recovery_per_s")
            for n in ("connection", "empathy", "curiosity", "achievement")
        ]
        jump = (
            scalar_bits(self.p["jump_notice_bits"])
            if "jump_notice_bits" in self.p
            else Q(self.p["jump_notice"])
        )
        if any(abs(r(a - b)) > jump for a, b in zip(after_values, before_values)):
            self.last["signals"] = ["Jump"]
        if self.mutant == "alarm-reset" and self.last["signals"]:
            self.c = C(self.p, self.original_case, self.fn)
        if measure_bounds:
            if self.mutant == "old-nb-bound":
                previous.mass = None
            self.last["bounds"] = bounds(previous, ev, self.c.last_rho)

    def state(self) -> dict[str, Any]:
        return self.c.state()

    def metrics(self) -> dict[str, Any]:
        c = self.c
        p = Q(self.case["p0"]) if c.mass is None else c.mass
        mass = c.e["weight"]
        if self.version == "B":
            p, mass = Q(self.case["p0"]), c.lifetime_mass
        denominator = r(p + mass)
        real = dict(c.exact)
        if self.c.mass is None:
            real["P"] = Q(self.case["p0"])
        if self.version == "B":
            real = dict(P=Q(self.case["p0"]), S=c.lifetime_mass, U=c.lifetime_sum)
        exact_mass = real["P"] + real["S"]
        image = (
            self.enclosure["P"] * Interval.point(Q(self.case["b0"]))
            + self.enclosure["U"]
        ) / (self.enclosure["P"] + self.enclosure["S"])
        real_b = (real["P"] * Q(self.case["b0"]) + real["U"]) / exact_mass
        stored_b = c.values["baseline"]["connection"]
        return dict(
            P=p,
            S=c.e["weight"],
            U=c.e["reaction_sum"],
            rho=c.last_rho,
            beta=r(p / denominator),
            alpha=r(Q(1) / denominator),
            real_reference=real,
            real_beta=real["P"] / exact_mass,
            baseline_enclosure=[image.low, image.high],
            stored_in_enclosure=image.low <= stored_b <= image.high,
            real_in_enclosure=image.low <= real_b <= image.high,
            baseline_error_exact=str(stored_b - real_b),
            qualified_ordinals=self.qualified,
            lifetime_mass=c.lifetime_mass,
            lifetime_sum=c.lifetime_sum,
        )
