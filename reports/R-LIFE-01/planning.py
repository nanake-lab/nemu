"""Analytic sensitivity and budget plans, never simulated model observations."""
from __future__ import annotations

import hashlib
import math

from analysis import binomial_tail, paired_power, power
from hosted_judge import PROVIDERS, estimate

PROTOCOL_VERSION = "rlife-prereg-v3-proposed"
H1_GROUPS = ("C", "E", "F", "A", "B")
H1_KEYS = tuple(f"{endpoint}:{group}" for endpoint in ("H1a", "H1b") for group in H1_GROUPS)
FORMAL_SEEDS = tuple(range(18101, 18261))
JUDGE_SAMPLE_OPTIONS = (160, 120, 80, 40)
SUBSET_DOMAIN = "R-LIFE-02/judge-subset/v2/"


def selected_seeds(size: int) -> list[int]:
    if size not in JUDGE_SAMPLE_OPTIONS:
        raise ValueError("Select and freeze a preregistered size before outcomes")
    ranked = sorted(FORMAL_SEEDS, key=lambda seed: hashlib.sha256((SUBSET_DOMAIN + str(seed)).encode()).digest())
    return sorted(ranked[:size])


def rank_one_power(n: int, positive_probability: float, alpha: float) -> dict[str, float | int]:
    """Exact quadratic-test power for d_s = +/- fixed vector, signs Bernoulli(pi)."""
    critical = next((k for k in range(n // 2 + 1, n + 1) if 2 * binomial_tail(k, n) <= alpha), n + 1)
    probability = (binomial_tail(critical, n, positive_probability)
                   + binomial_tail(critical, n, 1 - positive_probability)) if critical <= n else 0.0
    return {"n": n, "pi": positive_probability, "critical_upper": critical, "power": probability}


def budget(judged_pairs_per_group: int) -> dict[str, object]:
    if judged_pairs_per_group not in JUDGE_SAMPLE_OPTIONS:
        raise ValueError("Unregistered judge sample")
    actor_calls = 160 * 6 * 2 * 32
    pairs = 6 * judged_pairs_per_group
    judge_calls = pairs * (16 + 8 + 2) + 64
    return {"actor_calls": actor_calls, "judge_main": pairs * 16, "judge_null": pairs * 8,
            "judge_audits": pairs * 2, "judge_anchors": 64, "judge_sessions": judge_calls,
            "deepseek_expected_usd": actor_calls * (1000 * .30 + 120 * 1.20) / 1e6,
            "deepseek_upper_usd": actor_calls * (2500 * .30 + 256 * 1.20) / 1e6,
            "judge_api_usd": {name: {"expected": estimate(judge_calls, name),
                                     "upper": estimate(judge_calls, name, upper=True)} for name in PROVIDERS},
            "judge_visible_input_tokens_assumption": judge_calls * 2500,
            "judge_visible_output_tokens_assumption": judge_calls * 200,
            "unsampled_lives_requiring_manual_audit": 2 * 6 * (160 - judged_pairs_per_group)}


def plan() -> dict[str, object]:
    rows = []
    for size in JUDGE_SAMPLE_OPTIONS:
        pairing = [power(size, effect, .05 / len(H1_KEYS)) for effect in (.60, .65, .70)]
        critical = max(int(pairing[1]["critical_successes"]), math.ceil(.65 * size - 1e-10))
        rows.append({"judge_pairs_per_group": size, "selected_seeds": selected_seeds(size),
                     "budget": budget(size), "H1b_binary_test_power": pairing,
                     "H1b_p065_test_and_point_gate_only": binomial_tail(critical, size, .65),
                     "H2b_delta020_q050_test_power": paired_power(size, .20, .50, .05 / 8)})
    return {"version": PROTOCOL_VERSION, "kind": "analytic_planning_not_behavioral_results",
            "H1_family": H1_KEYS, "H1_family_alpha": .05, "H2_tests": 8,
            "H1a_rank_one_sensitivity": [rank_one_power(160, pi, .05 / len(H1_KEYS)) for pi in (.50, .60, .65, .70)],
            "H1a_assumption": "All six differences share one +/- direction, magnitude .10; no empirical calibration.",
            "judge_subset_domain": SUBSET_DOMAIN, "options": rows,
            "note": "Power covers idealized significance tests, not all validity/CI/capability gates; 16 judges are not 16 seeds."}
