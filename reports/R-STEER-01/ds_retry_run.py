"""Four logical slots, bounded raw retransmission and immutable attempt evidence."""
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

from ds_full2_windowless_run import supervise
from ds_high_resume import BASE
from ds_high_run import high_cost
from ds_protocol import HERE, load, sha, validate_rating, write
from ds_retry_budget import Ledger
from ds_retry_prepare import ORIGINAL, SESSION, preflight


def item(row: dict[str, Any], ledger: Ledger, output: Path) -> str:
    identifier = str(row['id'])
    folder = output / identifier
    folder.mkdir(exist_ok=True)
    body = (SESSION / 'requests' / (identifier + '.json')).read_bytes()
    if not (folder / 'request.json').exists():
        (folder / 'request.json').write_bytes(body)
    due = 0.0
    if row.get('previous_failure_utc'):
        elapsed = (datetime.now(timezone.utc) - datetime.fromisoformat(row['previous_failure_utc'])).total_seconds()
        due = time.monotonic() + max(0, ledger.plan['retry_seconds'] - elapsed)
    for number in range(int(row['attempts_already']) + 1, ledger.plan['max_retries'] + 2):
        try:
            if time.monotonic() < due:
                ledger.record({'event': 'retry_wait_started', 'id': identifier, 'attempt': number})
            while time.monotonic() < due and not ledger.stop.is_set():
                ledger.stop.wait(min(1, due - time.monotonic()))
            held = ledger.book(identifier, number, body)
            if held is None:
                return 'pending'
            attempt = folder / 'attempts' / f'{number:02d}'
            attempt.mkdir(parents=True, exist_ok=False)
            (attempt / 'request.json').write_bytes(body)
            ledger.record({'event': 'attempt_started', 'id': identifier, 'attempt': number,
                           'request_sha256': sha(attempt / 'request.json')})
            code, expired = supervise([str(BASE), '-B', str(HERE / 'ds_retry_http.py'), '--folder', str(attempt)],
                                     attempt, min(180, max(0, ledger.until - time.monotonic())))
            transport = load(attempt / 'transport.json')
            if expired or code not in (0, 2):
                raise ValueError('Process/HTTP/integrity failure; no retransmission')
            if code == 2:
                if transport['status'] != 'network_incomplete':
                    raise ValueError('Unknown process failure')
                ledger.record({'event': 'network_incomplete_full_reservation_retained', 'id': identifier,
                               'attempt': number, 'transport': transport})
                if transport['retry_eligible'] and number <= ledger.plan['max_retries'] and not ledger.stop.is_set():
                    due = time.monotonic() + ledger.plan['retry_seconds']
                    continue
                if ledger.stop.is_set():
                    return 'pending'
                write(folder / 'technical_failure.json', {'status': 'technical_failure', 'last_attempt': number,
                      'reason': 'retries_exhausted' if transport['retry_eligible'] else 'parseable_partial_no_retry',
                      'transport': transport})
                return 'technical_failure'
            if transport['response_complete'] is not True:
                raise ValueError('Complete-response marker required')
            response = load(attempt / 'response.json')
            actual = high_cost(response['usage'])
            ledger.settle(identifier, number, held, actual, response['usage'])
            if response.get('model') != ledger.plan['expected_response_model']:
                raise ValueError('Response model changed')
            choices = response['choices']
            if len(choices) != 1 or choices[0]['finish_reason'] != 'stop':
                raise ValueError('Completion termination differs')
            message = choices[0]['message']
            if message.get('tool_calls') or not isinstance(message.get('reasoning_content'), str) or not message['reasoning_content'].strip():
                raise ValueError('High thinking response differs')
            shutil.copyfile(attempt / 'response.json', folder / 'response.json')
            write(folder / 'attempt-selection.json', {'selected_attempt': number, 'retry': number > 1,
                  'response_sha256': sha(attempt / 'response.json'), 'full_response_never_retried': True})
            presented = json.loads(load(attempt / 'request.json')['messages'][1]['content'])
            try:
                content = message.get('content')
                if not isinstance(content, str) or not content.strip():
                    raise ValueError('Blank/nonstring grading content')
                rating = validate_rating(json.loads(content), presented['待评输出'], presented['评分事实参照'])
            except (ValueError, TypeError, KeyError, AttributeError) as error:
                write(folder / 'invalid.json', {'status': 'invalid', 'error_class': type(error).__name__,
                      'error': str(error), 'selected_attempt': number, 'retry_allowed': False})
                return 'invalid'
            write(folder / 'rating.json', rating)
            return 'rating'
        except Exception as error:
            ledger.record({'event': 'fatal_item_error', 'id': identifier, 'attempt': number,
                           'error_class': type(error).__name__})
            ledger.halt('process_or_integrity_failure')
            write(folder / 'fatal.json', {'status': 'fatal', 'attempt': number, 'error_class': type(error).__name__})
            return 'pending'
    raise AssertionError('Bounded attempt loop must return')


