"""Detached four-item continuation with an absolute per-attempt deadline."""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from ds_high_prepare import reserve
from ds_high_run import high_cost
from ds_pilot import event
from ds_protocol import HERE, MODEL, SOFT_LIMIT, load, sha, validate_rating, write

SESSION = Path('D:/Dev/cache/R-STEER-01/ds-flash-high-resume-01')
BASE = Path('C:/Users/nanake/AppData/Local/Programs/Python/Python312/python.exe')


def supervise(command: list[str], folder: Path, deadline: float) -> tuple[int, bool]:
    with (folder / 'worker.stdout').open('xb') as stdout, (folder / 'worker.stderr').open('xb') as stderr:
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr)
    write(folder / 'worker-started.json', {'pid': child.pid, 'parent_pid': os.getpid(),
          'utc': datetime.now(timezone.utc).isoformat(), 'deadline_seconds': deadline})
    until = time.monotonic() + deadline
    expired = False
    while child.poll() is None:
        event(folder / 'heartbeat.jsonl', {'event': 'worker_alive', 'pid': child.pid})
        remaining = until - time.monotonic()
        if remaining <= 0:
            child.kill()  # Only the exact child we created; never runner or other processes.
            child.wait()
            expired = True
            break
        try:
            child.wait(timeout=min(2, remaining))
        except subprocess.TimeoutExpired:
            pass
    code = child.wait()
    write(folder / 'worker-terminal.json', {'exit_code': code, 'wall_deadline_hit': expired,
          'utc': datetime.now(timezone.utc).isoformat()})
    return code, expired


def preflight(head: str) -> dict[str, Any]:
    root = HERE.parents[1]
    actual = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
    remote = subprocess.check_output(['git', 'ls-remote', 'origin', 'refs/heads/task/R-STEER-01'],
                                     cwd=root, text=True).split()[0]
    if actual != head or remote != head or subprocess.check_output(['git', 'status', '--porcelain'], cwd=root, text=True).strip():
        raise ValueError('Clean pushed registration head required')
    plan_path = HERE / 'inputs/ds-flash-high-resume-plan-01.json'
    plan = load(plan_path)
    if sha(plan_path) != sha(SESSION / 'plan.json') or sha(BASE) != plan['base_executable_sha256']:
        raise ValueError('Plan/interpreter differs')
    for row in plan['sources']:
        if sha(HERE / row['path']) != row['sha256']:
            raise ValueError('Source changed')
    for row in plan['requests']:
        request = SESSION / 'requests' / (row['id'] + '.json')
        original = Path('D:/Dev/cache/R-STEER-01/ds-flash-high-01/requests') / request.name
        if sha(request) != row['sha256'] or request.read_bytes() != original.read_bytes():
            raise ValueError('Same frozen payload required')
    if (plan['prior_upper_nano_usd'], plan['prior_attempts'], len(plan['requests'])) != (68386560, 37, 4):
        raise ValueError('Cumulative prior/call scope differs')
    price = load(SESSION / 'price-check.json')
    age = (datetime.now(timezone.utc) - datetime.fromisoformat(price['checked_utc'])).total_seconds()
    if (not 0 <= age <= 600 or price['model'] != MODEL or price['version'] != plan['documented_version']
        or price['peak_nano_usd_per_token'] != {'hit': 6, 'miss': 300, 'output': 1200}
        or price['source'] != 'https://api-docs.deepseek.com/quick_start/pricing/'):
        raise ValueError('Fresh unchanged official price/version required')
    return dict(plan)


