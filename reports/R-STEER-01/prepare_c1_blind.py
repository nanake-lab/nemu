"""Materialize the preregistered blind packet locally; never scores or calls an API."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import TypedDict

from c1_fixtures import make_jobs, prompt, scenes


class BlindItem(TypedDict):
    id: str
    input: str
    output: str
    concept_order: list[str]


def digest(text: str) -> str:
    return hashlib.sha256(('R-STEER-01-C1-blind-v1:'+text).encode('utf-8')).hexdigest()


def prepare(run: Path, destination: Path) -> None:
    jobs = make_jobs()
    frozen = json.loads((run/'jobs.json').read_text(encoding='utf-8'))
    if jobs != frozen:
        raise ValueError('Input corpus differs')
    destination.mkdir(parents=True,exist_ok=False)
    items: list[BlindItem] = []
    keys: list[dict[str, object]] = []
    concept_order = ['connection','empathy','curiosity','achievement']
    for scene in scenes():
        selected = [job for job in jobs if job['root']==scene['root'] and job['lang']==scene['lang'] and (
            (job['kind']=='target' and job['level']==0 and job['order']==0)
            or (job['kind'] in ('generic','fact') and job['level']==1)
            or job['kind']=='free')]
        repeats = {next(job['id'] for job in selected if job['kind']=='target' and job['form']=='base' and job['sign']==sign)
                   for sign in (1,-1)}
        repeats.add(next(job['id'] for job in selected if job['kind']=='free'))
        for job in selected:
            text = ((run/'requests'/job['id']/'output.bin').read_bytes().decode('utf-8')
                    if job['kind']=='free' else job['continuation'])
            context = job['prompt'] if job['kind'] in ('free','fact') else prompt(scene,0,0)
            for repeat in range(2 if job['id'] in repeats else 1):
                identifier = digest(job['id']+f':{repeat}')[:24]
                items.append(BlindItem(id=identifier,input=context,output=text,
                    concept_order=concept_order[::-1] if repeat else concept_order.copy()))
                keys.append({'id':identifier,'job_id':job['id'],'repeat':repeat,'root':job['root'],
                    'lang':job['lang'],'kind':job['kind'],'concept':job['concept'],
                    'expected_sign':job['sign'],'level':job['level'],'form':job['form'],'order':job['order']})
    if len(items)!=504 or sum(int(item['repeat']==0) for item in keys)!=432:
        raise ValueError('Blind packet size differs from preregistration')
    items.sort(key=lambda item:digest('display:'+item['id']))
    for name,value in (('blind-items.json',items),('mapping-with-expected.json',keys)):
        with (destination/name).open('x',encoding='utf-8',newline='\n') as stream:
            json.dump(value,stream,ensure_ascii=False,indent=2)
            stream.write('\n')
    (destination/'README.txt').write_text(
        'UNSCORED LOCAL PACKET. No external API or human rating performed.\n'
        'Give raters only blind-items.json and the frozen C1 rubric.\n'
        'Do not give raters mapping-with-expected.json, model metadata, or LL results.\n'
        '432 unique items + 72 reordered repeats = 504 presentations.\n'
        'input preserves the carrier template; remove no content after inspecting output.\n'
        'A public mapping cannot guarantee blinding if a rater reads it.\n',encoding='utf-8',newline='\n')


if __name__ == '__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--run',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    prepare(args.run,args.output)
