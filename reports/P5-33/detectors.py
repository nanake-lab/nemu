"""Synthetic, sign-only detector candidates; never imported by production code."""
from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import Protocol


def ordered_sum(values: list[float]) -> float:
    total = 0.0
    for value in values:
        total = total + value
    if not math.isfinite(total):
        raise ValueError("NonFiniteDetector")
    return total


class Detector(Protocol):
    def prediction(self) -> float: ...
    def observe(self, support: float) -> float: ...
    def state(self) -> object: ...


@dataclass
class Flips:
    """Original pre-group h. Beta diagnostic forecast uses nominal discount."""
    groups: int = 0
    flips: int = 0
    last: int = 0
    positive: float = 0.0
    negative: float = 0.0

    def prediction(self) -> float:
        return (1.0 + self.positive) / ((2.0 + self.positive) + self.negative)

    def observe(self, support: float) -> float:
        h = (1.0 + self.flips) / (10.0 + self.groups)
        sign = (support > 0) - (support < 0)
        if sign:
            self.positive = (1.0 - h) * self.positive + float(sign > 0)
            self.negative = (1.0 - h) * self.negative + float(sign < 0)
            self.flips += int(self.last != 0 and sign != self.last)
            self.last = sign
        self.groups += 1
        return h

    def state(self) -> object:
        return (self.groups, self.flips, self.last, self.positive.hex(), self.negative.hex())


@dataclass
class Run:
    length: int
    mass: float
    yes: int
    no: int


@dataclass
class ChangePoint:
    """Reset BEFORE y_t; h=P(boundary before y_t | y_1:t), not end-reset mass.

    Retain top K posterior nodes, ties shorter run first; sum in run order.
    Discarded mass is reported, never silently claimed to be exact BOCPD.
    """
    hazard: float = 0.001
    cap: int = 128
    runs: list[Run] = field(default_factory=lambda: [Run(0, 1.0, 0, 0)])
    dropped_total: float = 0.0
    dropped_max: float = 0.0
    observations: int = 0

    def prediction(self) -> float:
        mean = ordered_sum([r.mass * ((1.0 + r.yes) / (2.0 + r.yes + r.no))
                            for r in self.runs])
        return self.hazard * 0.5 + (1.0 - self.hazard) * mean

    def observe(self, support: float) -> float:
        if support == 0:
            return self.hazard  # no Bernoulli observation, no detector update
        y = int(support > 0)
        masses: list[float] = []
        for r in self.runs:
            likelihood = (1.0 + (r.yes if y else r.no)) / (2.0 + r.yes + r.no)
            masses.append(((1.0 - self.hazard) * r.mass) * likelihood)
        reset = self.hazard * 0.5
        evidence = reset + ordered_sum(masses)
        h = reset / evidence
        nodes = [Run(1, h, y, 1-y)]
        nodes.extend(Run(r.length+1, m/evidence, r.yes+y, r.no+1-y)
                     for r, m in zip(self.runs, masses, strict=True))
        # The initial empty run and the first reset are the same sufficient state.
        if self.observations == 0:
            nodes = [Run(1, 1.0, y, 1-y)]
        ranked = sorted(nodes, key=lambda r: (-r.mass, r.length))
        removed = ranked[self.cap:]
        lost = ordered_sum([r.mass for r in sorted(removed, key=lambda r: r.length)])
        self.dropped_total = self.dropped_total + lost
        self.dropped_max = max(self.dropped_max, lost)
        kept = sorted(ranked[:self.cap], key=lambda r: r.length)
        total = ordered_sum([r.mass for r in kept])
        self.runs = [Run(r.length, r.mass/total, r.yes, r.no) for r in kept]
        self.observations += 1
        if not 0 < h < 1:
            raise ValueError("DetectorPrecisionExhausted")
        return h

    def state(self) -> object:
        return (self.hazard.hex(), self.cap, self.observations,
                self.dropped_total.hex(), self.dropped_max.hex(),
                [(r.length, r.mass.hex(), r.yes, r.no) for r in self.runs])


@dataclass
class Mixture:
    """Finite discounted Beta experts with prior-weight mixing, research choice.

    Not a claim to reproduce any paper's complete DMA implementation.
    """
    rates: tuple[float, ...] = (0.01, 0.1, 0.4)
    prior: tuple[float, ...] = (0.5, 0.35, 0.15)  # E[h] = .1
    weights: list[float] = field(default_factory=lambda: [0.5, 0.35, 0.15])
    yes: list[float] = field(default_factory=lambda: [0.0, 0.0, 0.0])
    no: list[float] = field(default_factory=lambda: [0.0, 0.0, 0.0])
    switch: float = 0.01

    def priors(self) -> list[float]:
        return [(1.0-self.switch)*w + self.switch*p
                for w, p in zip(self.weights, self.prior, strict=True)]

    def means(self) -> list[float]:
        return [(1.0 + (1.0-h)*a) / ((2.0 + (1.0-h)*a) + (1.0-h)*b)
                for h, a, b in zip(self.rates, self.yes, self.no, strict=True)]

    def prediction(self) -> float:
        return ordered_sum([w*p for w, p in zip(self.priors(), self.means(), strict=True)])

    def observe(self, support: float) -> float:
        if support == 0:
            return ordered_sum([w*h for w, h in zip(self.weights, self.rates, strict=True)])
        y = int(support > 0)
        unnormalized = [w*(p if y else 1.0-p)
                        for w, p in zip(self.priors(), self.means(), strict=True)]
        total = ordered_sum(unnormalized)
        self.weights = [w/total for w in unnormalized]
        h = ordered_sum([w*r for w, r in zip(self.weights, self.rates, strict=True)])
        self.yes = [(1.0-r)*a + y for r, a in zip(self.rates, self.yes, strict=True)]
        self.no = [(1.0-r)*b + (1-y) for r, b in zip(self.rates, self.no, strict=True)]
        return h

    def state(self) -> object:
        return tuple(tuple(v.hex() for v in values) for values in
                     (self.rates, self.prior, self.weights, self.yes, self.no)) + (self.switch.hex(),)


METHODS = ("flips", "changepoint", "mixture", "fixed")


def make_detector(method: str, cap: int = 128, hazard: float = 0.001) -> Detector:
    if method == "flips":
        return Flips()
    if method == "changepoint":
        return ChangePoint(hazard=hazard, cap=cap)
    if method == "mixture":
        return Mixture()
    if method == "fixed":
        return Fixed()
    raise ValueError(method)


@dataclass
class Fixed(Flips):
    """Additional fixed-h control, same diagnostic forecast convention as Flips."""
    def observe(self, support: float) -> float:
        sign = (support > 0) - (support < 0)
        if sign:
            self.positive = 0.9*self.positive + float(sign > 0)
            self.negative = 0.9*self.negative + float(sign < 0)
        self.groups += 1
        return 0.1
