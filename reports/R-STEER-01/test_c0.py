"""Protect against the saturation/layout failure modes observed in R-CHOICE."""
from __future__ import annotations

import unittest
from collections import Counter

from analyze_c0 import Row, gates
from fixtures import make_jobs


def example_rows() -> list[Row]:
    return [Row(id=j['id'], root=j['root'], lang=j['lang'], kind=j['kind'], concept=j['concept'],
                style=j['style'], mapping=j['mapping'], order=j['order'], alphabet=j['alphabet'],
                q=0.5, mass=1.0, valid=True, correct=True)
            for j in make_jobs() if j['kind'] != 'free']


class CalibrationTests(unittest.TestCase):
    def test_complete_code_position_crossing(self) -> None:
        jobs = make_jobs()
        self.assertEqual(Counter(j['kind'] for j in jobs), {'expression': 384, 'oracle': 48, 'free': 24})
        for lang in ('zh', 'en', 'ja'):
            group = [j for j in jobs if j['kind'] == 'expression' and j['lang'] == lang]
            counts = Counter((j['root'], j['style'], j['alphabet'], j['target'], j['mapping'] ^ j['order']) for j in group)
            self.assertEqual(len(counts), 128)
            self.assertEqual(set(counts.values()), {1})
        self.assertEqual(len({j['id'] for j in jobs}), len(jobs))

    def test_saturation_cannot_be_rescued_by_perfect_oracle(self) -> None:
        rows = example_rows()
        self.assertTrue(gates(rows)['interface_passed'])
        for row in rows:
            if row['kind'] == 'expression':
                row['q'] = 0.999
        self.assertFalse(gates(rows)['interface_passed'])

    def test_layout_reversal_cannot_hide_in_half_probability_mean(self) -> None:
        rows = example_rows()
        for row in rows:
            if row['kind'] == 'expression':
                row['q'] = 0.9 if row['mapping'] == 0 else 0.1
        self.assertFalse(gates(rows)['interface_passed'])


if __name__ == '__main__':
    unittest.main()
