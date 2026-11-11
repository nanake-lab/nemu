"""P5-68 / SYS17 contract prerequisites; no storage or law implementation."""

from __future__ import annotations
import copy
import struct
from collections.abc import Callable
from typing import Any
from fixture_inputs import LIMITS, request
from fixture_law_state import ref


def verify(
    commands: dict[str, list[str]],
    probe: Callable[[list[str], list[dict[str, Any]]], list[dict[str, Any]]],
    legal: list[dict[str, Any]],
) -> None:
    predecessors = {
        name: [v for v in legal if v["request"]["name"] == name][:3]
        for name in (
            "FastStateSnapshotV2",
            "ThetaSnapshotV2",
            "ThetaEstimatorParameters",
        )
    }
    queries: list[dict[str, Any]] = []
    for old in predecessors["FastStateSnapshotV2"]:
        v = copy.deepcopy(old["request"]["value"])
        v.update(
            revision="synthetic-migrated",
            representation_version="ADR0035-A1-v1",
            slots=[],
            law_version=ref("echo.law-version", 1),
        )
        queries.append(request("FastStateSnapshotV3", v))
    outputs = {lang: probe(cmd, queries) for lang, cmd in commands.items()}
    authority = outputs["Rust"]
    records = []
    for old, new in zip(predecessors["FastStateSnapshotV2"], authority, strict=True):

        def snapshot_ref(tag: str, encoded: dict[str, Any]) -> dict[str, Any]:
            return dict(
                schema_version=1,
                digest=encoded["digest"],
                object_type=tag,
                resource_version=encoded["value"]["representation_version"],
                byte_length=str(len(bytes.fromhex(encoded["canonical"]))),
            )

        records.append(
            request(
                "FastStateMigrationRecord",
                dict(
                    schema_version=1,
                    previous_snapshot=snapshot_ref(
                        "echo.fast-state-v2", old["expected"]
                    ),
                    migrated_snapshot=snapshot_ref("echo.fast-state-v3", new),
                    prior_decomposition_unknown=True,
                ),
            )
        )
    encoded_records = {lang: probe(cmd, records) for lang, cmd in commands.items()}
    assert all(r["ok"] for r in encoded_records["Rust"])
    assert all(r == encoded_records["Rust"] for r in encoded_records.values())
    for lang, result in outputs.items():
        assert result == authority, (lang, "migration bytes differ")
        for old, new in zip(predecessors["FastStateSnapshotV2"], result, strict=True):
            assert new["ok"], (lang, new)
            before, after = old["expected"]["value"], new["value"]
            assert before["anchor_time_ns"] == after["anchor_time_ns"]
            assert not after["slots"]
            for key in ("connection", "empathy", "curiosity", "achievement"):
                assert struct.pack("<d", before["nature"][key]) == struct.pack(
                    "<d", after["nature"][key]
                )
            assert before["learned"] == after["learned"]
    # No guessing P from a ThetaSnapshotV2; neither transport permits retagging.
    bad: list[dict[str, Any]] = []
    for old_name, new_name in (
        ("FastStateSnapshotV2", "FastStateSnapshotV3"),
        ("ThetaSnapshotV2", "ThetaSnapshotV3"),
        ("ThetaEstimatorParameters", "ThetaEstimatorParametersV2"),
    ):
        new_rows = (
            authority
            if new_name == "FastStateSnapshotV3"
            else [v["expected"] for v in legal if v["request"]["name"] == new_name][:3]
        )
        for old, new in zip(predecessors[old_name], new_rows, strict=True):
            for op in ("canonical", "protobuf"):
                bad.append(
                    dict(name=new_name, op=op, hex=old["expected"][op], limits=LIMITS)
                )
                bad.append(dict(name=old_name, op=op, hex=new[op], limits=LIMITS))
    for lang, cmd in commands.items():
        assert all(not row["ok"] for row in probe(cmd, bad)), (
            lang,
            "cross-version decode",
        )
        # Rollback keeps original bytes readable with the original messages.
        old_queries = [
            dict(name=name, op=op, hex=v["expected"][op], limits=LIMITS)
            for name, rows in predecessors.items()
            for v in rows
            for op in ("canonical", "protobuf")
        ]
        expected = [
            v["expected"]
            for rows in predecessors.values()
            for v in rows
            for _ in range(2)
        ]
        assert probe(cmd, old_queries) == expected, (lang, "rollback old bytes")
    print(
        "P5-68: 3 empty-slot migrations preserve x/anchor; 36 cross-version rejections and 18 old-byte rollback checks per language PASS (SYS17 contract prerequisites only)"
    )
