"""ADR 0049. Validate a parentless bundle before trusted resource resolution."""

from typing import Any

from .runtime import Limits, fail, normalize

_STAGES = ("irl", "appraiser", "matcher", "fast", "growth", "dynamics", "compiler")


def _runtime(q: dict[str, Any]) -> None:
    env = q["environment"]
    for stage, resource in (
        ("irl", "irl"),
        ("appraiser", "nature_appraiser"),
        ("matcher", "situation_matcher"),
    ):
        fail(
            stage not in q or q[stage] == env.get(resource),
            "initial stage resource mismatch",
        )
    for stage in ("fast", "growth", "dynamics", "compiler"):
        fail(
            stage not in q or q[stage] in env["implementations"],
            "unbound initial implementation",
        )
    fail(
        not any(k in q for k in ("fast", "growth", "dynamics")) or "law_version" in env,
        "initial law missing",
    )
    fail("compiler" not in q or "direction_set" in env, "initial directions missing")


def _birth(b: dict[str, Any]) -> None:
    fail(
        b["manifest"]["object_type"] in ("echo.genesis-v2", "echo.genesis-v3"),
        "birth manifest type",
    )
    signature = b["signature_evidence"]
    fail(
        signature["object_type"] == "nemu.storage.raw-bytes-v1"
        and signature["resource_version"] == "1"
        and signature["byte_length"] > 0,
        "birth signature evidence",
    )


def check_initial_message(name: str, value: dict[str, Any], limits: Limits) -> None:
    if name not in (
        "EmptyPrefixRoot",
        "InitialRuntimeRoot",
        "BirthNameRecord",
        "InitialPublication",
    ):
        return
    v = normalize(name, value, limits)
    if name == "EmptyPrefixRoot":
        return
    if name == "InitialRuntimeRoot":
        _runtime(v)
        return
    if name == "BirthNameRecord":
        _birth(v)
        return
    q = v["q"]
    _runtime(q)
    fail(
        v["x"]["object_type"] in ("echo.fast-state-v2", "echo.fast-state-v3")
        and v["r"]["object_type"] == "echo.r-snapshot-v3"
        and v["theta"]["object_type"]
        in ("echo.theta-snapshot-v2", "echo.theta-snapshot-v3"),
        "initial snapshot type",
    )
    lineage = v["header"]["lineage"]
    if lineage == 1:
        fail("growth" not in q and "dynamics" not in q, "rehearsal growth")
    else:
        fail(all(k in q for k in _STAGES), "incomplete initial stages")
    if lineage == 2:
        fail("birth" in v, "missing birth record")
        b = v["birth"]
        _birth(b)
        fail(
            v["header"]["genesis_manifest_digest"] == b["manifest"]["digest"],
            "birth manifest digest",
        )
    else:
        fail(
            v.get("empty_history", {}).get("kind") == 1,
            "initial H requires empty history",
        )
