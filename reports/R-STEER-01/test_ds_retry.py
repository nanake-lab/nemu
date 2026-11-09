"""No paid calls: real loopback backoff plus finite retry and outcome boundaries."""
from __future__ import annotations

import json
import tempfile
import threading
import time
import unittest
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from unittest.mock import patch

import ds_retry_analysis
import ds_retry_run
from ds_full2_windowless_run import supervise
from ds_high_prepare import reserve
from ds_high_resume import BASE
from ds_protocol import HERE, example, load, write
from ds_retry_budget import Ledger
from ds_retry_http import parseable_content
from test_ds_full import fake_response


def fixture(seconds: int = 0) -> tuple[Path, dict[str, Any], dict[str, Any]]:
    folder = Path(tempfile.mkdtemp(dir='D:/Dev/cache/R-STEER-01', prefix='network-retry-offline-'))
    print('retained_offline_case=' + str(folder))
    (folder / 'requests').mkdir()
    body = {'messages': [{'role': 'system', 'content': 'OFFLINE_ONLY'},
                         {'role': 'user', 'content': json.dumps({'待评输出': 'OFFLINE_ONLY', '评分事实参照': 'OFFLINE_ONLY'})}]}
    (folder / 'requests/offline.json').write_text(json.dumps(body), encoding='utf-8')
    plan = {'prior_attempts': 0, 'prior_upper_nano_usd': 0, 'remaining_wall_seconds': 120,
            'soft_nano_usd': 4750000000, 'hard_nano_usd': 5000000000,
            'max_retries': 2, 'retry_seconds': seconds, 'expected_response_model': 'deepseek-flash'}
    return folder, plan, example(['connection', 'empathy', 'curiosity', 'achievement'])


