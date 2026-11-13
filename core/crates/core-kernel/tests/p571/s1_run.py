"""S1 allowlisted execution and new read-only analysis of immutable run02.

No full-matrix entry point, no resume, no implicit retry. The manifest is
written before this program is invoked and every input is hash-bound.
"""

import argparse
import csv
import json
import sys
import time
import traceback
from copy import deepcopy
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from arithmetic import Q, check_hand_vectors
from c_oracle import bit_tree, first_difference
from cf_research import CFResearch
from criteria import distance, distance_failure
from f_research import FResearch, scalar_bits
from full_run import dump, key, law_ref, run_row
from matrix_support import Budget, MatrixBridge, advance, fast_view, model_for
from prepare import binding, write
from s1_analysis import Analysis, point, add, sub, mul, clamp, serialize
from s1_refs import wrong_reference_checks


def load_rows(
    prepared: Path, report: Path
) -> tuple[list[dict[str, Any]], dict[int, dict[str, str]]]:
    with (report / "S1-ALLOWLIST.csv").open(encoding="utf-8", newline="") as s:
        allow = {int(r["run_index"]): r for r in csv.DictReader(s)}
    rows = [
        json.loads(line) for line in (prepared / "matrix.jsonl").open(encoding="utf-8")
    ]
    rows += json.loads((report / "S1-NEW-INPUTS.json").read_text(encoding="utf-8"))
    selected = [r for r in rows if r["run_index"] in allow]
    assert len(selected) == len(allow)
    for row in selected:
        if "input" in row:
            assert row["input_sha256"] == allow[row["run_index"]]["input_sha256"]
    return selected, allow


def stored_fast(item: dict[str, Any], row: dict[str, Any]) -> dict[str, Any]:
    if row["layer"] == "DUT":
        snapshot = item["snapshot"]["f"] if row["family"] == "CF" else item["snapshot"]
        return dict(bit_tree(fast_view(snapshot)))
    state = item["state"]["f"] if row["family"] == "CF" else item["state"]
    return {k: state[k] for k in ("x", "h", "targets", "anchor")}


def analyze(
    row: dict[str, Any],
    path: Path,
    resources: dict[str, Any],
    bridge: MatrixBridge,
    budget: Budget,
    destination: Path,
) -> dict[str, Any]:
    case, family = row["input"], row["family"]
    model = model_for(resources, case, family, bridge, {})
    f = model.f if isinstance(model, CFResearch) else model
    assert isinstance(f, FResearch)
    tracker = Analysis(f)
    failures: list[dict[str, Any]] = []
    unknown = 0
    count = 0
    with (
        path.open(encoding="utf-8") as source,
        destination.open("x", encoding="utf-8") as output,
    ):
        dump(
            output,
            dict(
                source=binding(path),
                source_path=str(path),
                input_sha256=row["input_sha256"],
            ),
        )
        for line in source:
            budget.check()
            item = json.loads(line)
            if "prefix" not in item:
                continue
            prefix = item["prefix"]
            before = deepcopy(f, {id(f.fn): f.fn})
            event = case["events"][prefix - 1] if prefix else None
            fe = event.get("f") if event and family == "CF" else event
            if prefix:
                assert event is not None
                advance(model, event, prefix - 1, False)
            projected = bool(
                prefix
                and (
                    (
                        family == "CF"
                        and event is not None
                        and event.get("c") is not None
                    )
                    or f.last.get("applied")
                )
            )
            readout = tracker.observe(before, f, fe, projected)
            expected = bit_tree(
                {k: f.state()[k] for k in ("x", "h", "targets", "anchor")}
            )
            difference = first_difference(
                expected, stored_fast(item, row), "analytic-point-oracle/source"
            )
            if (
                not readout["history_contains_h"]
                or readout["residual_enclosed"] is False
                or difference
            ):
                failures.append(
                    dict(prefix=prefix, difference=difference, readout=readout)
                )
            if readout["residual_enclosed"] is None:
                unknown += 1
            dump(output, dict(prefix=prefix, difference=difference, readout=readout))
            # Read-only observations preserve the stored anchor; analyze each
            # from the same independently generated state, never chain reads.
            deltas = event.get("readonly_after_ns", []) if event else []
            if prefix == len(case["events"]):
                deltas = case.get("reads_ns", []) or deltas
            raw_reads = item.get("reads") or []
            for delta, raw_read in zip(deltas, raw_reads, strict=True):
                budget.check()
                projected_f = deepcopy(f, {id(f.fn): f.fn})
                projected_f.project(f.anchor + delta)
                read_tracker = deepcopy(tracker)
                reading = read_tracker.observe(f, projected_f, None, True)
                expected_x = bit_tree(projected_f.x)
                actual_x = (
                    raw_read["x"]
                    if row["layer"] != "DUT"
                    else [
                        dict(f64=f"{v:016x}")
                        for v in raw_read["nature"] + raw_read["learned"]
                    ]
                )
                read_diff = first_difference(expected_x, actual_x, "readonly-x")
                if (
                    read_diff
                    or reading["residual_enclosed"] is False
                    or not reading["history_contains_h"]
                ):
                    failures.append(
                        dict(
                            prefix=prefix,
                            delta_ns=delta,
                            difference=read_diff,
                            readout=reading,
                        )
                    )
                dump(
                    output,
                    dict(
                        readonly_prefix=prefix,
                        delta_ns=delta,
                        difference=read_diff,
                        readout=reading,
                    ),
                )
            count += 1
    assert count == row["N"] + 1
    return dict(
        run_index=row["run_index"],
        id=row["id"],
        version=row["version"],
        layer=row["layer"],
        status="failed" if failures else "passed",
        prefixes=count,
        decomposition_non_identifiable_prefixes=unknown,
        first_failure=failures[0] if failures else None,
        failure_prefixes=len(failures),
        transcendental_error="not_run",
        evidence=destination.name,
        source=binding(path),
    )


