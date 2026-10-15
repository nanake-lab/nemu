"""Generated codec runtime template. No storage, clocks or authority writes."""

from __future__ import annotations

import hashlib
import json
import math
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Any

SCHEMA = json.loads(Path(__file__).with_name("schema.json").read_text(encoding="utf-8"))


@dataclass(frozen=True)
class Limits:
    max_bytes: int
    max_items: int
    max_depth: int


def fail(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def u32(n: int) -> bytes:
    fail(0 <= n <= 0xFFFFFFFF, "count overflow")
    return struct.pack("<I", n)


def blob(b: bytes) -> bytes:
    return u32(len(b)) + b


def scalar(f: dict[str, Any], v: Any, limits: Limits, depth: int) -> Any:
    t = f["type"]
    if t in SCHEMA["messages"]:
        return normalize(t, v, limits, depth + 1)
    if t in SCHEMA["enums"]:
        fail(
            type(v) is int and v != 0 and v in SCHEMA["enums"][t].values(),
            "unknown enum",
        )
    elif t == "bool":
        fail(type(v) is bool, "boolean")
    elif t in ("uint32", "uint64", "sint64"):
        low, high = (
            (0, 2**32 - 1)
            if t == "uint32"
            else (0, 2**64 - 1)
            if t == "uint64"
            else (-(2**63), 2**63 - 1)
        )
        fail(type(v) is int and low <= v <= high, "integer overflow")
    elif t == "double":
        fail(type(v) in (int, float) and math.isfinite(v), "nonfinite/invalid double")
        v = 0.0 if v == 0 else float(v)
    elif t == "string":
        fail(type(v) is str, "string")
        fail(
            len(v.encode("utf-8", errors="strict")) <= limits.max_bytes, "string limit"
        )
    elif t == "bytes":
        fail(type(v) is bytes and len(v) <= limits.max_bytes, "bytes limit/type")
    if "min" in f:
        fail(v >= f["min"], "below minimum")
    if "max" in f:
        fail(v <= f["max"], "above maximum")
    if f.get("nonempty"):
        fail(len(v) > 0, "empty identity/version")
    if "size" in f:
        fail(len(v) == f["size"], "byte size")
    return v


def normalize(name: str, value: Any, limits: Limits, depth: int = 0) -> dict[str, Any]:
    fail(0 <= depth <= limits.max_depth, "depth limit")
    fail(name in SCHEMA["messages"] and type(value) is dict, "message type")
    fields = SCHEMA["messages"][name]["fields"]
    fail(set(value) <= {f["name"] for f in fields}, "unknown field")
    out = {}
    for f in fields:
        key = f["name"]
        if key not in value or value[key] is None:
            fail(not f["required"], "missing required field: " + key)
            continue
        v = value[key]
        if f["repeated"]:
            fail(
                type(v) is list and len(v) <= min(limits.max_items, 0xFFFFFFFF),
                "collection limit/type",
            )
            v = [scalar(f, item, limits, depth) for item in v]
            if "set" in f:
                pairs = [(set_key(f, item, limits), item) for item in v]
                fail(len({k for k, _ in pairs}) == len(pairs), "duplicate set key")
                v = [item for _, item in sorted(pairs, key=lambda p: p[0])]
        else:
            v = scalar(f, v, limits, depth)
        out[key] = v
    fail(out.get("schema_version") == 1, "unknown schema")
    for key, count in SCHEMA["messages"][name]["rules"].get("lengths", {}).items():
        fail(len(out[key]) == out[count], "dimension mismatch")
    cases = SCHEMA["messages"][name]["rules"].get("cases")
    if cases:
        rules = SCHEMA["messages"][name]["rules"]
        discriminator = rules.get("discriminator", "kind")
        case = cases[str(out[discriminator])]
        required = case.get("required", [])
        allowed = set(
            required
            + case.get("optional", [])
            + rules.get("common", [])
            + ["schema_version", discriminator]
        )
        fail(set(required) <= out.keys() and out.keys() <= allowed, "variant fields")
        for key, values in case.get("values", {}).items():
            fail(out[key] in values, "variant enum")
    return out


def set_key(f: dict[str, Any], item: Any, limits: Limits) -> bytes:
    if f["set"] == "self":
        return canonical_scalar(f["type"], item, limits)
    key = next(
        k for k in SCHEMA["messages"][f["type"]]["fields"] if k["name"] == f["set"]
    )
    return canonical_scalar(key["type"], item[key["name"]], limits)


def canonical_scalar(t: str, v: Any, limits: Limits) -> bytes:
    if t in SCHEMA["messages"]:
        return blob(encode_canonical(t, v, limits))
    if t in SCHEMA["enums"] or t == "uint32":
        return u32(v)
    if t == "uint64":
        return struct.pack("<Q", v)
    if t == "sint64":
        return struct.pack("<q", v)
    if t == "double":
        return struct.pack("<d", v)
    if t == "bool":
        return bytes([int(v)])
    return blob(v.encode("utf-8") if t == "string" else v)


def encode_canonical(name: str, value: dict[str, Any], limits: Limits) -> bytes:
    v = normalize(name, value, limits)
    message = SCHEMA["messages"][name]
    out = bytearray(b"NCV1" + u32(1) + blob(message["tag"].encode("utf-8")))
    for f in message["fields"][1:]:
        key = f["name"]
        if f["repeated"]:
            out.extend(u32(len(v[key])))
            for item in v[key]:
                out.extend(canonical_scalar(f["type"], item, limits))
                fail(len(out) <= limits.max_bytes, "message size")
        elif not f["required"]:
            out.append(int(key in v))
            if key in v:
                out.extend(canonical_scalar(f["type"], v[key], limits))
        else:
            out.extend(canonical_scalar(f["type"], v[key], limits))
        fail(len(out) <= limits.max_bytes, "message size")
    fail(len(out) <= limits.max_bytes, "message size")
    return bytes(out)


class Reader:
    def __init__(self, data: bytes, limits: Limits):
        fail(len(data) <= limits.max_bytes, "message size")
        self.data, self.pos = data, 0

    def take(self, n: int) -> bytes:
        fail(0 <= n <= len(self.data) - self.pos, "truncated")
        b = self.data[self.pos : self.pos + n]
        self.pos += n
        return b

    def uint(self) -> int:
        return int(struct.unpack("<I", self.take(4))[0])

    def blob(self) -> bytes:
        return self.take(self.uint())

    def done(self) -> bool:
        return self.pos == len(self.data)


def decode_canonical(
    name: str, data: bytes, limits: Limits, depth: int = 0
) -> dict[str, Any]:
    fail(depth <= limits.max_depth and name in SCHEMA["messages"], "depth/type")
    r = Reader(data, limits)
    m = SCHEMA["messages"][name]
    fail(r.take(4) == b"NCV1" and r.uint() == 1, "magic/schema")
    fail(r.blob().decode("utf-8") == m["tag"], "type tag")

    def atom(t: str) -> Any:
        if t in SCHEMA["messages"]:
            return decode_canonical(t, r.blob(), limits, depth + 1)
        if t in SCHEMA["enums"] or t == "uint32":
            return r.uint()
        if t in ("uint64", "sint64", "double"):
            return struct.unpack(
                {"uint64": "<Q", "sint64": "<q", "double": "<d"}[t], r.take(8)
            )[0]
        if t == "bool":
            b = r.take(1)[0]
            fail(b < 2, "boolean")
            return bool(b)
        raw = r.blob()
        return raw.decode("utf-8") if t == "string" else raw

    value: dict[str, Any] = {"schema_version": 1}
    for f in m["fields"][1:]:
        if f["repeated"]:
            count = r.uint()
            fail(count <= limits.max_items, "count limit")
            value[f["name"]] = [atom(f["type"]) for _ in range(count)]
        elif f["required"]:
            value[f["name"]] = atom(f["type"])
        else:
            present = r.take(1)[0]
            fail(present < 2, "presence")
            if present:
                value[f["name"]] = atom(f["type"])
    fail(r.done(), "trailing bytes")
    out = normalize(name, value, limits, depth)
    # Reject noncanonical set order and negative zero in authority encoding.
    fail(encode_canonical(name, out, limits) == data, "noncanonical encoding")
    return out


def digest(name: str, value: dict[str, Any], limits: Limits) -> str:
    tag = SCHEMA["messages"][name]["tag"].encode("utf-8")
    return hashlib.sha256(
        b"NEMU-CANONICALV1\0" + blob(tag) + encode_canonical(name, value, limits)
    ).hexdigest()


def varint(n: int) -> bytes:
    fail(0 <= n <= 2**64 - 1, "varint overflow")
    out = bytearray()
    while n >= 128:
        out.append((n & 127) | 128)
        n >>= 7
    out.append(n)
    return bytes(out)


def read_varint(r: Reader) -> int:
    n = 0
    for i in range(10):
        b = r.take(1)[0]
        fail(i < 9 or b <= 1, "varint overflow")
        n |= (b & 127) << (7 * i)
        if b < 128:
            return n
    raise ValueError("varint overflow")


def wire_scalar(t: str, v: Any, limits: Limits) -> tuple[int, bytes]:
    if t in SCHEMA["messages"]:
        b = encode_protobuf(t, v, limits)
        return 2, varint(len(b)) + b
    if t in ("string", "bytes"):
        b = v.encode("utf-8") if t == "string" else v
        return 2, varint(len(b)) + b
    if t == "double":
        return 1, struct.pack("<d", v)
    if t == "sint64":
        return 0, varint((v << 1) ^ (v >> 63))
    return 0, varint(int(v))


def encode_protobuf(name: str, value: dict[str, Any], limits: Limits) -> bytes:
    v = normalize(name, value, limits)
    out = bytearray()
    for f in SCHEMA["messages"][name]["fields"]:
        if f["name"] not in v:
            continue
        items = v[f["name"]] if f["repeated"] else [v[f["name"]]]
        for item in items:
            wire, data = wire_scalar(f["type"], item, limits)
            out.extend(varint((f["number"] << 3) | wire) + data)
            fail(len(out) <= limits.max_bytes, "message size")
    return bytes(out)


def decode_protobuf(
    name: str, data: bytes, limits: Limits, depth: int = 0
) -> dict[str, Any]:
    fail(depth <= limits.max_depth and name in SCHEMA["messages"], "depth/type")
    r = Reader(data, limits)
    fields = SCHEMA["messages"][name]["fields"]
    by_id = {f["number"]: f for f in fields}
    value: dict[str, Any] = {f["name"]: [] for f in fields if f["repeated"]}

    def atom(t: str, wire: int, reader: Reader) -> Any:
        expected = (
            2
            if t in SCHEMA["messages"] or t in ("string", "bytes")
            else 1
            if t == "double"
            else 0
        )
        fail(wire == expected, "wire type")
        if wire == 2:
            b = reader.take(read_varint(reader))
            if t in SCHEMA["messages"]:
                return decode_protobuf(t, b, limits, depth + 1)
            return b.decode("utf-8") if t == "string" else b
        if wire == 1:
            return struct.unpack("<d", reader.take(8))[0]
        n = read_varint(reader)
        if t == "sint64":
            return (n >> 1) ^ -(n & 1)
        if t == "bool":
            fail(n < 2, "boolean")
            return bool(n)
        return n

    while not r.done():
        key = read_varint(r)
        number, wire = key >> 3, key & 7
        fail(number in by_id, "unknown field")
        f = by_id[number]
        field = f["name"]
        if not f["repeated"]:
            fail(field not in value, "duplicate singular")
            value[field] = atom(f["type"], wire, r)
        else:
            packed = (
                wire == 2
                and f["type"] not in SCHEMA["messages"]
                and f["type"] not in ("string", "bytes")
            )
            if packed:
                sub = Reader(r.take(read_varint(r)), limits)
                while not sub.done():
                    fail(len(value[field]) < limits.max_items, "count limit")
                    value[field].append(
                        atom(f["type"], 1 if f["type"] == "double" else 0, sub)
                    )
            else:
                fail(len(value[field]) < limits.max_items, "count limit")
                value[field].append(atom(f["type"], wire, r))
    return normalize(name, value, limits, depth)
