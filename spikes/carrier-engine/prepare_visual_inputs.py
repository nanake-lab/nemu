"""CPU-only P0-12 fixtures: original P0-10 messages, official nonthinking template."""
from __future__ import annotations

import gzip
import hashlib
import json
from pathlib import Path
from typing import Any, Callable, cast

from transformers import AutoTokenizer

from benchmark_plan import FIXED_SENTENCE, SYSTEM

ROOT = Path(__file__).resolve().parents[2]
CACHE = Path('D:/Dev/cache/P0-12')
REVISION = '851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a'


def main() -> None:
    output = ROOT / 'reports/P0-12/inputs'
    output.mkdir(exist_ok=False)
    model = CACHE / 'models/Qwen--Qwen3.5-4B' / REVISION
    tokenizer = cast(Callable[..., Any], AutoTokenizer.from_pretrained)(
        model, local_files_only=True, trust_remote_code=False)
    source = ROOT / 'spikes/codex-app-server'
    fixtures = json.loads((source / 'fixtures/irl-cases.json').read_text(encoding='utf-8'))['cases']
    wire = json.loads((source / 'semantic-candidate.wire.schema.json').read_text(encoding='utf-8'))
    if len(fixtures) != 200 or not all(sample['synthetic'] for sample in fixtures):
        raise ValueError('Original 200 synthetic fixtures required')
    jobs = []
    for index, arm in enumerate(['speech', 'unconstrained', 'wire', 'wire']):
        jobs.append((f'warmup-{index + 1}', arm, fixtures[0], True))
    jobs.extend((f'speech-{i + 1:02d}', 'speech', None, False) for i in range(20))
    for index, sample in enumerate(fixtures):
        arms = ['unconstrained', 'wire'] if index < 50 else ['wire']
        shift = index % len(arms)
        jobs.extend((sample['id'] + '-' + arm, arm, sample, False) for arm in arms[shift:] + arms[:shift])
    records = []
    for identifier, arm, sample, warmup in jobs:
        if arm == 'speech':
            messages = [{'role': 'user', 'content': '请逐字输出下面这句话，不添加其他内容：' + FIXED_SENTENCE}]
        else:
            prompt = ('Convert this single synthetic user utterance into an experimental '
                      'SemanticCandidate JSON object using the supplied schema. '
                      'Preserve only what the sentence says. Use sourceRef '
                      f"{sample['sourceRef']}; sourceSpan is a half-open range of Unicode code "
                      'points in the utterance. Do not fill missing facts. '
                      f"Language={sample['lang']}; utterance={json.dumps(sample['text'], ensure_ascii=False)}")
            messages = [{'role': 'system', 'content': SYSTEM + ' JSON schema: ' + json.dumps(wire)},
                        {'role': 'user', 'content': prompt}]
        text = tokenizer.apply_chat_template(messages, tokenize=False, add_generation_prompt=True,
                                             enable_thinking=False)
        ids = tokenizer(text, add_special_tokens=True)['input_ids']
        maximum = 64 if arm == 'speech' else 2048
        if len(ids) + maximum > 8192:
            raise ValueError('Input would exceed registered context')
        records.append(dict(id=identifier, arm=arm, sample=sample, warmup=warmup, messages=messages,
                            prompt=text, inputTokenIds=ids, maxNewTokens=maximum,
                            promptSha256=hashlib.sha256(text.encode()).hexdigest()))
    with gzip.open(output / 'jobs.json.gz', 'wt', encoding='utf-8', newline='\n') as stream:
        json.dump(records, stream, ensure_ascii=False)
    (output / 'schemas.json').write_text(json.dumps({'wire': wire}, ensure_ascii=False, indent=2), encoding='utf-8')
    prompts = {
        'zh': '请逐字输出下面这句话，不添加其他内容：' + FIXED_SENTENCE,
        'en': "Repeat exactly, adding nothing: Today's effort matters. Keep going with confidence.",
        'ja': '次の文だけをそのまま出力してください：今日の努力には価値があります。自信を持って進みましょう。',
    }
    for lang, prompt in prompts.items():
        rendered = tokenizer.apply_chat_template([{'role': 'user', 'content': prompt}], tokenize=False,
                                                 add_generation_prompt=True, enable_thinking=False)
        (output / f'injection-{lang}.txt').write_text(rendered, encoding='utf-8', newline='\n')
    plan = dict(revision=REVISION, jobs=len(records), formal=dict(speech=20, unconstrained=50, wire=200),
                warmups=4, context=8192, threads=4, batch=512, ubatch=512, seed=731,
                sampling='greedy', flashAttention='off', enable_thinking=False,
                fourthWarmup='Original compact warmup replaced by original wire, per user prohibition.',
                actualTemplate=str(model / 'chat_template.jinja'),
                templateSha256=hashlib.sha256((model / 'chat_template.jinja').read_bytes()).hexdigest(),
                layerMapping='round(17/36 * 32) = 15 (zero based)',
                bindings=[dict(path=str(path.relative_to(ROOT)), sha256=hashlib.sha256(path.read_bytes()).hexdigest())
                          for path in [source / 'fixtures/irl-cases.json', source / 'semantic-candidate.wire.schema.json']])
    (output / 'plan.json').write_text(json.dumps(plan, ensure_ascii=False, indent=2), encoding='utf-8')
    print(json.dumps(plan, ensure_ascii=False))


if __name__ == '__main__':
    main()