class RetryBoundary(unittest.TestCase):
    def test_partial_parseable_json_blocks_retry_without_inspecting_label(self) -> None:
        for value in ({'bad_schema': 'any'}, ['any'], None, 42):
            content = json.dumps(json.dumps(value))
            raw = ('{"choices":[{"message":{"content":' + content + '}').encode()
            self.assertTrue(parseable_content(raw))
        self.assertFalse(parseable_content(b'{"choices":[{"message":{"content":"unfinished'))

    def test_complete_invalid_rating_is_never_retried(self) -> None:
        folder, plan, rating = fixture()
        output = folder / 'run'
        output.mkdir()
        ledger = Ledger(plan, output / 'ledger.jsonl')
        calls = []
        def transport(command: list[str], dest: Path, deadline: float) -> tuple[int, bool]:
            calls.append(dest)
            rating['valence']['label'] = 'both'
            fake_response(dest, rating)
            write(dest / 'transport.json', {'response_complete': True})
            return 0, False
        with patch.object(ds_retry_run, 'SESSION', folder), patch.object(ds_retry_run, 'supervise', transport):
            outcome = ds_retry_run.item({'id': 'offline', 'attempts_already': 0}, ledger, output)
        self.assertEqual(outcome, 'invalid')
        self.assertEqual(len(calls), 1)
        self.assertTrue((output / 'offline/invalid.json').exists())

    def test_at_most_two_resends_keep_every_full_reservation(self) -> None:
        folder, plan, _ = fixture()
        output = folder / 'run'
        output.mkdir()
        ledger = Ledger(plan, output / 'ledger.jsonl')
        bodies = []
        def transport(command: list[str], dest: Path, deadline: float) -> tuple[int, bool]:
            bodies.append((dest / 'request.json').read_bytes())
            write(dest / 'transport.json', {'status': 'network_incomplete', 'retry_eligible': True})
            return 2, False
        with patch.object(ds_retry_run, 'SESSION', folder), patch.object(ds_retry_run, 'supervise', transport):
            outcome = ds_retry_run.item({'id': 'offline', 'attempts_already': 0}, ledger, output)
        self.assertEqual(outcome, 'technical_failure')
        self.assertEqual(len(bodies), 3)
        self.assertTrue(all(body == bodies[0] for body in bodies))
        self.assertEqual(ledger.total, 3 * reserve(bodies[0]))

    def test_existing_failed_attempt_uses_only_two_additional_calls(self) -> None:
        folder, plan, _ = fixture()
        output = folder / 'run'
        output.mkdir()
        ledger = Ledger(plan, output / 'ledger.jsonl')
        def transport(command: list[str], dest: Path, deadline: float) -> tuple[int, bool]:
            write(dest / 'transport.json', {'status': 'network_incomplete', 'retry_eligible': True})
            return 2, False
        with patch.object(ds_retry_run, 'SESSION', folder), patch.object(ds_retry_run, 'supervise', transport):
            outcome = ds_retry_run.item({'id': 'offline', 'attempts_already': 1}, ledger, output)
        self.assertEqual(outcome, 'technical_failure')
        self.assertEqual(ledger.attempts, 2)

    def test_consecutive_final_items_use_frozen_order_and_invalid_breaks_streak(self) -> None:
        rows = [{'id': str(i)} for i in range(5)]
        final = {'1': 'technical_failure', '2': 'technical_failure', '3': 'technical_failure'}
        self.assertTrue(ds_retry_run.streak(rows, final, 3))
        final['2'] = 'invalid'
        self.assertFalse(ds_retry_run.streak(rows, final, 3))

    def test_simultaneous_reservations_cannot_cross_soft_budget(self) -> None:
        folder, plan, _ = fixture()
        body = (folder / 'requests/offline.json').read_bytes()
        held = reserve(body)
        plan['soft_nano_usd'] = 2 * held
        ledger = Ledger(plan, folder / 'ledger.jsonl')
        barrier = threading.Barrier(4)
        booked: list[int | None] = []
        def book() -> None:
            barrier.wait(timeout=5)
            booked.append(ledger.book('offline', 1, body))
        threads = [threading.Thread(target=book) for _ in range(4)]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()
        self.assertEqual(sum(value is not None for value in booked), 1)
        self.assertEqual(ledger.total, held)
        self.assertEqual(ledger.reason, 'budget_stop')

    def test_unrecognized_process_failure_stops_without_retransmission(self) -> None:
        folder, plan, _ = fixture()
        output = folder / 'run'
        output.mkdir()
        ledger = Ledger(plan, output / 'ledger.jsonl')
        with patch.object(ds_retry_run, 'SESSION', folder), patch.object(ds_retry_run, 'supervise', return_value=(7, False)) as transport:
            outcome = ds_retry_run.item({'id': 'offline', 'attempts_already': 0}, ledger, output)
        self.assertEqual(outcome, 'pending')
        self.assertEqual(transport.call_count, 1)
        self.assertTrue(ledger.stop.is_set())

    def test_review_approval_required_before_any_formal_score_use(self) -> None:
        folder, _, _ = fixture()
        (folder / 'run').mkdir()
        write(folder / 'plan.json', {})
        write(folder / 'run/result.json', {'collection_complete': True})
        with patch.object(ds_retry_analysis, 'SESSION', folder):
            with self.assertRaises(FileNotFoundError) as error:
                ds_retry_analysis.analyze()
        self.assertIn('sample-review.json', str(error.exception))

    def test_real_loopback_truncation_waits_at_least_thirty_seconds_before_retry(self) -> None:
        folder, plan, rating = fixture(30)
        output = folder / 'run'
        output.mkdir()
        ledger = Ledger(plan, output / 'ledger.jsonl')
        times: list[float] = []
        class Handler(BaseHTTPRequestHandler):
            def do_POST(self) -> None:
                self.rfile.read(int(self.headers['Content-Length']))
                times.append(time.monotonic())
                if len(times) == 1:
                    body = b'{"choices":[{"message":{"reasoning_content":"OFFLINE_PARTIAL'
                    self.send_response(200)
                    self.send_header('Content-Length', str(len(body) + 100))
                    self.send_header('Connection', 'close')
                    self.end_headers()
                    self.wfile.write(body)
                    self.wfile.flush()
                    self.close_connection = True
                else:
                    value = {'model': 'deepseek-flash', 'choices': [{'finish_reason': 'stop', 'message': {
                        'reasoning_content': 'OFFLINE_FAKE_REASONING', 'content': json.dumps(rating)}}],
                        'usage': {'prompt_tokens': 100, 'prompt_cache_hit_tokens': 0, 'prompt_cache_miss_tokens': 100,
                                  'completion_tokens': 1000, 'total_tokens': 1100}}
                    body = json.dumps(value).encode()
                    self.send_response(200)
                    self.send_header('Content-Length', str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
            def log_message(self, format: str, *args: Any) -> None:
                pass
        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        endpoint = f'http://127.0.0.1:{server.server_port}/'
        code = ("import sys,os; from pathlib import Path; sys.path.insert(0,sys.argv[1]); "
                "os.environ['DEEPSEEK_API_KEY']='OFFLINE_FAKE_KEY'; "
                "from ds_retry_http import post; raise SystemExit(post(Path(sys.argv[2]),sys.argv[3]))")
        def transport(command: list[str], dest: Path, deadline: float) -> tuple[int, bool]:
            return supervise([str(BASE), '-I', '-B', '-c', code, str(HERE), str(dest), endpoint], dest, deadline)
        try:
            with patch.object(ds_retry_run, 'SESSION', folder), patch.object(ds_retry_run, 'supervise', transport):
                outcome = ds_retry_run.item({'id': 'offline', 'attempts_already': 0}, ledger, output)
        finally:
            server.shutdown()
            server.server_close()
            thread.join()
        self.assertEqual(outcome, 'rating')
        self.assertEqual(len(times), 2)
        self.assertGreaterEqual(times[1] - times[0], 30)
        self.assertTrue((output / 'offline/attempts/01/partial-response.bin').exists())
        self.assertEqual(ledger.total, 2 * reserve((folder / 'requests/offline.json').read_bytes()))
        write(folder / 'real-backoff-measurement.json', {'utc': datetime.now(timezone.utc).isoformat(),
              'request_gap_seconds': times[1] - times[0], 'endpoint': 'loopback_only', 'api_calls': 0})

    def test_http_error_body_is_preserved_without_retry_eligibility(self) -> None:
        folder, _, _ = fixture()
        target = folder / 'http-error'
        target.mkdir()
        (target / 'request.json').write_bytes(b'{}')
        body = b'{"error":{"message":"OFFLINE_ONLY"}}'
        class Handler(BaseHTTPRequestHandler):
            def do_POST(self) -> None:
                self.rfile.read(int(self.headers['Content-Length']))
                self.send_response(503)
                self.send_header('Content-Length', str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            def log_message(self, format: str, *args: Any) -> None:
                pass
        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        code = ("import sys,os; from pathlib import Path; sys.path.insert(0,sys.argv[1]); "
                "os.environ['DEEPSEEK_API_KEY']='OFFLINE_FAKE_KEY'; "
                "from ds_retry_http import post; raise SystemExit(post(Path(sys.argv[2]),sys.argv[3]))")
        try:
            result = supervise([str(BASE), '-I', '-B', '-c', code, str(HERE), str(target),
                                f'http://127.0.0.1:{server.server_port}/'], target, 10)
        finally:
            server.shutdown()
            server.server_close()
            thread.join()
        self.assertEqual(result, (3, False))
        self.assertEqual((target / 'http-error.body').read_bytes(), body)
        self.assertFalse(load(target / 'transport.json')['retry_eligible'])


if __name__ == '__main__':
    unittest.main()
