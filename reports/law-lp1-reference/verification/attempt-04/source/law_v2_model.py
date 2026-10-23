"""ADR 0019 candidate B: signal-limited common discount, research only.

All persisted arithmetic uses binary64. Fractions are used ONLY by the share
apportioner to assign exact dyadic shares, never to update evidence statistics.
No production imports, clock, model, filesystem or user data.
"""
from __future__ import annotations

import math
import struct
from dataclasses import dataclass
from fractions import Fraction

from check_law import Tendency as LegacyTendency


class NumericFault(ValueError):
    """An event must fail atomically; callers must not silently skip its evidence."""


def finite(value: float) -> float:
    if not math.isfinite(value):
        raise NumericFault("NonFinite")
    return 0.0 if value == 0.0 else value


@dataclass
class Tendency(LegacyTendency):
    """a = discounted signed evidence; b = discounted absolute evidence, excludes c.

    prior is a fixed genesis parameter, not evidence. Its bits never change.
    M/U use the same actual q; teaching groups only discount these two fields.
    """

    magnitude_sum: float = 0.0
    magnitude_weight: float = 0.0

    def __post_init__(self) -> None:
        self.validate()

    def validate(self) -> None:
        for value in (self.n0, self.kappa, self.prior, self.h0, self.nh, self.a, self.b,
                      self.magnitude_sum, self.magnitude_weight):
            finite(value)
        if not (self.n0 > 0 and self.kappa >= 0 and self.prior >= 0 and self.nh > 0):
            raise NumericFault("InvalidParameters")
        if not 0 <= self.h0 < 1 or not 0 <= self.flips <= self.groups < 2**53:
            raise NumericFault("InvalidParametersOrCounter")
        if self.fixed_rho is not None and not 0 < finite(self.fixed_rho) <= 1:
            raise NumericFault("InvalidRetention")
        if self.b < abs(self.a) or self.last_sign not in (-1, 0, 1):
            raise NumericFault("InvalidState")
        if self.magnitude_weight < 0:
            raise NumericFault("InvalidMagnitude")
        c = finite(self.n0 + self.kappa)
        denominator = finite(finite(c + self.prior) + self.b)
        if not -1 < finite((self.prior + self.a) / denominator) < 1:
            raise NumericFault("PrecisionExhausted")

    def pi(self) -> float:
        c = finite(self.n0 + self.kappa)
        numerator = finite(self.prior + self.a)
        denominator = finite(finite(c + self.prior) + self.b)
        return finite(numerator / denominator)

    def add(self, w: float, s: float, mu: float = 0.0, projection: float | None = None) -> float:
        """Full discount + insertion. Return actual q for diagnostic closed sums.

        Strict LP1 is checked against the pre-event pi, without tolerance.
        Positive input whose result cannot be represented is an explicit fault;
        no clamp, nextafter repair, minimum weight, fallback q=1, or retry.
        """
        self.validate()
        if not 0 <= finite(w) or not -1 <= finite(s) <= 1:
            raise NumericFault("InvalidEvidence")
        if not 0 <= finite(mu) <= 1:
            raise NumericFault("InvalidMagnitudeInput")
        if projection is not None:
            finite(projection)
        if self.groups + 1 >= 2**53:
            raise NumericFault("CounterExhausted")
        rho = finite(self.rho())
        if not 0 < rho <= 1:
            raise NumericFault("PrecisionExhaustedRetention")
        h = finite(1.0 - rho)
        before = self.pi()
        c = finite(self.n0 + self.kappa)
        anchor = finite(self.prior + c)
        drift = finite(self.prior - finite(before * anchor))
        signal = finite(w * abs(s))
        signed = finite(w * s)
        sign = (s > 0) - (s < 0)
        if w > 0 and s != 0 and signal == 0:
            raise NumericFault("UnderflowEvidence")
        q = rho
        if signal == 0:
            q = 1.0
        elif sign * drift < 0:
            gain = finite(signal * finite(1.0 - sign * before))
            opposing_drift = finite(h * abs(drift))
            # Reserve at least half of the incoming directional increment.
            # Attempt 01 used gain + opposing_drift: its O(w^2) increment
            # needlessly exhausted precision on ordinary small binary64 weights.
            divisor = finite(gain + finite(2.0 * opposing_drift))
            if gain <= 0 or divisor <= 0:
                raise NumericFault("UnderflowGain")
            fraction = finite(gain / divisor)
            discount = finite(h * fraction)
            q = finite(1.0 - discount)
        if not rho <= q <= 1:
            raise NumericFault("PrecisionExhaustedRetention")
        new_a = finite(finite(q * self.a) + signed)
        new_b = finite(finite(q * self.b) + signal)
        if new_b < abs(new_a):
            raise NumericFault("PrecisionExhaustedEvidence")
        after = finite(finite(self.prior + new_a) / finite(anchor + new_b))
        if not -1 < after < 1:
            raise NumericFault("PrecisionExhausted")
        if signal > 0 and not sign * (after - before) > 0:
            raise NumericFault("PrecisionExhaustedLP1")
        if signal == 0 and struct.pack(">d", before) != struct.pack(">d", after):
            raise NumericFault("PrecisionExhaustedZero")
        new_m = finite(q * self.magnitude_sum)
        new_u = finite(q * self.magnitude_weight)
        if projection is not None:
            new_m = finite(new_m + finite(mu * projection))
            new_u = finite(new_u + mu)
        # Commit only after every computation and postcondition succeeded.
        self.a, self.b = new_a, new_b
        self.magnitude_sum, self.magnitude_weight = new_m, new_u
        if sign:
            if self.last_sign and sign != self.last_sign:
                self.flips += 1
            self.last_sign = sign
        self.groups += 1
        return q