def clock_checks(out: Path) -> dict[str, Any]:
    checks = []
    by_world: dict[int, list[dict[str, Any]]] = {}
    # New short rows are four versions per world, fixed in the supplement.
    for start in (100000, 100004, 100008, 100012):
        traces = {}
        for offset in (1, 2, 3):
            traces[offset] = [
                json.loads(line)
                for line in (out / f"{start + offset:06}.jsonl")
                .read_text(encoding="utf-8")
                .splitlines()
                if '"prefix":' in line
            ]
        a, mutant, wall = (traces[k] for k in (1, 2, 3))
        by_world[start] = a
        ea, em = a[2]["state"]["estimator"], mutant[2]["state"]["estimator"]
        b1 = scalar_bits(a[1]["state"]["values"]["baseline"]["connection"]["f64"])
        premise = Q(float(Q(1, 6))) < Q(43, 256) < b1
        sign = ea["run_sign"] == ["-1"] * 4 and em["run_sign"] == ["1"] * 4
        wall_difference = first_difference(a[2]["state"], wall[2]["state"])
        closed = (
            ea["zero_duration_count"] == ["1"] * 4
            if start == 100004
            else all(scalar_bits(v["f64"]) == 1 for v in ea["duration_mass"])
        )
        final_sign = a[-1]["state"]["estimator"]["run_sign"] == (
            ["-1"] * 4 if start == 100012 else ["1"] * 4
        )
        checks.append(
            dict(
                world=start,
                separating_premise=premise,
                step2_sign_difference=sign,
                wall_clock_detected=wall_difference is not None,
                closed_segment=closed,
                final_sign=final_sign,
                status="passed"
                if premise
                and sign
                and closed
                and final_sign
                and (start != 100008 or wall_difference)
                else "failed",
            )
        )
    same = True
    for normal, stopped in zip(by_world[100000], by_world[100008], strict=True):
        n, s = deepcopy(normal["state"]), deepcopy(stopped["state"])
        for state in (n, s):
            state["estimator"].pop("last_event", None)
            state["estimator"].pop("last_running_time", None)
        same = same and first_difference(n, s) is None
    return dict(
        checks=checks,
        ordinary_stopped_same_dynamics=same,
        status="passed"
        if same and all(c["status"] == "passed" for c in checks)
        else "failed",
        original_07_11="unchanged failed 0/4",
    )


