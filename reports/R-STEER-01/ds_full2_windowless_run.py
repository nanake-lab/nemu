"""Bounded fresh full scoring, with four reserved requests and no automatic retry."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import time
from concurrent.futures import FIRST_COMPLETED, Future, ThreadPoolExecutor, wait
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from ds_full2_prepare import PREVIOUS, amended
from ds_full2_windowless_prepare import ORIGINAL, SESSION
from ds_high_prepare import reserve
from ds_high_resume import BASE as BASE
from ds_high_run import high_cost
from ds_pilot import event
from ds_protocol import HERE, MODEL, load, sha, validate_rating, write


def supervise(command: list[str], folder: Path, deadline: float) -> tuple[int, bool]:
    with (folder / 'worker.stdout').open('xb') as stdout, (folder / 'worker.stderr').open('xb') as stderr:
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr,
                                 creationflags=subprocess.CREATE_NO_WINDOW)
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
    actual = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True, creationflags=subprocess.CREATE_NO_WINDOW).strip()
    remote = subprocess.check_output(['git', 'ls-remote', 'origin', 'refs/heads/task/R-STEER-01'], cwd=root, text=True, creationflags=subprocess.CREATE_NO_WINDOW).split()[0]
    if actual != head or remote != head or subprocess.check_output(['git', 'status', '--porcelain'], cwd=root, text=True, creationflags=subprocess.CREATE_NO_WINDOW).strip():
        raise ValueError('Clean pushed head required')
    plan = load(SESSION / 'plan.json')
    if sha(SESSION / 'plan.json') != sha(HERE / 'inputs/ds-full-plan-02-windowless-02.json') or sha(BASE) != plan['base_executable_sha256']:
        raise ValueError('Frozen plan/interpreter differs')
    for row in plan['sources']:
        if sha(HERE / row['path']) != row['sha256']:
            raise ValueError('Frozen source changed')
    if (plan['prior_attempts'], plan['prior_upper_nano_usd'], len(plan['requests'])) != (48, 100000992, 504):
        raise ValueError('Scope/prior differs')
    for row in plan['requests']:
        path = SESSION / 'requests' / (row['id'] + '.json')
        original = PREVIOUS / 'requests' / path.name
        if (sha(path) != row['sha256'] or sha(original) != row['old_request_sha256']
            or load(path) != amended(load(original))):
            raise ValueError('Original messages or frozen high parameters differ')
    if (plan['resume_from_count'], plan['carried_completed'], plan['carried_invalid'],
        plan['carried_cumulative_upper_nano_usd']) != (70, 70, 0, 290715432):
        raise ValueError('Exact seventy-item continuation required')
    if (sha(ORIGINAL / 'run/result.json') != plan['carried_result_sha256']
        or sha(ORIGINAL / 'run/ledger.jsonl') != plan['carried_ledger_sha256']):
        raise ValueError('Original stopped ledger/result changed')
    for carried in plan['carried_files']:
        if sha(ORIGINAL / 'run' / carried['path']) != carried['sha256']:
            raise ValueError('Completed original V2 data changed')
    price = load(SESSION / 'price-check.json')
    age = (datetime.now(timezone.utc) - datetime.fromisoformat(price['checked_utc'])).total_seconds()
    if (not 0 <= age <= 600 or price['model'] != MODEL or price['version'] != plan['documented_version']
        or price['peak_nano_usd_per_token'] != {'hit': 6, 'miss': 300, 'output': 1200}
        or price['source'] != 'https://api-docs.deepseek.com/quick_start/pricing/'):
        raise ValueError('Fresh unchanged official price/version required')
    return dict(plan)


def execute(plan: dict[str, Any], output: Path, head: str) -> int:
    output.mkdir(exist_ok=False)
    write(output / 'binding.json', {'head': head, 'pid': os.getpid(), 'started_utc': datetime.now(timezone.utc).isoformat(),
          'plan_sha256': sha(SESSION / 'plan.json'), 'pilot_ratings_in_evidence': False})
    for carried_row in plan['requests'][:plan['resume_from_count']]:
        shutil.copytree(ORIGINAL / 'run' / carried_row['id'], output / carried_row['id'])
    for carried in plan['carried_files']:
        if sha(output / carried['path']) != carried['sha256']:
            raise ValueError('Carry copy differs')
    total = int(plan['carried_cumulative_upper_nano_usd'])
    submitted = int(plan['resume_from_count'])
    completed, invalid = int(plan['carried_completed']), int(plan['carried_invalid'])
    stopped: str | None = None
    model: str | None = plan['carried_model']
    pending: dict[Future[tuple[int, bool]], dict[str, Any]] = {}
    ledger = output / 'ledger.jsonl'
    event(ledger, {'event': 'carry_forward_original_v2_no_reposts', 'carried_posts': submitted,
          'completed': completed, 'invalid': invalid, 'cumulative_upper_nano_usd': total,
          'original_head': plan['carried_head'], 'result_sha256': plan['carried_result_sha256'],
          'ledger_sha256': plan['carried_ledger_sha256']})
    start = time.monotonic()
    with ThreadPoolExecutor(max_workers=plan['max_in_flight']) as pool:
        try:
            while pending or (stopped is None and submitted < len(plan['requests'])):
                if (SESSION / 'stop-request.json').exists():
                    stopped = stopped or 'user_stop'
                remaining = plan['wall_limit_seconds'] - (time.monotonic() - start)
                if remaining <= 0 and stopped is None:
                    stopped = 'time_stop'
                while stopped is None and len(pending) < plan['max_in_flight'] and submitted < len(plan['requests']):
                    if (SESSION / 'stop-request.json').exists():
                        stopped = 'user_stop'
                        break
                    row = plan['requests'][submitted]
                    body = (SESSION / 'requests' / (row['id'] + '.json')).read_bytes()
                    held = reserve(body)
                    if total + held >= plan['soft_nano_usd']:
                        stopped = 'budget_stop'
                        break
                    folder = output / row['id']
                    folder.mkdir()
                    (folder / 'request.json').write_bytes(body)
                    total += held
                    submitted += 1
                    event(ledger, {'event': 'reserve_before_post', 'id': row['id'], 'ordinal': submitted,
                          'overall_ordinal': plan['prior_attempts'] + submitted, 'reserved_nano_usd': held,
                          'cumulative_upper_nano_usd': total, 'request_sha256': row['sha256']})
                    command = [str(BASE), '-B', str(HERE / 'ds_http_once.py'), '--folder', str(folder)]
                    future = pool.submit(supervise, command, folder, min(180, max(0, remaining)))
                    pending[future] = {**row, 'ordinal': submitted, 'held': held}
                if not pending:
                    break
                done, _ = wait(pending, timeout=2, return_when=FIRST_COMPLETED)
                event(output / 'supervisor-heartbeat.jsonl', {'event': 'supervisor_alive', 'pid': os.getpid(),
                      'submitted': submitted, 'completed': completed, 'invalid': invalid, 'in_flight': len(pending),
                      'cumulative_upper_nano_usd': total, 'stop_reason': stopped})
                # Settle every observed completion before allowing another submission.
                for future in sorted(done, key=lambda f: pending[f]['ordinal']):
                    row = pending.pop(future)
                    folder = output / row['id']
                    try:
                        code, expired = future.result()
                        if code != 0 or expired:
                            event(ledger, {'event': 'transport_failed_no_retry', 'id': row['id'],
                                  'exit_code': code, 'wall_deadline_hit': expired})
                            raise ValueError('Child transport failed; reservation retained')
                        value = load(folder / 'response.json')
                        cost = high_cost(value['usage'])
                        total += cost - row['held']
                        event(ledger, {'event': 'settle_from_usage_peak_upper', 'id': row['id'],
                              'usage': value['usage'], 'upper_nano_usd': cost, 'cumulative_upper_nano_usd': total,
                              'response_model': value.get('model'), 'system_fingerprint': value.get('system_fingerprint'),
                              'response_id': value.get('id'), 'response_created': value.get('created')})
                        if cost > row['held'] or value.get('model') not in (MODEL, plan['documented_version']) or (model is not None and value['model'] != model):
                            raise ValueError('Model/reservation differs')
                        model = value['model']
                        choices = value['choices']
                        if len(choices) != 1 or choices[0]['finish_reason'] != 'stop':
                            raise ValueError('Nonstop completion')
                        message = choices[0]['message']
                        if message.get('tool_calls') or not isinstance(message.get('reasoning_content'), str) or not message['reasoning_content'].strip():
                            raise ValueError('Expected high thinking without tools')
                        content = message.get('content')
                        presented = json.loads(load(folder / 'request.json')['messages'][1]['content'])
                        try:
                            if not isinstance(content, str) or not content.strip():
                                raise ValueError('Blank/nonstring content; no repair')
                            rating = validate_rating(json.loads(content), presented['待评输出'], presented['评分事实参照'])
                        except (ValueError, TypeError, KeyError, AttributeError) as error:
                            if isinstance(content, str):
                                (folder / 'unvalidated-content.txt').write_text(content, encoding='utf-8', newline='')
                            write(folder / 'invalid.json', {'status': 'invalid', 'error_class': type(error).__name__,
                                  'error': str(error), 'policy': 'whole_item_failure_no_mapping_no_retry'})
                            invalid += 1
                            event(ledger, {'event': 'invalid_retained_continue', 'id': row['id'],
                                  'error_class': type(error).__name__})
                        else:
                            write(folder / 'rating.json', rating)
                            completed += 1
                    except Exception as error:
                        stopped = stopped or 'item_failure'
                        event(ledger, {'event': 'item_failed_stop_new_posts', 'id': row['id'], 'error_class': type(error).__name__})
        except Exception as error:
            stopped = stopped or 'supervisor_failure'
            event(ledger, {'event': 'supervisor_failure', 'error_class': type(error).__name__})
            # Outstanding exact children still have their own deadlines and terminal files.
            # Their unsettled full reservations remain; no replacement calls are made.
        finally:
            if pending:
                # An outer I/O failure prevents safe ledger settlement; retain all held amounts.
                wait(pending)
            complete = submitted == 504 and completed + invalid == 504 and stopped is None
            status = 'collection_complete_awaiting_random20_review' if complete else stopped or 'incomplete'
            write(output / 'result.json', {'status': status, 'post_attempts': submitted, 'completed_ratings': completed,
                  'carried_posts': plan['resume_from_count'], 'new_posts_this_segment': submitted - plan['resume_from_count'],
                  'collection_complete': complete, 'invalid_ratings': invalid, 'not_run': 504 - submitted,
                  'prior_attempts': plan['prior_attempts'],
                  'overall_attempts': plan['prior_attempts'] + submitted, 'prior_upper_nano_usd': plan['prior_upper_nano_usd'],
                  'new_upper_nano_usd': total - plan['prior_upper_nano_usd'], 'cumulative_upper_nano_usd': total,
                  'pilot_ratings_in_evidence': False, 'formal_analysis': 'not_run_pending_random20_user_review',
                  'ended_utc': datetime.now(timezone.utc).isoformat(), 'response_model': model})
            os.environ.pop('DEEPSEEK_API_KEY', None)
    return 0 if complete else 2


def worker(head: str) -> int:
    plan = preflight(head)
    if not os.environ.get('DEEPSEEK_API_KEY'):
        raise ValueError('Process credential missing')
    return execute(plan, SESSION / 'run', head)


def launch(head: str) -> int:
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
        raise SystemExit(launch(args.head) if args.mode == 'launch' else worker(args.head))
    except Exception as error:
        write(SESSION / 'supervisor-stop.json', {'status': 'preflight_or_supervisor_failure',
              'error_class': type(error).__name__, 'utc': datetime.now(timezone.utc).isoformat()})
        raise SystemExit(2) from None
