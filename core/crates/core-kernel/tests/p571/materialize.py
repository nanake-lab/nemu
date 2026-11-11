"""Expand the approved matrix before execution. Never calls DUT or an oracle."""

import argparse
import hashlib
import json
from copy import deepcopy
from pathlib import Path
from typing import Any

from inputs_c import c_case
from inputs_cf import cf_case
from inputs_f import f_case
from inputs_g import g_case
from prepare import BASE, PREREG, binding, ledger, resources, write


def negative(row: dict[str, Any]) -> tuple[dict[str, Any], str]:
    r = deepcopy(row)
    name, w = row["version"], row["world"]
    r["id"] = row["id"].removeprefix("NC-" + name + "-")
    r["version"] = (
        "A"
        if row["family"] == "C"
        else "A-A1"
        if row["family"] == "CF"
        else "G"
        if row["family"] == "G"
        else "A1"
    )
    if r["id"] == "07-02":
        r["world"], r["branch"] = dict(p0=(1, 2, 16)[w // 2], rho="dynamic"), w % 2
    else:
        variant = w % 2 if r["id"] == "09-07" else 0
        mirror = w // 2 if r["id"] == "09-07" else w
        r["world"] = dict(
            sign=(-1, 1)[mirror // 4], coordinate=mirror % 4, variant=variant
        )
        if row["family"] == "G":
            r["branch"] = 3
    return r, name


def expand(row: dict[str, Any]) -> dict[str, Any]:
    source = row
    control = ""
    if row["id"].startswith("NC-"):
        source, control = negative(row)
    family = row["family"]
    if family in ("C", "arithmetic"):
        case = c_case(source)
    elif family == "F":
        case = f_case(source)
    elif family == "CF":
        case = cf_case(source)
    elif family == "G":
        case = g_case(source)
    elif family == "candidate":
        case = dict(
            first_normal_step=dict(
                a=(-1, 1)[
                    row["world"] if row["id"] == "07-12-model" else row["world"] // 4
                ],
                p0=2,
                b0=0,
                rho="dynamic",
                time_ns=10**9,
            ),
            coordinate=row["branch"]["coordinate"],
            point=row["branch"]["point"],
            version=row["version"],
            resource="C",
            events=[dict(op="candidate_check")],
        )
    else:
        raise ValueError("unknown matrix family")
    if control:
        case["negative_control"] = control
        case["selected_base"] = source["version"]
        if control == "G-frozen":
            case["selected_base"] = "A1"
            case["f_route_bindings"] = ["A1"]
    if case.get("id") == "09-19":
        case["historical_bindings"] = []
        case["historical_objects"] = []
        for values in case["historical_inputs"]:
            objects = dict(
                theta=dict(
                    revision=values["revision"],
                    b=[0] * 4,
                    g=values["theta_g"],
                    lambda_bits=["3fe62e42fefa39ef"] * 4,
                ),
                effects=dict(
                    revision=values["revision"],
                    tendency_id="k",
                    s=values["effect_s"],
                    r=values["effect_r"],
                    direction=values["effect_direction"],
                ),
            )
            bindings = {}
            for key, body in objects.items():
                raw = json.dumps(body, sort_keys=True, separators=(",", ":")).encode()
                bindings[key] = dict(
                    sha256=hashlib.sha256(raw).hexdigest(), length=len(raw)
                )
            case["historical_objects"].append(objects)
            case["historical_bindings"].append(bindings)
        for event in case["events"]:
            event["recorded_objects"] = dict(
                interpretation=dict(
                    id=event["interpretation_id"],
                    a=event["a"],
                    mu=event["mu"],
                    people=event["involved_people"],
                ),
                governance=dict(
                    id=event["governance_id"], qualification=event["qualification"]
                ),
            )
            event["recorded_bindings"] = {}
            for key, body in event["recorded_objects"].items():
                raw = json.dumps(body, sort_keys=True, separators=(",", ":")).encode()
                event["recorded_bindings"][key] = dict(
                    sha256=hashlib.sha256(raw).hexdigest(), length=len(raw)
                )
    if len(case["events"]) != row["N"]:
        raise ValueError(f"N differs: {row['id']}")
    return case


def deferred() -> list[dict[str, Any]]:
    return [
        dict(
            id=id_,
            layer="production_wiring",
            family="handoff",
            version="not_applicable",
            world="not_applicable",
            branch="not_applicable",
            N=0,
            sum_N=0,
            prefix_recompute=0,
            planned_status="not_run",
            reason=reason,
        )
        for id_, reason in [
            ("07-05-G", "G/R no-candidate wiring; P5-74"),
            ("07-06-identity", "contact identity and runtime admission; P5-73/75"),
            ("07-07-governance", "real governance and transaction; P5-73/75"),
            (
                "09-03-consumption",
                "production consumption keys and atomic commit; P5-72/73/75",
            ),
            ("09-05-G", "derived-source G/R wiring; P5-74"),
            ("09-07-certificates", "production certificate judgment; P5-73/75"),
            ("09-11-recovery", "real restart and consumption recovery; P5-75"),
            ("E-C-D07-D05", "D05 learning and behavior-history feedback; P5-74/76"),
            (
                "E-C-D09-D11",
                "consumption, re-interpretation, backlog, atomicity; P5-72/73/74/75",
            ),
            (
                "E-C-D05-D09",
                "actual fitting, activation, eviction and support governance; P5-74",
            ),
            (
                "E-C-recovery",
                "H/x/R/theta/L/Q transaction recovery and real downtime; P5-74/75",
            ),
            (
                "E-C-faults",
                "structural fault, pause, fuse, notification and recovery wiring; P5-73/74/75",
            ),
            ("real-experience", "real experience and behavior quality; P5-77"),
        ]
    ]


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("output", type=Path)
    args = ap.parse_args()
    out: Path = args.output
    out.mkdir(parents=True, exist_ok=False)
    c, f, g = resources()
    alarm = dict(c, version="P5-71-v2-C-alarm", jump_notice_bits="0000000000000001")
    del alarm["jump_notice"]
    for name, resource in [
        ("C", c),
        ("C-alarm", alarm),
        ("F", f),
        (
            "F-interaction",
            dict(f, version="P5-71-v2-F-interaction", g_max=c["gain_max"]),
        ),
        ("G", g),
    ]:
        write(out / f"{name}.json", resource)
    totals: dict[str, dict[str, int]] = {}
    counts: dict[str, int] = {}
    with (out / "matrix.jsonl").open("x", encoding="utf-8", newline="\n") as stream:
        for index, row in enumerate(ledger() + deferred()):
            row = dict(row, run_index=index)
            if row["planned_status"] != "not_run":
                row["planned_status"] = "ready"
                row["input"] = expand(row)
                raw = json.dumps(
                    row["input"],
                    ensure_ascii=False,
                    sort_keys=True,
                    separators=(",", ":"),
                ).encode()
                row["input_sha256"] = hashlib.sha256(raw).hexdigest()
                if row["family"] == "G" and row["layer"] == "DUT":
                    routes = len(row["input"]["f_route_bindings"])
                    row["subject_trajectories"] = 1
                    row["G_route_calls"] = routes
                    row["sum_N"] *= routes
                    row["prefix_recompute"] *= routes
            stream.write(
                json.dumps(
                    row, ensure_ascii=False, sort_keys=True, separators=(",", ":")
                )
                + "\n"
            )
            key = row["layer"] + "/" + row["family"]
            t = totals.setdefault(
                key, dict(rows=0, sum_N=0, prefix_recompute=0, not_run_rows=0)
            )
            t["rows"] += 1
            for k in ("sum_N", "prefix_recompute"):
                t[k] += row[k]
            t["not_run_rows"] += row["planned_status"] == "not_run"
            counts[row["id"]] = counts.get(row["id"], 0) + 1
            if (
                row["id"] == "07-10"
                and row["family"] == "C"
                and row["layer"] == "independent_research"
            ):
                t = totals.setdefault(
                    "independent_research/baseline_subpath",
                    dict(rows=0, sum_N=0, prefix_recompute=0, not_run_rows=0),
                )
                t["rows"] += 1
                t["sum_N"] += row["N"]
                t["prefix_recompute"] += row["N"] * (row["N"] + 1) // 2
    write(out / "counts.json", counts)
    write(out / "totals.json", totals)
    write(
        out / "preparation.json",
        dict(
            base=BASE,
            prereg=PREREG,
            mode="full_input_materialization_only",
            seed="P5-71-v1-enumerated",
            time_limit_seconds=16 * 60 * 60,
            coordinate_expansion="C scalar parameters apply identically to all four coordinates; F theta vectors are explicit in each input",
            files={p.name: binding(p) for p in sorted(out.iterdir()) if p.is_file()},
        ),
    )


if __name__ == "__main__":
    main()
