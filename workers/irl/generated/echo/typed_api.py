"""Generated DTO bridge; wire names and field types come only from the IDL."""

from __future__ import annotations
from dataclasses import fields, is_dataclass
from enum import IntEnum
from typing import Any
from . import types
from .runtime import SCHEMA, Limits, normalize


def to_value(value: Any) -> Any:
    if isinstance(value, IntEnum):
        return int(value)
    if is_dataclass(value) and not isinstance(value, type):
        return {
            f.name.removesuffix("_"): to_value(getattr(value, f.name))
            for f in fields(value)
            if getattr(value, f.name) is not None
        }
    if isinstance(value, list):
        return [to_value(v) for v in value]
    return value


def from_value(name: str, value: dict[str, Any], limits: Limits) -> Any:
    validated = normalize(name, value, limits)

    def atom(t: str, v: Any) -> Any:
        if t in SCHEMA["messages"]:
            return from_value(t, v, limits)
        if t in SCHEMA["enums"]:
            return getattr(types, t)(v)
        return v

    kwargs = {}
    for f in SCHEMA["messages"][name]["fields"]:
        key = f["name"]
        v = validated.get(key)
        kwargs[key + "_" if key == "from" else key] = (
            None
            if v is None
            else (
                [atom(f["type"], x) for x in v] if f["repeated"] else atom(f["type"], v)
            )
        )
    return getattr(types, name)(**kwargs)
