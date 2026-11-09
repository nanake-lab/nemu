"""Freeze an execution-only continuation; preserve all seventy completed outcomes."""
from __future__ import annotations

import copy
import hashlib
import shutil

from ds_full2_prepare import SESSION as ORIGINAL_SESSION
from ds_protocol import HERE, load, sha, write

ORIGINAL = ORIGINAL_SESSION
SESSION = ORIGINAL.parent / 'ds-flash-full-02-windowless-v2'
EXTRA = ('DS-FULL-02-WINDOWLESS.md', 'ds_full2_windowless_prepare.py', 'ds_full2_windowless_run.py',
         'ds_full2_windowless_collect.py', 'ds_full2_windowless_entry.ps1', 'test_ds_full2_windowless.py')


def prepare() -> None:
    old = load(ORIGINAL / 'plan.json')
    terminal = load(ORIGINAL / 'run/result.json')
    if (terminal['status'], terminal['post_attempts'], terminal['completed_ratings'], terminal['invalid_ratings'],
        terminal['cumulative_upper_nano_usd']) != ('user_stop', 70, 70, 0, 290715432):
        raise ValueError('Exact original terminal required')
    for source in old['sources']:
        if sha(HERE / source['path']) != source['sha256']:
            raise ValueError('Original frozen source changed')
    plan = copy.deepcopy(old)
    plan.update({'phase': 'full504_high_v2_windowless', 'execution_revision': 2, 'resume_from_count': 70,
                 'carried_cumulative_upper_nano_usd': terminal['cumulative_upper_nano_usd'],
                 'carried_completed': 70, 'carried_invalid': 0, 'resumed_new_max_attempts': 434,
                 'carried_model': terminal['response_model'], 'carried_head': '144e6b1a364bf228b696f1d2ff5973c3ffafa3f9',
                 'carried_result_sha256': sha(ORIGINAL / 'run/result.json'),
                 'carried_ledger_sha256': sha(ORIGINAL / 'run/ledger.jsonl')})
    plan['sources'] += [{'path': name, 'sha256': sha(HERE / name)} for name in EXTRA]
    plan['carried_files'] = []
    for row in plan['requests'][:70]:
        folder = ORIGINAL / 'run' / row['id']
        for path in sorted(folder.rglob('*')):
            if path.is_file():
                plan['carried_files'].append({'path': path.relative_to(ORIGINAL / 'run').as_posix(), 'sha256': sha(path)})
    SESSION.mkdir(exist_ok=False)
    (SESSION / 'requests').mkdir()
    for row in plan['requests']:
        source = ORIGINAL / 'requests' / (row['id'] + '.json')
        if sha(source) != row['sha256']:
            raise ValueError('All 504 original V2 request bytes required')
        shutil.copyfile(source, SESSION / 'requests' / source.name)
    write(SESSION / 'plan.json', plan)
    shutil.copyfile(SESSION / 'plan.json', HERE / 'inputs/ds-full-plan-02-windowless-02.json')
    for row in plan['sources']:
        destination = SESSION / 'source-snapshot' / row['path']
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(HERE / row['path'], destination)
        if hashlib.sha256(destination.read_bytes()).hexdigest() != row['sha256']:
            raise ValueError('Snapshot differs')


if __name__ == '__main__':
    prepare()
