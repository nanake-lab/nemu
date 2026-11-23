"""Paired case-level estimates. Repeats do not increase the independent case count."""
from __future__ import annotations

import argparse
from collections import defaultdict
import json
from pathlib import Path
import random
from statistics import mean
from typing import Any

from run import ARMS, save


def interval(values: list[float]) -> list[float]:
    rng = random.Random(917301)
    boot = sorted(mean(rng.choices(values, k=len(values))) for _ in range(20000))
    return [boot[499], boot[19499]]


def paired(rows: list[dict[str, Any]], a: str, b: str, metric: str) -> dict[str, object]:
    groups: dict[str, dict[str, list[float]]] = defaultdict(lambda: defaultdict(list))
    for row in rows:
        if row["arm"] in (a, b):
            value = float(row[metric])
            groups[row["case"]][row["arm"]].append(value)
    deltas = {case: mean(arms[a])-mean(arms[b]) for case, arms in sorted(groups.items())}
    return {"difference": mean(deltas.values()), "case_bootstrap_95pct": interval(list(deltas.values())),
            "independent_cases": len(deltas), "case_differences": deltas}


def analyze(root: Path) -> dict[str, object]:
    rows = [json.loads(line) for line in (root / "episodes.jsonl").read_text(encoding="utf-8").splitlines()]
    expected = 18*2*4*7
    keys = {(r["case"], r["order"], r["seed"], r["arm"]) for r in rows}
    if len(rows) != expected or len(keys) != expected:
        raise ValueError("Incomplete/duplicate episodes; do not publish a completed comparison")
    labels = json.loads((root / "label-token-ids.json").read_text())
    for row in rows:
        row["retrieved"] = row["action"] == "relevant"
        row["irrelevant"] = row["action"] == "irrelevant"
        row["any_read"] = row["action"] != "submit"
        row["first_argmax_valid"] = row["first"]["unconstrained_argmax"] in labels
        row["final_argmax_valid"] = row["last"]["unconstrained_argmax"] in labels[:2]
    missing = [r for r in rows if r["missing"]]
    complete = [r for r in rows if not r["missing"]]
    arm_summary: dict[str, object] = {}
    for name, _, _ in ARMS:
        subset = [r for r in rows if r["arm"] == name]
        m = [r for r in subset if r["missing"]]
        c = [r for r in subset if not r["missing"]]
        arm_summary[name] = {"episodes": len(subset), "missing_retrieval_probability": mean(r["p_relevant"] for r in m),
            "missing_retrieved": sum(r["retrieved"] for r in m), "missing_trials": len(m),
            "missing_irrelevant": sum(r["irrelevant"] for r in m), "complete_any_read": sum(r["any_read"] for r in c),
            "complete_trials": len(c), "correct": sum(r["correct"] for r in subset), "accuracy": mean(r["correct"] for r in subset),
            "mean_score": mean(r["score"] for r in subset), "min_choice_allowed_mass": min(r["first"]["allowed_mass"] for r in subset),
            "mean_choice_allowed_mass": mean(r["first"]["allowed_mass"] for r in subset),
            "choice_argmax_valid": sum(r["first_argmax_valid"] for r in subset),
            "final_argmax_valid": sum(r["final_argmax_valid"] for r in subset)}
    primary = {b: paired(missing, "target4", b, "p_relevant") for b in ("zero", "random1907_4", "random2718_4")}
    actual = {b: paired(missing, "target4", b, "retrieved") for b in ("zero", "random1907_4", "random2718_4")}
    family = {f: {b: paired([r for r in missing if r["family"] == f], "target4", b, "p_relevant")
                   for b in ("zero", "random1907_4", "random2718_4")} for f in ("stock", "route", "device")}
    order = {str(o): paired([r for r in missing if r["order"] == o], "target4", "zero", "p_relevant") for o in (0, 1)}
    return {"episodes": len(rows), "arms": arm_summary, "primary_probability_differences": primary,
        "actual_retrieval_differences": actual, "by_family": family, "by_label_order": order,
        "accuracy_difference": paired(rows, "target4", "zero", "correct"),
        "unnecessary_reads_difference": paired(complete, "target4", "zero", "any_read"),
        "dose_difference_4_minus_1": paired(missing, "target4", "target1", "p_relevant"),
        "method": "Unweighted case means over both orders and four decoding repeats; paired bootstrap resamples whole cases (20,000 draws, seed 917301). Only 12 missing and 6 complete cases are independent. Single-token constrained decisions, not unconstrained dialogue."}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("run", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise SystemExit("Preserve prior analysis; use a new filename")
    save(args.output, analyze(args.run))
