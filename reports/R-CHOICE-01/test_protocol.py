"""CPU checks for the scoring boundary and fixed experimental design, not behavior evidence."""
from __future__ import annotations

import tempfile
from pathlib import Path
import unittest

from fixtures import CONTRASTS, cases, choice_prompt, final_prompt, tools
from run import ARMS, direction, floats, probabilities, select


class ProtocolTests(unittest.TestCase):
    def test_truth_is_derived_from_rules(self) -> None:
        data = cases()
        self.assertEqual(len(data), 18)
        self.assertEqual(len({c.id for c in data}), 18)
        for c in data:
            index = int(c.id[-1])
            expected = c.hidden < 8+index if c.family == "stock" else c.hidden < 5 if c.family == "route" else c.hidden == 1
            self.assertEqual(c.correct, "A" if expected else "B")
            for order in (0, 1):
                self.assertEqual({x[0] for x in tools(c, order)}, {"relevant", "irrelevant", "submit"})
                self.assertNotIn(c.evidence, choice_prompt(c, order))
            self.assertIn(c.evidence, final_prompt(c, "relevant"))
            self.assertNotIn(c.evidence, final_prompt(c, "irrelevant"))
        self.assertEqual(sum(c.missing for c in data), 12)

    def test_split_and_counterbalance(self) -> None:
        extracts = [stem+" "+v for stem, pos, neg in CONTRASTS for v in (pos, neg)]
        for c in cases():
            self.assertNotIn(c.context, extracts)
            self.assertEqual(tools(c, 0)[0][0], "relevant")
            self.assertEqual(tools(c, 1)[2][0], "relevant")
        self.assertEqual(len(ARMS), 7)
        for amp in (1, 4):
            self.assertEqual(sum(a == amp for _, _, a in ARMS), 3)

    def test_sampler_and_numeric_path(self) -> None:
        p = probabilities({"logits": [0.0, 0.0, 0.0]})
        self.assertEqual(p, [1/3]*3)
        self.assertEqual(select(p, 11, 0.0), 0)
        self.assertEqual(select(p, 11, 0.5), 1)
        self.assertEqual(select(p, 11, 0.9999), 2)
        self.assertEqual(select([0.1, 0.7, 0.2], 0, 0.0), 1)
        with tempfile.TemporaryDirectory(prefix="protocol-") as tmp:
            pth = Path(tmp)/"direction.f32"
            result = direction(pth, [3.0, 4.0])
            norm = result["saved_norm"]
            assert isinstance(norm, float)
            self.assertAlmostEqual(norm, 1, places=6)
            self.assertAlmostEqual(floats(pth)[0], 0.6, places=6)


if __name__ == "__main__":
    unittest.main()
