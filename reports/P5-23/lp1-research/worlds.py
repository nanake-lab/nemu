"""Paired synthetic worlds fixed by PLAN.md; metrics never control the law."""

from __future__ import annotations

import json
import random
import statistics
from typing import TextIO

from models import MODELS, PrecisionExhausted

SEED = 20261019
WORLD_NAMES = ("stable", "abrupt", "gradual", "periodic")


def probability(world: str, step: int) -> float:
    if world == "stable":
        return 0.8
    if world == "abrupt":
        return 0.8 if step < 1000 else 0.2
    if world == "gradual":
        return 0.8 - 0.6 * min(1.0, max(0.0, (step - 500) / 999))
    if world == "periodic":
        return 0.8 if (step // 250) % 2 == 0 else 0.2
    raise ValueError(world)


def delay(values: list[float], start: int, end: int, direction: int, run: int) -> int | None:
    count = 0
    for index in range(start, end):
        count = count + 1 if values[index] * direction > 0 else 0
        if count == run:
            return index - start + 1
    return None


def run_worlds(trace: TextIO) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for world in WORLD_NAMES:
        for offset in range(30):
            seed = SEED + 1000 + offset
            rng = random.Random(seed)
            states = {name: model(3.0, 1.0) for name, model in MODELS.items()}
            post: dict[str, list[float]] = {name: [] for name in states}
            losses: dict[str, list[float]] = {name: [] for name in states}
            latent: dict[str, list[float]] = {name: [] for name in states}
            failed: dict[str, str] = {}
            for step in range(2000):
                p = probability(world, step)
                s = 1.0 if rng.random() < p else -1.0
                w = rng.choice((0.01, 0.1, 1.0, 3.0))
                record: dict[str, object] = {"world": world, "seed": seed, "step": step,
                                            "p": p, "w": w, "s": s}
                for name, state in states.items():
                    if name in failed:
                        continue
                    prediction = (1 + state.pi()) / 2
                    losses[name].append((prediction - (s + 1) / 2) ** 2)
                    latent[name].append((prediction - p) ** 2)
                    try:
                        state.add(w, s)
                        post[name].append(state.pi())
                        record[name] = {"prediction": prediction, "pi_after": state.pi()}
                    except PrecisionExhausted as exc:
                        failed[name] = f"step {step}: {exc}"
                        record[name] = {"fault": failed[name]}
                trace.write(json.dumps(record, allow_nan=False) + "\n")
            for name in states:
                row: dict[str, object] = {"world": world, "seed": seed, "model": name,
                                          "ok": name not in failed}
                if name in failed:
                    rows.append(row | {"fault": failed[name]})
                    continue
                values = post[name]
                row |= {"brier": statistics.mean(losses[name]),
                        "latent_mse": statistics.mean(latent[name])}
                if world == "stable":
                    row["false_flips_per_1000"] = sum(
                        values[i - 1] >= 0 > values[i] for i in range(200, 2000)
                    ) * 1000 / 1800
                    row["wrong_direction_fraction"] = sum(v < 0 for v in values[200:]) / 1800
                if world == "abrupt":
                    row["delay"] = delay(values, 1000, 2000, -1, 1)
                    row["confirmed_delay"] = delay(values, 1000, 2000, -1, 5)
                if world == "periodic":
                    row["segment_delays"] = [
                        delay(values, start, start + 250, -1 if start // 250 % 2 else 1, 1)
                        for start in range(250, 2000, 250)
                    ]
                rows.append(row)
    return rows


def summarize(rows: list[dict[str, object]]) -> list[dict[str, object]]:
    output: list[dict[str, object]] = []
    metrics = ("brier", "latent_mse", "false_flips_per_1000", "wrong_direction_fraction",
               "delay", "confirmed_delay")
    for world in WORLD_NAMES:
        baseline = {r["seed"]: r for r in rows if r["world"] == world and r["model"] == "O"}
        for name in MODELS:
            subset = [r for r in rows if r["world"] == world and r["model"] == name]
            for metric in metrics:
                values: list[float] = []
                diffs: list[float] = []
                censored = 0
                for row in subset:
                    if metric not in row:
                        continue
                    value = row[metric]
                    other = baseline[row["seed"]].get(metric)
                    if value is None:
                        censored += 1
                    elif isinstance(value, (float, int)):
                        values.append(float(value))
                        if isinstance(other, (float, int)):
                            diffs.append(float(value - other))
                if values or censored:
                    output.append({"world": world, "model": name, "metric": metric,
                                   "n": len(values), "censored": censored,
                                   "mean": statistics.mean(values) if values else None,
                                   "sample_sd": statistics.stdev(values) if len(values) > 1 else None,
                                   "paired_n": len(diffs),
                                   "paired_delta_mean": statistics.mean(diffs) if diffs else None,
                                   "paired_delta_sd": statistics.stdev(diffs) if len(diffs) > 1 else None})
    return output
