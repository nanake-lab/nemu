"""Independent seed pairs are units; layouts and judge repetitions are not n."""
from __future__ import annotations

import math
import random
import statistics


def binomial_tail(k: int, n: int, p: float = 0.5) -> float:
    if not 0 <= k <= n or not 0 <= p <= 1:
        raise ValueError("Invalid binomial arguments")
    return math.fsum(math.comb(n, j) * p ** j * (1 - p) ** (n - j) for j in range(k, n + 1))


def power(n: int, effect: float, alpha: float) -> dict[str, float | int]:
    critical = next((k for k in range(n + 1) if binomial_tail(k, n) <= alpha), n + 1)
    return {"n": n, "p_alternative": effect, "alpha": alpha, "critical_successes": critical,
            "power": binomial_tail(critical, n, effect) if critical <= n else 0.0}


def paired_power(n: int, delta: float, discordance: float, alpha: float) -> float:
    """Ideal paired binary planning, averaging exact McNemar power over discordant n."""
    if not 0 < delta < discordance <= 1:
        raise ValueError("Invalid paired planning assumptions")
    conditional = (discordance + delta) / (2 * discordance)
    return math.fsum(math.comb(n, count) * discordance ** count * (1 - discordance) ** (n - count)
                     * float(power(count, conditional, alpha)["power"]) for count in range(1, n + 1))


def cluster_interval(values: list[float], seed: int = 1709) -> list[float]:
    if not values or any(not math.isfinite(v) for v in values):
        raise ValueError("Missing/nonfinite seed scores")
    rng = random.Random(seed)
    samples = sorted(statistics.fmean(rng.choices(values, k=len(values))) for _ in range(10000))
    return [samples[249], samples[9749]]


def sign_randomization(values: list[float], valid_fractions: list[float] | None = None) -> float:
    """Swap truth at seed level: valid decisions complement, invalids stay zero."""
    valid = valid_fractions if valid_fractions is not None else [1.0] * len(values)
    if (not values or len(valid) != len(values)
            or any(not 0 <= score <= fraction <= 1 for score, fraction in zip(values, valid))):
        raise ValueError("Invalid scores/validity")
    differences = [score - fraction / 2 for score, fraction in zip(values, valid)]
    observed = math.fsum(differences)
    rng = random.Random(291019)
    extreme = 1
    for _ in range(99999):
        statistic = math.fsum(d if rng.getrandbits(1) else -d for d in differences)
        extreme += statistic >= observed - 1e-12
    return extreme / 100000


def holm(pvalues: dict[str, float]) -> dict[str, float]:
    adjusted, previous = {}, 0.0
    for index, (key, value) in enumerate(sorted(pvalues.items(), key=lambda pair: pair[1])):
        previous = max(previous, min(1.0, (len(pvalues) - index) * value))
        adjusted[key] = previous
    return adjusted


def summarize(seed_scores: list[float], valid_fractions: list[float] | None = None) -> dict[str, object]:
    return {"independent_seed_pairs": len(seed_scores), "accuracy": statistics.fmean(seed_scores),
            "cluster_bootstrap_95": cluster_interval(seed_scores),
            "p_one_sided_sign_randomization": sign_randomization(seed_scores, valid_fractions)}
