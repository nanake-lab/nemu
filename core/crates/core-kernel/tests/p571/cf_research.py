"""Frozen §6 composition of the independently written C and F research flows."""

from copy import deepcopy
from typing import Any, Callable

from arithmetic import Q
from c_research import CResearch
from f_research import FResearch


class CFResearch:
    def __init__(
        self,
        cp: dict[str, Any],
        fp: dict[str, Any],
        case: dict[str, Any],
        fn: Callable[[str, Q], Q],
    ) -> None:
        self.case = case
        version = case["version"]
        self.mutant = (
            ""
            if version
            in ("oldC-oldF", "A-oldF", "oldC-A1", "A-A1", "B-A1", "A-A2", "A-B")
            else version
        )
        c_version, f_version = ("A", "A1") if self.mutant else version.split("-")
        self.c = CResearch(
            cp, dict(case, version="old" if c_version == "oldC" else c_version), fn
        )
        self.f = FResearch(
            fp,
            dict(
                case,
                version="old" if f_version == "oldF" else f_version,
                theta=dict(b=[0] * 4, g=[1] * 4, lambda_bits=["3fe62e42fefa39ef"] * 4),
            ),
            fn,
        )
        self.last: dict[str, Any] = {}

    def step(self, event: dict[str, Any], measure_bounds: bool = True) -> None:
        ce, fe = deepcopy(event["c"]), deepcopy(event["f"])
        self.last = dict(c=None, f=None)
        if ce is not None:
            if self.mutant == "x-as-a":
                ce["a"] = float(self.f.x[self.case["coordinate"]])
            if self.mutant == "clip-continue" and isinstance(ce["a"], str):
                ce["a"] = {"nan": 0.0, "inf": 1.0, "-inf": -1.0}[ce["a"]]
            self.c.step(ce, measure_bounds)
            self.last["c"] = self.c.last
            if self.c.last["fault"]:
                return
        # A C change needs the old-theta projection before publishing new
        # theta. F-only steps project inside F iff the role is admitted;
        # rejected B repeats must not introduce an otherwise absent anchor.
        if ce is not None:
            self.f.project(event["time_ns"])
        if fe is not None:
            if self.mutant == "revisit-current" and fe["op"] == "revisit":
                fe.update(op="update", target_mode="current_theta", a=fe["target"])
            if self.mutant == "new-old-state" and fe["op"] == "new":
                fe["target_mode"] = "direct"
            self.f.step(fe)
            self.last["f"] = self.f.last
            if (
                self.mutant == "internal-C-mass"
                and ce is not None
                and fe["op"] == "revisit"
            ):
                extra = dict(ce, id=ce["id"] + "-internal", ordinal=ce["ordinal"])
                self.c.step(extra, measure_bounds)
        if ce is not None and self.case.get("negative_control") != "C-frozen":
            names = ("connection", "empathy", "curiosity", "achievement")
            for key, field in (
                ("b", "baseline"),
                ("g", "gain"),
                ("lambda", "recovery_per_s"),
            ):
                self.f.theta[key] = [self.c.c.values[field][n] for n in names]
            if self.mutant == "recompute-targets":
                for root, old in self.f.history.items():
                    e = dict(old, target_mode="current_theta", a=old["target"])
                    self.f.targets[root] = self.f.target(e)
                    self.f.history[root]["recorded_target"] = list(self.f.targets[root])
            if self.mutant == "C-clears-x" or (
                self.mutant == "alarm-clears" and self.c.last["signals"]
            ):
                self.f.x = [Q(0)] * self.f.size
            if self.mutant == "alarm-clears" and self.c.last["signals"]:
                self.c.c = type(self.c.c)(self.c.p, self.c.original_case, self.c.fn)

    def state(self) -> dict[str, Any]:
        return dict(c=self.c.state(), f=self.f.state())
