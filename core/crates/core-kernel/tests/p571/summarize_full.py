"""Read-only aggregation of a single preserved matrix run; never executes laws.

Raw per-prefix files remain private. Counts are trajectories, not independent
subjects. A partial trajectory is not passed; model/DUT layers stay separate.
"""

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any

from arithmetic import Q
from f_research import scalar_bits
from full_run import selected_version
from prepare import binding, write


def resolved_status(
    row: dict[str, Any],
    raw: dict[str, Any],
    paired: dict[str, Any],
    complete: bool,
) -> dict[str, Any]:
    result = dict(raw)
    # Frozen interface exclusions remain distinct from a later run interruption.
    if row["planned_status"] == "not_run":
        result.update(status="not_run", reason=row["reason"])
        return result
    if row["layer"] == "DUT" and result.get("oracle_comparison"):
        result["oracle_comparison"] = paired.get("status", "not_run")
        if paired.get("status") == "failed":
            result["status"] = "failed"
            result["paired_differences"] = paired.get("differences", [])
        elif paired.get("status") != "passed" and result["status"] != "failed":
            result.update(
                status="not_run",
                reason="DUT prefixes retained; paired independent comparison incomplete",
            )
    if not complete and result["status"] == "passed":
        result.update(status="not_run", reason="incomplete prefix coverage")
    return result


def sensitivity_status(registered: int, completed: int, detected: int) -> str:
    # A detected witness does not discharge unexecuted frozen coverage.
    if completed != registered:
        return "not_run"
    return "passed" if detected else "failed"


def read_lines(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("prepared", type=Path)
    ap.add_argument("run", type=Path)
    ap.add_argument("output", type=Path)
    args = ap.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    plan = read_lines(args.prepared / "matrix.jsonl")
    results = {r["run_index"]: r for r in read_lines(args.run / "results.jsonl")}
    groups: dict[str, dict[str, Any]] = {}
    totals: dict[str, dict[str, int]] = {}
    measurements: dict[str, dict[str, int]] = {}
    times: dict[str, dict[str, int]] = {}
    delays: list[dict[str, Any]] = []
    resolved: list[dict[str, Any]] = []
    for row in plan:
        i = row["run_index"]
        result = dict(results.get(i, dict(status="not_run", reason="missing result")))
        raw_path = args.run / f"{i:06}.jsonl"
        prefixes = (
            [v for v in read_lines(raw_path) if "prefix" in v]
            if raw_path.exists()
            else []
        )
        complete = [v["prefix"] for v in prefixes] == list(range(row["N"] + 1))
        result = resolved_status(row, result, results.get(i + 1, {}), complete)
        category = row["layer"] + "/" + row["family"]
        total = totals.setdefault(
            category,
            dict(
                rows=0,
                completed_rows=0,
                retained_incremental_operations=0,
                retained_prefix_recompute=0,
            ),
        )
        total["rows"] += 1
        total["completed_rows"] += int(complete)
        n = max((v["prefix"] for v in prefixes), default=0)
        factor = row.get("G_route_calls", 1)
        total["retained_incremental_operations"] += n * factor
        total["retained_prefix_recompute"] += n * (n + 1) // 2 * factor
        timing = times.setdefault(
            category, dict(incremental_ns=0, replay_ns=0, timed_prefixes=0)
        )
        for prefix in prefixes:
            timing["incremental_ns"] += prefix.get("incremental_ns", 0)
            timing["replay_ns"] += prefix.get("replay_ns", 0)
            timing["timed_prefixes"] += int("incremental_ns" in prefix)
        group_key = "/".join((row["id"], row["layer"], row["version"]))
        if isinstance(row["world"], dict) and "rho" in row["world"]:
            group_key += "/rho=" + str(row["world"]["rho"])
        group = groups.setdefault(
            group_key,
            dict(
                passed=0,
                failed=0,
                not_run=0,
                non_identifiable=0,
                reasons={},
                first_failed=None,
            ),
        )
        group[result["status"]] += 1
        if result.get("identifiability"):
            group["non_identifiable"] += 1
        if result.get("reason"):
            reason = result["reason"]
            group["reasons"][reason] = group["reasons"].get(reason, 0) + 1
        if result["status"] == "failed" and group["first_failed"] is None:
            group["first_failed"] = i
        if row["planned_status"] != "not_run" and selected_version(row):
            measure = measurements.setdefault(
                group_key, dict(registered=0, completed=0, detected=0)
            )
            measure["registered"] += 1
            measure["completed"] += int(complete)
            measure["detected"] += int(result.get("mutation_killed") is True)
        if (
            row["family"] == "C"
            and row["layer"] == "independent_research"
            and row["id"] in ("07-04", "07-09")
            and (row["id"] != "07-04" or row["branch"] == 0)
            and prefixes
        ):
            start = 65 if row["id"] == "07-04" else 1025
            target = Q(-1, 2)
            relevant = [p for p in prefixes if p["prefix"] >= start]
            last_outside = start - 1
            for p in relevant:
                b = scalar_bits(p["state"]["values"]["baseline"]["connection"]["f64"])
                if abs(b - target) > Q(1, 8):
                    last_outside = p["prefix"]
            entered = last_outside + 1
            delays.append(
                dict(
                    run_index=i,
                    id=row["id"],
                    version=row["version"],
                    world=row["world"],
                    branch=row["branch"],
                    first_persistent_step=entered
                    if complete and entered <= row["N"]
                    else None,
                    right_censored=not complete or entered > row["N"],
                    observed_through=n,
                    scope="permanent branch only; finite-perturbation paired traces retained separately",
                )
            )
        resolved.append(
            dict(
                plan_index=i,
                result=result,
                retained_prefix=n if prefixes else None,
                complete=complete,
            )
        )
    sensitivity = {
        k: dict(
            v,
            status=sensitivity_status(v["registered"], v["completed"], v["detected"]),
        )
        for k, v in measurements.items()
    }
    summary = dict(
        source_end=json.loads((args.run / "END.json").read_text(encoding="utf-8")),
        operator_stop=json.loads(
            (args.run / "OPERATOR-STOP.json").read_text(encoding="utf-8-sig")
        )
        if (args.run / "OPERATOR-STOP.json").exists()
        else None,
        result_hash=binding(args.run / "results.jsonl"),
        planned_totals=json.loads(
            (args.prepared / "totals.json").read_text(encoding="utf-8")
        ),
        retained_totals=totals,
        groups=groups,
        measurement_sensitivity=sensitivity,
        timings=times,
        status_counts=dict(Counter(v["result"]["status"] for v in resolved)),
        limits=[
            "DUT success requires paired independent comparison",
            "Row statuses are comparison results, not whole-item mechanism acceptance",
            "Frozen not_run reasons restored from the pre-execution plan; raw results unchanged",
            "No subject-level statistical inference",
            "Sensitivity detected in a registered witness; untouched worlds need not all differ",
            "All complete positive trajectories and required sensitivity are necessary before signing an item passed",
            "Timings omit unfinished replay work; G sibling routes retained separately in raw files",
        ],
    )
    write(args.output / "SUMMARY.json", summary)
    write(args.output / "RESOLVED-RESULTS.json", resolved)
    write(args.output / "DELAYS.json", delays)
    print(
        json.dumps(
            dict(status_counts=summary["status_counts"], output=str(args.output)),
            ensure_ascii=False,
        )
    )


if __name__ == "__main__":
    main()
