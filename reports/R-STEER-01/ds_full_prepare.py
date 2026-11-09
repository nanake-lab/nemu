"""Freeze all 504 fresh high requests and a uniform fixed-seed audit sample."""
from __future__ import annotations

import hashlib
import json
import random
from pathlib import Path

from ds_high_prepare import high_request, reserve
from ds_high_resume import BASE
from ds_protocol import HERE, load, sha, write

SESSION = Path('D:/Dev/cache/R-STEER-01/ds-flash-full-01')
SAMPLE_SEED = 20261110  # Fixed before full responses; user requested random twenty of 504.
SOURCES = ('DS-FULL-01.md', 'ds_full_prepare.py', 'ds_full_run.py', 'ds_full_analysis.py',
           'ds_full_review.py', 'ds_full_entry.ps1', 'test_ds_full.py', 'ds_protocol.py',
           'ds_http_once.py', 'ds_high_prepare.py', 'ds_high_resume.py', 'ds_high_run.py', 'ds_pilot.py', 'ds_high_review.py',
           'DS-RUBRIC-01.txt', 'C1-BLIND-RUBRIC.md', 'SEMANTIC-RUBRIC.md',
           'results/c1-blind-items-02.json', 'results/c1-blind-key-02.json',
           'inputs/ds-flash-plan-01.json', 'results/ds-flash-high-complete-summary-01.json')


def audit_sample(ids: list[str]) -> list[str]:
    return random.Random(SAMPLE_SEED).sample(sorted(ids), 20)


def prepare() -> None:
    SESSION.mkdir(exist_ok=False)
    (SESSION / 'requests').mkdir()
    old = load(HERE / 'inputs/ds-flash-plan-01.json')
    prior = load(HERE / 'results/ds-flash-high-complete-summary-01.json')
    if (prior['overall_recorded_attempts'], prior['all_attempts_upper_nano_usd']) != (41, 78947532):
        raise ValueError('Prior accounting differs')
    rows = []
    for row in old['requests']:
        path = Path('D:/Dev/cache/R-STEER-01/ds-flash-preparation-01/requests') / (row['id'] + '.json')
        if sha(path) != row['sha256']:
            raise ValueError('Original 504 request changed')
        original = load(path)
        body = json.dumps(high_request(original), ensure_ascii=False, separators=(',', ':')).encode('utf-8')
        if len(body) > 16000:
            raise ValueError('Registered engineering byte bound')
        (SESSION / 'requests' / path.name).write_bytes(body)
        rows.append({'id': row['id'], 'sha256': hashlib.sha256(body).hexdigest(), 'bytes': len(body),
                     'old_request_sha256': row['sha256'], 'reserved_nano_usd': reserve(body)})
    if len(rows) != 504 or len({r['id'] for r in rows}) != 504:
        raise ValueError('Exactly 504 unique request IDs required')
    write(SESSION / 'plan.json', {'phase': 'full504_high', 'status': 'prepared_not_run',
          'model': 'deepseek-flash', 'documented_version': 'DeepSeek-V4.1-Flash-1017',
          'requests': rows, 'sample_seed': SAMPLE_SEED, 'sample_ids': audit_sample([r['id'] for r in rows]),
          'sampling': 'CPython3.12 Random(seed).sample(sorted(all504IDs),20); no replacement, no strata',
          'prior_attempts': 41, 'prior_upper_nano_usd': 78947532, 'new_max_attempts': 504,
          'overall_max_attempts': 545, 'max_in_flight': 4, 'wall_limit_seconds': 5400,
          'request_wall_seconds': 180, 'hard_nano_usd': 5000000000, 'soft_nano_usd': 4750000000,
          'base_executable': str(BASE), 'base_executable_sha256': sha(BASE),
          'sample_review_required_before_formal_analysis': True, 'pilot_ratings_in_formal_evidence': False,
          'sources': [{'path': name, 'sha256': sha(HERE / name)} for name in SOURCES]})


if __name__ == '__main__':
    prepare()
