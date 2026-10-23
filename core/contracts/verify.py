"""D10.8 / SYS21: frozen vectors, three implementations, adversarial transport.

This command never updates expected vectors. Probe build is explicit and checked.
"""

from __future__ import annotations
import collections
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
from typing import Any
from fixture_inputs import LIMITS, SCHEMA
from adversarial import cases

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def run(command: list[str]) -> None:
    subprocess.run(
        command, cwd=ROOT / "core" if command[0] == "cargo" else ROOT, check=True
    )


def probe(command: list[str], queries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    result = subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        encoding="utf-8",
        capture_output=True,
        input="".join(json.dumps(q) + "\n" for q in queries),
    )
    rows = [json.loads(line) for line in result.stdout.splitlines()]
    assert len(rows) == len(queries), (command, result.stderr)
    return rows


def varint(n: int) -> bytes:
    out = bytearray()
    while n >= 128:
        out.append((n & 127) | 128)
        n >>= 7
    out.append(n)
    return bytes(out)


def main() -> None:
    run([sys.executable, "core/contracts/generate.py", "--check"])
    run([sys.executable, "-m", "unittest", "discover", "-s", "core/contracts", "-p", "test_*schema.py"])
    # P5-21: no reused field number, enum value or type tag; frozen vectors unchanged.
    run([sys.executable, "core/contracts/check_compat.py"])
    run(
        [
            "cargo",
            "build",
            "--manifest-path",
            "Cargo.toml",
            "-p",
            "core-types",
            "--example",
            "contract_probe",
        ]
    )
    run(
        [
            shutil.which("pnpm") or "pnpm",
            "exec",
            "tsc",
            "--ignoreConfig",
            "--types",
            "node",
            "--module",
            "nodenext",
            "--target",
            "es2022",
            "--strict",
            "--skipLibCheck",
            "--outDir",
            "packages/contracts/src/generated/.test-build",
            "core/contracts/probe.ts",
        ]
    )
    legal = json.loads((HERE / "golden/legal.json").read_text(encoding="utf-8"))
    illegal = json.loads((HERE / "golden/illegal.json").read_text(encoding="utf-8"))
    for name in SCHEMA["messages"]:
        assert sum(v["request"]["name"] == name for v in legal) >= 3
        assert sum(v["request"]["name"] == name for v in illegal) >= 3
    queries = []
    expected = []
    for vector in legal:
        q, e = vector["request"], vector["expected"]
        for op in ("encode", "canonical", "protobuf"):
            queries.append(
                q
                if op == "encode"
                else dict(name=q["name"], op=op, hex=e[op], limits=LIMITS)
            )
            expected.append(e)
    for vector in illegal:
        queries.append(vector["request"])
        expected.append({"ok": False})

    def bad(q: dict[str, Any]) -> None:
        queries.append(q)
        expected.append({"ok": False})

    def wire(
        name: str, data: bytes, op: str = "protobuf", limits: dict[str, int] = LIMITS
    ) -> dict[str, Any]:
        return dict(name=name, hex=data.hex(), op=op, limits=limits)

    for vector in legal[::3]:
        q, e = vector["request"], vector["expected"]
        name = q["name"]
        pb, cb = bytes.fromhex(e["protobuf"]), bytes.fromhex(e["canonical"])
        bad(wire(name, pb + b"\x08\x01"))  # duplicate single schema_version
        bad(wire(name, pb + varint(500 << 3) + b"\0"))  # unknown field
        bad(wire(name, cb + b"\0", "canonical"))
        bad(wire(name, cb[:-1], "canonical"))
        bad(wire(name, cb, "canonical", {**LIMITS, "max_bytes": len(cb) - 1}))
        bad(wire(name, pb[:1]))
        for f in SCHEMA["messages"][name]["fields"]:
            typ, key = f["type"], f["name"]
            invalid: list[Any] = []
            if typ in SCHEMA["enums"]:
                invalid = [0, 4294967295]
            elif typ == "uint32":
                invalid = [-1, 2**32]
            elif typ == "uint64":
                invalid = ["-1", str(2**64)]
            elif typ == "sint64":
                invalid = [str(-(2**63) - 1), str(2**63)]
            elif typ == "double":
                # Inject raw IEEE NaN/+Inf/-Inf into protobuf, not JSON null.
                if key in q["value"]:
                    base = copy.deepcopy(q)
                    if f["required"]:
                        base["value"][key] = 0 if not f["repeated"] else []
                    else:
                        base["value"].pop(key, None)
                for bits in (
                    0x7FF8000000000001,
                    0x7FF0000000000000,
                    0xFFF0000000000000,
                ):
                    # A complete message prefix is unnecessary: known invalid scalar must be rejected.
                    bad(
                        wire(
                            name,
                            b"\x08\x01"
                            + varint((f["number"] << 3) | 1)
                            + struct.pack("<Q", bits),
                        )
                    )
                if "min" in f:
                    invalid.append(f["min"] - 1)
                if "max" in f:
                    invalid.append(f["max"] + 1)
            for value in invalid:
                bq = copy.deepcopy(q)
                bq["value"][key] = [value] if f["repeated"] else value
                bad(bq)
            if f["repeated"]:
                bad(wire(name, cb, "canonical", {**LIMITS, "max_items": 0})) if q[
                    "value"
                ][key] else None
            if f.get("nonempty"):
                bq = copy.deepcopy(q)
                bq["value"][key] = ""
                bad(bq)
    # Additional frozen ADR 0007 Unicode oracles; never regenerate in CI.
    for vector in json.loads((HERE / "golden/unicode.json").read_text(encoding="utf-8")):
        queries.append(vector["request"])
        expected.append(vector["expected"])
    for request, expectation in cases(legal):
        queries.append(request)
        expected.append(expectation)
    run(
        [
            "uv",
            "run",
            "--project",
            "core/contracts",
            "--locked",
            "python",
            "core/contracts/verify_protoc.py",
        ]
    )
    for vector in json.loads(
        (HERE / ".test-build/protoc-vectors.json").read_text(encoding="utf-8")
    ):
        queries.append(vector["request"])
        expected.append(vector["expected"])
    # Hand-calculated ADR 0007 oracle, independently hashed from literal input.
    manual = bytes.fromhex(
        "4e435631010000000f0000006563686f2e6f75747075742d656e64010000004101000000"
    )
    queries.append(
        dict(
            name="OutputEnd",
            op="encode",
            value=dict(schema_version=1, output_id="A", reason=1),
            limits=LIMITS,
        )
    )
    expected.append(
        dict(
            ok=True,
            canonical=manual.hex(),
            protobuf="08011201411801",
            value=dict(schema_version=1, output_id="A", reason=1),
            digest=hashlib.sha256(
                b"NEMU-CANONICALV1\0"
                + bytes.fromhex("0f0000006563686f2e6f75747075742d656e64")
                + manual
            ).hexdigest(),
        )
    )
    commands = {
        "Rust": [
            str(
                ROOT
                / (
                    "core/target/debug/examples/contract_probe.exe"
                    if os.name == "nt"
                    else "core/target/debug/examples/contract_probe"
                )
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
    for language, command in commands.items():
        actual = probe(command, queries)
        for i, (got, want) in enumerate(zip(actual, expected, strict=True)):
            assert got["ok"] == want["ok"], (language, i, queries[i], got, want)
            if want["ok"]:
                assert got == want, (language, i, queries[i]["name"], got, want)
        print(f"{language}: {len(actual)} cases PASS")
    counts = collections.Counter("legal" if e["ok"] else "illegal" for e in expected)
    print(
        json.dumps(
            dict(
                messages=len(SCHEMA["messages"]),
                cases_per_language=len(queries),
                **counts,
            )
        )
    )


if __name__ == "__main__":
    main()