def run(head: str) -> int:
    plan = preflight(head)
    if not os.environ.get('DEEPSEEK_API_KEY'):
        raise ValueError('Process credential missing')
    output = SESSION / 'run'
    output.mkdir(exist_ok=False)
    write(output / 'binding.json', {'head': head, 'pid': os.getpid(),
          'started_utc': datetime.now(timezone.utc).isoformat(), 'plan_sha256': sha(SESSION / 'plan.json')})
    total = int(plan['prior_upper_nano_usd'])
    attempts = completed = 0
    status = 'resume_stopped'
    model: str | None = None
    ledger = output / 'ledger.jsonl'
    start = time.monotonic()
    try:
        for row in plan['requests']:
            if attempts >= 4 or time.monotonic() - start >= 900:
                raise ValueError('Four calls/fifteen-minute scope')
            body = (SESSION / 'requests' / (row['id'] + '.json')).read_bytes()
            held = reserve(body)
            if total + held >= SOFT_LIMIT:
                status = 'budget_stop'
                break
            folder = output / row['id']
            folder.mkdir()
            (folder / 'request.json').write_bytes(body)
            total += held
            attempts += 1
            event(ledger, {'event': 'reserve_before_post', 'id': row['id'], 'ordinal': attempts,
                  'overall_ordinal': 37 + attempts, 'previous_attempt_unknown': row['previous_attempt_unknown'],
                  'reserved_nano_usd': held, 'cumulative_upper_nano_usd': total,
                  'request_sha256': row['sha256']})
            code, expired = supervise([str(BASE), '-B', str(HERE / 'ds_http_once.py'), '--folder', str(folder)],
                                     folder, min(180, 900 - (time.monotonic() - start)))
            if code != 0 or expired:
                event(ledger, {'event': 'transport_stop_no_retry', 'id': row['id'],
                      'exit_code': code, 'wall_deadline_hit': expired})
                raise ValueError('HTTP child failed or deadline hit; full reservation retained')
            value = load(folder / 'response.json')
            cost = high_cost(value['usage'])
            total += cost - held
            event(ledger, {'event': 'settle_from_usage_peak_upper', 'id': row['id'],
                  'usage': value['usage'], 'upper_nano_usd': cost, 'cumulative_upper_nano_usd': total,
                  'response_model': value.get('model'), 'system_fingerprint': value.get('system_fingerprint'),
                  'response_id': value.get('id'), 'response_created': value.get('created')})
            if cost > held or value.get('model') not in (MODEL, plan['documented_version']) or (model is not None and value['model'] != model):
                raise ValueError('Reservation/model differs')
            model = value['model']
            choices = value['choices']
            if len(choices) != 1 or choices[0]['finish_reason'] != 'stop':
                raise ValueError('Nonstop completion')
            message = choices[0]['message']
            if message.get('tool_calls') or not isinstance(message.get('reasoning_content'), str) or not message['reasoning_content'].strip():
                raise ValueError('Expected high thinking without tools')
            content = message['content']
            if not isinstance(content, str) or not content.strip():
                raise ValueError('Blank JSON; no repair')
            presented = json.loads(load(folder / 'request.json')['messages'][1]['content'])
            rating = validate_rating(json.loads(content), presented['待评输出'], presented['评分事实参照'])
            write(folder / 'rating.json', rating)
            completed += 1
        if completed == 4:
            status = 'resume_complete_awaiting_user_trust'
    except Exception as error:
        event(ledger, {'event': 'stop_no_retry', 'error_class': type(error).__name__, 'completed': completed})
    finally:
        write(output / 'result.json', {'status': status, 'post_attempts': attempts, 'completed_ratings': completed,
              'prior_attempts': 37, 'overall_attempts': 37 + attempts, 'full_post_attempts': 0,
              'prior_upper_nano_usd': plan['prior_upper_nano_usd'], 'new_upper_nano_usd': total - plan['prior_upper_nano_usd'],
              'cumulative_upper_nano_usd': total, 'prior_unknown_reservation_retained': True,
              'ended_utc': datetime.now(timezone.utc).isoformat(), 'response_model': model})
        os.environ.pop('DEEPSEEK_API_KEY', None)
    return 0 if completed == 4 else 2


def launch(head: str) -> int:
    # Break away from the input console; never keep its lifetime as the scoring lifetime.
    with (SESSION / 'supervisor.stdout').open('xb') as stdout, (SESSION / 'supervisor.stderr').open('xb') as stderr:
        child = subprocess.Popen([str(BASE), '-B', str(Path(__file__).resolve()), 'worker', '--head', head],
            stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr,
            creationflags=subprocess.DETACHED_PROCESS | subprocess.CREATE_BREAKAWAY_FROM_JOB)
    write(SESSION / 'supervisor-started.json', {'pid': child.pid, 'utc': datetime.now(timezone.utc).isoformat(),
          'input_window_may_close': True, 'key_persisted': False})
    return 0


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=('launch', 'worker'))
    parser.add_argument('--head', required=True)
    args = parser.parse_args()
    try:
        if args.mode == 'launch':
            raise SystemExit(launch(args.head))
        raise SystemExit(run(args.head))
    except Exception as error:
        write(SESSION / 'supervisor-stop.json', {'status': 'preflight_or_supervisor_failed',
              'error_class': type(error).__name__, 'utc': datetime.now(timezone.utc).isoformat()})
        raise SystemExit(2) from None
