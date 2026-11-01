"""Execute the unchanged original LAW functions at their original scale."""

from __future__ import annotations

import hashlib
import importlib.util
import sys
from collections.abc import Callable
from pathlib import Path
from typing import Protocol, cast

from models import Budgeted, MeanPreserving


class Result(Protocol):
    check_id: str
    spec_ref: str
    kind: str
    ok: bool
    detail: str


def run_original(source: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    digest = hashlib.sha256(source.read_bytes()).hexdigest()
    for name, replacement in (("O", None), ("M", MeanPreserving), ("B", Budgeted)):
        module_name = f"original_law_{name}"
        spec = importlib.util.spec_from_file_location(module_name, source)
        if spec is None or spec.loader is None:
            raise RuntimeError("cannot load original reference")
        module = importlib.util.module_from_spec(spec)
        sys.modules[module_name] = module
        spec.loader.exec_module(module)
        if replacement is not None:
            setattr(module, "Tendency", replacement)
        checks = cast(list[Callable[[], Result]], getattr(module, "CHECKS"))
        for check in checks[:10]:
            try:
                result = check()
                row: dict[str, object] = {
                    "model": name, "check": result.check_id, "ok": result.ok,
                    "detail": result.detail, "spec_ref": result.spec_ref,
                    "source_sha256": digest,
                }
            except ArithmeticError as exc:
                row = {
                    "model": name, "check": check.__name__, "ok": False,
                    "detail": f"{type(exc).__name__}: {exc}",
                    "source_sha256": digest, "incomplete_due_to_fault": True,
                }
            print(row, flush=True)
            rows.append(row)
        del sys.modules[module_name]
    if hashlib.sha256(source.read_bytes()).hexdigest() != digest:
        raise RuntimeError("original source changed during execution")
    return rows
