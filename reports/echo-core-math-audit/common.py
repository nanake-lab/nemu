"""Shared result type for the Echo Core math audit checks (stdlib only)."""

from __future__ import annotations

import math
from collections.abc import Callable
from dataclasses import dataclass


@dataclass(frozen=True)
class Result:
    """Outcome of one check.

    kind:
      "property"       -- a claimed property; ok means no violation was found.
      "counterexample" -- a counterexample the audit relies on; ok means it was
                          reproduced exactly as described in the audit report.
    """

    check_id: str
    spec_ref: str
    kind: str
    ok: bool
    detail: str


Check = Callable[[], Result]


def inf_norm(v: list[float]) -> float:
    return max((abs(a) for a in v), default=0.0)


def two_norm(v: list[float]) -> float:
    return math.sqrt(sum(a * a for a in v))
