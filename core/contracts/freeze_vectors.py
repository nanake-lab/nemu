"""Explicit development operation; NEVER invoked by acceptance or CI.

Run only when intentionally reviewing a new encoding fixture set. Rust is the
authority, and verify.py independently checks Python/TS plus the manual oracle.

Append-only merge (P5-21): frozen vectors are never regenerated. Before anything
is added, every frozen legal vector is re-encoded by the current Rust authority
and must reproduce its frozen request result byte for byte, and every frozen
illegal vector must still be rejected. Seeded vectors are then added only for
messages that have no frozen vector yet, followed by the targeted P5-21 vectors.
Build the probe first: cargo build -p core-types --example contract_probe.
"""

from __future__ import annotations
import json
import os
import subprocess
from pathlib import Path
from typing import Any
from fixture_inputs import requests, targeted_illegal, targeted_legal

ROOT = Path(__file__).resolve().parents[2]
TARGET = Path(__file__).parent / "golden"
PROBE = ROOT / (
    "core/target/debug/examples/contract_probe.exe"
    if os.name == "nt"
    else "core/target/debug/examples/contract_probe"
)


def probe(queries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    result = subprocess.run(
        [str(PROBE)],
        input="".join(json.dumps(q) + "\n" for q in queries),
        text=True,
        capture_output=True,
        check=True,
        encoding="utf-8",
    )
    values = [json.loads(s) for s in result.stdout.splitlines()]
    assert len(values) == len(queries)
    return values


def load(name: str) -> list[dict[str, Any]]:
    path = TARGET / name
    return json.loads(path.read_text(encoding="utf-8")) if path.exists() else []


def main() -> None:
    legal, illegal = load("legal.json"), load("illegal.json")
    # 1. Frozen vectors are re-verified unchanged; any byte difference aborts.
    for vector, got in zip(legal, probe([v["request"] for v in legal]), strict=True):
        assert got == vector["expected"], (
            "frozen legal vector changed",
            vector["request"]["name"],
        )
    for vector, got in zip(
        illegal, probe([v["request"] for v in illegal]), strict=True
    ):
        assert not got["ok"], (
            "frozen illegal vector now accepted",
            vector["request"]["name"],
        )
    frozen = {v["request"]["name"] for v in legal}
    known = [v["request"] for v in legal]
    # 2. Seeded vectors only for messages without frozen vectors, then targeted ones.
    fresh = [q for q in requests() if q["name"] not in frozen]
    fresh += [q for q in targeted_legal() if q not in known and q not in fresh]
    values = probe(fresh)
    for q, v in zip(fresh, values, strict=True):
        assert v["ok"], (q["name"], v)
    added_legal = [
        dict(request=q, expected=v) for q, v in zip(fresh, values, strict=True)
    ]
    seen: set[str] = set()
    added_illegal: list[dict[str, Any]] = []
    for q in fresh:
        if q["name"] in frozen or q["name"] in seen:
            continue
        seen.add(q["name"])
        for label, value in [
            (
                "missing-version",
                {k: v for k, v in q["value"].items() if k != "schema_version"},
            ),
            ("unknown-version", {**q["value"], "schema_version": 2}),
            ("unknown-field", {**q["value"], "undeclared": 0}),
        ]:
            added_illegal.append(dict(label=label, request={**q, "value": value}))
    old_illegal = [v["request"] for v in illegal]
    added_illegal += [v for v in targeted_illegal() if v["request"] not in old_illegal]
    for vector, got in zip(
        added_illegal, probe([v["request"] for v in added_illegal]), strict=True
    ):
        assert not got["ok"], (
            "illegal vector accepted",
            vector["label"],
            vector["request"]["name"],
        )
    TARGET.mkdir(exist_ok=True)
    (TARGET / "legal.json").write_text(
        json.dumps(legal + added_legal, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    (TARGET / "illegal.json").write_text(
        json.dumps(illegal + added_illegal, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(
        f"Re-verified {len(legal)} legal and {len(illegal)} illegal frozen vectors; "
        f"added {len(added_legal)} legal and {len(added_illegal)} illegal. Review before commit."
    )


if __name__ == "__main__":
    main()
