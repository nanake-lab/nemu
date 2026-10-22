"""Pure CPU checks for measurement boundaries."""
import unittest

from phase_timing import TokenClock, arm_order, decode_rate


class TimingTests(unittest.TestCase):
    def test_prompt_is_not_a_generated_token(self) -> None:
        times = iter([10.1, 10.3])
        clock = TokenClock(10, lambda: next(times))
        clock.put(object())
        self.assertIsNone(clock.first_ms)
        clock.put(object())
        clock.put(object())
        clock.end()
        self.assertEqual(clock.generated, 2)
        self.assertEqual(len(clock.token_ms), 2)
        self.assertEqual(clock.token_ms, [clock.first_ms, clock.last_ms])
        self.assertAlmostEqual(clock.first_ms or 0, 100)
        self.assertAlmostEqual(clock.last_ms or 0, 300)
        self.assertAlmostEqual(decode_rate(2, clock.first_ms, clock.last_ms) or 0, 5)

    def test_rates_do_not_invent_zero_or_missing_duration(self) -> None:
        for values in [(0, None, None), (1, 3., 3.), (2, 4., 3.), (2, None, 3.)]:
            self.assertIsNone(decode_rate(*values))

    def test_order_is_counterbalanced(self) -> None:
        self.assertEqual(arm_order(0), ("unconstrained", "constrained"))
        self.assertEqual(arm_order(1), ("constrained", "unconstrained"))


if __name__ == "__main__":
    unittest.main()
