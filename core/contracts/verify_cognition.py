"""ADR 0030 compatibility/rollback prerequisites, not SYS17 or cognition runtime."""

from __future__ import annotations

from copy import deepcopy
from collections.abc import Callable
from typing import Any
from fixture_inputs import LIMITS, SCHEMA


def verify(
    commands: dict[str, list[str]],
    probe: Callable[[list[str], list[dict[str, Any]]], list[dict[str, Any]]],
    legal: list[dict[str, Any]],
) -> None:
    # ADR 0030 staged rollout: marking V2 superseded would disable current store writes.
    assert "superseded" not in SCHEMA["messages"]["CoreTransactionV2"]
    assert SCHEMA["messages"]["CoreTransactionV3"]["retains"] == "CoreTransactionV2"
    # ADR 0007 / ADR 0026 §10: old event transactions keep their original bytes;
    # neither wire format may reinterpret them as an internal publication.
    rows = {
        name: [v for v in legal if v["request"]["name"] == name][:3]
        for name in ("CoreTransactionV2", "CoreTransactionV3")
    }
    good, expected, bad = [], [], []
    for name, values in rows.items():
        assert len(values) == 3
        other = (
            "CoreTransactionV3" if name == "CoreTransactionV2" else "CoreTransactionV2"
        )
        for row in values:
            for op in ("canonical", "protobuf"):
                good.append(
                    dict(name=name, op=op, hex=row["expected"][op], limits=LIMITS)
                )
                expected.append(row["expected"])
                bad.append(
                    dict(name=other, op=op, hex=row["expected"][op], limits=LIMITS)
                )
    # ADR 0046 §4. The same Q types can point to either record generation.
    new_refs, wrong_refs = [], []
    for name, key in (("ActivityState", "latest_step"), ("CognitiveInflightAction", "origin_record")):
        case = deepcopy(next(row["request"] for row in legal
                             if row["request"]["name"] == name and key in row["request"]["value"]))
        case["value"][key]["object_type"] = "echo.internal-activity-record-v2"
        new_refs.append(case)
        wrong = deepcopy(case)
        wrong["value"][key]["object_type"] = "echo.internal-activity-commit-v2"
        wrong_refs.append(wrong)
    reference = None
    for lang, command in commands.items():
        results = probe(command, new_refs)
        assert all(r["ok"] for r in results), (lang, "successor activity references")
        if reference is None:
            reference = results
        assert results == reference, (lang, "successor reference bytes")
        assert all(not r["ok"] for r in probe(command, wrong_refs)), (lang, "commit used as record")
        assert probe(command, good) == expected, (lang, "same-version restore bytes")
        assert all(not v["ok"] for v in probe(command, bad)), (
            lang,
            "cross-version reinterpretation",
        )
    print(
        "P5-37: 12 same-version restores + 12 cross-version rejections per language PASS; SYS17/runtime not_run"
    )
