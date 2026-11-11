"""Normal-CI protocol checks; not the formal matrix or acceptance sample.

Small hand fixtures exercise public adapters, independent encoders and rejects
before the one registered full run. Raw CI logs are separate evidence.
"""

import hashlib
import json
import sys
import tempfile
from pathlib import Path
from typing import Any, TextIO, cast

from matrix_support import (
    Budget,
    MatrixBridge,
    advance,
    audit_difference,
    compare,
    model_for,
)
from full_run import law_ref, request_for
from materialize import expand
from prepare import resources
from summarize_full import resolved_status, sensitivity_status


def summary_boundaries() -> None:
    # Report-only fixtures guard interruption and frozen-exclusion denominators.
    frozen = dict(planned_status="not_run", reason="无固定 ρ 公开入口", layer="DUT")
    assert (
        resolved_status(frozen, dict(status="not_run", reason="STOP"), {}, False)[
            "reason"
        ]
        == frozen["reason"]
    )
    ready = dict(planned_status="ready", layer="DUT")
    pending = dict(status="passed", oracle_comparison="pending")
    assert resolved_status(ready, pending, {}, True)["status"] == "not_run"
    assert (
        resolved_status(ready, pending, dict(status="passed"), True)["status"]
        == "passed"
    )
    assert (
        resolved_status(ready, dict(pending, status="failed"), {}, False)["status"]
        == "failed"
    )
    assert sensitivity_status(18, 1, 1) == "not_run"
    assert sensitivity_status(18, 18, 9) == "passed"
    assert sensitivity_status(18, 18, 0) == "failed"


def main() -> None:
    summary_boundaries()
    c, f, g = resources()
    alarm = dict(c, version="P5-71-v2-C-alarm", jump_notice_bits="0000000000000001")
    del alarm["jump_notice"]
    resource_map = {
        "C": c,
        "C-alarm": alarm,
        "F": f,
        "F-interaction": dict(f, g_max=3, version="P5-71-v2-F-interaction"),
        "G": g,
    }

    def ref(value: Any) -> dict[str, Any]:
        data = json.dumps(
            value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
        ).encode()
        return dict(sha256=hashlib.sha256(data).hexdigest(), length=len(data))

    refs = {name: ref(value) for name, value in resource_map.items()}
    rows: list[dict[str, Any]] = []
    for v in ("old", "A"):
        for branch in (0, 1, 2):
            rows.append(
                dict(id="07-12", family="C", world=1, branch=branch, version=v, N=4)
            )
    for v in ("old", "A1"):
        for variant in range(7):
            rows.append(
                dict(
                    id="09-11",
                    family="F",
                    world=dict(sign=1, coordinate=2, variant=variant),
                    branch=0,
                    version=v,
                    N=8,
                )
            )
        rows.append(
            dict(
                id="09-19",
                family="F",
                world=dict(sign=1, coordinate=1, variant=0),
                branch=0,
                version=v,
                N=7,
            )
        )
        rows.append(
            dict(
                id="09-09",
                family="F",
                world=dict(sign=1, coordinate=0, variant=0),
                branch=0,
                version=v,
                N=3,
            )
        )
    rows.append(
        dict(
            id="09-10-G",
            family="G",
            world=dict(sign=1, coordinate=0),
            branch=3,
            version="G",
            N=5,
        )
    )
    rows.append(
        dict(
            id="C-02",
            family="CF",
            world=dict(sign=1, coordinate=0),
            branch=0,
            version="A-A1",
            N=16,
        )
    )
    # No frozen full trajectory is used as an acceptance sample here. C/F
    # histories are short API checks; G checks its setup plus one backfill.
    with (
        tempfile.TemporaryDirectory(prefix="p571-tooling-") as folder,
        tempfile.TemporaryFile(mode="w+", encoding="utf-8") as log,
    ):
        budget = Budget(600, Path(folder) / "STOP")
        bridge = MatrixBridge(Path(sys.argv[1]), cast(TextIO, log))
        try:
            for row in rows:
                case = expand(row)
                if row["family"] == "G":
                    case["events"] = case["events"][:2]
                elif row["family"] == "CF":
                    case["events"] = case["events"][:2]
                # Change algebra amplitudes for maintenance hand fixtures;
                # original full matrix inputs remain untouched on disk.
                if case["id"] == "09-11":
                    for e in case["events"]:
                        e["target"] = [v / 2 for v in e["target"]]
                row["input"] = case
                row["input_sha256"] = ref(case)["sha256"]
                dut = [
                    x
                    for x in bridge.trajectory(
                        request_for(row, resource_map, refs, budget)
                    )
                    if "prefix" in x
                ]
                name = case.get("resource", case.get("f_resource"))
                assert isinstance(name, str)
                model = model_for(
                    resource_map, case, row["family"], bridge, law_ref(refs[name])
                )
                for prefix, item in enumerate(dut):
                    if prefix:
                        advance(model, case["events"][prefix - 1], prefix - 1, True)
                    difference = compare(
                        model, item, row["family"]
                    ) or audit_difference(model, item, prefix)
                    assert item["i23"] and difference is None, (
                        row["id"],
                        row["version"],
                        row["world"],
                        prefix,
                        difference,
                    )
        finally:
            try:
                if bridge.p.poll() is None:
                    bridge.close()
            finally:
                log.seek(0)
                print(log.read())
    print(
        f"P5-71 full adapter tooling checks: {len(rows)} small protocol fixtures passed"
    )


if __name__ == "__main__":
    main()
