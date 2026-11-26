"""P5-49 internal wire types and terminal ingress, using the three built probes."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any

from fixture_inputs import LIMITS, SCHEMA

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def cases() -> list[tuple[dict[str, Any], dict[str, Any]]]:
    def golden(name: str) -> list[dict[str, Any]]:
        vectors: list[dict[str, Any]] = json.loads(
            (HERE / "golden" / name).read_text(encoding="utf-8")
        )
        return vectors

    names = {n for n, m in SCHEMA["messages"].items() if m["file"] == "irl_pipe.proto"}
    legal = [v for v in golden("legal.json") if v["request"]["name"] in names]
    ingress = {v["label"]: v["request"] for v in golden("ingress.json")}
    chunk = ingress["upload-complete-chunk"]["chunk"]
    base = ingress["durable-receipt-pair"]
    request = {k: v for k, v in base["request"].items() if k != "event"} | dict(kind=2)
    session = base["session"]
    complete = ingress["canonical-object-complete"]

    def upload(value: dict[str, Any]) -> dict[str, Any]:
        return dict(
            name="TerminalRequest",
            op="ingress",
            limits=LIMITS,
            check="request",
            request=request | dict(upload=value),
            session=session,
        )

    pairs = [(upload(chunk), dict(ok=True))]
    covered: set[str] = set()
    for v in legal:
        q, e = v["request"], v["expected"]
        name = q["name"]
        pairs.append((q, e))
        for op in ("canonical", "protobuf"):
            pairs.append((dict(name=name, op=op, hex=e[op], limits=LIMITS), e))
        if name in covered:
            continue
        covered.add(name)
        size = len(bytes.fromhex(e["canonical"]))
        ref = chunk["object"] | dict(
            object_type=SCHEMA["messages"][name]["tag"],
            digest=e["digest"],
            byte_length=str(size),
        )
        # Keep the known-valid upload identical except for its type label.
        pairs.append(
            (
                upload(
                    chunk
                    | dict(
                        object=chunk["object"] | dict(object_type=ref["object_type"])
                    )
                ),
                dict(ok=False),
            )
        )
        assert size <= int(complete["budget"]["max_object_bytes"])
        pairs.append(
            (complete | dict(reference=ref, hex=e["canonical"]), dict(ok=False))
        )
    assert covered == names
    pairs.extend(
        (v["request"], dict(ok=False))
        for v in golden("illegal.json")
        if v["request"]["name"] in names
    )
    return pairs


if __name__ == "__main__":
    from verify import probe

    runs = {
        "Rust": [
            str(
                ROOT
                / "core/target/debug/examples"
                / ("contract_probe.exe" if os.name == "nt" else "contract_probe")
            )
        ],
        "Python": [sys.executable, str(HERE / "python_probe.py")],
        "TypeScript": [
            "node",
            str(
                ROOT
                / "packages/contracts/src/generated/.test-build/core/contracts/probe.js"
            ),
        ],
    }
    pairs = cases()
    for name, command in runs.items():
        replies = probe(command, [q for q, _ in pairs])
        for (q, expected), reply in zip(pairs, replies, strict=True):
            assert reply == expected or (not expected["ok"] and not reply["ok"]), (
                name,
                q,
                reply,
                expected,
            )
        print(f"{name}: {len(pairs)} IRL and terminal ingress cases PASS")