def main() -> None:
    sys.set_int_max_str_digits(0)
    ap = argparse.ArgumentParser()
    ap.add_argument("manifest", type=Path)
    ap.add_argument("output", type=Path)
    args = ap.parse_args()
    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    for path, expected in manifest["files"].items():
        assert binding(Path(path)) == expected, path
    assert manifest["optimized_build"] is True
    binary, prepared, report, old = [
        Path(manifest[k]) for k in ("binary_path", "prepared", "report", "run02")
    ]
    rows, allow = load_rows(prepared, report)
    args.output.mkdir(parents=True, exist_ok=False)
    out = args.output
    budget = Budget(4 * 3600, out / "STOP")
    write(
        out / "START.json",
        dict(
            utc=datetime.now(timezone.utc).isoformat(),
            limit_seconds=14400,
            manifest=binding(args.manifest),
        ),
    )
    resources = {
        n: json.loads((prepared / f"{n}.json").read_text(encoding="utf-8"))
        for n in ("C", "C-alarm", "F", "F-interaction", "G")
    }
    refs = {n: binding(prepared / f"{n}.json") for n in resources}
    references = {}
    # Baselines are explicitly original evidence; they are never marked re-executed.
    for line in (prepared / "matrix.jsonl").open(encoding="utf-8"):
        row = json.loads(line)
        path = old / f"{row['run_index']:06}.jsonl"
        if row["layer"] == "independent_research" and path.exists():
            references[key(row)] = path
    stopped = None
    timings: dict[str, float] = {}
    began = time.monotonic()
    with (out / "bridge.log").open("x", encoding="utf-8") as log:
        bridge = MatrixBridge(binary, log)
        try:
            write(out / "arithmetic.json", check_hand_vectors())
            phase = time.monotonic()
            with (out / "execution-results.jsonl").open(
                "x", encoding="utf-8"
            ) as results:
                for row in rows:
                    mode = allow[row["run_index"]]["mode"]
                    if "execute" not in mode and "regression" not in mode:
                        continue
                    result = run_row(
                        row, resources, refs, bridge, budget, out, references
                    )
                    dump(results, result)
                    write(
                        out / "PROGRESS.json",
                        dict(
                            phase="execution",
                            run_index=row["run_index"],
                            status=result["status"],
                        ),
                    )
            timings["execution_seconds"] = time.monotonic() - phase
            write(out / "clock-checks.json", clock_checks(out))
            # C-02 receipt checks are run at every applied prefix, with isolated corruptions.
            checks = []
            for row in rows:
                if row["id"] != "C-02" or row["layer"] != "independent_research":
                    continue
                source = out / f"{row['run_index']:06}.jsonl"
                dut = out / f"{row['run_index'] - 1:06}.jsonl"
                if not source.exists() or not dut.exists():
                    continue
                di = {
                    x["prefix"]: x
                    for x in map(
                        json.loads, dut.read_text(encoding="utf-8").splitlines()
                    )
                    if "prefix" in x
                }
                for line in source.read_text(encoding="utf-8").splitlines():
                    x = json.loads(line)
                    if "prefix" in x and (x["audit"].get("f") or {}).get("applied"):
                        killed = wrong_reference_checks(
                            x["audit"]["f"]["synthetic_h"],
                            di[x["prefix"]]["audit"]["f"],
                        )
                        checks.append(
                            dict(
                                run_index=row["run_index"],
                                prefix=x["prefix"],
                                killed=killed,
                            )
                        )
            write(out / "reference-negative-checks.json", checks)
            phase = time.monotonic()
            with (out / "analysis-results.jsonl").open(
                "x", encoding="utf-8"
            ) as results:
                for row in rows:
                    if "analyze" not in allow[row["run_index"]]["mode"]:
                        continue
                    budget.check()
                    if row["planned_status"] == "not_run":
                        dump(
                            results,
                            dict(
                                run_index=row["run_index"],
                                status="not_run",
                                reason=row["reason"],
                            ),
                        )
                        continue
                    path = out / f"{row['run_index']:06}.jsonl"
                    if not path.exists():
                        path = old / path.name
                    result = analyze(
                        row,
                        path,
                        resources,
                        bridge,
                        budget,
                        out / f"analysis-{row['run_index']:06}.jsonl",
                    )
                    dump(results, result)
                    write(
                        out / "PROGRESS.json",
                        dict(
                            phase="analysis",
                            run_index=row["run_index"],
                            status=result["status"],
                        ),
                    )
            timings["analysis_seconds"] = time.monotonic() - phase
            phase = time.monotonic()
            paired_checks(rows, old, out, budget, bridge)
            timings["distance_seconds"] = time.monotonic() - phase
            # The original reducer handles only new execution witnesses in out.
            from reduce_witness import reduce_all
            from full_run import request_for, selected_version, comparable_state

            phase = time.monotonic()
            reduce_all(
                out,
                resources,
                refs,
                bridge,
                budget,
                request_for,
                law_ref,
                selected_version,
                comparable_state,
            )
            timings["reduction_seconds"] = time.monotonic() - phase
        except (Exception, KeyboardInterrupt) as error:
            stopped = f"{type(error).__name__}: {error}"
            traceback.print_exc(file=log)
        finally:
            if stopped:
                bridge.p.terminate()
                bridge.p.wait(timeout=30)
            else:
                bridge.close()
    write(
        out / "END.json",
        dict(
            utc=datetime.now(timezone.utc).isoformat(),
            stopped=stopped,
            seconds=time.monotonic() - began,
            timings=timings,
            incomplete="unlisted completed rows are not_run" if stopped else None,
        ),
    )
    if stopped:
        raise SystemExit(2)


