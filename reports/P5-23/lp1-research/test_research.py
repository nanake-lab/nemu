"""Harness checks independent of candidate acceptance; exact-rational witnesses."""

from __future__ import annotations

import importlib.util
import random
import sys
import unittest
from fractions import Fraction as F
from pathlib import Path
from typing import cast

from models import (
    MODELS, Budgeted, CapacityExhausted, MeanPreserving, PrecisionExhausted,
    Reference, capacity_choice, replay,
)
from properties import apply_action, rebuild
from worlds import delay, probability


def exact(name: str, history: list[tuple[F, F]], prior: F = F(0), h0: F = F(3, 10),
          base: F = F(1)) -> F:
    a = b = F(0)
    p = prior / (prior + base)
    flips = last = 0
    for n, (w, s) in enumerate(history):
        rho = 1 - (h0 * 10 + flips) / (10 + n)
        q, v = (s > 0) - (s < 0), w * abs(s)
        c, total, u = base + prior, base + prior + b, prior + a
        if name == "B":
            drift = prior * b - a * c
            if v > 0 and q * drift < 0:
                budget = v * (total - q * u)
                rho = 1 - (1 - rho) * budget / (-q * drift + budget)
        if name == "M":
            p += v / (c + rho * b + v) * (q - p)
        a, b = rho * a + w * s, rho * b + v
        if name != "M":
            p = (prior + a) / (c + b)
        if q:
            flips += bool(last and last != q)
            last = q
    return p


class HarnessTests(unittest.TestCase):
    def test_original_adapter_matches_source_bitwise(self) -> None:
        source = Path(__file__).resolve().parents[2] / "echo-core-v2-draft/check_law.py"
        spec = importlib.util.spec_from_file_location("source_for_test", source)
        assert spec is not None and spec.loader is not None
        module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = module
        spec.loader.exec_module(module)
        factory = cast(type[Reference], getattr(module, "Tendency"))
        original, adapter = factory(3.0, 1.0, prior=60), Reference(3.0, 1.0, prior=60)
        rng = random.Random(20261019)
        for _ in range(1000):
            w, s = rng.uniform(0, 8), rng.uniform(-1, 1)
            self.assertEqual(original.add(w, s).hex(), adapter.add(w, s).hex())
            self.assertEqual((original.a.hex(), original.b.hex(), original.pi().hex()),
                             (adapter.a.hex(), adapter.b.hex(), adapter.pi().hex()))

    def test_original_counterexample_exact(self) -> None:
        first = [(F(3), F(1))] * 3
        self.assertEqual(exact("O", first), F(303, 347))
        self.assertEqual(exact("O", first + [(F(1, 100), F(1))]), F(75893, 90193))
        for name in ("M", "B"):
            self.assertGreater(exact(name, first + [(F(1, 100), F(1))]), exact(name, first))

    def test_candidates_match_exact_short_sequence(self) -> None:
        values = [(3.0, 1.0), (0.01, 1.0), (0.1, -0.5), (0.01, 1.0)]
        for name in ("M", "B"):
            t = replay(MODELS[name], values, n0=1, kappa=0, h0=0.3)
            rational = exact(name, [(F(w), F(s)) for w, s in values])
            self.assertAlmostEqual(t.pi(), float(rational), delta=8 * 2 ** -52)

    def test_budgeted_fixed_rho_memory_bound_counterexample(self) -> None:
        t = Budgeted(1, 0, fixed_rho=0.5)
        for _ in range(20):
            t.add(1, 1)
        self.assertGreater(t.consistent(), 1 / (1 - 0.5))

    def test_weak_evidence_fault_is_atomic(self) -> None:
        for model in (MeanPreserving, Budgeted):
            t = replay(model, [(3, 1)])
            before = t.bits()
            with self.assertRaises(PrecisionExhausted):
                t.add(float.fromhex("0x0.0000000000001p-1022"), 1)
            self.assertEqual(t.bits(), before)

    def test_capacity_fault_and_tie_order(self) -> None:
        states = [replay(Reference, [(8, 1)]) for _ in range(3)]
        with self.assertRaises(CapacityExhausted):
            capacity_choice(states, 2)
        self.assertEqual(capacity_choice([Reference(3, 1)] * 3, 2), 0)

    def test_all_d5_action_kinds_replay_mean_state(self) -> None:
        ledger: list[tuple[float, list[float], list[float]]] = []
        active = [0, 1, 2]
        states = rebuild(MeanPreserving, ledger, active)
        for action in ((0, 0, 1.0, [1.0, 0.5, -1.0]),
                       (0, 0, 2.0, [-1.0, -0.5, 1.0]),
                       (1, 0, 1.0, [1.0, 0.5, -1.0]),
                       (2, 1, 1.0, [0.5, 0.5, -0.5]),
                       (3, 0, 1.0, [0.0, 0.0, 0.0])):
            states, _, _ = apply_action(MeanPreserving, ledger, active, states, action)
            self.assertEqual([s.bits() for s in states],
                             [s.bits() for s in rebuild(MeanPreserving, ledger, active)])
        self.assertEqual(len(active), 2)

    def test_world_boundaries_and_censored_delay(self) -> None:
        self.assertEqual(probability("abrupt", 999), 0.8)
        self.assertEqual(probability("abrupt", 1000), 0.2)
        self.assertEqual(probability("periodic", 250), 0.2)
        self.assertAlmostEqual(probability("gradual", 1499), 0.2)
        self.assertIsNone(delay([1, 1, 1], 0, 3, -1, 1))
        self.assertEqual(delay([1, -1, -1], 0, 3, -1, 2), 3)


if __name__ == "__main__":
    unittest.main()
