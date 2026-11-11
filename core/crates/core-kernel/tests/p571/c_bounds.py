"""Independent interval evaluation of ADR0035 §3.4 and V2-96 identities.

Endpoints are exact rationals of binary64 numbers. Each algebraic operation
rounds once then widens to the two adjacent floats as the registered numeric
enclosure requires. No DUT bounds or recurrence outputs are used as inputs.
"""

import math
from dataclasses import dataclass
from typing import Any, Callable

from arithmetic import Q, rounded as r
from c_oracle import C


def neighbour(x: Q, up: bool) -> Q:
    value = math.nextafter(float(x), math.inf if up else -math.inf)
    if not math.isfinite(value):
        raise ArithmeticError("nonfinite enclosure")
    return Q(value)


@dataclass(frozen=True)
class Interval:
    low: Q
    high: Q

    @staticmethod
    def point(x: Q | int) -> "Interval":
        return Interval(Q(x), Q(x))

    @staticmethod
    def enclose(low: Q, high: Q) -> "Interval":
        return Interval(neighbour(r(low), False), neighbour(r(high), True))

    def __add__(self, b: "Interval") -> "Interval":
        return self.enclose(self.low + b.low, self.high + b.high)

    def __sub__(self, b: "Interval") -> "Interval":
        return self.enclose(self.low - b.high, self.high - b.low)

    def __mul__(self, b: "Interval") -> "Interval":
        products = [a * c for a in (self.low, self.high) for c in (b.low, b.high)]
        return self.enclose(min(products), max(products))

    def __truediv__(self, b: "Interval") -> "Interval":
        if b.low <= 0 <= b.high:
            raise ArithmeticError("zero in denominator interval")
        quotients = [a / c for a in (self.low, self.high) for c in (b.low, b.high)]
        return self.enclose(min(quotients), max(quotients))

    def magnitude(self) -> Q:
        return max(abs(self.low), abs(self.high))

    def width(self) -> Q:
        return neighbour(r(self.high - self.low), True)

    def clip(self, low: Q, high: Q) -> "Interval":
        return Interval(max(low, min(high, self.low)), max(low, min(high, self.high)))

    def elementary(self, name: str, fn: Callable[[str, Q], Q]) -> "Interval":
        if name == "sqrt" and self.low <= 0:
            raise ArithmeticError("nonpositive square root enclosure")
        a, b = fn(name, self.low), fn(name, self.high)
        if name == "exp":
            a, b = neighbour(a, False), neighbour(b, True)
        return self.enclose(a, b)


P = Interval.point


def moment(
    n: Q,
    prior: Interval,
    total: Q,
    mass: Q,
    current: Q,
    observation: Interval,
    added: Q,
    rho: Q,
) -> tuple[Interval, Q]:
    n_i, s, u, x, a, d = P(n), P(total), P(mass), P(current), P(added), P(rho)
    defect = (n_i * prior + s) - x * (n_i + u)
    identity = (n_i * (P(1) - d) * (prior - x) + a * (observation - x) + d * defect) / (
        n_i + (d * u + a)
    )
    image = (n_i * prior + (d * s + a * observation)) / (n_i + (d * u + a))
    error = (image - (x + identity)).magnitude()
    limit = neighbour((P(identity.magnitude()) + P(error)).high, True)
    return image, limit


def bounds(before: C, event: dict[str, Any], rho: Q) -> dict[str, object]:
    e, p = before.e, before.p
    a = Q(event["a"])
    channels = ("connection", "empathy", "curiosity", "achievement")
    ranges = (
        [Q(2)] * 4
        + [(P(p["gain_max"]) - P(p["gain_min"])).high] * 4
        + [(P(p["recovery_max"]) - P(p["recovery_min"])).high] * 4
    )
    single = list(ranges)
    fallback = [False] * 12
    for j, name in enumerate(channels):
        b = before.values["baseline"][name]
        try:
            if before.mass is None:
                _, limit = moment(
                    p["p0"],
                    P(p["b0"]),
                    e["reaction_sum"][j],
                    e["weight"],
                    b,
                    P(a),
                    Q(1),
                    rho,
                )
            else:
                mass, s, w, d = (
                    P(before.mass),
                    P(e["reaction_sum"][j]),
                    P(e["weight"]),
                    P(rho),
                )
                x, prior = P(b), P(p["b0"])
                defect = (mass * prior + s) - x * (mass + w)
                identity = (d * defect + (P(a) - x)) / (d * (mass + w) + P(1))
                image = ((d * mass) * prior + (d * s + P(a))) / (
                    (d * mass) + (d * w + P(1))
                )
                error = (image - (x + identity)).magnitude()
                limit = neighbour((P(identity.magnitude()) + P(error)).high, True)
            single[j] = limit
        except ArithmeticError:
            fallback[j] = True
        try:
            residual = P(a) - P(b)
            image, delta = moment(
                p["variance_weight"],
                P(p["sigma_prior"]) * P(p["sigma_prior"]),
                e["square_sum"][j],
                e["weight"],
                e["variance"][j],
                residual * residual,
                Q(1),
                rho,
            )
            old_root = P(e["variance"][j]).elementary("sqrt", before.fn)
            new_root = image.elementary("sqrt", before.fn)
            k = P(p["gain_prior"]) * P(p["sigma_prior"])
            after = (k / new_root).clip(p["gain_min"], p["gain_max"])
            previous = (k / old_root).clip(p["gain_min"], p["gain_max"])
            slope = k * P(delta) / (new_root * old_root * (new_root + old_root))
            error = (previous - P(before.values["gain"][name])).magnitude()
            single[4 + j] = neighbour((slope + P(error) + P(after.width())).high, True)
        except ArithmeticError:
            fallback[4 + j] = True
        try:
            sign = (a > b) - (a < b)
            added, observation = Q(0), Q(0)
            if int(e["run_sign"][j]) and sign != int(e["run_sign"][j]):
                duration = int(event["running_ns"]) - int(e["run_start_ns"][j])
                if duration > 0:
                    added, observation = Q(1), before.fn("log", r(Q(duration) / 10**9))
            image, delta = moment(
                p["duration_weight"],
                P(before.fn("log", p["tau_prior_s"])),
                e["duration_log_sum"][j],
                e["duration_mass"][j],
                e["duration_log_mean"][j],
                P(observation),
                added,
                rho,
            )
            after = (P(1) / image.elementary("exp", before.fn)).clip(
                p["recovery_min"], p["recovery_max"]
            )
            previous = (
                P(1) / P(e["duration_log_mean"][j]).elementary("exp", before.fn)
            ).clip(p["recovery_min"], p["recovery_max"])
            slope = (P(0) - P(min(image.low, e["duration_log_mean"][j]))).elementary(
                "exp", before.fn
            )
            error = (previous - P(before.values["recovery_per_s"][name])).magnitude()
            single[8 + j] = neighbour(
                (slope * P(delta) + P(error) + P(after.width())).high, True
            )
        except ArithmeticError:
            fallback[8 + j] = True
    return dict(
        range=ranges,
        single_event=single,
        range_only=fallback,
        limits=[min(a, b) for a, b in zip(ranges, single)],
    )
