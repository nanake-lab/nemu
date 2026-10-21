"""Targeted research properties; not AC2 or production acceptance."""
import itertools
import math
import unittest
from dataclasses import replace
from evidence import Criterion, Record, canonical, eligible, estimate, justified_order, predictions
from learning import Adaptive, Change


def integral(segment: tuple[int, ...]) -> float:
    n, s = len(segment), sum(segment)
    return math.factorial(s)*math.factorial(n-s)/math.factorial(n+1)


def enumerated(history: tuple[int, ...], h: float) -> float:
    if not history:
        return .5
    numerator = denominator = 0.0
    for changes in itertools.product((0, 1), repeat=len(history)-1):
        starts = [0] + [i+1 for i, flag in enumerate(changes) if flag]
        ends = starts[1:] + [len(history)]
        w = h**sum(changes)*(1-h)**(len(changes)-sum(changes))
        for start, end in zip(starts, ends):
            w *= integral(history[start:end])
        last = history[starts[-1]:]
        numerator += w*(h*.5+(1-h)*(1+sum(last))/(2+len(last)))
        denominator += w
    return numerator/denominator


class ResearchProperties(unittest.TestCase):
    def setUp(self) -> None:
        self.row = Record("g1", 0, 0, "free", "a", "task", outcome=1)
        self.goal = Criterion("task")

    def test_r01_duplicate_correction_and_real_descendant(self) -> None:
        history = [self.row]*100
        self.assertEqual(canonical(history), [self.row])
        descendant = replace(self.row, root="action_result", order=1)
        corrected = replace(self.row, revision=1, retracted=True)
        history += [descendant, corrected]
        self.assertEqual(eligible(history, self.goal), [descendant])
        self.assertEqual(len(history), 102)  # Audit history itself is retained.

    def test_r02_revision_conflict_late_arrival_no_recency_refresh(self) -> None:
        newer = replace(self.row, revision=1, outcome=0)
        self.assertEqual(canonical([newer, self.row]), [newer])
        with self.assertRaises(ValueError):
            canonical([self.row, replace(self.row, outcome=0)])
        with self.assertRaises(ValueError):
            canonical([self.row, replace(newer, order=9)])

    def test_r03_owner_and_claim_kind(self) -> None:
        data = [replace(self.row, owner="user"),
                replace(self.row, root="g2", kind="user_preference"),
                replace(self.row, root="g3", kind="self_description")]
        self.assertEqual(eligible(data, self.goal), [])

    def test_r04_unknown_is_not_negative_or_a_success(self) -> None:
        unknown = estimate([replace(self.row, outcome=None)]*20)
        self.assertEqual(unknown.forecast, .5)
        self.assertEqual((unknown.lower, unknown.upper), (0, 1))
        self.assertIsNone(justified_order(unknown, unknown))

    def test_r05_criterion_revaluation(self) -> None:
        data = [replace(self.row, root=f"a{i}", order=i, outcome=int(i < 90), cost=.9)
                for i in range(100)]
        data += [replace(self.row, root=f"b{i}", order=100+i, action="b", outcome=int(i < 60), cost=.1)
                 for i in range(100)]
        old, new = Criterion("task", cost_weight=.1), Criterion("task", cost_weight=.8)
        a, b = predictions(data, old, "free")
        self.assertGreater(a.value(old), b.value(old))
        self.assertLess(a.value(new), b.value(new))
        self.assertEqual(len(data), 200)

    def test_r06_pause_replay_and_label_migration_then_learning(self) -> None:
        data = [self.row, replace(self.row, root="g2", order=1, action="b", outcome=0)]
        incremental: list[Record] = []
        for row in data:
            incremental.append(row)
            predictions(incremental, self.goal, "free")
        self.assertEqual(predictions(data, self.goal, "free"), predictions(incremental, self.goal, "free"))
        migrated = [replace(r, action="b" if r.action == "a" else "a") for r in data]
        a, b = predictions(migrated, self.goal, "free")
        self.assertEqual((b, a), predictions(data, self.goal, "free"))
        next_row = replace(self.row, root="g3", order=2, outcome=0)
        data.append(next_row)
        migrated.append(replace(next_row, action="b"))
        a, b = predictions(migrated, self.goal, "free")
        self.assertEqual((b, a), predictions(data, self.goal, "free"))

    def test_r07_no_read_or_candidate_evidence(self) -> None:
        data = [self.row]
        before = predictions(data, self.goal, "free")
        for _ in range(100):
            self.assertEqual(predictions(data, self.goal, "free"), before)
        clones = [replace(self.row, root=f"hypothesis{i}", kind="self_description") for i in range(100)]
        self.assertEqual(predictions(data+clones, self.goal, "free"), before)

    def test_r08_exact_changepoint_enumeration(self) -> None:
        for length in range(7):
            for history in itertools.product((0, 1), repeat=length):
                for h in (.002, .01, .05):
                    model = Change(h)
                    for y in history:
                        model.update(y)
                    self.assertAlmostEqual(model.predict(), enumerated(history, h), delta=1e-12)

    def test_r09_adaptive_replay_and_first_experience(self) -> None:
        original, replay = Adaptive(), Adaptive()
        original.update(1)
        self.assertGreater(original.predict(), .5)
        for y in [0, 1, 1, 0]:
            original.update(y)
        for y in [1, 0, 1, 1, 0]:
            replay.update(y)
        self.assertEqual(original.predict(), replay.predict())
        self.assertEqual(original.weights, replay.weights)


if __name__ == "__main__":
    unittest.main()
