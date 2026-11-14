"""One registered full execution, with a 16-hour wall cap and durable prefixes.

Never changes inputs/resources/criteria, never retries a failed worker, and never
resumes a partial output directory. STOP, interruption, tool failure and budget
expiration all preserve the completed prefix and enumerate the remaining rows.
"""

import argparse
import json
import sys
import time
from copy import deepcopy
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, TextIO

from arithmetic import check_hand_vectors
from c_oracle import bit_tree, first_difference
from c_research import CResearch
from f_research import FResearch
from matrix_support import (
    Model,
    Budget,
    MatrixBridge,
    advance,
    audit_difference,
    compare,
    exact_metrics,
    model_for,
    rational_json,
    read_difference,
    read_points,
)
from prepare import binding, write
from probe import peak_bytes
from sensitivity import BaselinePath, arithmetic, candidate
from materialize import negative
from criteria import distance, distance_failure, measurement_mutation
from reduce_witness import reduce_all


def dump(stream: TextIO, value: Any) -> None:
    stream.write(
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    )
    stream.flush()


def law_ref(ref: dict[str, Any]) -> dict[str, Any]:
    return dict(
        schema_version=1,
        digest=ref["sha256"],
        byte_length=str(ref["length"]),
        object_type="echo.law-version",
        resource_version="P5-71-v2",
    )


def request_for(
    row: dict[str, Any], resources: dict[str, Any], refs: dict[str, Any], budget: Budget
) -> dict[str, Any]:
    case = row["input"]
    name = case.get("resource", case.get("f_resource"))
    request = dict(
        op="matrix",
        family=row["family"],
        case={k: v for k, v in case.items() if k != "b_admissions"},
        resource=resources[name],
        resource_ref=refs[name],
        input_ref=dict(
            sha256=row["input_sha256"],
            length=len(
                json.dumps(
                    case, ensure_ascii=False, sort_keys=True, separators=(",", ":")
                ).encode()
            ),
        ),
        remaining_ms=budget.remaining_ms(),
    )
    if row["family"] == "CF":
        request.update(
            c_resource=resources[case["c_resource"]],
            c_resource_ref=refs[case["c_resource"]],
        )
    return request


def selected_version(row: dict[str, Any]) -> str | None:
    version = row["version"]
    if version in (
        "old",
        "A",
        "A1",
        "A2",
        "B",
        "oldC-oldF",
        "A-oldF",
        "oldC-A1",
        "A-A1",
        "B-A1",
        "A-A2",
        "A-B",
        "G",
        "normal",
    ):
        return None
    if row["family"] == "candidate":
        return "normal"
    if row["family"] in ("C", "arithmetic"):
        return "A"
    if row["family"] == "CF":
        return "A-A1"
    if row["family"] == "G":
        return "G"
    return "old" if version == "kappa-one" else "A1"


def key(row: dict[str, Any], version: str | None = None) -> str:
    world = deepcopy(row["world"])
    if isinstance(world, dict) and row["family"] in ("G", "CF"):
        world.pop("variant", None)
    return json.dumps(
        [row["id"], world, row["branch"], version or row["version"]], sort_keys=True
    )


def comparable_state(value: dict[str, Any], family: str) -> dict[str, Any]:
    value = deepcopy(value)
    if family == "C":
        value.pop("prior_mass", None)
    elif family == "CF":
        value["c"].pop("prior_mass", None)
    return value


