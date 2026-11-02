"""Post-hoc failure diagnosis declared in POSTHOC.md; never changes the study."""

from __future__ import annotations

import argparse
import gzip
import json
import statistics
from fractions import Fraction as F
from pathlib import Path
from typing import cast

from models import Budgeted, PrecisionExhausted, replay
from test_research import exact


def failure_records(path: Path) -> list[dict[str, object]]:
    with gzip.open(path, "rt", encoding="utf-8") as source:
        return [cast(dict[str, object], json.loads(line)) for line in source]


def params(row: dict[str, object]) -> tuple[list[tuple[float, float]], int, float, float, float]:
    history = cast(list[tuple[float, float]], row["history"])
    return (history, cast(int, row["selected"]), cast(float, row["factor"]),
            cast(float, row["prior"]), cast(float, row["h0"]))


def rational_difference(row: dict[str, object], name: str) -> F:
    history, selected, factor, prior, h0 = params(row)
    before = [(F(w), F(s)) for w, s in history]
    after = list(before)
    w, s = history[selected]
    after[selected] = F(w * factor), F(s)  # Same binary64 input as the actual replay.
    return (exact(name, after, F(prior), F(h0), F(4))
            - exact(name, before, F(prior), F(h0), F(4))) * ((s > 0) - (s < 0))


def minimize(row: dict[str, object]) -> dict[str, object]:
    history, selected, factor, prior, h0 = params(row)
    history = list(history)

    def violates(events: list[tuple[float, float]], index: int) -> bool:
        altered = list(events)
        w, s = events[index]
        altered[index] = w * factor, s
        try:
            before = replay(Budgeted, events, prior=prior, h0=h0).pi()
            after = replay(Budgeted, altered, prior=prior, h0=h0).pi()
            return (after - before) * s > 1e-6  # Only choose a clear diagnostic witness.
        except PrecisionExhausted:
            return False

    changed = True
    while changed:
        changed = False
        for i in range(len(history)):
            if i == selected:
                continue
            shorter = history[:i] + history[i + 1:]
            new_index = selected - (i < selected)
            if violates(shorter, new_index):
                history, selected, changed = shorter, new_index, True
                break
    witness = row | {"history": history, "selected": selected}
    delta = rational_difference(witness, "B")
    altered = list(history)
    w, s = history[selected]
    altered[selected] = w * factor, s
    return witness | {
        "exact_directional_delta": str(delta), "exact_violation": delta > 0,
        "before": replay(Budgeted, history, prior=prior, h0=h0).pi(),
        "after": replay(Budgeted, altered, prior=prior, h0=h0).pi(),
    }


def analyze(directory: Path) -> dict[str, object]:
    records = failure_records(directory / "failures.jsonl.gz")
    old = [r for r in records if r["check"] == "LP1-3" and r["model"] == "O"]
    budget = [r for r in records if r["check"] == "LP1-3" and r["model"] == "B"]
    largest = max(budget, key=lambda r: abs(cast(float, r["after"]) - cast(float, r["before"])))
    pre: dict[int, dict[str, float]] = {}
    with gzip.open(directory / "world-trajectories.jsonl.gz", "rt", encoding="utf-8") as stream:
        for line in stream:
            row = cast(dict[str, object], json.loads(line))
            if row["world"] == "abrupt" and row["step"] == 999:
                pre[cast(int, row["seed"])] = {
                    name: cast(float, cast(dict[str, object], row[name])["pi_after"])
                    for name in ("O", "M", "B")
                }
    common = [seed for seed, values in pre.items() if all(p >= 0 for p in values.values())]
    worlds = cast(list[dict[str, object]], json.loads((directory / "world-seeds.json").read_text()))
    subset: list[dict[str, object]] = []
    for name in ("O", "M", "B"):
        rows = [r for r in worlds if r["world"] == "abrupt" and r["model"] == name
                and r["seed"] in common]
        subset.append({"model": name, "n": len(rows),
                       "already_wrong_before_change": sum(v[name] < 0 for v in pre.values()),
                       "delay_mean": statistics.mean(cast(float, r["delay"]) for r in rows),
                       "confirmed_delay_mean": statistics.mean(
                           cast(float, r["confirmed_delay"]) for r in rows)})
    return {"original_lp1_3": {"binary64_violations": len(old),
                               "real_arithmetic_violations": sum(rational_difference(r, "O") > 0 for r in old)},
            "budget_witness": minimize(largest), "common_correct_seeds": common,
            "common_correct_diagnostic": subset}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = analyze(args.run)
    with args.output.open("x", encoding="utf-8") as out:
        json.dump(result, out, ensure_ascii=False, allow_nan=False, indent=2)
        out.write("\n")
    print(json.dumps(result, ensure_ascii=False))


if __name__ == "__main__":
    main()
