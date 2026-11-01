"""P0-12 items 5, 6, 7, in order; frozen images/prompts and pinned server."""
from __future__ import annotations

import base64
from datetime import datetime, timezone
import json
from pathlib import Path
from urllib.request import urlopen

from jinja2 import Environment

from http_probe import ROOT, Server, quiet, write_json

CACHE = Path('D:/Dev/cache/P0-12')
RUN = CACHE / 'gpu-02'
INPUTS = ROOT / 'reports/P0-12/inputs'


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


def main() -> None:
    deadline = datetime.fromisoformat(json.loads((RUN / 'window.json').read_text())['deadlineUtc'])
    images = json.loads((INPUTS / 'images/manifest.json').read_text(encoding='utf-8'))['images']
    template_path = ROOT / 'reports/P0-12/preparation/model/chat_template.jinja'
    template = Environment().from_string(template_path.read_text(encoding='utf-8'))
    # Same official single-user template rendering as the prepared injection input.
    english = "Repeat exactly, adding nothing: Today's effort matters. Keep going with confidence."
    check = template.render(messages=[dict(role='user', content=english)],
                            add_generation_prompt=True, enable_thinking=False)
    if check != (INPUTS / 'injection-en.txt').read_text(encoding='utf-8'):
        raise ValueError('Official template rendering differs from frozen input')
    output = RUN / 'items-5-7'
    output.mkdir(exist_ok=False)
    write_json(output / 'plan.json', dict(imageCount=len(images), imageGenerationLimit=64,
        imageText='Describe this synthetic image briefly. If there is text, transcribe it.',
        memoryInputs=['red-1024.png', 'geometry-1024.png', 'ja-1024.png'],
        stabilityInputs=['injection-zh.txt', 'injection-en.txt', 'injection-ja.txt'],
        stabilityRepeats=3, stabilityGenerationLimit=64, sampling='same greedy seed 731',
        template=str(template_path), enable_thinking=False,
        measurementOrder=[5, 6, 7], note='No model selection or quality threshold.'))
    with Server(Path('D:/Dev/cache/P0-10/llama.cpp/b11669'), CACHE / 'gguf/carrier-Q4_K_M.gguf',
                output / 'server', visual_carrier=True, mmproj=CACHE / 'gguf/mmproj-F16.gguf') as server:
        with urlopen(server.endpoint + '/v1/models', timeout=180) as reply:
            models = json.load(reply)
        write_json(output / 'models.json', models)
        if not any('multimodal' in item.get('capabilities', []) for item in models['models']):
            raise RuntimeError('Pinned server did not advertise multimodal capability')
        with urlopen(server.endpoint + '/props', timeout=180) as reply:
            props = json.load(reply)
        write_json(output / 'props.json', props)
        marker = props['media_marker']
        prompt = template.render(messages=[dict(role='user', content=marker + '\n' +
            'Describe this synthetic image briefly. If there is text, transcribe it.')],
            add_generation_prompt=True, enable_thinking=False)
        (output / 'actual-image-prompt.txt').write_text(prompt, encoding='utf-8', newline='\n')
        for stage in [5, 6, 7]:
            folder = output / f'item-{stage}'
            folder.mkdir()
            write_json(folder / 'started.json', dict(startedUtc=now()))
            if stage == 5:
                jobs = [(entry['file'], entry['file'], None) for entry in images]
            elif stage == 6:
                jobs = [(name, name, None) for name in ['red-1024.png', 'geometry-1024.png', 'ja-1024.png']]
            else:
                jobs = [(f'{lang}-{repeat}', None, lang) for lang in ['zh', 'en', 'ja'] for repeat in range(1, 4)]
            with (folder / 'samples.jsonl').open('x', encoding='utf-8') as stream:
                for identifier, image_name, lang in jobs:
                    if datetime.now(timezone.utc) >= deadline:
                        raise TimeoutError('Three-hour cumulative GPU limit')
                    quiet()
                    erase = server.post('/slots/0?action=erase', {})
                    if image_name is not None:
                        encoded = base64.b64encode((INPUTS / 'images' / image_name).read_bytes()).decode('ascii')
                        request_prompt = dict(prompt_string=prompt, multimodal_data=[encoded])
                    else:
                        text = (INPUTS / f'injection-{lang}.txt').read_text(encoding='utf-8')
                        request_prompt = server.post('/tokenize', dict(content=text, add_special=False, parse_special=True))['tokens']
                    payload = dict(prompt=request_prompt, n_predict=64, stream=True, return_tokens=True,
                                   temperature=0, seed=731, repeat_penalty=1.0, presence_penalty=0.0,
                                   frequency_penalty=0.0, cache_prompt=False, id_slot=0)
                    stderr = output / 'server/server.stderr.log'
                    start_offset = stderr.stat().st_size
                    started = now()
                    raw = server.completion(payload, folder / 'requests' / identifier)
                    write_json(folder / 'requests' / identifier / 'erase.json', erase)
                    row = dict(id=identifier, image=image_name, language=lang, startedUtc=started, endedUtc=now(),
                               ttftClientMs=raw['ttftClientMs'], completedClientMs=raw['completedClientMs'],
                               timings=raw['final'].get('timings'), content=raw['content'],
                               serverLogStartByte=start_offset, serverLogEndByte=stderr.stat().st_size)
                    stream.write(json.dumps(row, ensure_ascii=False) + '\n')
                    stream.flush()
                    print(f'item-{stage} {identifier} {raw["completedClientMs"]:.1f} ms', flush=True)
            write_json(folder / 'completed.json', dict(endedUtc=now(), samples=len(jobs)))


if __name__ == '__main__':
    main()