def run_row(
    row: dict[str, Any],
    resources: dict[str, Any],
    refs: dict[str, Any],
    bridge: MatrixBridge,
    budget: Budget,
    out: Path,
    reference_paths: dict[str, Path],
) -> dict[str, Any]:
    index = row["run_index"]
    result = {
        k: row[k]
        for k in (
            "id",
            "family",
            "layer",
            "world",
            "branch",
            "version",
            "N",
            "run_index",
        )
    }
    result.update(
        status="not_run", completed_prefix=None, differences=[], mutation_killed=None
    )
    if row["planned_status"] == "not_run":
        return dict(result, reason=row["reason"])
    budget.check()
    case = row["input"]
    destination = out / f"{index:06}.jsonl"
    result["evidence"] = destination.name
    selected = selected_version(row)
    reference_path = reference_paths.get(key(row, selected)) if selected else None
    if row["id"].startswith("NC-"):
        positive, _ = negative(row)
        reference_path = reference_paths.get(key(positive, selected))
    with destination.open("x", encoding="utf-8", newline="\n") as raw:
        dump(raw, dict(plan=row))
        if row["layer"] == "DUT":
            for item in bridge.trajectory(request_for(row, resources, refs, budget)):
                dump(raw, item)
                if "completion" in item:
                    result["status"] = (
                        "passed"
                        if item["completion"]["status"] == "completed"
                        else "not_run"
                    )
                    if result["status"] == "not_run":
                        result["reason"] = item["completion"].get("reason")
                        raise InterruptedError(str(result["reason"]))
                else:
                    result["completed_prefix"] = item["prefix"]
                    if not item["i23"]:
                        result["differences"].append(
                            dict(
                                prefix=item["prefix"],
                                kind="I23",
                                detail=item["replay_difference"],
                            )
                        )
                budget.check()
            if result["differences"]:
                result["status"] = "failed"
            # Comparison is finalized by the paired independent row below.
            result["oracle_comparison"] = "pending paired independent row"
            return result
        if row["family"] in ("candidate", "arithmetic"):
            if row["family"] == "candidate":
                measured = candidate(
                    resources[case["resource"]], case, bridge.primitive
                )
                repeated = candidate(
                    resources[case["resource"]], case, bridge.primitive
                )
            else:
                measured, repeated = arithmetic(case), arithmetic(case)
            difference = first_difference(measured, repeated, "standalone-I23")
            dump(raw, dict(prefix=0, state="initial arithmetic inputs", i23=True))
            dump(
                raw,
                dict(
                    prefix=1,
                    state=measured,
                    i23=difference is None,
                    difference=difference,
                ),
            )
            result.update(status=measured.get("status", "passed"), completed_prefix=1)
            if row["family"] == "arithmetic" and (
                measured["rounded_bits"] != measured["nominal_expected_bits"]
                or measured["fault"]
            ):
                result["status"] = "failed"
            if selected:
                result["mutation_killed"] = result["status"] == "failed"
            if difference:
                result["differences"].append(
                    dict(prefix=1, kind="I23", detail=difference)
                )
                result["status"] = "failed"
            return result
        family = row["family"]
        name = case.get("resource", case.get("f_resource"))

        def initial() -> Model:
            return model_for(resources, case, family, bridge, law_ref(refs[name]))

        model = initial()
        dut_path = out / f"{index - 1:06}.jsonl"
        has_dut = False
        if dut_path.exists():
            with dut_path.open(encoding="utf-8") as prior_trace:
                prior_plan = json.loads(next(prior_trace))["plan"]
            has_dut = (
                prior_plan["layer"] == "DUT"
                and prior_plan["input_sha256"] == row["input_sha256"]
            )
        dut_lines = (
            iter(dut_path.read_text(encoding="utf-8").splitlines()[1:])
            if has_dut
            else iter([])
        )
        baseline_lines = (
            iter(reference_path.read_text(encoding="utf-8").splitlines()[1:])
            if reference_path
            else iter([])
        )
        events = case["events"]
        bpath = BaselinePath(case) if case["id"] == "07-10" else None
        for prefix in range(len(events) + 1):
            budget.check()
            start = time.perf_counter_ns()
            if prefix:
                advance(model, events[prefix - 1], prefix - 1, True)
                if bpath:
                    bpath.step(events[prefix - 1])
            incremental_ns = time.perf_counter_ns() - start
            start = time.perf_counter_ns()
            replay = initial()
            replay_bpath = BaselinePath(case) if bpath else None
            for j, event in enumerate(events[:prefix]):
                budget.check()
                advance(replay, event, j, False)
                if replay_bpath:
                    replay_bpath.step(event)
            replay_ns = time.perf_counter_ns() - start
            state = bit_tree(model.state())
            difference = first_difference(
                state, bit_tree(replay.state()), "independent-I23"
            )
            if bpath and replay_bpath:
                difference = difference or first_difference(
                    rational_json(bpath.state()),
                    rational_json(replay_bpath.state()),
                    "b-path-I23",
                )
            dut = json.loads(next(dut_lines, "null"))
            self_difference = difference
            cross_difference = None
            reference_difference = None
            if dut is not None and "prefix" in dut:
                cross_difference = compare(model, dut, family)
                reference_difference = audit_difference(model, dut, prefix)
                difference = difference or cross_difference or reference_difference
            paired = distance(case) if case["id"] == "09-18" else None
            if paired is not None:
                difference = difference or distance_failure(
                    model.case["version"], paired
                )
            if (
                prefix
                and isinstance(model, CResearch)
                and model.last.get("fault")
                and not isinstance(events[prefix - 1].get("a"), str)
            ):
                difference = difference or (
                    "finite-input-numerical-fault",
                    "finite admitted transition",
                    model.last["fault"],
                )
            read_case = case
            if prefix and events[prefix - 1].get("readonly_after_ns") is not None:
                read_case = dict(case, reads_ns=events[prefix - 1]["readonly_after_ns"])
            points = (
                read_points(model, read_case)
                if isinstance(model, (FResearch, CResearch))
                and read_case.get("reads_ns")
                and (prefix == len(events) or read_case is not case)
                else []
            )
            if points and dut:
                difference = difference or read_difference(points, dut)
            metrics = exact_metrics(model) if isinstance(model, CResearch) else None
            if metrics and (
                not metrics["binary"]["stored_in_enclosure"]
                or not metrics["binary"]["real_in_enclosure"]
            ):
                difference = difference or (
                    "propagated-baseline-enclosure",
                    True,
                    metrics["binary"],
                )
            item = dict(
                prefix=prefix,
                state=state,
                audit=bit_tree(model.last),
                i23=self_difference is None,
                self_replay_difference=self_difference,
                oracle_dut_difference=cross_difference,
                reference_difference=reference_difference,
                incremental_ns=incremental_ns,
                replay_ns=replay_ns,
                difference=difference,
                reads=points,
            )
            if metrics is not None:
                item["metrics"] = metrics
            if bpath:
                item["independent_baseline_subpath"] = rational_json(bpath.state())
            if case["id"] == "09-18":
                item["paired_distance"] = paired
            baseline = json.loads(next(baseline_lines, "null"))
            if baseline is not None and prefix > 0:
                mutation = first_difference(
                    comparable_state(baseline["state"], family),
                    comparable_state(state, family),
                    "selected-law-criterion",
                )
                mutation = mutation or first_difference(
                    baseline.get("reads", []), points, "same-anchor-read"
                )
                mutation = mutation or measurement_mutation(
                    row["version"], prefix, len(events), case, baseline, item
                )
                if mutation and result["mutation_killed"] is not True:
                    result["mutation_killed"] = True
                    result["first_mutation_witness"] = dict(
                        prefix=prefix, difference=mutation
                    )
                    write(
                        out / f"{index:06}-mutation-witness.json",
                        dict(
                            plan=row,
                            earliest_prefix=prefix,
                            difference=mutation,
                            reference_evidence=reference_path.name
                            if reference_path
                            else None,
                            original_trace=destination.name,
                            comparison_layer="DUT_oracle" if has_dut else "independent",
                            reduction_status="pending_after_matrix",
                            events=events[:prefix],
                        ),
                    )
            dump(raw, item)
            result["completed_prefix"] = prefix
            if difference:
                result["differences"].append(
                    dict(prefix=prefix, kind="comparison", detail=difference)
                )
                if len(result["differences"]) == 1:
                    write(
                        out / f"{index:06}-first-failure.json",
                        dict(
                            plan=row,
                            earliest_prefix=prefix,
                            difference=difference,
                            original_trace=destination.name,
                            comparison_layer="DUT_oracle" if has_dut else "independent",
                            reduction_status="pending_after_matrix",
                            events=events[:prefix],
                        ),
                    )
            budget.check()
        result["status"] = "failed" if result["differences"] else "passed"
        if selected:
            result["mutation_killed"] = bool(result["mutation_killed"])
            # A detected deliberately broken law is expected; a surviving one
            # is a measurement failure, never counted as a robust mechanism.
            result["measurement_status"] = (
                "passed" if result["mutation_killed"] else "failed"
            )
            result["status"] = "failed"
            result["expected_mutation_failure"] = result["mutation_killed"]
        if "novelty_witness" in case or "decomposition_witness" in case:
            result["identifiability"] = dict(
                status="non_identifiable",
                witness=case.get("novelty_witness", case.get("decomposition_witness")),
            )
    reference_paths[key(row)] = destination
    return result


