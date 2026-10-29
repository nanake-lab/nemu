"""Typed boundary around the existing jsonschema runtime, with no stub dependency."""
from __future__ import annotations

from collections.abc import Callable
import importlib
from typing import Any, Protocol, cast


class Validator(Protocol):
    def validate(self, instance: object) -> None: ...


_api = importlib.import_module("jsonschema")
ValidationError = cast(type[Exception], _api.ValidationError)
_factory = cast(Callable[[dict[str, Any]], Validator], _api.Draft202012Validator)


def validator(schema: dict[str, Any]) -> Validator:
    return _factory(schema)
