"""Freeze four unchanged payloads and cumulative accounting; no credential/network."""
from __future__ import annotations

from ds_high_resume import BASE, SESSION
from ds_protocol import HERE, load, sha, write

SOURCES = ('DS-FLASH-HIGH-RESUME-01.md', 'ds_resume_prepare.py', 'ds_high_resume.py',
           'ds_http_once.py', 'ds_resume_entry.ps1', 'test_ds_resume.py', 'ds_high_combined_review.py',
           'ds_protocol.py', 'ds_high_prepare.py', 'ds_high_run.py', 'ds_pilot.py', 'ds_high_review.py',
           'inputs/ds-flash-high-plan-01.json', 'results/ds-flash-high-summary-01.json', 'DS-RUBRIC-01.txt')


def prepare() -> None:
    SESSION.mkdir(exist_ok=False)
    (SESSION / 'requests').mkdir()
    original = load(HERE / 'inputs/ds-flash-high-plan-01.json')
    prior = load(HERE / 'results/ds-flash-high-summary-01.json')
    for entry in original['sources']:
        if sha(HERE / entry['path']) != entry['sha256']:
            raise ValueError('Original high source changed')
    if prior['cumulative_upper_nano_usd'] != 68386560 or prior['overall_post_attempts'] != 37:
        raise ValueError('Prior cumulative accounting differs')
    rows = []
    for index, entry in enumerate(original['requests'][16:]):
        source = SESSION.parent / 'ds-flash-high-01/requests' / (entry['id'] + '.json')
        if sha(source) != entry['sha256']:
            raise ValueError('Original request changed')
        (SESSION / 'requests' / source.name).write_bytes(source.read_bytes())
        rows.append({**entry, 'previous_attempt_unknown': index == 0})
    if [r['id'] for r in rows] != prior['unknown_outcome_ids'] + prior['not_run_ids']:
        raise ValueError('Only remaining four may run')
    write(SESSION / 'plan.json', {'status': 'prepared_not_run', 'phase': 'high_resume_four',
          'model': 'deepseek-flash', 'documented_version': original['documented_version'],
          'prior_attempts': 37, 'prior_upper_nano_usd': 68386560, 'new_max_attempts': 4,
          'maximum_pilot_attempts_after_resume': 41, 'full_post_attempts_authorized': 0,
          'prior_unknown_reservation_retained': True, 'max_tokens': 8192, 'reasoning_effort': 'high',
          'base_executable': str(BASE), 'base_executable_sha256': sha(BASE),
          'request_wall_deadline_seconds': 180, 'supervisor_deadline_seconds': 900,
          'requests': rows, 'sources': [{'path': name, 'sha256': sha(HERE / name)} for name in SOURCES]})


if __name__ == '__main__':
    prepare()
