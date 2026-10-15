"""Independent transport mutations of COMPLETE valid messages, including SYS21."""

from __future__ import annotations
import copy
import struct
from typing import Any
from fixture_inputs import LIMITS, SCHEMA


def varint(n: int) -> bytes:
    out = bytearray()
    while n >= 128:
        out.append((n & 127) | 128)
        n >>= 7
    out.append(n)
    return bytes(out)


def parts(data: bytes) -> list[tuple[int, int, bytes]]:
    pos = 0

    def read() -> int:
        nonlocal pos
        n = 0
        for i in range(10):
            b = data[pos]
            pos += 1
            n |= (b & 127) << (7 * i)
            if b < 128:
                return n
        raise AssertionError("fixture varint")

    result = []
    while pos < len(data):
        start = pos
        tag = read()
        number, wire = tag >> 3, tag & 7
        if wire == 0:
            read()
        elif wire == 1:
            pos += 8
        else:
            assert wire == 2
            size = read()
            pos += size
        result.append((number, wire, data[start:pos]))
    return result


def cases(legal: list[dict[str, Any]]) -> list[tuple[dict[str, Any], dict[str, Any]]]:
    output = []

    def add(q: dict[str, Any], e: dict[str, Any] | None = None) -> None:
        output.append((q, e if e is not None else {"ok": False}))

    for vector in legal:
        q, e = vector["request"], vector["expected"]
        name = q["name"]
        pb = parts(bytes.fromhex(e["protobuf"]))

        def wire(data: bytes, op: str = "protobuf") -> dict[str, Any]:
            return dict(name=name, op=op, hex=data.hex(), limits=LIMITS)

        add(wire(b"".join(p[2] for p in sorted(pb, key=lambda p: -p[0]))), e)
        if any(
            f["type"] in SCHEMA["messages"] and bool(q["value"].get(f["name"]))
            for f in SCHEMA["messages"][name]["fields"]
        ):
            add({**q, "limits": {**LIMITS, "max_depth": 0}})
        for f in SCHEMA["messages"][name]["fields"]:
            key, typ, number = f["name"], f["type"], f["number"]
            if f["required"]:
                missing = copy.deepcopy(q)
                missing["value"].pop(key)
                add(missing)
            values = q["value"].get(key)
            if values is None:
                continue
            if f.get("set") and values:
                duplicate = copy.deepcopy(q)
                duplicate["value"][key].append(values[0])
                add(duplicate)
                reverse = copy.deepcopy(q)
                reverse["value"][key].reverse()
                add(reverse, e)
            if f["repeated"] and values:
                add({**q, "limits": {**LIMITS, "max_items": len(values) - 1}})
            if typ == "double":
                for bits in (
                    0x7FF8000000000001,
                    0x7FF0000000000000,
                    0xFFF0000000000000,
                ):
                    altered = b"".join(
                        varint((number << 3) | 1) + struct.pack("<Q", bits)
                        if n == number
                        else raw
                        for n, w, raw in pb
                    )
                    if any(n == number for n, w, raw in pb):
                        add(wire(altered))
                if (not f["repeated"] and values == 0) or (
                    f["repeated"] and values and all(v == 0 for v in values)
                ):
                    altered = b"".join(
                        varint((number << 3) | 1) + struct.pack("<Q", 1 << 63)
                        if n == number
                        else raw
                        for n, w, raw in pb
                    )
                    add(wire(altered), e)  # -0 transport normalizes to +0
                if f["repeated"] and values:
                    payload = b"".join(struct.pack("<d", v) for v in values)
                    packed = (
                        b"".join(raw for n, w, raw in pb if n != number)
                        + varint((number << 3) | 2)
                        + varint(len(payload))
                        + payload
                    )
                    add(wire(packed), e)
            if typ == "string":
                altered = b"".join(
                    varint((number << 3) | 2) + b"\x01\xff" if n == number else raw
                    for n, w, raw in pb
                )
                if any(n == number for n, w, raw in pb):
                    add(wire(altered))
            if typ in SCHEMA["enums"]:
                altered = b"".join(
                    varint(number << 3) + b"\0" if n == number else raw
                    for n, w, raw in pb
                )
                if any(n == number for n, w, raw in pb):
                    add(wire(altered))
    # Binary canonical count/presence mutations are otherwise complete messages.
    v = next(v for v in legal if v["request"]["name"] == "FastStateSnapshot")
    cb = bytearray.fromhex(v["expected"]["canonical"])
    pos = 12 + len("echo.fast-state")
    for _ in range(2):
        pos += 4 + int.from_bytes(cb[pos : pos + 4], "little")
    pos += 4  # dimension
    overflow = cb.copy()
    overflow[pos : pos + 4] = b"\xff" * 4
    add(
        dict(
            name="FastStateSnapshot", op="canonical", hex=overflow.hex(), limits=LIMITS
        )
    )
    negative = cb.copy()
    negative[pos + 4 : pos + 12] = struct.pack("<Q", 1 << 63)
    add(
        dict(
            name="FastStateSnapshot", op="canonical", hex=negative.hex(), limits=LIMITS
        )
    )
    v = next(v for v in legal if v["request"]["name"] == "TouchObservation")
    cb = bytearray.fromhex(v["expected"]["canonical"])
    cb[-1] = 2
    add(dict(name="TouchObservation", op="canonical", hex=cb.hex(), limits=LIMITS))
    return output
