"""Paired E2 worlds from closed PR #40, pinned in ADR 0019. No thresholds.

Only the scalar evidence recurrence changes. y maps to S=2*y-1, W=1;
(1+pi)/2 is an evaluation readout, NOT a calibrated-probability claim.
"""
from __future__ import annotations

import argparse
import csv
import json
import random
import statistics
from pathlib import Path

from check_law import Tendency as Old
from law_v2_model import Tendency as New

Row = dict[str, str | int | float]


def truth(world: str, t: int) -> float:
    if world == "stable":
        return .7
    if world == "abrupt":
        return .7 if t < 320 else .3
    if world == "gradual":
        return .7 - .4 * min(1, max(0, (t - 192) / 192))
    return .7 if (t // 128) % 2 == 0 else .3


def csv_write(path: Path, rows: list[Row]) -> None:
    keys = list(dict.fromkeys(key for row in rows for key in row))
    with path.open("x", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=keys)
        writer.writeheader()
        writer.writerows(rows)


def run(out: Path) -> dict[str, object]:
    out.mkdir(parents=True, exist_ok=False)
    rows: list[Row] = []
    traces: list[Row] = []
    tapes: list[Row] = []
    for seed in range(41000, 41128):
        rng = random.Random(seed)
        tape = [rng.random() for _ in range(512)]
        tapes.extend(dict(seed=seed, t=t, uniform=u) for t, u in enumerate(tape))
        for world in ("stable", "abrupt", "gradual", "cycle"):
            for name, cls in (("old", Old), ("new", New)):
                model = cls(n0=3., kappa=1., h0=.1, nh=10.)
                mse = late = brier = wrong = flips = down = 0.0
                pre = 0.0
                delay, consecutive = 193, 0
                previous = .5
                for t, uniform in enumerate(tape):
                    p = truth(world, t)
                    prediction = (1 + model.pi()) / 2
                    y = int(uniform < p)
                    if t >= 64:
                        mse += (prediction - p)**2
                        brier += (prediction - y)**2
                        wrong += int(prediction < .5)
                        flips += int((prediction < .5) != (previous < .5))
                        down += int(prediction < .5 <= previous)
                    if 64 <= t < 320:
                        pre += (prediction - p)**2
                    if t >= 320:
                        late += (prediction - p)**2
                        consecutive = consecutive + 1 if prediction < .5 else 0
                        if consecutive >= 5 and delay == 193:
                            delay = t - 320 + 1
                    nominal = model.rho()
                    actual = model.add(1., 2. * y - 1.)
                    if seed == 41000:
                        traces.append(dict(seed=seed, world=world, form=name, t=t, truth=p,
                                           prediction=prediction, y=y, nominal_rho=nominal,
                                           actual_q=actual, after_pi=model.pi()))
                    previous = prediction
                row: Row = dict(seed=seed, world=world, form=name, mse=mse / 448,
                                pre_mse=pre / 256, late_mse=late / 192, brier=brier / 448,
                                mean_h=1-model.rho())
                if world == "stable":
                    row.update(wrong_side=wrong / 448, flip_rate=flips / 448,
                               false_downcross_rate=down / 448)
                if world == "abrupt":
                    row.update(delay_capped=delay, censored=int(delay == 193))
                rows.append(row)
    csv_write(out / "per_seed.csv", rows)
    csv_write(out / "trace_seed41000.csv", traces)
    csv_write(out / "uniform_tapes.csv", tapes)
    summary: dict[str, object] = {}
    for world in ("stable", "abrupt", "gradual", "cycle"):
        for name in ("old", "new"):
            selected = [r for r in rows if r["world"] == world and r["form"] == name]
            metrics = [k for k in selected[0] if k not in ("seed", "world", "form")]
            values = {key: statistics.mean(float(r[key]) for r in selected) for key in metrics}
            if world == "abrupt":
                observed = [float(r["delay_capped"]) for r in selected if r["censored"] == 0]
                values["detected_only_mean_delay"] = statistics.mean(observed) if observed else 0.
                values["detected_count"] = float(len(observed))
            summary[f"{world}/{name}"] = values
        old = [r for r in rows if r["world"] == world and r["form"] == "old"]
        new = [r for r in rows if r["world"] == world and r["form"] == "new"]
        differences = [float(n["mse"])-float(o["mse"]) for o, n in zip(old, new, strict=True)]
        summary[f"{world}/paired_mse_new_minus_old"] = dict(
            mean=statistics.mean(differences), standard_error=statistics.stdev(differences)/(128**.5),
            seeds_new_lower=sum(d < 0 for d in differences))
    manifest = dict(seeds=[41000, 41127], seeds_count=128, steps=512, forms=2,
                    world_source="f06e0e114495bf6231b59a3378cb29ea5eed5c2a:E2",
                    parameters=dict(n0=3., kappa=1., prior=0., h0=.1, nh=10.),
                    prediction="(1+pi)/2 before update", delay="5 consecutive predictions < .5; 193=censored",
                    row_count=len(rows), summary=summary)
    (out / "summary.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(run(args.out), indent=2))
