"""Freeze the same twenty prompts with high thinking; no credential or API access."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from ds_protocol import HERE, load, sha, write

SESSION = Path('D:/Dev/cache/R-STEER-01/ds-flash-high-01')
MAX_TOKENS = 8192  # Registered total reasoning + JSON allowance, not a quality threshold.
SOURCES = ('DS-FLASH-HIGH-01.md', 'ds_high_prepare.py', 'ds_high_run.py',
           'ds_high_entry.ps1', 'ds_high_review.py', 'test_ds_high.py', 'ds_protocol.py', 'ds_pilot.py',
           'DS-RUBRIC-01.txt', 'inputs/ds-flash-plan-01.json',
           'results/ds-flash-pilot-summary-01.json')


def high_request(original: dict[str, object]) -> dict[str, object]:
    return {**original, 'thinking': {'type': 'enabled'}, 'reasoning_effort': 'high',
            'max_tokens': MAX_TOKENS}


def reserve(body: bytes) -> int:
    return (len(body) + 1024) * 300 + MAX_TOKENS * 1200


def prepare() -> None:
    SESSION.mkdir(exist_ok=False)
    (SESSION / 'requests').mkdir()
    old = load(HERE / 'inputs/ds-flash-plan-01.json')
    prior = load(HERE / 'results/ds-flash-pilot-summary-01.json')
    rows = []
    for identifier in old['pilot_ids']:
        path = HERE / 'ds-flash-review-01' / identifier / 'request.json'
        original = load(path)
        body = json.dumps(high_request(original), ensure_ascii=False,
                          separators=(',', ':')).encode('utf-8')
        if len(body) > 16000:
            raise ValueError('Registered request size limit')
        (SESSION / 'requests' / (identifier + '.json')).write_bytes(body)
        rows.append({'id': identifier, 'old_request_sha256': sha(path),
                     'sha256': hashlib.sha256(body).hexdigest(), 'bytes': len(body),
                     'reserved_nano_usd': reserve(body)})
    write(SESSION / 'plan.json', {'phase': 'high_twenty_only', 'status': 'prepared_not_run',
          'model': 'deepseek-flash', 'documented_version': old['documented_version'],
          'thinking': 'enabled', 'reasoning_effort': 'high', 'max_tokens': MAX_TOKENS,
          'prior_posts': prior['post_attempts'], 'prior_upper_nano_usd': prior['fee_upper_nano_usd'],
          'new_max_posts': 20, 'full_posts_authorized_now': 0,
          'peak_nano_usd_per_token': old['peak_nano_usd_per_token'],
          'price_source': old['price_source'], 'requests': rows,
          'sources': [{'path': name, 'sha256': sha(HERE / name)} for name in SOURCES]})


if __name__ == '__main__':
    argparse.ArgumentParser(description=__doc__).parse_args()
    prepare()
