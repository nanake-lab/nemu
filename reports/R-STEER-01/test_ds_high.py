"""Offline checks for unchanged messages, reasoning budget and quote boundaries."""
from __future__ import annotations

import json
import unittest

from ds_high_prepare import MAX_TOKENS, high_request, reserve
from ds_high_run import high_cost
from ds_protocol import HERE, SOFT_LIMIT, load, validate_rating


class HighBoundaryChecks(unittest.TestCase):
    def test_all_twenty_have_identical_messages_and_only_registered_parameter_changes(self) -> None:
        for identifier in load(HERE / 'inputs/ds-flash-plan-01.json')['pilot_ids']:
            old = load(HERE / 'ds-flash-review-01' / identifier / 'request.json')
            new = high_request(old)
            self.assertEqual(new['messages'], old['messages'])
            self.assertEqual({k for k in old if new[k] != old[k]}, {'thinking', 'max_tokens'})
            self.assertEqual(set(new) - set(old), {'reasoning_effort'})
            self.assertEqual(new['reasoning_effort'], 'high')
            self.assertEqual(new['thinking'], {'type': 'enabled'})
            self.assertEqual(new['max_tokens'], MAX_TOKENS)

    def test_reasoning_count_is_not_double_charged_and_over_limit_rejected(self) -> None:
        usage = {'prompt_tokens': 100, 'prompt_cache_hit_tokens': 40,
                 'prompt_cache_miss_tokens': 60, 'completion_tokens': 8192,
                 'total_tokens': 8292, 'completion_tokens_details': {'reasoning_tokens': 7000}}
        self.assertEqual(high_cost(usage), 40 * 6 + 60 * 300 + 8192 * 1200)
        self.assertLess(11447052 + 20 * reserve(bytes(16000)), SOFT_LIMIT)
        usage['completion_tokens'] = 8193
        usage['total_tokens'] = 8293
        with self.assertRaises(ValueError):
            high_cost(usage)

    def test_original_quote_validator_rejects_new_evidence(self) -> None:
        identifier = load(HERE / 'inputs/ds-flash-plan-01.json')['pilot_ids'][0]
        path = HERE / 'ds-flash-review-01' / identifier
        item = json.loads(load(path / 'request.json')['messages'][1]['content'])
        rating = load(path / 'rating.json')
        validate_rating(rating, item['待评输出'], item['评分事实参照'])
        rating['facts']['quote'] = 'NONEXISTENT_EVIDENCE_0123456789'
        with self.assertRaises(ValueError):
            validate_rating(rating, item['待评输出'], item['评分事实参照'])


if __name__ == '__main__':
    unittest.main()
