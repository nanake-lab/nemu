"""Deterministic source fixtures. Expected bytes are frozen separately, never in verification."""

from __future__ import annotations
import json
from pathlib import Path
from typing import Any

SCHEMA = json.loads(
    (Path(__file__).parent / "generated/schema.json").read_text(encoding="utf-8")
)
LIMITS = dict(max_bytes=1048576, max_items=4096, max_depth=32)


def sample(name: str, seed: int, depth: int = 0) -> dict[str, Any]:
    message = SCHEMA["messages"][name]

    def atom(f: dict[str, Any], n: int) -> Any:
        t = f["type"]
        if t in SCHEMA["messages"]:
            return sample(t, n, depth + 1)
        if t in SCHEMA["enums"]:
            values = [v for v in SCHEMA["enums"][t].values() if v]
            return values[n % len(values)]
        if t == "string":
            return ["A", "中文🙂", "\ufeffzero\0"][n % 3] + str(n)
        if t == "bytes":
            return bytes([n % 256] * f.get("size", n % 3)).hex()
        if t == "bool":
            return n % 2 == 0
        if t == "uint64":
            return str([0, 2**64 - 1, 2**53 + 1][n % 3])
        if t == "sint64":
            return str([0, -(2**63), 2**63 - 1][n % 3])
        if t == "uint32":
            return max(int(f.get("min", 0)), [0, 2**32 - 1, 17][n % 3])
        return max(
            f.get("min", -1e300),
            min(
                f.get("max", 1e300),
                [0.0, f.get("min", -1.25), f.get("max", 2.5)][n % 3],
            ),
        )

    out: dict[str, Any] = {}
    for f in message["fields"]:
        k = f["name"]
        if k == "schema_version":
            out[k] = 1
        elif f["repeated"]:
            out[k] = [atom(f, seed + j) for j in range(seed % 3 if depth < 2 else 0)]
        elif f["required"] or seed % 3:
            out[k] = atom(f, seed)
    for key, count in message["rules"].get("lengths", {}).items():
        out[key] = [atom(next(f for f in message["fields"] if f["name"] == key), seed)]
        out[count] = len(out[key])
    cases = message["rules"].get("cases")
    if cases:
        discriminator = message["rules"].get("discriminator", "kind")
        case = cases[str(out[discriminator])]
        required = case.get("required", [])
        allowed = set(
            required
            + case.get("optional", [])
            + message["rules"].get("common", [])
            + ["schema_version", discriminator]
        )
        out = {k: v for k, v in out.items() if k in allowed}
        for k in required:
            if k not in out:
                out[k] = atom(
                    next(f for f in message["fields"] if f["name"] == k), seed
                )
        for k, values in case.get("values", {}).items():
            out[k] = values[seed % len(values)]
    return out


def requests() -> list[dict[str, Any]]:
    base = [
        dict(name=n, op="encode", value=sample(n, s), limits=LIMITS)
        for n in SCHEMA["messages"]
        for s in range(3)
    ]
    for name in ("Anchor", "GazeTarget", "BodyAction"):
        base.extend(
            dict(name=name, op="encode", value=sample(name, seed), limits=LIMITS)
            for seed in range(3, len(SCHEMA["messages"][name]["rules"]["cases"]))
        )
    return base
