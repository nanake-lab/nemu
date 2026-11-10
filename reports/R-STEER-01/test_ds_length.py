"""Complete length-limited grading is invalid once, and does not stop the next item."""
from __future__ import annotations

import json
import shutil
import unittest
from pathlib import Path
from unittest.mock import patch

import ds_length_run
from ds_protocol import load, write
from ds_retry_budget import Ledger
from test_ds_retry import fixture


class CompleteInvalid(unittest.TestCase):
    def test_complete_empty_length_is_not_retried_and_next_item_can_finish(self) -> None:
        folder, plan, rating = fixture()
        shutil.copyfile(folder / 'requests/offline.json', folder / 'requests/offline2.json')
        output = folder / 'run'
        output.mkdir()
        ledger = Ledger(plan, output / 'ledger.jsonl')
        calls: list[str] = []
        def transport(command: list[str], target: Path, deadline: float) -> tuple[int, bool]:
            identifier = target.parents[1].name
            calls.append(identifier)
            bad = identifier == 'offline'
            tokens = 8192 if bad else 1000
            write(target / 'transport.json', {'response_complete': True})
            write(target / 'response.json', {'model': 'deepseek-flash', 'choices': [{
                'finish_reason': 'length' if bad else 'stop',
                'message': {'reasoning_content': 'OFFLINE_ONLY', 'content': '' if bad else json.dumps(rating)}}],
                'usage': {'prompt_tokens': 100, 'prompt_cache_hit_tokens': 0, 'prompt_cache_miss_tokens': 100,
                          'completion_tokens': tokens, 'total_tokens': tokens + 100}})
            return 0, False
        with patch.object(ds_length_run, 'SESSION', folder), patch.object(ds_length_run, 'supervise', transport):
            first = ds_length_run.item({'id': 'offline', 'attempts_already': 0}, ledger, output)
            second = ds_length_run.item({'id': 'offline2', 'attempts_already': 0}, ledger, output)
        self.assertEqual((first, second), ('invalid', 'rating'))
        self.assertEqual(calls, ['offline', 'offline2'])
        self.assertFalse(ledger.stop.is_set())
        self.assertFalse((output / 'offline/rating.json').exists())
        self.assertEqual(load(output / 'offline/response.json')['usage']['completion_tokens'], 8192)


if __name__ == '__main__':
    unittest.main()
