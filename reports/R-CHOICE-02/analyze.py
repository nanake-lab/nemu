"""Offline reanalysis; never loads a model or drops invalid records."""
import argparse
from collections import Counter
import json
from pathlib import Path
import statistics
from typing import Any

from calibrate import summarize
from probe import save


def analyze(root: Path) -> dict[str, Any]:
    rows = [json.loads(line) for line in (root / "episodes.jsonl").read_text(encoding="utf-8").splitlines()]
    result = summarize(rows)
    result["observations"] = {}
    for missing in (False, True):
        subset = [r for r in rows if r["missing"] == missing]
        mean_p = statistics.mean(r["p_relevant_conditional"] for r in subset)
        result["observations"]["missing" if missing else "complete"] = {
            "rows": len(subset), "actual_actions": dict(Counter(r["actual_action"] for r in subset)),
            "p_relevant_conditional_mean": mean_p, "maximum_upward_headroom": 1 - mean_p,
            "floor_layouts": sum(r["p_relevant_conditional"] <= .05 for r in subset),
            "ceiling_layouts": sum(r["p_relevant_conditional"] >= .95 for r in subset)}
    result["unique_prompts"] = {slot: len({r[slot]["prompt"] for r in rows}) for slot in ("first", "last")}
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise SystemExit("Preserve existing analysis")
    save(args.output, analyze(args.root))
