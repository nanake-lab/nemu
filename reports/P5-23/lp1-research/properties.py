"""Seeded LP1 checks, replay witnesses and numeric boundaries; retain every failure."""

from __future__ import annotations

import copy
import json
import math
import random
import struct
from collections.abc import Callable
from typing import TextIO

from models import (
    MODELS, Budgeted, CapacityExhausted, MeanPreserving, PrecisionExhausted,
    Reference, capacity_choice, replay, shares,
)

SEED = 20261019


def save_failure(out: TextIO, record: dict[str, object]) -> None:
    out.write(json.dumps(record, ensure_ascii=False, allow_nan=False) + "\n")


def lp1_checks(out: TextIO) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for check in (1, 2, 3):
        for name, model in MODELS.items():
            rng = random.Random(SEED + 100 + check)
            bad = faults = positive = negative = 0
            for trial in range(20000):
                prior = rng.choice((0.0, 60.0))
                h0 = rng.uniform(0.02, 0.3)
                history = [
                    (10 ** rng.uniform(-10, math.log10(8)),
                     rng.choice((-1, 1)) * rng.uniform(0.05, 1))
                    for _ in range(rng.randint(1, 40))
                ]
                new = (10 ** rng.uniform(-10, math.log10(8)), rng.choice((-1.0, 1.0)))
                selected = rng.randrange(len(history))
                factor = 0.0 if trial % 5 == 0 else rng.random()
                updated = list(history)
                w, s = updated[selected]
                updated[selected] = w * factor, s
                record: dict[str, object] = {
                    "check": f"LP1-{check}", "model": name, "trial": trial,
                    "prior": prior, "h0": h0, "history": history,
                    "selected": selected, "factor": factor, "new": new,
                }
                try:
                    if check == 1:
                        before = replay(model, history, prior=prior, h0=h0).pi()
                        after = replay(model, history + [new], prior=prior, h0=h0).pi()
                        direction = new[1]
                        ok = (after - before) * direction > 0
                    elif check == 2:
                        # Freeze exactly the same updated old allocations and ordinal order.
                        before = replay(model, updated, prior=prior, h0=h0).pi()
                        after = replay(model, updated + [new], prior=prior, h0=h0).pi()
                        direction = new[1]
                        ok = (after - before) * direction > 0
                    else:
                        before = replay(model, history, prior=prior, h0=h0).pi()
                        after = replay(model, updated, prior=prior, h0=h0).pi()
                        direction = s
                        ok = (after - before) * direction <= 0
                    positive += direction > 0
                    negative += direction < 0
                    if not ok:
                        bad += 1
                        save_failure(out, record | {"before": before, "after": after})
                except PrecisionExhausted as exc:
                    faults += 1
                    save_failure(out, record | {"fault": str(exc)})
            rows.append({"check": f"LP1-{check}", "model": name, "trials": 20000,
                         "violations": bad, "faults": faults, "positive": positive,
                         "negative": negative, "ok": bad == 0 and faults == 0})
    return rows


def lp7_checks(out: TextIO) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for name, model in MODELS.items():
        rng = random.Random(SEED + 104)
        bad = faults = 0
        for trial in range(20000):
            # Hold pi, rho and c fixed; vary only discounted evidence amount.
            p, c = rng.uniform(-0.8, 0.8), rng.uniform(0.5, 10)
            b = c * abs(p) / (1 - abs(p)) + rng.uniform(1, 20)
            larger = b + rng.uniform(1, 100)
            rho = rng.uniform(0.1, 1)
            w, s = rng.uniform(0.01, 8), rng.choice((-1.0, 1.0))
            low = model(c, 0.0, fixed_rho=rho, a=p * (c + b), b=b)
            high = model(c, 0.0, fixed_rho=rho, a=p * (c + larger), b=larger)
            if isinstance(low, MeanPreserving) and isinstance(high, MeanPreserving):
                low.mean = high.mean = p
            try:
                low_before, high_before = low.pi(), high.pi()
                low.add(w, s)
                high.add(w, s)
                small = abs(low.pi() - low_before)
                large = abs(high.pi() - high_before)
                if large > small:
                    bad += 1
                    save_failure(out, {"check": "LP7-fixed-pi", "model": name,
                                       "trial": trial, "p": p, "c": c, "b": b,
                                       "larger": larger, "rho": rho, "w": w, "s": s,
                                       "small_delta": small, "large_delta": large})
            except PrecisionExhausted as exc:
                faults += 1
                save_failure(out, {"check": "LP7-fixed-pi", "model": name,
                                   "trial": trial, "fault": str(exc)})
        rows.append({"check": "LP7-fixed-pi", "model": name, "trials": 20000,
                     "violations": bad, "faults": faults, "ok": bad == 0 and faults == 0})
    return rows


# A projected-input replay model, deliberately not a second implementation of G.
Group = tuple[float, list[float], list[float]]  # raw weight, support, allocated weights
Action = tuple[int, int, float, list[float]]


def rebuild(model: type[Reference], ledger: list[Group], active: list[int]) -> list[Reference]:
    return [replay(model, [(weights[i], supports[i]) for _, supports, weights in ledger])
            for i in active]


