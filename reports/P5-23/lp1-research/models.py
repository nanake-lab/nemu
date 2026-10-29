"""Unapproved research laws; no dependency on Rust or authoritative storage."""

from __future__ import annotations

import math
import struct
from dataclasses import dataclass


class PrecisionExhausted(ArithmeticError):
    """A finite, strictly directional result cannot be represented."""


class CapacityExhausted(ArithmeticError):
    """No non-personality candidate can be removed."""


@dataclass
class Reference:
    n0: float
    kappa: float
    prior: float = 0.0
    h0: float = 0.1
    nh: float = 10.0
    fixed_rho: float | None = None
    a: float = 0.0
    b: float = 0.0
    groups: int = 0
    flips: int = 0
    last_sign: int = 0

    def pi(self) -> float:
        return (self.prior + self.a) / (self.n0 + self.kappa + self.prior + self.b)

    def rho(self) -> float:
        if self.fixed_rho is not None:
            return self.fixed_rho
        return 1.0 - (self.h0 * self.nh + self.flips) / (self.nh + self.groups)

    def count(self, s: float) -> None:
        sign = (s > 0) - (s < 0)
        if sign:
            if self.last_sign and sign != self.last_sign:
                self.flips += 1
            self.last_sign = sign
        self.groups += 1

    def add(self, w: float, s: float) -> float:
        rho = self.rho()
        self.a = rho * self.a + w * s
        self.b = rho * self.b + w * abs(s)
        self.count(s)
        return rho

    def consistent(self) -> float:
        return (self.b + self.a) / 2

    def contrary(self) -> float:
        return (self.b - self.a) / 2

    def values(self) -> tuple[float, ...]:
        return self.a, self.b, self.pi(), self.rho()

    def bits(self) -> str:
        return struct.pack(
            ">ddddQQb", *self.values(), self.groups, self.flips, self.last_sign
        ).hex()


def validate(w: float, s: float, a: float, b: float, p: float) -> None:
    if not all(math.isfinite(x) for x in (w, s, a, b, p)):
        raise PrecisionExhausted("non-finite input or intermediate")
    if w < 0 or not -1 <= s <= 1 or b < 0:
        raise ValueError("invalid evidence")
    if not -1 < p < 1:
        raise PrecisionExhausted("pi outside the open interval")


def directional(w: float, s: float, before: float, after: float) -> None:
    if w > 0 and s != 0 and (after - before) * s <= 0:
        raise PrecisionExhausted("strict directional change not representable")


@dataclass
class MeanPreserving(Reference):
    """M: discounted evidence mass controls gain, not an autonomous pi drift."""

    mean: float | None = None

    def pi(self) -> float:
        if self.mean is None:
            return self.prior / (self.n0 + self.kappa + self.prior)
        return self.mean

    def add(self, w: float, s: float) -> float:
        before, rho = self.pi(), self.rho()
        a = rho * self.a + w * s
        discounted = rho * self.b
        v = w * abs(s)
        b = discounted + v
        denominator = self.n0 + self.kappa + self.prior + b
        q = (s > 0) - (s < 0)
        after = before + (v / denominator) * (q - before)
        validate(w, s, a, b, after)
        if not math.isfinite(denominator):
            raise PrecisionExhausted("gain denominator overflow")
        directional(w, s, before, after)
        self.a, self.b, self.mean = a, b, after
        self.count(s)
        return rho


@dataclass
class Budgeted(Reference):
    """B: shrink forgetting when its drift would oppose the incoming evidence."""

    def add(self, w: float, s: float) -> float:
        before, rho = self.pi(), self.rho()
        c = self.n0 + self.kappa + self.prior
        u, total = self.prior + self.a, c + self.b
        q, v = (s > 0) - (s < 0), w * abs(s)
        drift = self.prior * self.b - self.a * c
        if v > 0 and q * drift < 0:
            budget = v * (total - q * u)
            cost = -q * drift
            rho = 1.0 - (1.0 - rho) * (budget / (cost + budget))
        a = rho * self.a + w * s
        b = rho * self.b + v
        after = (self.prior + a) / (c + b)
        validate(w, s, a, b, after)
        if not all(math.isfinite(x) for x in (c, total, drift, c + b)):
            raise PrecisionExhausted("budget intermediate overflow")
        directional(w, s, before, after)
        self.a, self.b = a, b
        self.count(s)
        return rho


MODELS: dict[str, type[Reference]] = {
    "O": Reference,
    "M": MeanPreserving,
    "B": Budgeted,
}


def replay(
    model: type[Reference], history: list[tuple[float, float]],
    n0: float = 3.0, kappa: float = 1.0, prior: float = 0.0,
    h0: float = 0.1,
) -> Reference:
    state = model(n0=n0, kappa=kappa, prior=prior, h0=h0)
    for w, s in history:
        state.add(w, s)
    return state


def shares(pis: list[float], supports: list[float]) -> list[float]:
    """Same side and canonical list order as the immutable original reference."""
    result = [0.0] * len(pis)
    for side in (1, -1):
        indices = [i for i, s in enumerate(supports) if (s > 0) - (s < 0) == side]
        total = sum((1 + pis[i]) / 2 for i in indices)
        for i in indices:
            if total <= 0 or not math.isfinite(total):
                raise PrecisionExhausted("invalid share denominator")
            result[i] = (1 + pis[i]) / 2 / total
    return result


def capacity_choice(states: list[Reference], limit: int) -> int | None:
    if len(states) <= limit:
        return None
    candidates = [(s.b + s.prior, i) for i, s in enumerate(states) if s.pi() <= 0.5]
    if not candidates:
        raise CapacityExhausted("only personalities remain")
    return min(candidates)[1]
