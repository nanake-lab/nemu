"""Real no-window child startup and no-duplicate carry accounting, without API."""
from __future__ import annotations

import json
import subprocess
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest.mock import patch

import ds_full2_windowless_run as runner
from ds_protocol import load, sha
from test_ds_full import case, fake_response


class Windowless(unittest.TestCase):
    def test_real_child_has_no_window_and_retains_original_terminal_protocol(self) -> None:
        folder = Path(tempfile.mkdtemp(dir='D:/Dev/cache/R-STEER-01', prefix='full2-nowindow-'))
        print('retained_offline_case=' + str(folder))
        result: list[tuple[int, bool]] = []
        worker = threading.Thread(target=lambda: result.append(runner.supervise(
            [str(runner.BASE), '-I', '-B', '-c', 'import time; print("OFFLINE_ONLY"); time.sleep(6)'], folder, 10)))
        worker.start()
        for _ in range(100):
            if (folder / 'worker-started.json').exists():
                break
            time.sleep(0.02)
        pid = load(folder / 'worker-started.json')['pid']
        command = f'$p=Get-Process -Id {pid}; [long]$p.MainWindowHandle'
        for _ in range(3):
            raw = subprocess.check_output(['C:/Users/nanake/AppData/Local/Microsoft/WindowsApps/pwsh.exe',
                '-NoProfile', '-NonInteractive', '-Command', command], creationflags=subprocess.CREATE_NO_WINDOW, text=True)
            self.assertEqual(raw.strip(), '0')
        worker.join(timeout=12)
        self.assertEqual(result, [(0, False)])
        self.assertEqual((folder / 'worker.stdout').read_text().strip(), 'OFFLINE_ONLY')
        self.assertEqual(load(folder / 'worker-terminal.json')['exit_code'], 0)

    def test_completed_v2_outcomes_are_copied_exactly_not_reposted(self) -> None:
        folder, plan, rating = case(8)
        original = Path(tempfile.mkdtemp(dir='D:/Dev/cache/R-STEER-01', prefix='full2-carry-'))
        print('retained_offline_case=' + str(original))
        (original / 'run').mkdir()
        carried = []
        for row in plan['requests'][:2]:
            destination = original / 'run' / row['id']
            destination.mkdir()
            fake_response(destination, rating)
            for path in destination.iterdir():
                carried.append({'path': path.relative_to(original / 'run').as_posix(), 'sha256': sha(path)})
        plan.update({'prior_attempts': 48, 'prior_upper_nano_usd': 100000992, 'resume_from_count': 2,
                     'carried_completed': 2, 'carried_invalid': 0, 'carried_cumulative_upper_nano_usd': 102460992,
                     'carried_model': 'deepseek-flash', 'carried_files': carried, 'carried_head': 'offline',
                     'carried_result_sha256': 'offline', 'carried_ledger_sha256': 'offline'})
        sent: list[str] = []
        def transport(command: list[str], dest: Path, deadline: float) -> tuple[int, bool]:
            sent.append(dest.name)
            fake_response(dest, rating)
            return 0, False
        with patch.object(runner, 'ORIGINAL', original), patch.object(runner, 'SESSION', folder), patch.object(runner, 'supervise', transport):
            runner.execute(plan, folder / 'run', 'offline')
        result = load(folder / 'run/result.json')
        self.assertEqual(len(sent), 6)
        self.assertFalse({'offline000', 'offline001'} & set(sent))
        self.assertEqual((result['carried_posts'], result['new_posts_this_segment'], result['post_attempts']), (2, 6, 8))
        self.assertEqual(result['cumulative_upper_nano_usd'], 102460992 + 6 * 1230000)
        for row in carried:
            self.assertEqual(sha(folder / 'run' / row['path']), row['sha256'])
        first = json.loads((folder / 'run/ledger.jsonl').read_text().splitlines()[0])
        self.assertEqual(first['event'], 'carry_forward_original_v2_no_reposts')


if __name__ == '__main__':
    unittest.main()
