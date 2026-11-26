"""Offline boundaries for missing samples, PID sums, and independent peaks."""
import copy
import unittest

from summarize import Row, derive_memory, memory_stats, quantile


class SummaryTests(unittest.TestCase):
    def row(self) -> Row:
        return {'utc': 10.0, 'owned': [11, 12], 'rss': 30, 'private': 40,
                'process_memory': [{'pid': 11, 'rss': 10, 'private': 15},
                                   {'pid': 12, 'rss': 20, 'private': 25}],
                'counters': [{'Name': 'pid_11_luid_a', 'DedicatedUsage': 100, 'SharedUsage': 3},
                             {'Name': 'pid_12_luid_a', 'DedicatedUsage': 200, 'SharedUsage': 5},
                             {'Name': 'pid_99_luid_a', 'DedicatedUsage': 9999, 'SharedUsage': 9999}]}

    def test_post_complete_empty_is_retained_not_zero(self) -> None:
        row = self.row()
        row.update(utc=21.0, process_memory=[], rss=None, private=None)
        original = copy.deepcopy(row)
        result = derive_memory([row], 20.0)
        self.assertEqual(len(result), 1)
        self.assertIsNone(result[0]['rss'])
        self.assertIsNone(result[0]['dedicated'])
        self.assertIn('未采到', result[0]['status'])
        self.assertEqual(row, original)

    def test_runtime_or_partial_empty_is_rejected(self) -> None:
        row = self.row()
        row.update(process_memory=[], rss=None, private=None)
        for utc in (10.0, 20.0):
            row['utc'] = utc
            with self.assertRaisesRegex(ValueError, '运行期'):
                derive_memory([row], 20.0)
        row.update(utc=21.0, private=0)
        with self.assertRaisesRegex(ValueError, '退出条件'):
            derive_memory([row], 20.0)

    def test_wddm_sums_actual_tree_excluding_background(self) -> None:
        result = derive_memory([self.row()], 20.0)[0]
        self.assertEqual(result['dedicated'], 300)
        self.assertEqual(result['shared'], 8)
        self.assertEqual(result['wddm_by_pid'][12]['dedicated'], 200)

    def test_missing_wddm_is_not_zero(self) -> None:
        row = self.row()
        row['counters'] = []
        result = derive_memory([row], 20.0)[0]
        self.assertIsNone(result['dedicated'])
        self.assertEqual(result['rss'], 30)

    def test_independent_peaks_and_metric_coverage(self) -> None:
        rows: list[Row] = [
            {'utc': 1., 'dedicated': 100, 'shared': 10, 'rss': 30, 'private': 40},
            {'utc': 2., 'dedicated': 80, 'shared': 40, 'rss': 31, 'private': 41},
            {'utc': 3., 'dedicated': None, 'shared': None, 'rss': None, 'private': None}]
        result = memory_stats(rows)
        self.assertEqual(result['dedicated_plus_shared_independent_peaks_bytes'], 140)
        self.assertEqual(result['metrics']['rss']['coverage']['n'], 2)
        self.assertEqual(result['metrics']['rss']['coverage']['last_utc'], 2.)
        self.assertEqual(result['metrics']['rss']['missing_n'], 1)
        self.assertAlmostEqual(quantile([1., 2., 3., 4.], .9), 3.7)


if __name__ == '__main__':
    unittest.main()
