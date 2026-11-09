"""C1 prepare/run driver; no model use in prepare, no implicit window approval."""
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

from c1_fixtures import make_jobs, render

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CACHE = Path('D:/Dev/cache/R-STEER-01').resolve()
ENGINE = Path('D:/Dev/cache/P0-10/llama.cpp/b11669')
MODEL = Path('D:/Dev/cache/P0-12/gguf/carrier-Q4_K_M.gguf')
MODEL_HASH = '045b7d309c8021d4d7bcebe1156ae94a1e3e128df271d05f5adfcea38ee20b27'
MAIN = 'a7e8ae8790d297ac697956f6f1256d2e2b9403d5'
SOURCES = ['c1_fixtures.py','run_c1.py','analyze_c1.py','test_c1.py','check_c1_inputs.py','c1_probe.cpp',
           'build_c1.ps1','PREREG-C1.md','TEXT-STATE-CONTROL.md','C1-BLIND-RUBRIC.md','inputs/c1-jobs.json']


def digest(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def write_json(path: Path, value: object) -> None:
    with path.open('x', encoding='utf-8', newline='\n') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


def processes() -> list[dict[str, object]]:
    result = subprocess.run(['powershell.exe','-NoProfile','-Command',
        'Get-Process | Select-Object ProcessName,Id | ConvertTo-Json -Compress'],
        capture_output=True, check=True, encoding='utf-8')
    rows: list[dict[str, object]] = json.loads(result.stdout)
    if not rows:
        raise ValueError('Cannot confirm process inventory')
    return rows


def require_quiet() -> list[dict[str, object]]:
    rows = processes()
    blocked = {'runner.listener','runner.worker','llama-server','blender'}
    if any(str(row['ProcessName']).lower() in blocked for row in rows):
        raise ValueError('Quiet window conflict; no process was stopped')
    return rows


def prepare(output: Path) -> None:
    output.mkdir(parents=True, exist_ok=False)
    for folder in ('prompts','continuations','requests'):
        (output / folder).mkdir()
    jobs = make_jobs()
    frozen = json.loads((HERE / 'inputs/c1-jobs.json').read_text(encoding='utf-8'))
    if frozen != jobs or len(jobs) != 1686:
        raise ValueError('Jobs differ from frozen C1 corpus')
    reference = ROOT / 'reports/P0-12/inputs/injection-en.txt'
    if reference.read_text(encoding='utf-8') != render("Repeat exactly, adding nothing: Today's effort matters. Keep going with confidence."):
        raise ValueError('P0-12 nonthinking template differs')
    for job in jobs:
        for folder, text in (('prompts',job['prompt']),('continuations',job['continuation'])):
            (output / folder / (job['id']+'.txt')).write_text(text,encoding='utf-8',newline='\n')
    write_json(output / 'jobs.json',jobs)
    paths = [HERE / name for name in SOURCES] + [ROOT / 'spikes/carrier-engine/native_probe.hpp']
    write_json(output / 'preparation.json',{
        'status':'prepared_not_run','jobs':len(jobs),'template_sha256':digest(reference),
        'jobs_sha256':digest(output / 'jobs.json'),
        'inputs':[{'path':str(path.relative_to(output)).replace('\\','/'),'sha256':digest(path)}
                  for folder in ('prompts','continuations') for path in sorted((output / folder).glob('*.txt'))],
        'sources':[{'path':path.relative_to(ROOT).as_posix(),'sha256':digest(path)} for path in paths]})


def run(output: Path, executable: Path) -> None:
    preparation = json.loads((output / 'preparation.json').read_text(encoding='utf-8'))
    for entry in preparation['sources']:
        if digest(ROOT / entry['path']) != entry['sha256']:
            raise ValueError('Frozen source changed: '+entry['path'])
    for entry in preparation['inputs']:
        if digest(output / entry['path']) != entry['sha256']:
            raise ValueError('Frozen input changed: '+entry['path'])
    if digest(output / 'jobs.json') != preparation['jobs_sha256']:
        raise ValueError('Frozen jobs changed')
    head = subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    remote = subprocess.check_output(['git','ls-remote','origin','refs/heads/task/R-STEER-01'],cwd=ROOT,text=True).split()[0]
    dirty = subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip()
    main = subprocess.check_output(['git','rev-parse','origin/main'],cwd=ROOT,text=True).strip()
    if dirty or head != remote or main != MAIN:
        raise ValueError('Use clean pushed preregistration head and registered main; do not adapt silently')
    background = require_quiet()
    if digest(MODEL) != MODEL_HASH or shutil.disk_usage(output).free < 10_000_000_000:
        raise ValueError('Model hash or evidence reserve mismatch')
    expected = json.loads((ROOT / 'reports/P0-12/continuation-02/engine-check.json').read_text(encoding='utf-8'))
    actual = []
    if sorted(path.name for path in ENGINE.iterdir() if path.is_file()) != sorted(entry['path'] for entry in expected['files']):
        raise ValueError('Engine file set differs')
    for entry in expected['files']:
        sha = digest(ENGINE / entry['path'])
        actual.append({'path':entry['path'],'actual_sha256':sha,'expected_sha256':entry['expected']})
        if sha != entry['expected']:
            raise ValueError('Pinned engine mismatch: '+entry['path'])
    build = json.loads((executable.parent / 'build.json').read_text(encoding='utf-8-sig'))
    if (build['exitCode'] != 0 or digest(executable) != build['executableSha256']
            or digest(HERE / 'c1_probe.cpp') != build['sourceSha256']
            or digest(ROOT / 'spikes/carrier-engine/native_probe.hpp') != build['nativeHeaderSha256']):
        raise ValueError('Build binding mismatch')
    write_json(output / 'binding.json',{'head':head,'remote_head':remote,'main_baseline':MAIN,
        'started_utc':datetime.now(timezone.utc).isoformat(),'engine_files':actual,
        'model_sha256':MODEL_HASH,'executable_sha256':digest(executable),'background_processes':background,
        'quiet_window':'Requires a new user approval for C1; CLI flag alone is not authorization'})
    jobs = json.loads((output / 'jobs.json').read_text(encoding='utf-8'))
    started = time.monotonic()
    completed = 0
    code: int | None = None
    with (output / 'native-stderr.log').open('xb') as stderr, (output / 'protocol.log').open('x',encoding='utf-8') as protocol:
        process = subprocess.Popen([str(executable),str(ENGINE),str(MODEL),str(output),'--quiet-window-confirmed'],
            stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,encoding='utf-8',creationflags=subprocess.CREATE_NO_WINDOW)
        lines: queue.Queue[str] = queue.Queue()

        def read_lines() -> None:
            assert process.stdout is not None
            for line in process.stdout:
                lines.put(line)
            lines.put('EOF')

        def kill_owned() -> None:
            if process.poll() is None:
                subprocess.run(['taskkill.exe','/PID',str(process.pid),'/T','/F'],capture_output=True,
                    check=False,creationflags=subprocess.CREATE_NO_WINDOW)

        timer = threading.Timer(1800,kill_owned)
        timer.start()
        threading.Thread(target=read_lines,daemon=True).start()
        try:
            if lines.get(timeout=60).strip() != 'READY':
                raise RuntimeError('Native startup failed; preserve original log')
            assert process.stdin is not None
            for job in jobs:
                require_quiet()
                mode = 'free' if job['kind'] == 'free' else 'score'
                keep = 'full' if job['kind'] == 'guard' else 'summary'
                process.stdin.write(f"{job['id']}\t{mode}\t{keep}\n")
                process.stdin.flush()
                line = lines.get(timeout=60)
                protocol.write(line)
                protocol.flush()
                if not line.startswith('OK\t'+job['id']+'\t'):
                    raise RuntimeError('Native request failed; no retry or item replacement')
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
            write_json(output / 'exit.json',{'native_exit_code':process.returncode,'completed':completed,
                'expected':len(jobs),'wall_seconds':time.monotonic()-started,'all_requests_completed':code == 0})


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('mode',choices=('prepare','run'))
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--executable',type=Path)
    parser.add_argument('--quiet-window-confirmed',action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    if output == CACHE or not output.is_relative_to(CACHE):
        raise ValueError('Use a new child of task cache on D')
    if args.mode == 'prepare':
        prepare(output)
    elif args.quiet_window_confirmed and args.executable:
        run(output,args.executable.resolve())
    else:
        raise ValueError('Run requires a new quiet window and bound executable')


if __name__ == '__main__':
    main()
