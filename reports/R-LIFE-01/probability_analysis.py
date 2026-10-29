"""Judge-independent, CPU-only paired probability endpoint; no model execution."""
from __future__ import annotations

import argparse
from itertools import product
import json
import math
from pathlib import Path
import random
import statistics
from typing import cast

from analysis import cluster_interval, holm
from io_support import CACHE, digest, save
from planning import H1_KEYS
from phases import COMPLETE
from worlds import LAYOUTS, PROBES, SEEDS


def semantic_probabilities(life: dict[str, object]) -> list[tuple[float, float]]:
    """Undo labels then average all four layouts; never renormalize valid mass."""
    rows = life.get("probes")
    if not isinstance(rows, list) or len(rows) != len(PROBES) * len(LAYOUTS):
        raise ValueError("Complete six-probe/four-layout records required")
    cells: dict[tuple[int, tuple[bool, bool]], tuple[float, float]] = {}
    for value in rows:
        if not isinstance(value, dict):
            raise ValueError("Invalid probe row")
        item, layout, measurement = value.get("item"), value.get("layout"), value.get("measurement")
        if (type(item) is not int or not 0 <= item < len(PROBES)
                or not isinstance(layout, (list, tuple)) or len(layout) != 2
                or any(type(bit) is not bool for bit in layout) or not isinstance(measurement, dict)):
            raise ValueError("Invalid probe identity")
        key = (item, (layout[0], layout[1]))
        if key in cells:
            raise ValueError("Duplicate layout")
        pa, pb = measurement.get("p_A_full"), measurement.get("p_B_full")
        if (isinstance(pa, bool) or not isinstance(pa, (int, float))
                or isinstance(pb, bool) or not isinstance(pb, (int, float))):
            raise ValueError("Numeric full-vocabulary probabilities required")
        if (not math.isfinite(pa) or not math.isfinite(pb)
                or not 0 <= pa <= 1 or not 0 <= pb <= 1 or pa + pb > 1 + 1e-10):
            raise ValueError("Invalid full-vocabulary probabilities")
        cells[key] = (float(pb), float(pa)) if layout[0] else (float(pa), float(pb))
    return [(statistics.fmean(cells[(item, layout)][0] for layout in LAYOUTS),
             statistics.fmean(cells[(item, layout)][1] for layout in LAYOUTS))
            for item in range(len(PROBES))]


def quadratic_statistic(differences: list[list[float]], signs: tuple[int, ...]) -> float:
    n, dimensions = len(differences), len(differences[0])
    return math.fsum((math.fsum(sign * row[j] for sign, row in zip(signs, differences)) / n) ** 2
                     for j in range(dimensions)) * n / dimensions


def paired_probability_test(differences: list[list[float]], draws: int = 99999) -> dict[str, object]:
    if (not differences or not differences[0] or draws < 1
            or any(len(row) != len(differences[0]) or any(not math.isfinite(v) or abs(v) > 1 for v in row)
                   for row in differences)):
        raise ValueError("Finite, complete seed vectors required")
    n = len(differences)
    observed = quadratic_statistic(differences, (1,) * n)
    if n <= 16:
        extreme = sum(quadratic_statistic(differences, signs) >= observed - 1e-15
                      for signs in product((-1, 1), repeat=n))
        p_value, method, count = extreme / 2 ** n, "exact_seed_swap", 2 ** n
    else:
        rng = random.Random(291020)
        extreme = 1
        for _ in range(draws):
            signs = tuple(1 if rng.getrandbits(1) else -1 for _ in range(n))
            extreme += quadratic_statistic(differences, signs) >= observed - 1e-15
        p_value, method, count = extreme / (draws + 1), "monte_carlo_seed_swap_plus_one", draws
    return {"statistic": observed, "p_value": p_value, "method": method, "permutations": count,
            "independent_seed_pairs": n, "dimensions": len(differences[0])}


def analyze_pairs(pairs: list[tuple[dict[str, object], dict[str, object]]]) -> dict[str, object]:
    differences, magnitudes, raw_distances, invalid_changes = [], [], [], []
    for left, right in pairs:
        p0, p1 = semantic_probabilities(left), semantic_probabilities(right)
        # Half the change in unconditional semantic balance: equal losses to INVALID cancel.
        delta = [(right_probs[0] - right_probs[1] - left_probs[0] + left_probs[1]) / 2
                 for left_probs, right_probs in zip(p0, p1)]
        differences.append(delta)
        magnitudes.append(statistics.fmean(abs(value) for value in delta))
        raw_distances.append(statistics.fmean((abs(right_probs[0] - left_probs[0]) + abs(right_probs[1] - left_probs[1])) / 2
                                               for left_probs, right_probs in zip(p0, p1)))
        invalid_changes.append([sum(left_probs) - sum(right_probs) for left_probs, right_probs in zip(p0, p1)])
    return {"endpoint": "H1a", "test": paired_probability_test(differences),
            "seed_semantic_balance_differences": differences, "seed_magnitudes": magnitudes,
            "mean_magnitude": statistics.fmean(magnitudes), "magnitude_ci95": cluster_interval(magnitudes),
            "seed_raw_semantic_half_l1": raw_distances, "seed_invalid_mass_changes": invalid_changes,
            "status": "provisional_until_full_input_and_quality_audit",
            "limitation": "Mean-vector test may miss seed-specific differences with cancelling directions; non-significance is not identity."}


def adjust_primary(pvalues: dict[str, float], *, pilot: bool = False) -> dict[str, float]:
    """Never silently shrink the co-primary family or include structural-zero D."""
    expected = {"H1a:C", "H1b:C"} if pilot else set(H1_KEYS)
    if set(pvalues) != expected or any(not math.isfinite(value) or not 0 <= value <= 1 for value in pvalues.values()):
        raise ValueError("Complete preregistered H1 family required; D is diagnostic only")
    return holm(pvalues)


def analyze_root(root: Path) -> dict[str, object]:
    root = root.resolve()
    if not root.is_relative_to(CACHE.resolve()) or root == CACHE.resolve():
        raise ValueError("Experiment cache only")
    status = json.loads((root / "status.json").read_text(encoding="utf-8"))
    if status.get("status") not in (COMPLETE, "C_behavior_complete_pending_blind_audit"):
        raise ValueError("No complete C behavior; never analyze preparation as observations")
    pairs = []
    bindings = {}
    for seed in SEEDS:
        lives = []
        for world in (0, 1):
            path = root / f"life-{seed}-{world}.json"
            life = cast(dict[str, object], json.loads(path.read_text(encoding="utf-8")))
            if life.get("seed") != seed or life.get("world") != world:
                raise ValueError("Pair identity mismatch")
            lives.append(life)
            bindings[path.name] = digest(path)
        pairs.append((lives[0], lives[1]))
    return {**analyze_pairs(pairs), "seeds": list(SEEDS), "source_sha256": bindings,
            "judge_calls": 0, "gpu_calls": 0, "paid_api_calls": 0}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("run", type=Path)
    args = parser.parse_args()
    save(args.run.resolve() / "probability-summary.json", analyze_root(args.run))
    print("Offline probability endpoint saved; no judge/API/GPU calls.")


if __name__ == "__main__":
    main()