def paired_checks(
    rows: list[dict[str, Any]],
    old: Path,
    out: Path,
    budget: Budget,
    bridge: MatrixBridge,
) -> None:
    pairs: dict[str, list[dict[str, Any]]] = {}
    for row in rows:
        if row["id"] != "09-18" or row["version"] != "A1" or "input" not in row:
            continue
        c = row["input"]
        k = json.dumps([row["layer"], row["world"], c["distance_pair"]], sort_keys=True)
        pairs.setdefault(k, []).append(row)
    with (out / "paired-distance.jsonl").open("x", encoding="utf-8") as stream:
        for group in pairs.values():
            budget.check()
            assert len(group) == 2
            group.sort(key=lambda r: r["branch"])
            observed = []
            intervals = []
            for row in group:
                path = old / f"{row['run_index']:06}.jsonl"
                items = [
                    json.loads(line)
                    for line in path.read_text(encoding="utf-8").splitlines()
                ]
                state = stored_fast(next(i for i in items if i.get("prefix") == 1), row)
                j = row["input"]["coordinate"]
                observed.append(
                    (
                        scalar_bits(state["x"][j]["f64"]),
                        scalar_bits(state["h"]["r1"][j]["f64"]),
                    )
                )
                c = row["input"]
                x, h = Q(c["initial"]["x"]), Q(c["initial"]["h"])
                rate = scalar_bits(c["theta"]["lambda_bits"][j])
                pull = -bridge.primitive("expm1", -rate)
                xp = clamp(add(point(x), mul(point(pull), point(-x))))
                hp = add(point(h), mul(point(pull), point(-h)))
                v = Q(c["events"][0]["target"][j])
                total = add(xp, sub(point(v), hp))
                xi = clamp(total)
                hi = point(v)
                if total[0] >= 1 or total[1] <= -1:
                    wall = Q(1) if total[0] >= 1 else Q(-1)
                    t = add(hp, sub(point(wall), xp))
                    hi = clamp(t, min(hp[0], v), max(hp[1], v))
                elif total[0] <= -1 or total[1] >= 1:
                    candidates = [point(v)] + [
                        clamp(add(hp, sub(point(w), xp)), min(hp[0], v), max(hp[1], v))
                        for w in (Q(-1), Q(1))
                    ]
                    hi = min(t[0] for t in candidates), max(t[1] for t in candidates)
                intervals.append((xi, hi))
            real = distance(group[0]["input"])
            actual = abs(observed[0][0] - observed[1][0]) + abs(
                observed[0][1] - observed[1][1]
            )

            # Actual histories contain Phi before the pulse; separately report
            # the original pure-event exact property and complete-path distance.
            def absolute(i: tuple[Q, Q]) -> tuple[Q, Q]:
                return (
                    Q(0) if i[0] <= 0 <= i[1] else min(abs(i[0]), abs(i[1])),
                    max(abs(i[0]), abs(i[1])),
                )

            envelope = add(
                absolute(sub(intervals[0][0], intervals[1][0])),
                absolute(sub(intervals[0][1], intervals[1][1])),
            )
            failed = (
                distance_failure("A1", real) is not None
                or not envelope[0] <= actual <= envelope[1]
            )
            dump(
                stream,
                dict(
                    rows=[r["run_index"] for r in group],
                    real=real,
                    actual_after=str(actual),
                    binary_distance_envelope=serialize(envelope),
                    status="failed" if failed else "passed",
                    scope="pure real event plus observed Phi/event path; no certified libm error",
                ),
            )


if __name__ == "__main__":
    main()
