"""P5-21 byte review of the golden vectors (evidence script; changes nothing).

1. Frozen prefix: the first N entries of each golden file are byte-identical to the
   file at the base ref (default origin/main), compared as text, not re-serialized.
2. Added vectors: every added legal vector's canonical bytes and digest are
   recomputed by the small encoder below, written directly from ADR 0007 rules
   (header, ascending field numbers, presence bytes, little-endian integers,
   binary64 with -0 -> +0, u32-length strings/bytes, nested blobs with their own
   header, sets sorted by canonical key bytes) and independent of the generated
   runtimes, then compared with the frozen hex.

Usage (repository root): python reports/P5-21/review_vectors.py [--base origin/main]
"""

from __future__ import annotations

import argparse
import hashlib
import json
import struct
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
SCHEMA = json.loads(
    (ROOT / "core/contracts/generated/schema.json").read_text(encoding="utf-8")
)


def u32(n: int) -> bytes:
    assert 0 <= n < 2**32
    return struct.pack("<I", n)


def blob(b: bytes) -> bytes:
    return u32(len(b)) + b


def atom(t: str, v: Any) -> bytes:
    if t in SCHEMA["messages"]:
        return blob(message(t, v))
    if t in SCHEMA["enums"] or t == "uint32":
        return u32(v)
    if t == "uint64":
        return struct.pack("<Q", int(v))
    if t == "sint64":
        return struct.pack("<q", int(v))
    if t == "double":
        return struct.pack("<d", 0.0 if v == 0 else float(v))
    if t == "bool":
        return b"\x01" if v else b"\x00"
    if t == "string":
        return blob(v.encode("utf-8"))
    if t == "bytes":
        return blob(bytes.fromhex(v))
    raise AssertionError(t)


def message(name: str, v: dict[str, Any]) -> bytes:
    m = SCHEMA["messages"][name]
    out = b"NCV1" + u32(1) + blob(m["tag"].encode("utf-8"))
    for f in sorted(m["fields"][1:], key=lambda f: f["number"]):
        value = v.get(f["name"])
        if f["repeated"]:
            items = [atom(f["type"], x) for x in value]
            if f.get("set"):
                key = f["set"]
                keyed = [
                    (
                        atom(f["type"], x)
                        if key == "self"
                        else atom(
                            next(
                                g["type"]
                                for g in SCHEMA["messages"][f["type"]]["fields"]
                                if g["name"] == key
                            ),
                            x[key],
                        ),
                        enc,
                    )
                    for x, enc in zip(value, items, strict=True)
                ]
                items = [enc for _, enc in sorted(keyed)]
            out += u32(len(items)) + b"".join(items)
        elif f["required"]:
            out += atom(f["type"], value)
        else:
            out += b"\x01" + atom(f["type"], value) if value is not None else b"\x00"
    return out


def digest(name: str, canonical: bytes) -> str:
    tag = SCHEMA["messages"][name]["tag"].encode("utf-8")
    return hashlib.sha256(b"NEMU-CANONICALV1\0" + blob(tag) + canonical).hexdigest()


def at(ref: str, path: str) -> str:
    return subprocess.run(
        ["git", "show", f"{ref}:{path}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    ).stdout


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", default="origin/main")
    base = parser.parse_args().base
    problems = 0
    for name in ("legal", "illegal", "unicode"):
        path = f"core/contracts/golden/{name}.json"
        old_text = at(base, path)
        new_text = (ROOT / path).read_text(encoding="utf-8")
        old, new = json.loads(old_text), json.loads(new_text)
        # Text prefix: the old file minus its closing "\n]\n" must start the new file.
        head = old_text.rstrip()[:-1].rstrip()
        same_text = new_text.startswith(head) if old else True
        same_entries = new[: len(old)] == old
        print(
            f"{name}: {len(old)} frozen, {len(new) - len(old)} added; "
            f"frozen text prefix identical: {same_text}; entries identical: {same_entries}"
        )
        problems += (not same_text) + (not same_entries)
    legal_old = len(json.loads(at(base, "core/contracts/golden/legal.json")))
    legal = json.loads(
        (ROOT / "core/contracts/golden/legal.json").read_text(encoding="utf-8")
    )
    checked = 0
    for vector in legal[legal_old:]:
        q, e = vector["request"], vector["expected"]
        canonical = message(q["name"], e["value"])
        if (
            canonical.hex() != e["canonical"]
            or digest(q["name"], canonical) != e["digest"]
        ):
            print("MISMATCH", q["name"])
            problems += 1
        checked += 1
    print(
        f"added legal vectors recomputed independently: {checked}, mismatches: {problems}"
    )
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
