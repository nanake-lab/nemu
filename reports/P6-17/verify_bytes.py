"""Independent P6-17 field-by-field byte review; never rewrites expectations."""

import hashlib
import json
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[2]
TAG = b"echo.touch-v2"


def u32(n):
    return struct.pack("<I", n)


def string(value):
    raw = value.encode("utf-8")
    return u32(len(raw)) + raw


def varint(n):
    out = bytearray()
    while n >= 128:
        out.append((n & 127) | 128)
        n >>= 7
    out.append(n)
    return bytes(out)


def wire_string(number, value):
    raw = value.encode("utf-8")
    return varint(number << 3 | 2) + varint(len(raw)) + raw


def canonical(v):
    out = b"NCV1" + u32(1) + u32(len(TAG)) + TAG + string(v["region"])
    out += b"\x01" + struct.pack("<d", v["speed"]) if "speed" in v else b"\x00"
    out += u32(v["kind"])
    out += b"\x01" + string(v["contact_id"]) if "contact_id" in v else b"\x00"
    out += b"\x01" + struct.pack("<d", v["elapsed_ms"]) if "elapsed_ms" in v else b"\x00"
    return out


def protobuf(v):
    out = b"\x08\x01" + wire_string(2, v["region"])
    if "speed" in v:
        out += b"\x21" + struct.pack("<d", v["speed"])
    out += b"\x28" + varint(v["kind"])
    if "contact_id" in v:
        out += wire_string(6, v["contact_id"])
    if "elapsed_ms" in v:
        out += b"\x39" + struct.pack("<d", v["elapsed_ms"])
    return out


counts = {}
for name in ("legal", "illegal"):
    path = f"core/contracts/golden/{name}.json"
    before = json.loads(subprocess.check_output(["git", "show", f"67cc296c:{path}"], cwd=ROOT))
    after = json.loads((ROOT / path).read_text(encoding="utf-8"))
    assert after[:len(before)] == before, f"{name}: frozen prefix changed"
    counts[name] = dict(preserved=len(before), added=len(after) - len(before))

legal = json.loads((ROOT / "core/contracts/golden/legal.json").read_text(encoding="utf-8"))
vectors = [v for v in legal if v["request"]["name"] == "TouchObservationV2"]
for vector in vectors:
    value, expected = vector["request"]["value"], vector["expected"]
    cb = canonical(value)
    assert expected["canonical"] == cb.hex()
    assert expected["protobuf"] == protobuf(value).hex()
    assert expected["digest"] == hashlib.sha256(b"NEMU-CANONICALV1\0" + u32(len(TAG)) + TAG + cb).hexdigest()
assert {v["request"]["value"]["kind"] for v in vectors} == set(range(1, 10))
print(json.dumps(dict(baseline="67cc296c8b960ae429aceb1f1e0110ef0dbc2f9e",
                     frozen_prefixes=counts, independent_new_byte_checks=len(vectors),
                     result="passed"), indent=2))
