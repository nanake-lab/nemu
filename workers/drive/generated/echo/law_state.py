"""ADR 0035 §4 / P5-68: explicit boundary after structural decoding.

References require trusted H/resource resolution by P5-75. This checks their
representation, not provenance, F eligibility, or a h/current-target bound.
"""

from __future__ import annotations
from typing import Any
from .runtime import Limits, fail, normalize

NAMES = {
    "FastBasisIdentity",
    "LawLearnedCoordinate",
    "LawStateVector",
    "FastBasisSlot",
    "FastStateSnapshotV3",
    "FastEffectRecord",
    "FastStateMigrationRecord",
}


def coordinates(v: dict[str, Any]) -> list[str]:
    return [str(item["tendency_id"]) for item in v["learned"]]


def slot(v: dict[str, Any]) -> None:
    # §4.1: identical coordinate identities, not just equal vector lengths.
    fail(coordinates(v["current"]) == coordinates(v["target"]), "slot coordinates")


def check_law_state_message(name: str, value: dict[str, Any], limits: Limits) -> None:
    if name not in NAMES:
        return
    v = normalize(name, value, limits)
    if name == "FastBasisSlot":
        slot(v)
    elif name == "FastStateSnapshotV3":
        for item in v["slots"]:
            slot(item)
            fail(coordinates(item["current"]) == coordinates(v), "snapshot coordinates")
        # Singleton identity + set uniqueness already excludes two internal branches.
        fail(
            v["law_version"]["object_type"] == "echo.law-version",
            "snapshot law reference",
        )
    elif name == "FastEffectRecord":
        fail(
            bool(v["interpretation_refs"]) and bool(v["governance_refs"]),
            "effect references",
        )
        fail(
            v["theta_before"]["object_type"] == "echo.theta-snapshot-v3",
            "effect theta reference",
        )
        fail(
            v["law_version"]["object_type"] == "echo.law-version",
            "effect law reference",
        )
        fail(v["role"] != 4 or v["basis"]["kind"] == 2, "internal role")
        fail(v["role"] != 1 or v["basis"]["kind"] == 1, "new basis role")
    elif name == "FastStateMigrationRecord":
        fail(v["prior_decomposition_unknown"] is True, "unknown prior decomposition")
        fail(
            v["previous_snapshot"]["object_type"] == "echo.fast-state-v2",
            "migration predecessor",
        )
        fail(
            v["migrated_snapshot"]["object_type"] == "echo.fast-state-v3",
            "migration successor",
        )
