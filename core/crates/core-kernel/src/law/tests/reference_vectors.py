"""Stream same-input scalar/share vectors from the unmodified ADR 0019 suites.

Run through Rust reference_parity; no user stores. Results and stream digest are
printed to Rust's test log. Full finite states and fault atomicity are compared.
"""
from __future__ import annotations

import contextlib
import hashlib
import io
import json
from pathlib import Path
import struct
import sys
from collections.abc import Iterator
from typing import Any

ROOT = Path(__file__).resolve().parents[6]
sys.path.insert(0, str(ROOT / "reports/echo-core-v2-draft"))
import check_law_v2 as checks  # noqa: E402
import law_v2_model as model  # noqa: E402
import check_law as legacy  # noqa: E402
from law_v2_event_checks import event_checks  # noqa: E402
from law_v2_properties import property_checks  # noqa: E402
from law_v2_review_checks import review_checks  # noqa: E402
from law_v2_side_checks import side_checks  # noqa: E402

digest = hashlib.sha256()
counts = {"scalar": 0, "share": 0, "fault": 0}


def f(value: float) -> str:
    return struct.pack(">d", value).hex()


def emit(parts: Iterator[str] | list[str]) -> None:
    row = " ".join(parts) + "\n"
    digest.update(row.encode())
    sys.stdout.write(row)


original_add = model.Tendency.add
original_shares = model.exact_shares


def add(self: model.Tendency, w: float, s: float, mu: float = 0., projection: float | None = None) -> float:
    pre = [self.n0, self.kappa, self.prior, self.h0, self.nh, self.a, self.b,
           self.magnitude_sum, self.magnitude_weight]
    head = ["A", *map(f, pre), str(self.groups), str(self.flips), str(self.last_sign),
            "none" if self.fixed_rho is None else f(self.fixed_rho), f(w), f(s), f(mu),
            "none" if projection is None else f(projection)]
    before = model.bits(self)
    try:
        q = original_add(self, w, s, mu, projection)
    except model.NumericFault as exc:
        assert model.bits(self) == before, f"reference scalar fault changed state: {exc}"
        counts["fault"] += 1
        emit([*head, "fault", str(exc)])
        raise
    counts["scalar"] += 1
    expected = [q, self.a, self.b, self.pi(), self.magnitude_sum, self.magnitude_weight,
                (self.h0*self.nh+self.flips)/(self.nh+self.groups),
                model.finite(model.finite(self.n0 + self.kappa) + self.b),
                0.0 if self.magnitude_weight == 0.0 else
                model.finite(max(0.0, min(1.0, self.magnitude_sum / self.magnitude_weight)))]
    assert list(map(f, pre[:5])) == list(map(f, (self.n0, self.kappa, self.prior, self.h0, self.nh)))
    assert head[13] == ("none" if self.fixed_rho is None else f(self.fixed_rho))
    emit([*head, "ok", *map(f, expected), str(self.groups), str(self.flips), str(self.last_sign)])
    return q


def shares(pis: list[float], supports: list[float]) -> list[float]:
    head = ["S", str(len(pis)), *map(f, pis), *map(f, supports)]
    try:
        result = original_shares(pis, supports)
    except model.NumericFault as exc:
        counts["fault"] += 1
        emit([*head, "fault", str(exc)])
        raise
    counts["share"] += 1
    emit([*head, "ok", *map(f, result)])
    return result


def main() -> None:
    legacy_tendency = legacy.Tendency
    setattr(model.Tendency, "add", add)
    # Modules imported exact_shares by value; replace only references to that
    # function, while retaining every original check/seed/loop/predicate.
    for module in list(sys.modules.values()):
        if getattr(module, "exact_shares", None) is original_shares:
            setattr(module, "exact_shares", shares)
    for name, fn in (("legacy", checks.original_checks),
                     ("whole", checks.whole_steps),
                     ("counterexamples", checks.counterexamples),
                     ("replay_shares", checks.replay_and_shares),
                     ("numeric", checks.numerical_edges),
                     ("events", event_checks),
                     ("properties", property_checks),
                     ("review", review_checks),
                     ("sides", side_checks)):
        # Vector emission bypasses the check's human-readable stdout capture.
        global emit
        buffer = io.StringIO()
        wire = sys.stdout
        previous = emit

        def emitting(parts: Iterator[str] | list[str]) -> None:
            with contextlib.redirect_stdout(wire):
                previous(parts)

        emit = emitting
        with contextlib.redirect_stdout(buffer):
            result: Any = fn()
        emit = previous
        if name == "legacy":
            setattr(legacy, "Tendency", legacy_tendency)
        sys.stdout.write("R " + json.dumps({"suite": name, "result": result,
                                            "stdout": buffer.getvalue()}, ensure_ascii=True) + "\n")
        if name == "legacy":
            rows = [json.loads(row) for row in buffer.getvalue().splitlines()]
            assert result == 1 and rows[-1]["failed"] == ["LAW-6"]
        elif name == "whole":
            assert result["planned_events"] == 80000 and result["violations"] == 0 and result["faults"] == 0
        elif name == "replay_shares":
            assert result["bitwise_mismatches"] == 0 and result["exact_share_failures"] == 0
        elif name == "events":
            assert all(result[key] == 0 for key in ("append_violations", "marginal_violations",
                       "withdrawal_violations", "replay_mismatches", "conservation_failures", "oracle_mismatches", "faults"))
        else:
            assert result["ok"] is True
    sys.stdout.write("R " + json.dumps({"vectors": counts, "sha256": digest.hexdigest()}) + "\n")


if __name__ == "__main__":
    main()
