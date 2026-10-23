"""Replay the original event/review/side tapes, emit each distinct full input.

The Rust adapter supplies prepared tendencies and W/S plans. Event ordering,
share planning, stored-q revision, first insertion, cf and publication use G.
Candidate/alias construction, appraisals, relations and capacity need G tests.
"""
from __future__ import annotations

import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import struct
import sys
from dataclasses import asdict

sys.path.insert(0, str(Path(__file__).resolve().parents[6] / "reports/echo-core-v2-draft"))
from law_v2_events import Change, Engine, Event  # noqa: E402
from law_v2_model import NumericFault, finite  # noqa: E402
from law_v2_event_checks import event_checks  # noqa: E402
from law_v2_review_checks import review_checks  # noqa: E402
from law_v2_side_checks import side_checks  # noqa: E402

wire = sys.stdout
seen: set[str] = set()
digest = hashlib.sha256()
original_apply = Engine.apply
original_append = Engine.append
successes = faults = 0


def f(value: float) -> str:
    return struct.pack(">d", value).hex()


def states(engine: Engine) -> list[str]:
    out = [str(len(engine.tendencies))]
    for name, t in sorted(engine.tendencies.items()):
        magnitude = 0.0 if t.magnitude_weight == 0.0 else finite(max(0.0, min(1.0, t.magnitude_sum / t.magnitude_weight)))
        out += [name, *map(f, (t.a, t.b, t.magnitude_sum, t.magnitude_weight, t.pi(),
                              finite(finite(t.n0 + t.kappa) + t.b),
                              (t.h0*t.nh+t.flips)/(t.nh+t.groups), magnitude)),
                str(t.groups), str(t.flips), str(t.last_sign)]
    return out


def allocations(engine: Engine) -> list[str]:
    out = [str(sum(map(len, engine.allocations.values())))]
    for name, rows in sorted(engine.allocations.items()):
        for ordinal, a in enumerate(rows):
            out += [name, a.group, str(ordinal), *map(f, (a.weight, a.support, a.share, a.q))]
    return out


def emit(before: Engine, event: Event, after: Engine, cf: dict[str, float], fault: str = "") -> None:
    global successes, faults
    identity = hashlib.sha256((before.snapshot() + json.dumps(asdict(event), sort_keys=True)).encode()).hexdigest()
    if identity in seen:
        return
    seen.add(identity)
    parts = ["E", f(before.prior), event.revision_group or "none", event.competitor or "none",
             f(event.competitor_support), event.new_group or "none", f(event.new_weight),
             str(len(event.new_supports))]
    for name, support in event.new_supports:
        parts += [name, f(support)]
    parts += states(before) + allocations(before)
    if fault:
        assert before.snapshot() == after.snapshot(), f"reference event fault changed state: {fault}"
        faults += 1
        parts += ["fault", fault]
    else:
        successes += 1
        parts += ["ok", str(int(before.snapshot() != after.snapshot())), str(len(cf))]
        for name, pi in sorted(cf.items()):
            parts += [name, f(pi)]
        parts += states(after) + allocations(after)
    row = " ".join(parts) + "\n"
    digest.update(row.encode())
    wire.write(row)


def apply(self: Engine, event: Event) -> tuple[dict[str, float], list[Change]]:
    before = copy.deepcopy(self)
    try:
        result = original_apply(self, event)
    except NumericFault as exc:
        emit(before, event, self, {}, str(exc))
        raise
    emit(before, event, self, result[0])
    return result


def append(self: Engine, group: str, weight: float, supports: tuple[tuple[str, float], ...]) -> None:
    before = copy.deepcopy(self)
    event = Event(new_group=group, new_weight=weight, new_supports=supports)
    try:
        original_append(self, group, weight, supports)
    except NumericFault as exc:
        emit(before, event, self, {}, str(exc))
        raise
    cf = {name: before.tendencies[name].pi() if name in before.tendencies else self.prior/(self.prior+4.)
          for name in self.tendencies}
    emit(before, event, self, cf)


def main() -> None:
    setattr(Engine, "apply", apply)
    setattr(Engine, "append", append)
    for name, fn in (("events", event_checks), ("review", review_checks), ("sides", side_checks)):
        with contextlib.redirect_stdout(io.StringIO()):
            result = fn()
        wire.write("R " + json.dumps({"suite": name, "result": result}) + "\n")
        if name == "events":
            assert all(result[key] == 0 for key in ("append_violations", "marginal_violations",
                       "withdrawal_violations", "replay_mismatches", "conservation_failures", "oracle_mismatches", "faults"))
        else:
            assert result["ok"] is True
    wire.write("R " + json.dumps({"unique_events": successes, "fault_events": faults,
                                  "sha256": digest.hexdigest()}) + "\n")


if __name__ == "__main__":
    main()
