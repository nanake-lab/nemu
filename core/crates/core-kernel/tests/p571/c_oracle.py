"""Independent C research recurrence from v2 §8 / ADR0035 §3.

Only arithmetic.py and primitive libm RPC are shared. No DUT or previous oracle
imports, and no DUT trace is an input. Complete estimator fields are retained;
the exact baseline history is separate from the rounded-state recurrence.
"""

from copy import deepcopy
from typing import Any, Callable
from arithmetic import Q, rounded as r, bits


def vec(v: list[Q]) -> dict[str, Any]:
    return dict(
        schema_version=1,
        **dict(zip(("connection", "empathy", "curiosity", "achievement"), v)),
    )


def sign(x: Q) -> int:
    return (x > 0) - (x < 0)


def clamp(x: Q, lo: Q, hi: Q) -> Q:
    return max(lo, min(hi, x))


def strict_product(a: Q, b: Q) -> Q:
    value = r(a * b)
    if a and b and not value:
        raise ArithmeticError("Underflow")
    return value


class C:
    def __init__(
        self,
        resource: dict[str, Any],
        case: dict[str, Any],
        primitive: Callable[[str, Q], Q],
    ) -> None:
        self.p = {k: Q(v) for k, v in resource.items() if isinstance(v, (float, int))}
        self.p["b0"], self.p["p0"] = Q(case["b0"]), Q(case["p0"])
        self.version = case["version"]
        self.fixed_rho = (
            None if case.get("rho", "dynamic") == "dynamic" else Q(case["rho"])
        )
        self.lifetime_mass, self.lifetime_sum = Q(0), Q(0)
        self.last_rho = Q(1)
        self.fn = primitive
        logmean = r(
            r(self.p["duration_weight"] * self.fn("log", self.p["tau_prior_s"]))
            / self.p["duration_weight"]
        )
        tau = self.fn("exp", logmean)
        recovery = clamp(r(1 / tau), self.p["recovery_min"], self.p["recovery_max"])
        self.values: dict[str, Any] = dict(
            schema_version=1,
            baseline=vec([self.p["b0"]] * 4),
            gain=vec([self.p["gain_prior"]] * 4),
            recovery_per_s=vec([recovery] * 4),
        )
        self.e: dict[str, Any] = dict(
            schema_version=1,
            weight=Q(0),
            reaction_sum=[Q(0)] * 4,
            square_sum=[Q(0)] * 4,
            sigma=vec([self.p["sigma_prior"]] * 4),
            variance=[r(self.p["sigma_prior"] ** 2)] * 4,
            duration_log_sum=[Q(0)] * 4,
            duration_mass=[Q(0)] * 4,
            duration_log_mean=[logmean] * 4,
            zero_duration_count=["0"] * 4,
            tau_s=vec([tau] * 4),
            run_sign=["0"] * 4,
            run_start_ns=["0"] * 4,
            detector_segment_count="0",
            detector_changes="0",
            detector_mean=[Q(0)] * 4,
            detector_positive=[Q(0)] * 4,
            detector_negative=[Q(0)] * 4,
            event_count="0",
            h=max(self.p["change_prior"], self.p["change_min"]),
            drift_sign=["0"] * 12,
            drift_count=["0"] * 12,
        )
        self.mass = self.p["p0"] if self.version != "old" else None
        self.exact = dict(P=self.p["p0"], S=Q(0), U=Q(0))

    def step(self, event: dict[str, Any]) -> None:
        p, e = self.p, self.e
        a, t = Q(event["a"]), event.get("running_ns", event["time_ns"])
        n = int(e["event_count"]) + 1
        segments = int(e["detector_segment_count"]) + 1
        changed = False
        for j in range(4):
            if segments == 1:
                e["detector_mean"][j] = a
                continue
            variance = r(p["sigma_prior"] ** 2)
            shift = r(p["detector_shift_sigma"] * p["sigma_prior"])
            residual = r(a - e["detector_mean"][j])
            linear = r(r(shift * residual) / variance)
            penalty = r(r(shift * shift) / r(2 * variance))
            for key, val in [
                ("detector_positive", linear),
                ("detector_negative", -linear),
            ]:
                e[key][j] = max(Q(0), r(e[key][j] + r(val - penalty)))
                changed |= e[key][j] >= p["detector_threshold"]
            e["detector_mean"][j] = r(e["detector_mean"][j] + r(residual / segments))
        if changed:
            e["detector_changes"] = str(int(e["detector_changes"]) + 1)
            segments = 1
            e["detector_mean"], e["detector_positive"], e["detector_negative"] = (
                [a] * 4,
                [Q(0)] * 4,
                [Q(0)] * 4,
            )
        e["detector_segment_count"] = str(segments)
        h = max(
            p["change_min"],
            r(
                r(
                    r(p["change_prior_weight"] * p["change_prior"])
                    + int(e["detector_changes"])
                )
                / r(p["change_prior_weight"] + n)
            ),
        )
        rho = r(1 - h)
        if self.fixed_rho is not None:
            rho, h = self.fixed_rho, r(1 - self.fixed_rho)
        self.last_rho = rho
        self.lifetime_mass = r(self.lifetime_mass + 1)
        self.lifetime_sum = r(self.lifetime_sum + a)
        e["weight"] = r(strict_product(rho, e["weight"]) + 1)
        if self.mass is not None and self.version != "B":
            self.mass = r(rho * self.mass)
        old = deepcopy(self.values)
        new_b, new_g, new_l, sigma, tau = [], [], [], [], []
        channels = ("connection", "empathy", "curiosity", "achievement")
        for j, name in enumerate(channels):
            residual = r(a - old["baseline"][name])
            e["square_sum"][j] = r(
                strict_product(rho, e["square_sum"][j])
                + strict_product(residual, residual)
            )
            e["reaction_sum"][j] = r(r(rho * e["reaction_sum"][j]) + a)
            mass = p["p0"] if self.mass is None else self.mass
            new_b.append(
                r(r(r(mass * p["b0"]) + e["reaction_sum"][j]) / r(mass + e["weight"]))
            )
            if self.version == "B":
                new_b[-1] = r(
                    r(r(p["p0"] * p["b0"]) + self.lifetime_sum)
                    / r(p["p0"] + self.lifetime_mass)
                )
            variance = r(
                r(
                    r(p["variance_weight"] * r(p["sigma_prior"] ** 2))
                    + e["square_sum"][j]
                )
                / r(p["variance_weight"] + e["weight"])
            )
            e["variance"][j] = variance
            sigma.append(self.fn("sqrt", variance))
            new_g.append(
                clamp(
                    r(r(p["gain_prior"] * p["sigma_prior"]) / sigma[-1]),
                    p["gain_min"],
                    p["gain_max"],
                )
            )
            e["duration_log_sum"][j] = strict_product(rho, e["duration_log_sum"][j])
            e["duration_mass"][j] = strict_product(rho, e["duration_mass"][j])
            if "duration_baseline" in event:
                residual = r(a - Q(event["duration_baseline"]))
            previous_sign, current_sign = int(e["run_sign"][j]), sign(residual)
            if previous_sign and current_sign != previous_sign:
                duration = t - int(e["run_start_ns"][j])
                if duration == 0:
                    e["zero_duration_count"][j] = str(
                        int(e["zero_duration_count"][j]) + 1
                    )
                else:
                    e["duration_log_sum"][j] = r(
                        e["duration_log_sum"][j]
                        + self.fn("log", r(Q(duration) / 10**9))
                    )
                    e["duration_mass"][j] = r(e["duration_mass"][j] + 1)
            if previous_sign != current_sign:
                e["run_sign"][j], e["run_start_ns"][j] = (
                    str(current_sign),
                    str(t if current_sign else 0),
                )
            z = r(
                r(
                    r(p["duration_weight"] * self.fn("log", p["tau_prior_s"]))
                    + e["duration_log_sum"][j]
                )
                / r(p["duration_weight"] + e["duration_mass"][j])
            )
            e["duration_log_mean"][j] = z
            tau.append(self.fn("exp", z))
            new_l.append(clamp(r(1 / tau[-1]), p["recovery_min"], p["recovery_max"]))
        self.values = dict(
            schema_version=1,
            baseline=vec(new_b),
            gain=vec(new_g),
            recovery_per_s=vec(new_l),
        )
        e["sigma"], e["tau_s"] = vec(sigma), vec(tau)
        before = [
            old[k][name]
            for k in ("baseline", "gain", "recovery_per_s")
            for name in channels
        ]
        after = new_b + new_g + new_l
        for j, (x, y) in enumerate(zip(before, after)):
            direction = sign(r(y - x))
            e["drift_count"][j] = str(
                0
                if not direction
                else int(e["drift_count"][j]) + 1
                if direction == int(e["drift_sign"][j])
                else 1
            )
            e["drift_sign"][j] = str(direction)
        e["event_count"], e["h"] = str(n), h
        e["last_event"] = dict(
            schema_version=1,
            ordinal=str(event["ordinal"]),
            time_ns=str(event["time_ns"]),
            event_id=event["id"],
        )
        e["last_running_time"] = dict(
            schema_version=1,
            origin_ns="0",
            event_ns=str(event["time_ns"]),
            stopped_ns=str(event.get("stopped_ns", 0)),
            running_ns=str(t),
        )
        # P07 conditional common-rho real reference, derived only from this oracle.
        self.exact["P"] *= rho
        self.exact["S"] = rho * self.exact["S"] + 1
        self.exact["U"] = rho * self.exact["U"] + a

    def state(self) -> dict[str, Any]:
        d: dict[str, Any] = dict(values=self.values, estimator=self.e)
        if self.mass is not None:
            d["prior_mass"] = self.mass
        return d


def bit_tree(x: Any) -> Any:
    if isinstance(x, (Q, float)):
        return {"f64": bits(x)}
    if isinstance(x, dict):
        return {k: bit_tree(v) for k, v in x.items()}
    if isinstance(x, list):
        return [bit_tree(v) for v in x]
    return x


def first_difference(
    expected: Any, actual: Any, path: str = ""
) -> tuple[str, Any, Any] | None:
    if type(expected) is not type(actual):
        return (path, expected, actual)
    if isinstance(expected, dict):
        if expected.keys() != actual.keys():
            return (path + ".keys", sorted(expected), sorted(actual))
        for k in expected:
            d = first_difference(expected[k], actual[k], path + "." + k)
            if d:
                return d
    elif isinstance(expected, list):
        if len(expected) != len(actual):
            return (path + ".length", len(expected), len(actual))
        for j, (a, b) in enumerate(zip(expected, actual)):
            d = first_difference(a, b, f"{path}[{j}]")
            if d:
                return d
    elif expected != actual:
        return (path, expected, actual)
    return None