def streak(work: list[dict[str, Any]], outcomes: dict[str, str], limit: int) -> bool:
    return any(all(outcomes.get(row['id']) == 'technical_failure' for row in work[start:start + limit])
               for start in range(len(work) - limit + 1))


def execute(plan: dict[str, Any], output: Path, head: str) -> int:
    output.mkdir(exist_ok=False)
    write(output / 'binding.json', {'head': head, 'pid': os.getpid(), 'started_utc': datetime.now(timezone.utc).isoformat(),
          'plan_sha256': sha(SESSION / 'plan.json'), 'pilot_or_full01_scores_used': False})
    for identifier in plan['carried_ids']:
        shutil.copytree(ORIGINAL / 'run' / identifier, output / identifier)
    for row in plan['carried_files']:
        if sha(output / row['path']) != row['sha256']:
            raise ValueError('Copied original evidence differs')
    ledger = Ledger(plan, output / 'ledger.jsonl')
    ledger.record({'event': 'carry_forward', 'prior_attempts': plan['prior_attempts'],
                   'cumulative_upper_nano_usd': ledger.total, 'carried_ratings': len(plan['carried_rating_ids']),
                   'original_result_sha256': plan['original_result_sha256']})
    outcomes = {identifier: 'rating' for identifier in plan['carried_rating_ids']}
    submitted = 0
    pending: dict[Future[str], dict[str, Any]] = {}
    work = plan['worklist']
    with ThreadPoolExecutor(max_workers=plan['max_in_flight']) as pool:
        while pending or (not ledger.stop.is_set() and submitted < len(work)):
            if (SESSION / 'stop-request.json').exists():
                ledger.halt('user_stop')
            if time.monotonic() >= ledger.until:
                ledger.halt('time_stop')
            while not ledger.stop.is_set() and len(pending) < plan['max_in_flight'] and submitted < len(work):
                row = work[submitted]
                submitted += 1
                pending[pool.submit(item, row, ledger, output)] = row
            if not pending:
                break
            done, _ = wait(pending, timeout=2, return_when=FIRST_COMPLETED)
            for future in done:
                row = pending.pop(future)
                try:
                    outcomes[row['id']] = future.result()
                except Exception as error:
                    ledger.record({'event': 'worker_thread_failure', 'error_class': type(error).__name__})
                    ledger.halt('supervisor_failure')
                if streak(work, outcomes, plan['technical_streak_limit']):
                    ledger.halt('three_consecutive_technical_failures')
            ledger.record({'event': 'supervisor_alive', 'submitted_work_items': submitted,
                           'logical_in_flight': len(pending), 'final_items': sum(v != 'pending' for v in outcomes.values()),
                           'cumulative_upper_nano_usd': ledger.total, 'reason': ledger.reason})
    complete = len(outcomes) == 504 and all(v in ('rating', 'invalid', 'technical_failure') for v in outcomes.values()) and not ledger.stop.is_set()
    value = {'status': 'collection_complete_awaiting_random20_review' if complete else ledger.reason or 'incomplete',
             'collection_complete': complete, 'completed_ratings': sum(v == 'rating' for v in outcomes.values()),
             'invalid_ratings': sum(v == 'invalid' for v in outcomes.values()),
             'technical_failures': sum(v == 'technical_failure' for v in outcomes.values()),
             'new_attempts': ledger.attempts - plan['prior_attempts'], 'overall_attempts': ledger.attempts,
             'cumulative_upper_nano_usd': ledger.total, 'new_upper_nano_usd': ledger.total - plan['prior_upper_nano_usd'],
             'unstarted_work_items': len(work) - submitted, 'outcomes': outcomes,
             'formal_analysis': 'not_run_pending_random20_user_review', 'ended_utc': datetime.now(timezone.utc).isoformat()}
    write(output / 'result.json', value)
    os.environ.pop('DEEPSEEK_API_KEY', None)
    return 0 if complete else 2


def launch(head: str) -> int:
    with (SESSION / 'supervisor.stdout').open('xb') as stdout, (SESSION / 'supervisor.stderr').open('xb') as stderr:
        process = subprocess.Popen([str(BASE), '-B', str(Path(__file__).resolve()), 'worker', '--head', head],
            stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr,
            creationflags=subprocess.DETACHED_PROCESS | subprocess.CREATE_BREAKAWAY_FROM_JOB)
    write(SESSION / 'supervisor-started.json', {'pid': process.pid, 'utc': datetime.now(timezone.utc).isoformat(), 'key_persisted': False})
    return 0


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=('launch', 'worker'))
    parser.add_argument('--head', required=True)
    args = parser.parse_args()
    try:
        if args.mode == 'launch':
            raise SystemExit(launch(args.head))
        plan_value = preflight(args.head)
        if not os.environ.get('DEEPSEEK_API_KEY'):
            raise ValueError('Process credential required')
        raise SystemExit(execute(plan_value, SESSION / 'run', args.head))
    except Exception as error:
        write(SESSION / 'supervisor-stop.json', {'status': 'preflight_or_supervisor_failure', 'error_class': type(error).__name__})
        raise SystemExit(2) from None
