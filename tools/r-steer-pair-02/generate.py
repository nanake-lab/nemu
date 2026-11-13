"""Fixed local paired generation; no network or scoring. Run only in approved window."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import queue
import re
import subprocess
import sys
import threading
import time
from resources import snapshot

ROOT = Path('D:/Dev/cache/R-STEER-PAIR-02')
REPO = Path(__file__).resolve().parents[2]
ENGINE = Path('D:/Dev/cache/P0-10/llama.cpp/b11669')
MODEL = Path('D:/Dev/cache/P0-12/gguf/carrier-Q4_K_M.gguf')
EXE = ROOT/'build/pair_probe.exe'
SOURCE = Path('D:/Dev/nemu-evidence-wt/R-STEER-01-J-v3/R-STEER-01/c3-2a-01')

def sha(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')

def prepare() -> None:
    prereg = REPO/'reports/R-STEER-PAIR-02/PREREGISTRATION.md'
    text = prereg.read_text(encoding='utf-8-sig')
    material = text.split('共22对44条')[0]
    rows = re.findall(r'^(\d+)\. (.+)$', material, re.M)
    assert [int(i) for i, _ in rows] == list(range(1, 23))
    intro = '你叫 Nemu。下面是一个跟你很熟的人对你说的话，自然地回他一句。'
    jobs = []
    for number, opening in rows:
        ident = f'{int(number):02d}'
        prompt = '<|im_start|>user\n'+intro+'\n'+opening+'<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n'
        (ROOT/f'inputs/prompts/p{ident}.txt').write_bytes(prompt.encode('utf-8'))
        for condition in ('zero', 'p08'):
            jobs.append({'id':f'p{ident}-{condition}', 'material_id':f'p{ident}', 'mode':'free',
                         'layer':15 if condition == 'p08' else -1,
                         'vector':'connection-l15-p08' if condition == 'p08' else 'none',
                         'seed':20261113, 'seed_used':False})
    jobs.sort(key=lambda row: hashlib.sha256(('R-STEER-PAIR-02:generate:'+row['material_id'][1:]+':'+row['id'].split('-')[1]).encode()).hexdigest())
    guard = {'material_id':'p01', 'mode':'guard', 'layer':-1, 'vector':'none'}
    write(ROOT/'inputs/jobs.json', [{'id':'guard-start', **guard}]+jobs+[{'id':'guard-end', **guard}])
    assert sha(MODEL) == '045b7d309c8021d4d7bcebe1156ae94a1e3e128df271d05f5adfcea38ee20b27'
    assert sha(SOURCE/'code/c3_2a_probe.cpp') == 'f57cf6d85a415de4defde9430266e1bd05af23e06b8a7d16c224ca2973676117'
    expected = (SOURCE/'code/c3_2a_probe.cpp').read_text(encoding='utf-8').replace('mode=="free"?96:1','mode=="free"?384:1')
    assert (REPO/'tools/r-steer-pair-02/probe.cpp').read_text(encoding='utf-8-sig').strip() == expected.strip()
    assert sha(ROOT/'inputs/vectors/connection-l15-p08.f32') == sha(SOURCE/'inputs/vectors/connection-l15-p08.f32')
    inventory = json.loads(Path('D:/Dev/nemu-wt/R-STEER-01/reports/P0-12/continuation-02/engine-check.json').read_text())
    for row in inventory['files']:
        assert sha(ENGINE/row['path']) == row['expected']
    files = list((ROOT/'inputs').rglob('*'))+list((REPO/'tools/r-steer-pair-02').glob('*'))+[EXE, MODEL, prereg]
    write(ROOT/'execution-manifest.json', {'files':[{'path':str(p), 'sha256':sha(p)} for p in files if p.is_file()],
          'source_commit':'a8c57eb35cf5e823f8ee0a650efc8c3c707055e4', 'expected_answers':44, 'guards':2,
          'sampling':'greedy', 'seed':20261113, 'seed_used':False, 'max_new_tokens':384,
          'model_api_calls':0, 'engine_files':inventory['files']})
    print('Prepared 44 requests and 2 guards; source, model and engine hashes verified.')

def run() -> None:
    manifest = json.loads((ROOT/'execution-manifest.json').read_text())
    for row in manifest['files']:
        assert sha(Path(row['path'])) == row['sha256']
    state = snapshot()
    assert not state['heavy_load_categories'], 'Resource conflict'
    out = ROOT/'generation-01'
    out.mkdir(exist_ok=False)
    (out/'requests').mkdir()
    write(out/'preflight.json', state)
    commands = json.loads((ROOT/'inputs/jobs.json').read_text())
    began = time.monotonic()
    stop = threading.Event()
    errors: list[str] = []
    completed = 0
    process: subprocess.Popen[str] | None = None
    def kill_owned() -> None:
        if process is not None and process.poll() is None:
            process.kill()
    def monitor() -> None:
        try:
            while not stop.is_set():
                value = snapshot(process.pid if process else None)
                with (out/'resources.jsonl').open('a', encoding='utf-8') as stream:
                    stream.write(json.dumps(value)+'\n')
                if value['heavy_load_categories'] or time.monotonic()-began >= 870:
                    errors.append('resource_conflict_or_deadline')
                    kill_owned()
                    return
                stop.wait(5)
        except Exception as error:
            errors.append(type(error).__name__)
            kill_owned()
    timer = threading.Timer(870, kill_owned)
    success = False
    with (out/'stderr.log').open('x', encoding='utf-8') as stderr:
        try:
            process = subprocess.Popen([str(EXE),str(ENGINE),str(MODEL),str(ROOT/'inputs'),str(out),'--quiet-window-confirmed'],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, encoding='utf-8', creationflags=subprocess.CREATE_NO_WINDOW)
            lines: queue.Queue[str] = queue.Queue()
            def reader() -> None:
                assert process is not None and process.stdout is not None
                for line in process.stdout:
                    lines.put(line)
                lines.put('EOF')
            threading.Thread(target=reader, daemon=True).start()
            watcher = threading.Thread(target=monitor, daemon=True)
            watcher.start()
            timer.start()
            assert lines.get(timeout=90).strip() == 'READY', 'Startup failed'
            assert process.stdin is not None
            for row in commands:
                assert not errors
                process.stdin.write('\t'.join(str(row[k]) for k in ('id','material_id','mode','layer','vector'))+'\n')
                process.stdin.flush()
                line = lines.get(timeout=min(120, max(1,870-(time.monotonic()-began))))
                assert line.startswith('OK\t'+row['id']+'\t'), 'Request failed'
                completed += 1
                completion = (out/'requests'/row['id']/'completion.tsv').read_text().splitlines()[1].split('\t')
                if row['mode'] == 'free':
                    assert completion[1] == '1', 'Output cap reached; stop without retry'
                    (out/'requests'/row['id']/'output.bin').read_bytes().decode('utf-8', errors='strict')
                print(f'Completed {completed}/46', flush=True)
            process.stdin.write('QUIT\n')
            process.stdin.flush()
            assert process.wait(timeout=20) == 0 and not errors
            first, last = out/'requests/guard-start', out/'requests/guard-end'
            for name in ('first-logits.f32','prompt-ids.tsv'):
                assert (first/name).read_bytes() == (last/name).read_bytes(), 'Zero drift'
            assert (first/'native-zero-logits.f32').read_bytes() == (first/'first-logits.f32').read_bytes()
            success = True
        except Exception as error:
            errors.append(str(error) or type(error).__name__)
        finally:
            kill_owned()
            if process:
                process.wait(timeout=20)
            stop.set()
            timer.cancel()
            if 'watcher' in locals():
                watcher.join(timeout=25)
            result = {'status':'completed' if success else 'technical_stop', 'completed':completed,
                      'wall_seconds':time.monotonic()-began, 'errors':errors, 'api_calls':0, 'analysis':'not_run'}
            write(out/'exit.json', result)
            print(json.dumps(result), flush=True)
    if not success:
        raise SystemExit(1)

if __name__ == '__main__':
    if sys.argv[1:] == ['prepare']:
        prepare()
    elif sys.argv[1:] == ['run', '--user-approved']:
        run()
    else:
        raise SystemExit('Use prepare or run --user-approved only after actual user approval')
