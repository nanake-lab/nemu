"""Checks for measurement scoring and counterbalancing, not model success."""
from collections import Counter
import unittest

from calibration_fixtures import LAYOUTS, cases, prompt
from calibrate import summarize


class CalibrationTests(unittest.TestCase):
    def test_balancing_and_fixtures(self) -> None:
        self.assertEqual(len(cases()), 24)
        self.assertEqual(sum(c.missing for c in cases()), 8)
        self.assertEqual(Counter(m.index("relevant") for m, _ in LAYOUTS), {0: 4, 1: 4, 2: 4})
        self.assertEqual(Counter(2-m.index("relevant") if reverse else m.index("relevant")
                                 for m, reverse in LAYOUTS), {0: 4, 1: 4, 2: 4})
        for version in ("I0", "I1"):
            for case in cases():
                for layout, (_, reverse) in enumerate(LAYOUTS):
                    expected = chr(65 + (int(not case.truth) ^ int(reverse)))
                    answer = case.yes if case.truth else case.no
                    self.assertIn(f"{expected}: {answer}", prompt(case, layout, version, True))

    def test_invalid_is_not_discarded(self) -> None:
        rows = [{"case": c.id, "family": c.family, "missing": c.missing, "layout": layout,
                 "reverse": reverse, "choice_valid": False, "final_valid": False, "correct": False,
                 "choice_mass_ok": False, "final_mass_ok": False, "actual_action": "INVALID",
                 "p_relevant_conditional": .5, "relevant_label": chr(65 + m.index("relevant")),
                 "relevant_position": 2-m.index("relevant") if reverse else m.index("relevant")}
                for c in cases() for layout, (m, reverse) in enumerate(LAYOUTS)]
        result = summarize(rows)
        self.assertFalse(result["usable"])
        self.assertEqual(result["n_rows"], 288)
        self.assertEqual(result["overall"]["correct"], 0)
        self.assertEqual(result["per_case"][cases()[0].id]["actions"], {"INVALID": 12})


if __name__ == "__main__":
    unittest.main()
