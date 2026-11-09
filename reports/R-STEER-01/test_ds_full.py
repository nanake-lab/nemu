"""Offline safeguards for four-slot accounting, stop behavior and review-gated use."""
from __future__ import annotations

import hashlib
import json
import tempfile
import threading
import time
import unittest
from pathlib import Path
from typing import Any
from unittest.mock import patch

import ds_full_analysis
import ds_full_run
from ds_full_analysis import absence, at_least, metrics, score
from ds_full_prepare import audit_sample
from ds_high_prepare import high_request, reserve
from ds_protocol import HERE, example, load, payload, write


def case(n: int) -> tuple[Path, dict[str, Any], dict[str, Any]]:
    folder = Path(tempfile.mkdtemp(dir='D:/Dev/cache/R-STEER-01', prefix='full-offline-'))
    print('retained_offline_case=' + str(folder))
    (folder / 'requests').mkdir()
    item = load(HERE / 'results/c1-blind-items-02.json')[0]
    body = json.dumps(high_request(payload(item)), ensure_ascii=False, separators=(',', ':')).encode('utf-8')
    rows = []
    for index in range(n):
        identifier = f'offline{index:03d}'
        (folder / 'requests' / (identifier + '.json')).write_bytes(body)
        rows.append({'id': identifier, 'sha256': hashlib.sha256(body).hexdigest()})
    plan = {'requests': rows, 'documented_version': 'DeepSeek-V4.1-Flash-1017',
            'prior_upper_nano_usd': 78947532, 'max_in_flight': 4,
            'soft_nano_usd': 4750000000, 'wall_limit_seconds': 30}
    write(folder / 'plan.json', plan)
    return folder, plan, example(item['concept_order'])


def fake_response(folder: Path, rating: dict[str, Any]) -> None:
    write(folder / 'response.json', {'model': 'deepseek-flash', 'system_fingerprint': 'OFFLINE_ONLY',
          'choices': [{'finish_reason': 'stop', 'message': {'reasoning_content': 'OFFLINE_FAKE_REASONING',
                      'content': json.dumps(rating, ensure_ascii=False)}}],
          'usage': {'prompt_tokens': 100, 'prompt_cache_hit_tokens': 0, 'prompt_cache_miss_tokens': 100,
                    'completion_tokens': 1000, 'total_tokens': 1100}})


class SchedulingBoundary(unittest.TestCase):
    def test_four_slot_cap_and_all_reservations_are_settled_without_duplicate_posts(self) -> None:
        folder, plan, rating = case(8)
        active = peak = 0
        lock = threading.Lock()
        def transport(command: list[str], dest: Path, deadline: float) -> tuple[int, bool]:
            nonlocal active, peak
            with lock:
                active += 1
                peak = max(peak, active)
            time.sleep(0.04)
            fake_response(dest, rating)
            with lock:
                active -= 1
            return 0, False
        with patch.object(ds_full_run, 'SESSION', folder), patch.object(ds_full_run, 'supervise', transport):
            ds_full_run.execute(plan, folder / 'run', 'offline')
        result = load(folder / 'run/result.json')
        self.assertEqual(peak, 4)
        self.assertEqual(result['post_attempts'], 8)
        self.assertEqual(result['completed_ratings'], 8)
        self.assertEqual(result['cumulative_upper_nano_usd'], 78947532 + 8 * 1230000)
        self.assertFalse(result['formal_full_complete'])  # A small fixture cannot masquerade as 504.

    def test_first_failure_stops_new_posts_and_keeps_failed_reservation(self) -> None:
        folder, plan, rating = case(8)
        barrier = threading.Barrier(4)
        def transport(command: list[str], dest: Path, deadline: float) -> tuple[int, bool]:
            barrier.wait(timeout=5)
            if dest.name == 'offline000':
                return 7, False
            time.sleep(0.1)
            fake_response(dest, rating)
            return 0, False
        held = reserve((folder / 'requests/offline000.json').read_bytes())
        with patch.object(ds_full_run, 'SESSION', folder), patch.object(ds_full_run, 'supervise', transport):
            ds_full_run.execute(plan, folder / 'run', 'offline')
        result = load(folder / 'run/result.json')
        self.assertEqual(result['status'], 'item_failure')
        self.assertEqual(result['post_attempts'], 4)
        self.assertEqual(result['completed_ratings'], 3)
        self.assertEqual(result['cumulative_upper_nano_usd'], 78947532 + held + 3 * 1230000)
        self.assertFalse((folder / 'run/offline004').exists())

    def test_budget_pre_reserves_all_inflight_and_does_not_auto_resume_after_settlement(self) -> None:
        folder, plan, rating = case(8)
        held = reserve((folder / 'requests/offline000.json').read_bytes())
        plan['soft_nano_usd'] = 78947532 + 3 * held
        def transport(command: list[str], dest: Path, deadline: float) -> tuple[int, bool]:
            time.sleep(0.04)
            fake_response(dest, rating)
            return 0, False
        with patch.object(ds_full_run, 'SESSION', folder), patch.object(ds_full_run, 'supervise', transport):
            ds_full_run.execute(plan, folder / 'run', 'offline')
        result = load(folder / 'run/result.json')
        self.assertEqual(result['status'], 'budget_stop')
        self.assertEqual(result['post_attempts'], 2)
        self.assertEqual(result['completed_ratings'], 2)


class EvidenceBoundary(unittest.TestCase):
    def test_uniform_sample_is_fixed_over_all_504_and_not_response_order(self) -> None:
        ids = [r['id'] for r in load(HERE / 'results/c1-blind-items-02.json')]
        selected = audit_sample(ids)
        self.assertEqual(selected, audit_sample(list(reversed(ids))))
        self.assertEqual(len(selected), 20)
        self.assertEqual(len(set(selected)), 20)
        self.assertTrue(set(selected).issubset(ids))

    def test_existing_threshold_edges_and_absent_neutral_uncertain_are_not_merged(self) -> None:
        for bad, good, n, percent in ((57, 58, 64, 90), (61, 62, 72, 85), (19, 20, 24, 80), (43, 44, 48, 90)):
            self.assertFalse(at_least(bad, n, percent))
            self.assertTrue(at_least(good, n, percent))
        rating = example(['connection', 'empathy', 'curiosity', 'achievement'])
        self.assertTrue(absence(rating))
        rating['concepts']['curiosity']['label'] = 'neutral'
        self.assertFalse(absence(rating))
        rating['concepts']['curiosity']['label'] = 'uncertain'
        self.assertFalse(absence(rating))
        self.assertEqual(score('uncertain'), 0)
        keys = load(HERE / 'results/c1-blind-key-02.json')
        fabricated = {r['id']: example(['connection', 'empathy', 'curiosity', 'achievement']) for r in keys}
        value = metrics(keys, fabricated)
        self.assertEqual(value['semantic_decision'], 'C1_semantic_insufficient_stop')
        self.assertEqual(value['automatic_decision_unchanged'], 'C1_insufficient_stop')
        self.assertFalse(value['pilot_scores_used'])

    def test_missing_user_random20_review_stops_before_any_formal_rating_is_read(self) -> None:
        folder, _, _ = case(1)
        (folder / 'run').mkdir()
        write(folder / 'run/result.json', {'formal_full_complete': True})
        with patch.object(ds_full_analysis, 'SESSION', folder):
            with self.assertRaises(FileNotFoundError) as error:
                ds_full_analysis.analyze()
        self.assertIn('sample-review.json', str(error.exception))
        self.assertFalse((folder / 'formal-analysis.json').exists())


if __name__ == '__main__':
    unittest.main()
