"""Guard the authorized new prompt, invalid continuation, fatal stops and denominators."""
from __future__ import annotations

import tempfile
import time
import unittest
from pathlib import Path
from typing import Any
from unittest.mock import patch

import ds_full2_analysis
import ds_full2_run
from ds_full2_analysis import metrics
from ds_full2_diagnostics import diagnostic
from ds_full2_prepare import ENUM_LINE, PREVIOUS, amended, rubric2
from ds_full_prepare import audit_sample
from ds_protocol import HERE, example, load, write
from test_ds_full import case, fake_response


class NewStage(unittest.TestCase):
    def test_prompt_only_adds_approved_line_and_preserves_504_draw(self) -> None:
        old = (HERE / 'DS-RUBRIC-01.txt').read_text(encoding='utf-8')
        self.assertEqual(rubric2().replace(ENUM_LINE + '\n', '', 1), old)
        plan = load(PREVIOUS / 'plan.json')
        for row in plan['requests']:
            before = load(PREVIOUS / 'requests' / (row['id'] + '.json'))
            after = amended(before)
            after['messages'][0]['content'] = after['messages'][0]['content'].replace(ENUM_LINE + '\n', '', 1)
            self.assertEqual(after, before)
        self.assertEqual(audit_sample([r['id'] for r in plan['requests']]), plan['sample_ids'])

    def test_bad_schema_and_quote_are_invalid_without_retry_or_termination(self) -> None:
        folder, plan, rating = case(8)
        plan['prior_attempts'] = 48
        plan['prior_upper_nano_usd'] = 100000992
        def transport(command: list[str], dest: Path, deadline: float) -> tuple[int, bool]:
            current = example(list(rating['concepts']))
            if dest.name == 'offline000':
                current['valence']['label'] = 'both'
            if dest.name == 'offline001':
                current['valence']['quote'] = 'NOT_IN_OUTPUT_SYNTHETIC_TEST'
            fake_response(dest, current)
            return 0, False
        with patch.object(ds_full2_run, 'SESSION', folder), patch.object(ds_full2_run, 'supervise', transport):
            ds_full2_run.execute(plan, folder / 'run', 'offline')
        result = load(folder / 'run/result.json')
        self.assertEqual((result['post_attempts'], result['completed_ratings'], result['invalid_ratings']), (8, 6, 2))
        self.assertEqual(result['cumulative_upper_nano_usd'], 100000992 + 8 * 1230000)
        for identifier in ('offline000', 'offline001'):
            self.assertTrue((folder / 'run' / identifier / 'invalid.json').exists())
            self.assertFalse((folder / 'run' / identifier / 'rating.json').exists())
        self.assertFalse(result['collection_complete'])

    def test_transport_failure_still_stops_new_posts(self) -> None:
        folder, plan, rating = case(8)
        plan['prior_attempts'] = 48
        def transport(command: list[str], dest: Path, deadline: float) -> tuple[int, bool]:
            if dest.name == 'offline000':
                return 7, False
            time.sleep(0.05)
            fake_response(dest, rating)
            return 0, False
        with patch.object(ds_full2_run, 'SESSION', folder), patch.object(ds_full2_run, 'supervise', transport):
            ds_full2_run.execute(plan, folder / 'run', 'offline')
        result = load(folder / 'run/result.json')
        self.assertEqual(result['status'], 'item_failure')
        self.assertEqual(result['post_attempts'], 4)
        self.assertEqual(result['invalid_ratings'], 0)

    def test_invalid_keeps_denominators_fails_repeat_and_blocks_effect_estimation(self) -> None:
        keys = load(HERE / 'results/c1-blind-key-02.json')
        ratings = {row['id']: example(['connection', 'empathy', 'curiosity', 'achievement']) for row in keys}
        bad = next(r for r in keys if r['kind'] == 'free' and r['repeat'] == 0)
        del ratings[bad['id']]
        value = metrics(keys, ratings)
        cell = value['language_gates'][bad['lang']]['free']
        self.assertEqual(cell['n'], 48)
        self.assertIsNone(cell['signed_effects'][bad['concept']]['equal_root_mean_delta'])
        self.assertFalse(cell['signed_effects'][bad['concept']]['passed'])
        pair = next(p for p in value['repeat_pairs'] if p['original_id'] == bad['id'])
        self.assertFalse(any(pair[k] for k in ('concept_vector', 'facts', 'coherence_vector')))

    def test_analysis_requires_new_random20_approval_before_reading_scores(self) -> None:
        folder = Path(tempfile.mkdtemp(dir='D:/Dev/cache/R-STEER-01', prefix='full2-gate-'))
        print('retained_offline_case=' + str(folder))
        (folder / 'run').mkdir()
        write(folder / 'plan.json', {})
        write(folder / 'run/result.json', {'collection_complete': True})
        with patch.object(ds_full2_analysis, 'SESSION', folder):
            with self.assertRaises(FileNotFoundError) as error:
                ds_full2_analysis.analyze()
        self.assertIn('sample-review.json', str(error.exception))

    def test_mixed_diagnostic_uses_frozen_output_cues_and_keeps_unreadable(self) -> None:
        keys: list[dict[str, Any]] = [{'id': 'a', 'lang': 'ja', 'kind': 'free'}, {'id': 'b', 'lang': 'en', 'kind': 'free'}]
        value = diagnostic(keys, {'a': {'output': '隔たりを感じる。ありがとう。'}, 'b': {'output': 'happy'}},
                           {'a': '{', 'b': '{"valence":{"label":"positive"}}'}, {'a'})
        self.assertEqual(value['counts']['cue_group']['mixed_cue_candidate:invalid'], 1)
        self.assertEqual(value['counts']['returned_valence']['unreadable:invalid'], 1)
        self.assertEqual(value['independent_mixed_valence_gold_labels'], 'not_available')


if __name__ == '__main__':
    unittest.main()
