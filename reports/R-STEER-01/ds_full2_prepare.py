"""Freeze an enum-only prompt addition, all 504 fresh calls and unchanged draw."""
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any

from ds_full_prepare import SOURCES as OLD_SOURCES
from ds_full_prepare import audit_sample
from ds_high_prepare import reserve
from ds_high_resume import BASE
from ds_protocol import HERE, load, sha, write

SESSION = Path('D:/Dev/cache/R-STEER-01/ds-flash-full-02')
PREVIOUS = SESSION.parent / 'ds-flash-full-01'
ENUM_LINE = 'valence.label允许值仅为positive、negative、mixed、neutral、uncertain；混合正负的一般效价用mixed。四个concepts通道的正负并存标签仍为both；两个字段使用各自的枚举。'
SOURCES = (*OLD_SOURCES, 'DS-FULL-02.md', 'DS-RUBRIC-02.txt', 'ds_full2_prepare.py',
           'ds_full2_run.py', 'ds_full2_analysis.py', 'ds_full2_review.py',
           'ds_full2_diagnostics.py', 'ds_full2_entry.ps1', 'test_ds_full2.py',
           'results/ds-full-stop-summary-01.json')


def rubric2() -> str:
    old = (HERE / 'DS-RUBRIC-01.txt').read_text(encoding='utf-8')
    lines = old.splitlines(keepends=True)
    anchors = [i for i, line in enumerate(lines) if line.startswith('valence是一般正负情感，')]
    if len(anchors) != 1:
        raise ValueError('Exactly one original valence line required')
    lines.insert(anchors[0] + 1, ENUM_LINE + '\n')
    return ''.join(lines)


def amended(original: dict[str, Any]) -> dict[str, Any]:
    new = copy.deepcopy(original)
    old = (HERE / 'DS-RUBRIC-01.txt').read_text(encoding='utf-8')
    system = new['messages'][0]['content']
    if not system.startswith(old):
        raise ValueError('Original system prefix differs')
    new['messages'][0]['content'] = rubric2() + system[len(old):]
    return dict(new)


def prepare() -> None:
    if (HERE / 'DS-RUBRIC-02.txt').read_text(encoding='utf-8') != rubric2():
        raise ValueError('Only approved enum line may be added')
    prior = load(HERE / 'results/ds-full-stop-summary-01.json')
    if (prior['overall_recorded_attempts'], prior['cumulative_upper_nano_usd']) != (48, 100000992):
        raise ValueError('Prior accounting differs')
    previous = load(PREVIOUS / 'plan.json')
    SESSION.mkdir(exist_ok=False)
    (SESSION / 'requests').mkdir()
    rows = []
    for row in previous['requests']:
        path = PREVIOUS / 'requests' / (row['id'] + '.json')
        if sha(path) != row['sha256']:
            raise ValueError('Previous frozen payload differs')
        body = json.dumps(amended(load(path)), ensure_ascii=False, separators=(',', ':')).encode('utf-8')
        if len(body) > 16000:
            raise ValueError('Registered request byte bound')
        (SESSION / 'requests' / path.name).write_bytes(body)
        rows.append({'id': row['id'], 'sha256': hashlib.sha256(body).hexdigest(), 'bytes': len(body),
                     'old_request_sha256': row['sha256'], 'reserved_nano_usd': reserve(body)})
    selected = audit_sample([r['id'] for r in rows])
    if len(rows) != 504 or selected != previous['sample_ids']:
        raise ValueError('Population and original fixed draw required')
    write(SESSION / 'plan.json', {'phase': 'full504_high_v2', 'status': 'prepared_not_run',
          'model': 'deepseek-flash', 'documented_version': 'DeepSeek-V4.1-Flash-1017',
          'requests': rows, 'sample_seed': 20261110, 'sample_ids': selected, 'sampling': previous['sampling'],
          'prior_attempts': 48, 'prior_upper_nano_usd': 100000992, 'new_max_attempts': 504,
          'overall_max_attempts': 552, 'max_in_flight': 4, 'wall_limit_seconds': 5400,
          'request_wall_seconds': 180, 'hard_nano_usd': 5000000000, 'soft_nano_usd': 4750000000,
          'base_executable': str(BASE), 'base_executable_sha256': sha(BASE),
          'schema_invalid_policy': 'retain_invalid_count_failure_no_retry_continue_collection',
          'sample_review_required_before_formal_analysis': True, 'pilot_ratings_in_formal_evidence': False,
          'sources': [{'path': name, 'sha256': sha(HERE / name)} for name in SOURCES]})


if __name__ == '__main__':
    prepare()
