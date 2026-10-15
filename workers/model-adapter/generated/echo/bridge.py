"""Explicit JSON fixture conversion; not an authority representation."""

from __future__ import annotations

from typing import Any
from .runtime import SCHEMA


def from_json(name: str, obj: dict[str, Any]) -> dict[str, Any]:
    fields = {f["name"]: f for f in SCHEMA["messages"][name]["fields"]}

    def atom(typ: str, value: Any) -> Any:
        if typ in SCHEMA["messages"] and isinstance(value, dict):
            return from_json(typ, value)
        if typ in ("uint64", "sint64") and isinstance(value, str):
            number = int(value)
            if str(number) != value:
                raise ValueError("decimal spelling")
            return number
        if typ == "bytes" and isinstance(value, str):
            if len(value) % 2 or any(c not in "0123456789abcdefABCDEF" for c in value):
                raise ValueError("hex")
            return bytes.fromhex(value)
        return value

    out = {}
    for key, value in obj.items():
        f = fields.get(key)
        if f is None or value is None:
            out[key] = value
        elif f["repeated"] and isinstance(value, list):
            out[key] = [atom(f["type"], v) for v in value]
        else:
            out[key] = atom(f["type"], value)
    return out


def to_json(name: str, obj: dict[str, Any]) -> dict[str, Any]:
    fields = {f["name"]: f for f in SCHEMA["messages"][name]["fields"]}

    def atom(typ: str, value: Any) -> Any:
        if typ in SCHEMA["messages"]:
            return to_json(typ, value)
        if typ in ("uint64", "sint64"):
            return str(value)
        if typ == "bytes":
            return value.hex()
        return value

    return {
        k: [atom(fields[k]["type"], v) for v in value]
        if fields[k]["repeated"]
        else atom(fields[k]["type"], value)
        for k, value in obj.items()
    }
