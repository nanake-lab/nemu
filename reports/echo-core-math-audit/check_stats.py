"""Hoeffding radius and the α_j / α_jn error allocation (O §6.2), with Monte Carlo evidence.

Spec references: O §6.2 (radius, CI, improvement/equivalence rules, α_j = 6α/(π²j²),
α_jn = α_j/[n(n+1)]); S §15; G09; D II §3.5.
"""

from __future__ import annotations

import math
import random

from common import Check, Result

SEED = 20261021
ALPHA = 0.05


def alpha_j(j: int, alpha: float = ALPHA) -> float:
    return 6.0 * alpha / (math.pi ** 2 * j * j)


def alpha_jn(j: int, n: int, alpha: float = ALPHA) -> float:
    return alpha_j(j, alpha) / (n * (n + 1))


def radius(j: int, n: int, alpha: float = ALPHA) -> float:
    return math.sqrt(2.0 * math.log(2.0 / alpha_jn(j, n, alpha)) / n)


def check_allocation_sums() -> Result:
    jmax, nmax = 1_000_000, 1_000_000
    sum_j = math.fsum(alpha_j(j) for j in range(1, jmax + 1))
    tail_j = 6.0 * ALPHA / math.pi ** 2 / jmax  # Σ_{j>J} 1/j² < 1/J
    sum_n = nmax / (nmax + 1)  # telescoping Σ_{n≤N} 1/(n(n+1)) = N/(N+1)
    direct_n = math.fsum(1.0 / (n * (n + 1)) for n in range(1, 10001))
    ok = (sum_j <= ALPHA and ALPHA - sum_j <= tail_j + 1e-15
          and abs(direct_n - 10000 / 10001) < 1e-12 and sum_n < 1.0)
    return Result(
        "ST-1",
        "O §6.2 Σ_j Σ_n α_jn = α",
        "property",
        ok,
        f"Σ_(j≤1e6) α_j = {sum_j:.12f} (α={ALPHA}, remaining tail < {tail_j:.2e}); "
        f"Σ_(n≤N) 1/(n(n+1)) = N/(N+1) checked to N=1e4 → total family-wise budget = α",
    )


def check_radius_is_hoeffding() -> Result:
    worst = 0.0
    for j in (1, 2, 5, 10, 100, 1000):
        for n in (1, 2, 5, 10, 50, 100, 1000, 10000):
            r = radius(j, n)
            # Hoeffding for i.i.d. d_i ∈ [-1, 1] (range 2): P(|mean-μ| ≥ r) ≤ 2 exp(-2nr²/2²).
            bound = 2.0 * math.exp(-2.0 * n * r * r / 4.0)
            worst = max(worst, abs(bound / alpha_jn(j, n) - 1.0))
    return Result(
        "ST-2",
        "O §6.2 radius(j,n) = sqrt(2 log(2/α_jn)/n) is the Hoeffding inversion for range [-1,1]",
        "property",
        worst < 1e-12,
        f"max relative mismatch between 2exp(-n r²/2) and α_jn over a (j,n) grid: {worst:.2e}",
    )


def _running_means_exceed(rng: random.Random, n_max: int, thresholds: list[float]) -> bool:
    bits = rng.getrandbits(n_max)
    s = 0
    for n in range(1, n_max + 1):
        s += 1 if (bits >> (n - 1)) & 1 else -1
        if abs(s) / n > thresholds[n - 1]:
            return True
    return False


def check_fwer_monte_carlo() -> Result:
    rng = random.Random(SEED)
    reps, n_max = 4000, 400
    # Allocated procedure, j = 1 (the largest share α_1), looking after every block n = 1..n_max.
    thr_alloc = [radius(1, n) for n in range(1, n_max + 1)]
    hits_alloc = sum(_running_means_exceed(rng, n_max, thr_alloc) for _ in range(reps))
    # Naive procedure for contrast (G09): normal-approximation 95% interval re-checked after every block.
    thr_naive = [1.959964 / math.sqrt(n) for n in range(1, n_max + 1)]
    hits_naive = sum(_running_means_exceed(rng, n_max, thr_naive) for _ in range(reps))
    p_alloc, p_naive = hits_alloc / reps, hits_naive / reps
    return Result(
        "ST-3",
        "O §6.2 sequential looks covered by α_jn; G09 (repeated looks must not inflate error)",
        "property",
        p_alloc <= alpha_j(1) and p_naive > ALPHA,
        f"Rademacher null (μ=0), {reps} streams × {n_max} looks: allocated rule miscoverage "
        f"{p_alloc:.4f} ≤ α_1={alpha_j(1):.4f}; naive per-look 95% rule {p_naive:.3f} > α={ALPHA}",
    )


def min_n(j: int, margin: float) -> int:
    n = 1
    while radius(j, n) > margin:
        n += 1
    return n


def check_sample_size_table() -> Result:
    table = {(j, m): min_n(j, m) for j in (1, 10, 100, 1000) for m in (0.2, 0.1)}
    monotone = all(table[(1, m)] <= table[(10, m)] <= table[(100, m)] <= table[(1000, m)]
                   for m in (0.2, 0.1))
    cells = "; ".join(f"j={j},r≤{m}: n≥{table[(j, m)]}" for (j, m) in table)
    return Result(
        "ST-4",
        "O §6.2 / S §15 sample size grows with the opportunity index j (informational)",
        "property",
        monotone,
        f"smallest n with radius(j,n) ≤ r at α={ALPHA}: {cells}",
    )


def check_equivalence_vacuous() -> Result:
    # "pass equivalence iff CI is wholly inside [-ε, +ε]"; CI ⊆ [-1, 1] after intersection.
    eps, j, n, mean = 1.0, 1, 1, 0.9
    lo, hi = max(-1.0, mean - radius(j, n)), min(1.0, mean + radius(j, n))
    passes = -eps <= lo and hi <= eps
    return Result(
        "ST-5",
        "O §6.2 equivalence rule with ε ≥ 1 (design suggestion: require ε < 1)",
        "counterexample",
        passes,
        f"ε=1, n=1, mean=0.9: CI=[{lo:g},{hi:g}] ⊆ [-1,1] → 'equivalent' with one block "
        f"(the conclusion |μ| ≤ 1 is true but carries no information)",
    )


CHECKS: list[Check] = [
    check_allocation_sums,
    check_radius_is_hoeffding,
    check_fwer_monte_carlo,
    check_sample_size_table,
    check_equivalence_vacuous,
]
