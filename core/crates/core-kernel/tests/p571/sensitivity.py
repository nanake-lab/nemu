"""Frozen standalone arithmetic and private-boundary sensitivity models."""

import math
from typing import Any, Callable

from arithmetic import Q, rounded as r, bits
from c_research import CResearch
from f_research import scalar_bits
from matrix_support import rational_json


def arithmetic(case: dict[str, Any]) -> dict[str, Any]:
    initial = case["initial"]
    prior = scalar_bits(initial["P_bits"])
    exact = prior * Q(initial["rho"])
    rounded = r(exact)
    fault = None
    if case["version"] == "underflow-fault" and exact < Q(1, 2**1022):
        rounded, fault = prior, "Underflow"
    if case["version"] == "flush-subnormal" and abs(rounded) < Q(1, 2**1022):
        rounded = Q(0)
    return dict(
        prior_bits=bits(prior),
        exact=str(exact),
        rounded_bits=bits(rounded),
        fault=fault,
        nominal_expected_bits=bits(r(exact)),
    )


def candidate(
    resource: dict[str, Any], case: dict[str, Any], fn: Callable[[str, Q], Q]
) -> dict[str, Any]:
    step = case["first_normal_step"]
    ccase = dict(p0=step["p0"], b0=step["b0"], rho=step["rho"], version="A")
    model = CResearch(resource, ccase, fn)
    event = dict(
        id="candidate-base",
        ordinal=1,
        time_ns=step["time_ns"],
        running_ns=step["time_ns"],
        stopped_ns=0,
        a=step["a"],
        eligible=True,
        source="other_person",
        stops=[],
    )
    model.step(event)
    bound = model.last["bounds"]["limits"][case["coordinate"]]
    point = case["point"]
    if point == "equal":
        value = float(bound)
    elif point == "next_up":
        value = math.nextafter(float(bound), math.inf)
    else:
        value = {"nan": math.nan, "inf": math.inf, "-inf": -math.inf}[point]
    if point == "next_up" and not math.isfinite(value):
        return dict(
            status="not_run",
            reason="finite next_up unavailable",
            bound=rational_json(bound),
        )
    expected = point == "equal"
    version = case["version"]
    actual = math.isfinite(value) and abs(value) <= float(bound)
    if version == "clip":
        actual = True
    elif version == "ignore-nonfinite" and not math.isfinite(value):
        actual = True
    elif version == "greater-equal" and value == float(bound):
        actual = False
    return dict(
        status="passed" if expected == actual else "failed",
        expected_accept=expected,
        actual_accept=actual,
        point=point,
        bound=rational_json(bound),
        candidate_bits=bits(value),
    )


class BaselinePath:
    """The explicitly registered 07-10 b-path, apart from full C fault state."""

    def __init__(self, case: dict[str, Any]) -> None:
        initial = case.get("initial", {})
        self.version = case["version"]
        self.p = (
            scalar_bits(initial["P_bits"])
            if "P_bits" in initial and self.version not in ("old", "B")
            else Q(case["p0"])
        )
        self.s, self.u = Q(initial.get("S", 0)), Q(initial.get("U", 0))
        self.b0, self.rho = Q(case["b0"]), Q(case["rho"])
        self.exact = dict(P=self.p, S=self.s, U=self.u)
        self.b = r(r(r(self.p * self.b0) + self.u) / r(self.p + self.s))
        self.fault: str | None = None

    def step(self, e: dict[str, Any]) -> None:
        a = Q(e["a"])
        # §3.1 old fixed n_b / P07-A common discount / P07-B lifetime mean.
        # The separately reported b-path must not silently run A for all labels.
        discount = Q(1) if self.version == "B" else self.rho
        prior = self.p if self.version in ("old", "B") else r(discount * self.p)
        self.fault = None
        if self.version == "underflow-fault" and prior < Q(1, 2**1022):
            self.fault = "Underflow"
            return
        if self.version == "flush-subnormal" and prior < Q(1, 2**1022):
            prior = Q(0)
        self.s = r(r(discount * self.s) + 1)
        self.p = prior
        self.u = r(r(discount * self.u) + a)
        self.b = r(r(r(self.p * self.b0) + self.u) / r(self.p + self.s))
        self.exact = dict(
            P=self.exact["P"]
            if self.version in ("old", "B")
            else self.rho * self.exact["P"],
            S=discount * self.exact["S"] + 1,
            U=discount * self.exact["U"] + a,
        )

    def state(self) -> dict[str, Any]:
        return dict(
            P=self.p, S=self.s, U=self.u, b=self.b, exact=self.exact, fault=self.fault
        )
