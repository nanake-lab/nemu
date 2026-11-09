"""Freeze unchanged 504 requests, carried 110 ratings and a bounded network worklist."""
from __future__ import annotations

import copy
import math
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from ds_protocol import HERE, load, sha, write

ORIGINAL = Path('D:/Dev/cache/R-STEER-01/ds-flash-full-02-windowless-v2')
SESSION = ORIGINAL.parent / 'ds-flash-full-02-network-retry-02'
FAILED = '53dfb69ce6b166eae7a2f26e'
EXTRA = ('FULL02-NETWORK-RETRY-01.md', 'ds_retry_http.py', 'ds_retry_budget.py', 'ds_retry_prepare.py',
         'ds_retry_run.py', 'ds_retry_review.py', 'ds_retry_analysis.py', 'ds_retry_entry.ps1', 'test_ds_retry.py')


def prepare() -> None:
    old = load(ORIGINAL / 'plan.json')
    result = load(ORIGINAL / 'run/result.json')
    if (result['status'], result['post_attempts'], result['completed_ratings'], result['cumulative_upper_nano_usd']) != ('item_failure', 111, 110, 403365240):
        raise ValueError('Exact previous technical stop required')
    plan = copy.deepcopy(old)
    plan.update({'phase': 'full02_network_retry', 'registration_revision': 2, 'prior_attempts': 159, 'prior_upper_nano_usd': 403365240,
                 'new_max_attempts': 1181, 'overall_max_attempts': 1340, 'max_retries': 2, 'retry_seconds': 30,
                 'technical_streak_limit': 3, 'original_result_sha256': sha(ORIGINAL / 'run/result.json'),
                 'original_ledger_sha256': sha(ORIGINAL / 'run/ledger.jsonl'), 'expected_response_model': 'deepseek-flash'})
    first = load(ORIGINAL.parent / 'ds-flash-full-02/run/binding.json')
    first_end = load(ORIGINAL.parent / 'ds-flash-full-02/run/result.json')
    second = load(ORIGINAL / 'run/binding.json')
    elapsed = sum((datetime.fromisoformat(end['ended_utc']) - datetime.fromisoformat(start['started_utc'])).total_seconds()
                  for start, end in ((first, first_end), (second, result)))
    plan['prior_active_seconds'] = elapsed
    plan['remaining_wall_seconds'] = 5400 - math.ceil(elapsed)
    plan['carried_ids'] = [r['id'] for r in old['requests'][:111]]
    plan['carried_rating_ids'] = [i for i in plan['carried_ids'] if (ORIGINAL / 'run' / i / 'rating.json').exists()]
    if len(plan['carried_rating_ids']) != 110:
        raise ValueError('Exactly 110 complete scores required')
    plan['worklist'] = [{'id': FAILED, 'attempts_already': 1,
                         'previous_failure_utc': load(ORIGINAL / 'run' / FAILED / 'transport.json')['ended_utc']}]
    plan['worklist'] += [{'id': r['id'], 'attempts_already': 0} for r in old['requests'][111:]]
    plan['carried_files'] = [{'path': p.relative_to(ORIGINAL / 'run').as_posix(), 'sha256': sha(p)}
                             for i in plan['carried_ids'] for p in sorted((ORIGINAL / 'run' / i).rglob('*')) if p.is_file()]
    plan['sources'] += [{'path': name, 'sha256': sha(HERE / name)} for name in EXTRA]
    SESSION.mkdir(exist_ok=False)
    (SESSION / 'requests').mkdir()
    for row in plan['requests']:
        source = ORIGINAL / 'requests' / (row['id'] + '.json')
        if sha(source) != row['sha256']:
            raise ValueError('Original V2 request changed')
        shutil.copyfile(source, SESSION / 'requests' / source.name)
    write(SESSION / 'plan.json', plan)
    shutil.copyfile(SESSION / 'plan.json', HERE / 'inputs/ds-full02-network-retry-plan-02.json')
    for row in plan['sources']:
        destination = SESSION / 'source-snapshot' / row['path']
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(HERE / row['path'], destination)


def preflight(head: str) -> dict[str, Any]:
    root = HERE.parents[1]
    def git(*args: str) -> str:
        return subprocess.check_output(['git', *args], cwd=root, text=True, creationflags=subprocess.CREATE_NO_WINDOW).strip()
    if git('rev-parse', 'HEAD') != head or git('ls-remote', 'origin', 'refs/heads/task/R-STEER-01').split()[0] != head or git('status', '--porcelain'):
        raise ValueError('Clean pushed registration required')
    plan = load(SESSION / 'plan.json')
    if sha(SESSION / 'plan.json') != sha(HERE / 'inputs/ds-full02-network-retry-plan-02.json'):
        raise ValueError('Plan differs')
    if sha(Path(plan['base_executable'])) != plan['base_executable_sha256']:
        raise ValueError('Interpreter differs')
    for row in plan['sources']:
        if sha(HERE / row['path']) != row['sha256']:
            raise ValueError('Frozen source differs')
    for row in plan['requests']:
        if sha(SESSION / 'requests' / (row['id'] + '.json')) != row['sha256']:
            raise ValueError('Request differs')
    for row in plan['carried_files']:
        if sha(ORIGINAL / 'run' / row['path']) != row['sha256']:
            raise ValueError('Original data differs')
    if (sha(ORIGINAL / 'run/result.json') != plan['original_result_sha256']
        or sha(ORIGINAL / 'run/ledger.jsonl') != plan['original_ledger_sha256']):
        raise ValueError('Original terminal accounting differs')
    price = load(SESSION / 'price-check.json')
    age = (datetime.now(timezone.utc) - datetime.fromisoformat(price['checked_utc'])).total_seconds()
    if (not 0 <= age <= 600 or price['model'] != 'deepseek-flash' or price['version'] != plan['documented_version']
        or price['peak_nano_usd_per_token'] != {'hit': 6, 'miss': 300, 'output': 1200}
        or price['source'] != 'https://api-docs.deepseek.com/quick_start/pricing/'):
        raise ValueError('Fresh unchanged official price/version required')
    return dict(plan)


if __name__ == '__main__':
    prepare()
