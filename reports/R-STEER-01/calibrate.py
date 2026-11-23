"""Prepare or run C0; stdlib only. Never loads the carrier during preparation."""
from __future__ import annotations

import argparse
import hashlib
import json
import queue
import shutil
import subprocess
import threading
import time
from datetime import datetime, timezone
from pathlib import Path

from fixtures import make_jobs, render

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ENGINE = Path('D:/Dev/cache/P0-10/llama.cpp/b11669')
MODEL = Path('D:/Dev/cache/P0-12/gguf/carrier-Q4_K_M.gguf')
MODEL_HASH = '045b7d309c8021d4d7bcebe1156ae94a1e3e128df271d05f5adfcea38ee20b27'


def digest(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def write_json(path: Path, value: object) -> None:
    with path.open('x', encoding='utf-8', newline='\n') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


def inventory() -> list[dict[str, str]]:
    completed = subprocess.run(
        ['powershell.exe', '-NoProfile', '-Command',
         'Get-Process | Select-Object ProcessName,Id | ConvertTo-Json -Compress'],
        check=True, capture_output=True, encoding='utf-8')
    records: list[dict[str, str]] = json.loads(completed.stdout)
    return records


def require_quiet() -> list[dict[str, str]]:
    processes = inventory()
    blocked = {'runner.listener', 'runner.worker', 'llama-server', 'blender'}
    if any(str(row['ProcessName']).lower() in blocked for row in processes):
        raise RuntimeError('Quiet window conflict; no process has been stopped')
    return processes


def prepare(output: Path) -> None:
    output.mkdir(parents=True, exist_ok=False)
    (output / 'prompts').mkdir()
    (output / 'requests').mkdir()
    reference = (ROOT / 'reports/P0-12/inputs/injection-en.txt').read_text(encoding='utf-8')
    if reference != render("Repeat exactly, adding nothing: Today's effort matters. Keep going with confidence."):
        raise ValueError('Rendered template differs from P0-12 official nonthinking sample')
    jobs = make_jobs()
    # First three requests and repeats are identical zero-modulation byte controls, not extra cases.
    guards = []
    for i, language in enumerate(('zh', 'en', 'ja')):
        original = next(job for job in jobs if job['lang'] == language and job['kind'] == 'oracle')
        for repeat in (0, 1):
            guard = original.copy()
            guard['id'] = f'guard-{i}-{repeat}'
            guard['kind'] = 'guard'
            guards.append(guard)
    jobs = guards[::2] + jobs + guards[1::2]
    frozen = HERE / 'inputs/c0-jobs.json'
    if json.loads(frozen.read_text(encoding='utf-8')) != jobs:
        raise ValueError('Generated jobs differ from pushed input corpus')
    for job in jobs:
        (output / 'prompts' / (job['id'] + '.txt')).write_text(job['prompt'], encoding='utf-8', newline='\n')
    write_json(output / 'jobs.json', jobs)
    # Execution definitions are frozen here; REPORT may later record independent Base preparation.
    paths = sorted(HERE.glob('*.py')) + [HERE / 'calibration_probe.cpp', HERE / 'build.ps1', HERE / 'PREREG-C0.md',
                                      frozen, ROOT / 'spikes/carrier-engine/native_probe.hpp']
    write_json(output / 'preparation.json', {
        'status': 'prepared_not_run', 'jobs': len(jobs), 'template_reference': digest(ROOT / 'reports/P0-12/inputs/injection-en.txt'),
        'jobs_sha256': digest(output / 'jobs.json'),
        'prompts': [{'id': job['id'], 'sha256': digest(output / 'prompts' / (job['id'] + '.txt'))} for job in jobs],
        'sources': [{'path': path.relative_to(ROOT).as_posix(), 'sha256': digest(path)} for path in paths]})


def run(output: Path, executable: Path) -> None:
    preparation = json.loads((output / 'preparation.json').read_text(encoding='utf-8'))
    for entry in preparation['sources']:
        if digest(ROOT / entry['path']) != entry['sha256']:
            raise ValueError('Frozen source changed: ' + entry['path'])
    if digest(output / 'jobs.json') != preparation['jobs_sha256']:
        raise ValueError('Frozen jobs changed')
    for entry in preparation['prompts']:
        if digest(output / 'prompts' / (entry['id'] + '.txt')) != entry['sha256']:
            raise ValueError('Frozen prompt changed')
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    remote = subprocess.check_output(['git', 'ls-remote', 'origin', 'refs/heads/task/R-STEER-01'], cwd=ROOT, text=True).split()[0]
    dirty = subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip()
    if dirty or head != remote:
        raise ValueError('Run only clean, pushed preregistration head')
    background = require_quiet()
    if digest(MODEL) != MODEL_HASH:
        raise ValueError('P0-12 model hash mismatch')
    expected = json.loads((ROOT / 'reports/P0-12/continuation-02/engine-check.json').read_text(encoding='utf-8'))
    for entry in expected['files']:
        if digest(ENGINE / entry['path']) != entry['expected']:
            raise ValueError('Pinned engine mismatch: ' + entry['path'])
    if shutil.disk_usage(output).free < 10_000_000_000:
        raise ValueError('Less than 10 GB evidence reserve')  # Engineering reserve, not behavioral criterion.
    build = json.loads((executable.parent / 'build.json').read_text(encoding='utf-8-sig'))
    if (build['exitCode'] != 0 or digest(executable) != build['executableSha256']
            or digest(HERE / 'calibration_probe.cpp') != build['sourceSha256']
            or digest(ROOT / 'spikes/carrier-engine/native_probe.hpp') != build['nativeHeaderSha256']):
        raise ValueError('Build binding mismatch')
    write_json(output / 'binding.json', {'head': head, 'remote_head': remote, 'main_baseline': 'cb95af4dbf5e6d39b1e8664baee5530776b46265',
               'started_utc': datetime.now(timezone.utc).isoformat(), 'model_sha256': MODEL_HASH,
               'executable_sha256': digest(executable), 'background_processes': background,
               'quiet_window_confirmation': 'required by --quiet-window-confirmed; user message remains authority'})
    jobs = json.loads((output / 'jobs.json').read_text(encoding='utf-8'))
    started = time.monotonic()
    code: int | None = None
    completed = 0
    with (output / 'native-stderr.log').open('xb') as stderr, (output / 'protocol.log').open('x', encoding='utf-8') as protocol:
        process = subprocess.Popen([str(executable), str(ENGINE), str(MODEL), str(output), '--quiet-window-confirmed'],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   text=True, encoding='utf-8', creationflags=subprocess.CREATE_NO_WINDOW)
        lines: queue.Queue[str] = queue.Queue()

        def read_lines() -> None:
            assert process.stdout is not None
            for line in process.stdout:
                lines.put(line)
            lines.put('EOF')

        def kill_owned() -> None:
            if process.poll() is None:
                subprocess.run(['taskkill.exe', '/PID', str(process.pid), '/T', '/F'],
                               capture_output=True, check=False, creationflags=subprocess.CREATE_NO_WINDOW)

        timer = threading.Timer(600, kill_owned)  # Bounded feasibility window; no automatic retry.
        timer.start()
        threading.Thread(target=read_lines, daemon=True).start()
        try:
            if lines.get(timeout=60).strip() != 'READY':
                raise RuntimeError('Native startup failed; inspect original stderr')
            assert process.stdin is not None
            for job in jobs:
                require_quiet()
                keep = 'full' if job['kind'] == 'guard' else 'summary'
                process.stdin.write(f"{job['id']}\t{job['limit']}\t{keep}\n")
                process.stdin.flush()
                line = lines.get(timeout=60)
                protocol.write(line)
                protocol.flush()
                if not line.startswith('OK\t' + job['id'] + '\t'):
                    raise RuntimeError('Native request failed; preserve partial evidence')
                completed += 1
            process.stdin.write('QUIT\n')
            process.stdin.flush()
            code = process.wait(timeout=30)
            if code != 0:
                raise RuntimeError('Native exited unsuccessfully')
        finally:
            kill_owned()
            process.wait(timeout=30)
            timer.cancel()
            write_json(output / 'exit.json', {'native_exit_code': process.returncode, 'completed': completed,
                       'expected': len(jobs), 'wall_seconds': time.monotonic() - started, 'all_requests_completed': code == 0})


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=('prepare', 'run'))
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--executable', type=Path)
    parser.add_argument('--quiet-window-confirmed', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(Path('D:/Dev/cache/R-STEER-01').resolve()):
        raise ValueError('Use task cache on D')
    if args.mode == 'prepare':
        prepare(output)
    elif args.quiet_window_confirmed and args.executable:
        run(output, args.executable.resolve())
    else:
        raise ValueError('Run requires an explicit quiet window and a bound executable')


if __name__ == '__main__':
    main()
