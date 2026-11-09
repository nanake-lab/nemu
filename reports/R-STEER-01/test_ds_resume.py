"""Offline failure-boundary checks, using loopback and an explicitly fake token only."""
from __future__ import annotations

import http.server
import tempfile
import threading
import time
import unittest
from pathlib import Path

from ds_high_resume import BASE, supervise
from ds_protocol import HERE, load


class KeepAlive(http.server.BaseHTTPRequestHandler):
    started = threading.Event()

    def do_POST(self) -> None:
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.started.set()
        try:
            for _ in range(100):
                self.wfile.write(b'\n')
                self.wfile.flush()
                time.sleep(0.05)
        except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
            pass

    def log_message(self, format: str, *args: object) -> None:
        pass


class ResumeBoundaries(unittest.TestCase):
    def test_wall_deadline_stops_keepalive_http_child_and_records_terminal(self) -> None:
        # Construct a fresh environment. Never read the real key or inherited environment.
        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), KeepAlive)
        KeepAlive.started.clear()
        server.daemon_threads = True
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            with tempfile.TemporaryDirectory(dir='D:/Dev/cache/R-STEER-01', prefix='resume-offline-', delete=False) as temp:
                print('retained_offline_case=' + temp)
                folder = Path(temp)
                (folder / 'request.json').write_text('{}', encoding='utf-8')
                command = [str(BASE), '-B', '-c',
                    "import os,sys;from pathlib import Path;sys.path.insert(0,sys.argv[1]);"
                    "os.environ['DEEPSEEK_API_KEY']='OFFLINE_FAKE_TOKEN';"
                    "from ds_http_once import post;raise SystemExit(post(Path(sys.argv[2]),sys.argv[3]))",
                    str(HERE), str(folder), f'http://127.0.0.1:{server.server_port}']
                started = time.monotonic()
                code, hit = supervise(command, folder, 0.6)
                self.assertTrue(KeepAlive.started.is_set())
                self.assertTrue(hit)
                self.assertNotEqual(code, 0)
                self.assertLess(time.monotonic() - started, 3)
                self.assertTrue(load(folder / 'worker-terminal.json')['wall_deadline_hit'])
                self.assertFalse((folder / 'response.json').exists())
        finally:
            server.shutdown()
            server.server_close()

    def test_unexpected_exit_is_preserved_without_starting_another_process(self) -> None:
        with tempfile.TemporaryDirectory(dir='D:/Dev/cache/R-STEER-01', prefix='resume-exit-', delete=False) as temp:
            print('retained_offline_case=' + temp)
            folder = Path(temp)
            code, hit = supervise([str(BASE), '-B', '-c', 'raise SystemExit(7)'], folder, 3)
            self.assertEqual(code, 7)
            self.assertFalse(hit)
            self.assertEqual(load(folder / 'worker-terminal.json')['exit_code'], 7)
            self.assertEqual(len(list(folder.glob('worker-started.json'))), 1)

    def test_only_unknown_item_and_three_unrun_items_are_selected(self) -> None:
        old = load(HERE / 'inputs/ds-flash-high-plan-01.json')['requests']
        result = load(HERE / 'results/ds-flash-high-summary-01.json')
        remaining = [r['id'] for r in old[16:]]
        self.assertEqual(remaining, result['unknown_outcome_ids'] + result['not_run_ids'])
        self.assertEqual(len(remaining), 4)
        self.assertEqual(result['cumulative_upper_nano_usd'], 68386560)


if __name__ == '__main__':
    unittest.main()