def apply_action(
    model: type[Reference], ledger: list[Group], active: list[int],
    states: list[Reference], action: Action,
) -> tuple[list[Reference], int, int]:
    kind, index, weight, supports = action
    faults = checks = 0
    if kind == 3:
        try:
            removed = capacity_choice(states, 2)
        except CapacityExhausted:
            return states, 1, 0  # No mutation on an explicit capacity fault.
        if removed is None:
            return states, 0, 0
        del active[removed]
        del states[removed]
        targets = range(len(ledger))
    elif kind == 0 or not ledger:
        ledger.append((weight, list(supports), [0.0] * 3))
        targets = range(len(ledger) - 1, len(ledger))
    else:
        index %= len(ledger)
        old_weight, old_supports, old_alloc = ledger[index]
        # Back-sharing keeps S; late members can change S. Both preserve the ordinal.
        ledger[index] = (old_weight if kind == 1 else weight,
                         old_supports if kind == 1 else list(supports), old_alloc)
        targets = range(index, index + 1)
    pis = [state.pi() for state in states]
    for pos in targets:
        raw, sup, _ = ledger[pos]
        sh = shares(pis, [sup[i] for i in active])
        allocation = [0.0] * 3
        for j, i in enumerate(active):
            allocation[i] = raw * sh[j]
        for side in (1, -1):
            indices = [j for j, i in enumerate(active) if (sup[i] > 0) - (sup[i] < 0) == side]
            if indices:
                checks += 1
                if abs(sum(sh[j] for j in indices) - 1) > 1e-12:
                    raise AssertionError("I25 side share sum")
                if sum(allocation[active[j]] * abs(sup[active[j]]) for j in indices) > raw * (1 + 1e-12):
                    raise AssertionError("I25 side credit budget")
        ledger[pos] = raw, sup, allocation
    if kind == 0:
        _, sup, allocation = ledger[-1]
        for state, i in zip(states, active, strict=True):
            state.add(allocation[i], sup[i])
    else:
        states = rebuild(model, ledger, active)
    return states, faults, checks


def roundtrip(states: list[Reference]) -> None:
    for state in states:
        a, b, p, _ = struct.unpack(">dddd", struct.pack(">dddd", *state.values()))
        state.a, state.b = a, b
        if isinstance(state, MeanPreserving):
            state.mean = p


def replay_checks(out: TextIO) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for name, model in MODELS.items():
        rng = random.Random(SEED + 105)
        bad = faults = capacity_faults = share_checks = prefixes = 0
        for trial in range(500):
            actions: list[Action] = [
                (3 if j == 31 else j % 3, rng.randrange(40), rng.uniform(0.01, 8),
                 [rng.uniform(-1, 1) for _ in range(3)]) for j in range(40)
            ]
            ledger: list[Group] = []
            active = [0, 1, 2]
            states = rebuild(model, ledger, active)
            try:
                for end, action in enumerate(actions, 1):
                    states, cf, sc = apply_action(model, ledger, active, states, action)
                    capacity_faults += cf
                    share_checks += sc
                    roundtrip(states)
                    rebuilt_ledger: list[Group] = []
                    rebuilt_active = [0, 1, 2]
                    full = rebuild(model, rebuilt_ledger, rebuilt_active)
                    for past in actions[:end]:
                        full, _, _ = apply_action(model, rebuilt_ledger, rebuilt_active, full, past)
                    replayed = rebuild(model, ledger, active)
                    prefixes += 1
                    if (active != rebuilt_active or ledger != rebuilt_ledger
                            or [s.bits() for s in states] != [s.bits() for s in full]
                            or [s.bits() for s in states] != [s.bits() for s in replayed]):
                        bad += 1
                        save_failure(out, {"check": "I23-projected", "model": name,
                                           "trial": trial, "end": end, "actions": actions[:end]})
            except (PrecisionExhausted, AssertionError) as exc:
                faults += 1
                save_failure(out, {"check": "I23-projected", "model": name,
                                   "trial": trial, "actions": actions, "fault": str(exc)})
        rows.append({"check": "I23-I25-projected", "model": name, "prefixes": prefixes,
                     "share_checks": share_checks, "violations": bad, "faults": faults,
                     "capacity_faults": capacity_faults, "ok": bad == 0 and faults == 0})
    return rows


def edge_checks() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    history = [(3.0, 1.0)] * 3
    for name, model in MODELS.items():
        t = replay(model, history, n0=1.0, kappa=0.0, h0=0.3)
        before = t.pi()
        t.add(0.01, 1.0)
        rows.append({"check": "ADR0018", "model": name, "before": before,
                     "after": t.pi(), "ok": t.pi() > before})
    for model in (MeanPreserving, Budgeted):
        for label, action in (
            ("weak-subnormal", lambda t: t.add(math.ulp(0.0), 1.0)),
            ("nonfinite", lambda t: t.add(float("inf"), 1.0)),
            ("overflow", lambda t: t.add(1.7976931348623157e308, 1.0)),
        ):
            t = replay(model, [(3.0, 1.0)])
            if label == "overflow":
                t.b = t.a = 1.7976931348623157e308
            old = copy.deepcopy(t)
            status = "silent"
            try:
                cast_action: Callable[[Reference], float] = action
                cast_action(t)
            except PrecisionExhausted:
                status = "PrecisionExhausted"
            rows.append({"check": label, "model": model.__name__, "status": status,
                         "atomic": t == old, "ok": status == "PrecisionExhausted" and t == old})
    return rows