def main() -> None:
    sys.set_int_max_str_digits(0)
    ap = argparse.ArgumentParser()
    ap.add_argument("prepared", type=Path)
    ap.add_argument("binary", type=Path)
    ap.add_argument("output", type=Path)
    args = ap.parse_args()
    manifest = json.loads(
        (args.prepared / "RUN-MANIFEST.json").read_text(encoding="utf-8")
    )
    for name, expected in manifest["prepared_files"].items():
        if binding(args.prepared / name) != expected:
            raise ValueError(f"input hash mismatch {name}")
    for name, expected in manifest["code_files"].items():
        if binding(Path(__file__).parent / name) != expected:
            raise ValueError(f"code hash mismatch {name}")
    if binding(args.binary) != manifest["binary"]:
        raise ValueError("DUT binary hash mismatch")
    args.output.mkdir(parents=True, exist_ok=False)
    start = datetime.now(timezone.utc)
    budget = Budget(16 * 60 * 60, args.output / "STOP")
    write(
        args.output / "START.json",
        dict(
            start_utc=start.isoformat(),
            limit_seconds=16 * 60 * 60,
            manifest=binding(args.prepared / "RUN-MANIFEST.json"),
            code=manifest["code_commit"],
        ),
    )
    resources = {
        name: json.loads((args.prepared / f"{name}.json").read_text(encoding="utf-8"))
        for name in ("C", "C-alarm", "F", "F-interaction", "G")
    }
    refs = {name: binding(args.prepared / f"{name}.json") for name in resources}
    rows = [
        json.loads(line)
        for line in (args.prepared / "matrix.jsonl")
        .read_text(encoding="utf-8")
        .splitlines()
    ]
    references: dict[str, Path] = {}
    stopped = None
    with (
        (args.output / "bridge.log").open("x", encoding="utf-8") as log,
        (args.output / "results.jsonl").open(
            "x", encoding="utf-8", newline="\n"
        ) as results,
    ):
        bridge = MatrixBridge(args.binary, log)
        try:
            write(args.output / "hand-vectors.json", check_hand_vectors())
            for row in rows:
                if stopped:
                    dump(
                        results,
                        dict(
                            run_index=row["run_index"],
                            id=row["id"],
                            layer=row["layer"],
                            status="not_run",
                            reason=stopped,
                        ),
                    )
                    continue
                try:
                    result = run_row(
                        row, resources, refs, bridge, budget, args.output, references
                    )
                    dump(results, result)
                    write(
                        args.output / "PROGRESS.json",
                        dict(
                            run_index=row["run_index"],
                            id=row["id"],
                            layer=row["layer"],
                            utc=datetime.now(timezone.utc).isoformat(),
                            status=result["status"],
                            completed_prefix=result["completed_prefix"],
                        ),
                    )
                except (InterruptedError, KeyboardInterrupt) as error:
                    stopped = str(error) or "user interruption"
                    dump(
                        results,
                        dict(
                            run_index=row["run_index"],
                            id=row["id"],
                            layer=row["layer"],
                            status="not_run",
                            reason=stopped,
                            prefix_evidence=f"{row['run_index']:06}.jsonl",
                        ),
                    )
                except Exception as error:
                    stopped = f"tool failure: {type(error).__name__}: {error}"
                    dump(
                        results,
                        dict(
                            run_index=row["run_index"],
                            id=row["id"],
                            layer=row["layer"],
                            status="not_run",
                            reason=stopped,
                            prefix_evidence=f"{row['run_index']:06}.jsonl",
                        ),
                    )
                    import traceback

                    traceback.print_exc(file=log)
                    log.flush()
            if not stopped:
                try:
                    reduce_all(
                        args.output,
                        resources,
                        refs,
                        bridge,
                        budget,
                        request_for,
                        law_ref,
                        selected_version,
                        comparable_state,
                    )
                except (InterruptedError, KeyboardInterrupt) as error:
                    stopped = str(error) or "interrupted during witness reduction"
                except Exception as error:
                    stopped = f"witness tool failure: {type(error).__name__}: {error}"
                    import traceback

                    traceback.print_exc(file=log)
                    log.flush()
        finally:
            peak = peak_bytes()
            if stopped:
                bridge.p.terminate()
                bridge.p.wait(timeout=30)
            else:
                bridge.close()
    write(
        args.output / "END.json",
        dict(
            start_utc=start.isoformat(),
            end_utc=datetime.now(timezone.utc).isoformat(),
            stopped=stopped,
            peak_working_set=peak,
            remaining_not_run=stopped is not None,
            witness_reduction="completed"
            if not stopped
            else "partial or not_run; inspect individual witnesses",
            next="summarize without rerunning or changing criteria",
        ),
    )
    print(
        json.dumps(
            dict(stopped=stopped, result_file=str(args.output / "results.jsonl")),
            ensure_ascii=False,
        )
    )
    if stopped:
        sys.exit(2)


if __name__ == "__main__":
    main()
