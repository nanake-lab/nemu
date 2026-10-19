"""Autonomous-action timing: non-explosion, thinning, minimum interval.

Spec references: F §11 (λ(s) = g(s)Λ(...), survival exp(−∫λ), Pr(lim T_n = ∞) = 1,
"λ 必须可测、局部可积"), F §16.14, O §17.3 (λ_sched,max, minimum interval), S §20 (thinning,
minimum start interval), SYS10.
"""

from __future__ import annotations

import math
import random

from common import Check, Result

SEED = 20261025


def check_explosion_counterexample() -> Result:
    # History-dependent intensity λ = 2^n after the n-th action: every path is piecewise constant
    # (measurable, locally integrable), yet E[T_∞] = Σ 2^{-n} = 2 s < ∞, so T_n → T_∞ < ∞ a.s.
    rng = random.Random(SEED)
    sims, depth, horizon = 2000, 60, 10.0
    t_inf = []
    for _ in range(sims):
        t = 0.0
        for n in range(depth):
            t += rng.expovariate(2.0 ** n)
        t_inf.append(t)
    frac = sum(t <= horizon for t in t_inf) / sims
    mean = sum(t_inf) / sims
    return Result(
        "NX-1",
        "F §11 local integrability does not imply Pr(lim T_n=∞)=1 (closed by O §17.3 / S §20)",
        "counterexample",
        frac > 0.99 and abs(mean - 2.0) < 0.2,
        f"λ_n = 2^n /s: {frac:.3f} of {sims} paths start {depth} actions within {horizon:g} s; "
        f"mean T_{depth} = {mean:.3f} s (theory E[T_∞] = 2 s) → explosion with probability 1",
    )


def check_bounded_intensity_and_gap() -> Result:
    rng = random.Random(SEED + 1)
    lam_max, gap, horizon = 2.0, 0.5, 10.0
    worst = 0
    for _ in range(2000):
        t, n, last = 0.0, 0, -math.inf
        while True:
            t += rng.expovariate(lam_max)  # thinning envelope
            if t > horizon:
                break
            lam = min(lam_max, 2.0 ** n)  # same explosive rule, clipped
            gate = 1.0 if t - last >= gap else 0.0  # minimum interval as g(s) = 0
            if rng.random() < gate * lam / lam_max:
                n, last = n + 1, t
        worst = max(worst, n)
    cap = int(horizon / gap) + 1
    return Result(
        "NX-2",
        "O §17.3 λ ≤ λ_sched,max and minimum interval ⇒ N(t) ≤ ⌊t/δ⌋+1",
        "property",
        worst <= cap,
        f"2000 paths with the explosive rule clipped at λ_max={lam_max:g}/s and δ={gap:g} s: "
        f"max actions in {horizon:g} s = {worst} ≤ {cap}",
    )


def _lam(s: float) -> float:
    gate = 0.0 if 1.0 <= s < 1.5 else 1.0
    return gate * (0.2 + 0.8 * math.sin(s) ** 2)


def _cum(s: float, steps: int = 4000) -> float:
    h = s / steps
    return h * (0.5 * _lam(0.0) + sum(_lam(i * h) for i in range(1, steps)) + 0.5 * _lam(s))


def check_thinning_survival() -> Result:
    rng = random.Random(SEED + 2)
    lam_max, samples, cap = 1.0, 20000, 6.0
    firsts = []
    for _ in range(samples):
        t = 0.0
        while True:
            t += rng.expovariate(lam_max)
            if t > cap or rng.random() < _lam(t) / lam_max:
                break
        firsts.append(t)
    worst_z = 0.0
    for s in (0.25, 0.5, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0):
        emp = sum(f > s for f in firsts) / samples
        theo = math.exp(-_cum(s))
        sd = math.sqrt(theo * (1 - theo) / samples)
        worst_z = max(worst_z, abs(emp - theo) / sd)
    return Result(
        "TH-1",
        "F §11 Pr(Δ>s) = exp(−∫λ) realised by S §20 bounded thinning with gate g(s)",
        "property",
        worst_z < 4.5,
        f"{samples} first-arrival samples, λ(s)=g(s)(0.2+0.8sin²s), gate closed on [1,1.5): "
        f"max |empirical − exp(−∫λ)| = {worst_z:.2f} standard errors",
    )


def check_gap_deferral() -> Result:
    # Minimum interval δ after an action at time 0, constant λ = 1/s.
    rng = random.Random(SEED + 3)
    gap, eps, samples = 0.5, 0.01, 20000
    gate_hits = defer_hits = 0
    for _ in range(samples):
        raw = rng.expovariate(1.0)
        gated = gap + rng.expovariate(1.0)  # g = 0 on (0, δ): memoryless restart at δ
        deferred = max(raw, gap)  # draws inside (0, δ) are postponed to δ
        gate_hits += gated <= gap + eps
        defer_hits += deferred <= gap + eps
    theory_gate = 1 - math.exp(-eps)
    return Result(
        "TH-2",
        "F §11 survival law with S §20 minimum interval: gate (rejection) vs deferral",
        "counterexample",
        abs(gate_hits / samples - theory_gate) < 0.005 and defer_hits / samples > 0.35,
        f"P(T ≤ δ+0.01): gate {gate_hits / samples:.4f} (exp(−∫gλ) predicts {theory_gate:.4f}); "
        f"deferral {defer_hits / samples:.4f} — deferring breaks the stated survival function",
    )


CHECKS: list[Check] = [
    check_explosion_counterexample,
    check_bounded_intensity_and_gap,
    check_thinning_survival,
    check_gap_deferral,
]