def exact_shares(pis: list[float], supports: list[float]) -> list[float]:
    """Canonical-ID order; largest remainder on 2^-52 lattice, ties by ID order.

    Every populated side sums to exactly 1 as real dyadics AND ordered binary64.
    A positive quota rounded to zero is a fault, not discarded evidence.
    This is an explicit proposed numeric change from independently rounded ratios.
    """
    if len(pis) != len(supports):
        raise NumericFault("InvalidShares")
    for p, s in zip(pis, supports, strict=True):
        if not -1 < finite(p) < 1 or not -1 <= finite(s) <= 1:
            raise NumericFault("InvalidShares")
    units = 2**52
    out = [0.0] * len(pis)
    for side in (1, -1):
        indices = [i for i, s in enumerate(supports) if (s > 0) - (s < 0) == side]
        if not indices:
            continue
        weights = [Fraction.from_float((1.0 + pis[i]) / 2.0) for i in indices]
        total = sum(weights, Fraction())
        quotas = [weight * units / total for weight in weights]
        counts = [q.numerator // q.denominator for q in quotas]
        order = sorted(range(len(indices)), key=lambda j: (-(quotas[j] - counts[j]), indices[j]))
        for j in order[:units - sum(counts)]:
            counts[j] += 1
        if any(count == 0 for count in counts):
            raise NumericFault("PrecisionExhaustedShare")
        for i, count in zip(indices, counts, strict=True):
            out[i] = count / units
    return out


def bits(t: LegacyTendency) -> str:
    """Serialization boundary for the complete scalar state, not just pi equality."""
    floats = (t.n0, t.kappa, t.prior, t.h0, t.nh, t.a, t.b)
    fixed = "none" if t.fixed_rho is None else t.fixed_rho.hex()
    moments = (t.magnitude_sum, t.magnitude_weight) if isinstance(t, Tendency) else (0., 0.)
    return struct.pack(">9dQQb", *floats, *moments, t.groups, t.flips, t.last_sign).hex() + ":" + fixed
