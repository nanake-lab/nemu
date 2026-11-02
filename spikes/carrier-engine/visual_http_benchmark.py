"""P0-12 item 4: original fixed text inputs, explicit hybrid slot erase."""
from __future__ import annotations

from datetime import datetime, timezone
import gzip
import json
from pathlib import Path

from http_probe import ROOT, Server, quiet, write_json

CACHE = Path('D:/Dev/cache/P0-12')


def main() -> None:
    window = json.loads((CACHE / 'gpu-02/window.json').read_text(encoding='utf-8'))
    deadline = datetime.fromisoformat(window['deadlineUtc'])
    output = CACHE / 'gpu-02/item-4'
    output.mkdir(exist_ok=False)
    prepared = ROOT / 'reports/P0-12/inputs'
    with gzip.open(prepared / 'jobs.json.gz', 'rt', encoding='utf-8') as stream:
        jobs = json.load(stream)
    schemas = json.loads((prepared / 'schemas.json').read_text(encoding='utf-8'))
    with Server(Path('D:/Dev/cache/P0-10/llama.cpp/b11669'), CACHE / 'gguf/carrier-Q4_K_M.gguf',
                output / 'server', visual_carrier=True) as server:
        native = server.post('/tokenize', {'content': jobs[0]['prompt'], 'add_special': False, 'parse_special': True})
        write_json(output / 'tokenizer.json', dict(native=native, prepared=jobs[0]['inputTokenIds'],
                                                 equal=native.get('tokens') == jobs[0]['inputTokenIds']))
        with (output / 'samples.jsonl').open('x', encoding='utf-8') as stream:
            for job in jobs:
                if datetime.now(timezone.utc) >= deadline:
                    raise TimeoutError('Three-hour GPU deadline')
                quiet()
                erase = server.post('/slots/0?action=erase', {})
                payload = dict(prompt=job['inputTokenIds'], n_predict=job['maxNewTokens'], stream=True,
                               return_tokens=True, temperature=0, seed=731, repeat_penalty=1.0,
                               presence_penalty=0.0, frequency_penalty=0.0, cache_prompt=False, id_slot=0)
                if job['arm'] == 'wire':
                    payload['json_schema'] = schemas['wire']
                raw = server.completion(payload, output / 'requests' / job['id'])
                write_json(output / 'requests' / job['id'] / 'erase.json', erase)
                row = dict(id=job['id'], arm=job['arm'], warmup=job['warmup'],
                           lang=job['sample']['lang'] if job['sample'] else 'zh',
                           ttftClientMs=raw['ttftClientMs'], completedClientMs=raw['completedClientMs'],
                           timings=raw['final'].get('timings'), content=raw['content'])
                stream.write(json.dumps(row, ensure_ascii=False) + '\n')
                stream.flush()
                print(f"{job['id']} {raw['completedClientMs']:.1f} ms", flush=True)
    write_json(output / 'completed.json', dict(endedUtc=datetime.now(timezone.utc).isoformat(), jobs=len(jobs)))


if __name__ == '__main__':
    main()
