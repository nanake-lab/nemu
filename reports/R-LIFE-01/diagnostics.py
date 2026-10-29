"""Offline descriptive quality metrics; no model execution or acceptance substitution."""
from __future__ import annotations

from collections import defaultdict
import json
from pathlib import Path
import statistics
from typing import cast

from analysis import cluster_interval
from worlds import SEEDS


def quality(root: Path) -> dict[str, object]:
    intermediate = {str(world): [0] * 6 for world in (0, 1)}
    valid_actions: list[float] = []
    high_mass: list[float] = []
    semantic_choices: dict[tuple[int, int], set[int]] = defaultdict(set)
    speeches: list[str] = []
    deltas: dict[int, list[float]] = {0: [], 1: []}
    baseline_scores: list[float] = []
    label_scores: dict[int, list[float]] = {0: [], 1: []}
    for seed in SEEDS:
        baseline = json.loads((root / f"ability-pre-{seed}.json").read_text(encoding="utf-8"))
        pre = statistics.fmean(float(row["correct"]) for row in baseline)
        baseline_scores.append(pre)
        for row in baseline:
            label_scores[int(row["layout"][0])].append(float(row["correct"]))
            valid_actions.append(float(row["index"] >= 0))
            high_mass.append(float(row["measurement"]["valid_mass"] >= .95))
        for world in (0, 1):
            life = json.loads((root / f"life-{seed}-{world}.json").read_text(encoding="utf-8"))
            speeches.extend(row["utterance"] for row in life["transcript"])
            post = statistics.fmean(float(row["correct"]) for row in life["ability_post"])
            deltas[world].append(post - pre)
            for row in life["transcript"] + life["probes"] + life["ability_post"]:
                valid_actions.append(float(row["index"] >= 0))
                high_mass.append(float(row["measurement"]["valid_mass"] >= .95))
            for item in range(6):
                rows = [row for row in life["probes"] if row["item"] == item]
                probability = statistics.fmean(row["measurement"]["p_B_full" if row["layout"][0] else "p_A_full"] for row in rows)
                intermediate[str(world)][item] += .10 < probability < .90
                semantic_choices[(world, item)].update(row["index"] for row in rows if row["index"] >= 0)
    return {"intermediate_seed_counts_by_world_and_probe": intermediate,
            "valid_actions_fraction_including_ability": statistics.fmean(valid_actions),
            "high_label_mass_fraction_including_ability": statistics.fmean(high_mass),
            "action_variety_by_world_probe": {str(key): sorted(value) for key, value in semantic_choices.items()},
            "unique_utterance_fraction": len(set(speeches)) / len(speeches),
            "ability_pre": statistics.fmean(baseline_scores),
            "ability_pre_by_label_swap": {str(label): statistics.fmean(values) for label, values in label_scores.items()},
            "ability_world_delta": {str(w): {"mean": statistics.fmean(d), "ci95": cluster_interval(d)} for w, d in deltas.items()},
            "note": "All lives must pass semantic audit. Reaction categories require event-cited manual coding."}


def margins(layout_scores: list[float]) -> dict[str, float]:
    if len(layout_scores) != 8:
        raise ValueError("Eight complete layouts required")
    return {name: abs(statistics.fmean(layout_scores[i] for i in range(8) if i & bit)
                      - statistics.fmean(layout_scores[i] for i in range(8) if not i & bit))
            for name, bit in (("history_order", 4), ("behavior_order", 2), ("label_vocabulary", 1))}


def audit_scores(root: Path) -> dict[str, object]:
    scores = []
    for seed in SEEDS:
        for world in (0, 1):
            raw = json.loads((root / f"audit-{seed}-{world}.json").read_text(encoding="utf-8"))
            audit = cast(dict[str, object], json.loads(raw["raw"]))
            value = audit["realism"]
            if type(value) is not int:
                raise ValueError("Invalid realism score")
            scores.append(value)
    return {"realism_median": statistics.median(scores), "realism_at_least_3": statistics.fmean(float(s >= 3) for s in scores)}
