"""Freeze all 504 payloads and the 20-item pilot; never reads a key or calls API."""
from __future__ import annotations

import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path

from ds_protocol import HERE, MODEL, choose_pilot, load, payload, reservation, sha, write

SOURCES=('ds_protocol.py','ds_flash_prepare.py','ds_pilot.py','ds_judge.py','ds_key_entry.ps1',
         'test_ds_judge.py','DS-RUBRIC-01.txt','DS-FLASH-01.md','C1-BLIND-RUBRIC.md','SEMANTIC-RUBRIC.md',
         'results/c1-blind-items-02.json','results/c1-blind-key-02.json')


def prepare(output: Path) -> None:
    output.mkdir(parents=True,exist_ok=False)
    (output/'requests').mkdir()
    items=load(HERE/'results/c1-blind-items-02.json')
    keys=load(HERE/'results/c1-blind-key-02.json')
    if len(items)!=504 or len(keys)!=504:
        raise ValueError('Frozen packet size differs')
    pilot=choose_pilot(items,keys)
    keymap={key['id']:key for key in keys}
    requests=[]
    for item in items:
        body=json.dumps(payload(item),ensure_ascii=False,separators=(',',':')).encode('utf-8')
        if len(body)>16_000:  # Engineering request bound, not a semantic criterion.
            raise ValueError('Request exceeds registered byte bound')
        (output/'requests'/(item['id']+'.json')).write_bytes(body)
        requests.append({'id':item['id'],'sha256':hashlib.sha256(body).hexdigest(),
                         'bytes':len(body),'reservation_nano_usd':reservation(body)})
    plan={'status':'prepared_not_run','model':MODEL,'documented_version':'DeepSeek-V4.1-Flash-1017',
        'main_baseline':'a7e8ae8790d297ac697956f6f1256d2e2b9403d5',
        'phase':'pilot_first_user_review_required','pilot_ids':pilot,'full_ids':[item['id'] for item in items],
        'pilot_n':20,'full_n':504,'overall_max_post_calls':524,'fee_hard_limit_usd':'5.00',
        'fee_soft_stop_usd':'4.75','peak_nano_usd_per_token':{'hit':6,'miss':300,'output':1200},
        'price_source':'https://api-docs.deepseek.com/quick_start/pricing/',
        'requests':requests,'sources':[{'path':name,'sha256':sha(HERE/name)} for name in SOURCES],
        'pilot_strata':dict(Counter(keymap[i]['kind'] for i in pilot)),
        'pilot_languages':dict(Counter(keymap[i]['lang'] for i in pilot)),
        'selection':'frozen blind order after deterministic metadata strata; includes known truncation c1-1234',
        'pilot_ratings_used_in_full_gates':False}
    write(output/'plan.json',plan)
    write(output/'pilot-index.json',[keymap[i] for i in pilot])


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    prepare(args.output)
