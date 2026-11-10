"""Freeze the complete-empty classification fix without replaying completed items."""
from __future__ import annotations

import copy
import math
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from ds_protocol import HERE, load, sha, write
from ds_retry_prepare import SESSION as PREVIOUS_SESSION

ORIGINAL = PREVIOUS_SESSION
SESSION = ORIGINAL.parent / 'ds-flash-full-02-complete-invalid-01'
EMPTY = '3bffc4de3ca549e250b0327c'
EXTRA = ('FULL02-COMPLETE-INVALID-01.md', 'ds_length_prepare.py', 'ds_length_run.py',
         'ds_length_collect.py', 'ds_length_entry.ps1', 'test_ds_length.py')


def prepare() -> None:
    old = load(ORIGINAL / 'plan.json')
    terminal = load(ORIGINAL / 'run/result.json')
    if (terminal['status'], terminal['completed_ratings'], terminal['overall_attempts'], terminal['cumulative_upper_nano_usd']) != ('process_or_integrity_failure', 265, 315, 850617732):
        raise ValueError('Exact previous terminal required')
    response = load(ORIGINAL / 'run' / EMPTY / 'attempts/01/response.json')
    if (response['choices'][0]['finish_reason'], response['choices'][0]['message']['content'], response['usage']['completion_tokens']) != ('length', '', 8192):
        raise ValueError('Exact complete-empty evidence required')
    plan = copy.deepcopy(old)
    elapsed = (datetime.fromisoformat(terminal['ended_utc']) - datetime.fromisoformat(load(ORIGINAL / 'run/binding.json')['started_utc'])).total_seconds()
    active = old['prior_active_seconds'] + elapsed
    plan.update({'phase': 'full02_complete_invalid', 'prior_attempts': 315, 'prior_upper_nano_usd': 850617732,
                 'new_max_attempts': 714, 'overall_max_attempts': 1029, 'prior_active_seconds': active,
                 'remaining_wall_seconds': 5400 - math.ceil(active),
                 'original_result_sha256': sha(ORIGINAL / 'run/result.json'),
                 'original_ledger_sha256': sha(ORIGINAL / 'run/ledger.jsonl')})
    plan['carried_ids'] = [r['id'] for r in old['requests'] if (ORIGINAL / 'run' / r['id']).is_dir()]
    plan['carried_rating_ids'] = [i for i in plan['carried_ids'] if (ORIGINAL / 'run' / i / 'rating.json').exists()]
    plan['carried_invalid_ids'] = [EMPTY]
    plan['worklist'] = [r for r in old['worklist'] if r['id'] not in plan['carried_ids']]
    if (len(plan['carried_ids']), len(plan['carried_rating_ids']), len(plan['worklist'])) != (266, 265, 238):
        raise ValueError('Only the remaining 238 may be posted')
    plan['carried_files'] = [{'path': p.relative_to(ORIGINAL / 'run').as_posix(), 'sha256': sha(p)}
                             for i in plan['carried_ids'] for p in sorted((ORIGINAL / 'run' / i).rglob('*')) if p.is_file()]
    plan['sources'] += [{'path': n, 'sha256': sha(HERE / n)} for n in EXTRA]
    SESSION.mkdir(exist_ok=False)
    (SESSION / 'requests').mkdir()
    for row in plan['requests']:
        source = ORIGINAL / 'requests' / (row['id'] + '.json')
        if sha(source) != row['sha256']:
            raise ValueError('Same frozen request bytes required')
        shutil.copyfile(source, SESSION / 'requests' / source.name)
    write(SESSION / 'plan.json', plan)
    shutil.copyfile(SESSION / 'plan.json', HERE / 'inputs/ds-full02-complete-invalid-plan-01.json')
    for row in plan['sources']:
        target = SESSION / 'source-snapshot' / row['path']
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(HERE / row['path'], target)


def preflight(head: str) -> dict[str, Any]:
    root = HERE.parents[1]
    def git(*args: str) -> str:
        return subprocess.check_output(['git', *args], cwd=root, text=True, creationflags=subprocess.CREATE_NO_WINDOW).strip()
    if git('rev-parse', 'HEAD') != head or git('ls-remote', 'origin', 'refs/heads/task/R-STEER-01').split()[0] != head or git('status', '--porcelain'):
        raise ValueError('Clean pushed registration required')
    plan = load(SESSION / 'plan.json')
    if sha(SESSION / 'plan.json') != sha(HERE / 'inputs/ds-full02-complete-invalid-plan-01.json'):
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
