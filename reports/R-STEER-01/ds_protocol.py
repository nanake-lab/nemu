"""Frozen Flash rating format and deterministic pilot selection; no network."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

HERE=Path(__file__).resolve().parent
MODEL='deepseek-flash'
CONCEPTS=('connection','empathy','curiosity','achievement')
POLARITY=('positive','negative','both','neutral','absent','uncertain')
COHERENCE=('unclear_reference','contradiction','repetition_break','off_topic','truncated')
FLAGS=('politeness','customer_service','self_blame_apology','unsolicited_identity',
       'task_asks_feeling','state_meta_report','stable_trait','extreme_claim',
       'action_promise','invented_relationship_experience','method_inferable')
MAX_OUTPUT=2048
# Official peak rates: USD .30/M miss, .006/M hit, 1.20/M output.
# Integer nano-USD avoids rounding a reservation below the actual bound.
MISS_RATE=300
HIT_RATE=6
OUTPUT_RATE=1200
HARD_LIMIT=5_000_000_000
SOFT_LIMIT=4_750_000_000  # Stop before 95% of the user-authorized $5 ceiling.


def sha(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream,'sha256').hexdigest()


def load(path: Path) -> Any:
    return json.loads(path.read_text(encoding='utf-8-sig'))


def write(path: Path, value: object) -> None:
    with path.open('x',encoding='utf-8',newline='\n') as stream:
        json.dump(value,stream,ensure_ascii=False,indent=2)
        stream.write('\n')


def choose_pilot(items: list[dict[str,Any]], keys: list[dict[str,Any]]) -> list[str]:
    candidates=[key for key in keys if key['repeat']==0]
    def first(rows: list[dict[str,Any]]) -> str:
        return str(min(rows,key=lambda row:hashlib.sha256(('DS-FLASH-PILOT-01:'+row['id']).encode()).hexdigest())['id'])
    selected=[]
    # Eight target items: all four concepts, both signs, all four forms; language 3/3/2.
    for index,(concept,sign) in enumerate((c,s) for c in sorted(CONCEPTS) for s in (1,-1)):
        lang=('zh','en','ja')[index%3]
        form=('base','paraphrase','long','courtesy')[index%4]
        selected.append(first([k for k in candidates if k['kind']=='target' and k['concept']==concept
                               and k['lang']==lang and k['form']==form and k['expected_sign']==sign]))
    for index,lang in enumerate(('zh','en','ja')):
        selected.append(first([k for k in candidates if k['kind']=='free' and k['lang']==lang and k['level']==0 and k['order']==0]))
        # A known 64-token truncated output is an explicit diagnostic stratum, not random sampling.
        selected.append(first([k for k in candidates if k['kind']=='free' and k['lang']==lang
                               and (k['job_id']=='c1-1234' if lang=='ja' else k['level']==(-1 if index==0 else 1) and k['order']==1)]))
        selected.append(first([k for k in candidates if k['kind']=='fact' and k['lang']==lang and k['expected_sign']==(1 if index==0 else -1)]))
        selected.append(first([k for k in candidates if k['kind']=='generic' and k['lang']==lang and k['expected_sign']==(-1 if index==1 else 1)]))
    if len(selected)!=20 or len(set(selected))!=20 or not set(selected).issubset({item['id'] for item in items}):
        raise ValueError('Pilot selection differs')
    # Keep the already frozen blind presentation order, not the stratification order.
    return [item['id'] for item in items if item['id'] in selected]


def reference(text: str) -> str:
    prefix='<|im_start|>user\n'
    suffix='<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n'
    if not text.startswith(prefix) or not text.endswith(suffix):
        raise ValueError('Unexpected saved carrier wrapper')
    return text[len(prefix):-len(suffix)]


def example(order: list[str]) -> dict[str,Any]:
    mark={'label':'no','quote':''}
    return {'concepts':{key:{'label':'absent','quote':'','object':''} for key in order},
        'valence':{'label':'neutral','quote':''},'facts':{'label':'faithful','quote':'','reason':''},
        'coherence':{key:mark.copy() for key in COHERENCE},
        'flags':{key:mark.copy() for key in FLAGS},'method_guess':{'guess':'unknown','quote':''}}


def payload(item: dict[str,Any]) -> dict[str,Any]:
    rubric=(HERE/'DS-RUBRIC-01.txt').read_text(encoding='utf-8')
    rubric+='\njson格式示例（字段完整；示例标签不是本题答案）：\n'+json.dumps(example(item['concept_order']),ensure_ascii=False,separators=(',',':'))
    content={'待评输出':item['output'],'评分事实参照':reference(item['input'])}
    return {'model':MODEL,'messages':[{'role':'system','content':rubric},
        {'role':'user','content':json.dumps(content,ensure_ascii=False,separators=(',',':'))}],
        'thinking':{'type':'disabled'},'temperature':0,'top_p':1,'max_tokens':MAX_OUTPUT,
        'response_format':{'type':'json_object'},'stream':False}


def reservation(body: bytes) -> int:
    # UTF-8 bytes are a conservative prompt-token bound; 1024 covers chat framing.
    return (len(body)+1024)*MISS_RATE+MAX_OUTPUT*OUTPUT_RATE


def usage_cost(usage: dict[str,Any]) -> int:
    names=('prompt_tokens','completion_tokens','prompt_cache_hit_tokens','prompt_cache_miss_tokens','total_tokens')
    if any(type(usage.get(key)) is not int or usage[key]<0 for key in names):
        raise ValueError('Missing/noninteger usage')
    if usage['prompt_tokens']!=usage['prompt_cache_hit_tokens']+usage['prompt_cache_miss_tokens'] or usage['total_tokens']!=usage['prompt_tokens']+usage['completion_tokens'] or usage['completion_tokens']>MAX_OUTPUT:
        raise ValueError('Usage totals differ')
    return int(usage['prompt_cache_hit_tokens']*HIT_RATE+usage['prompt_cache_miss_tokens']*MISS_RATE+usage['completion_tokens']*OUTPUT_RATE)


def validate_rating(value: object, text: str, context: str) -> dict[str,Any]:
    if not isinstance(value,dict) or set(value)!={'concepts','valence','facts','coherence','flags','method_guess'}:
        raise ValueError('Invalid rating fields')
    def annotation(row: object, labels: tuple[str,...], allowed: str, required: bool=False) -> None:
        if not isinstance(row,dict) or not isinstance(row.get('quote'),str) or row.get('label') not in labels:
            raise ValueError('Invalid annotation')
        if len(row['quote'])>400 or (row['quote'] and row['quote'] not in allowed) or (required and not row['quote']):
            raise ValueError('Evidence is absent from allowed text')
    concepts=value['concepts']
    if not isinstance(concepts,dict) or set(concepts)!=set(CONCEPTS):
        raise ValueError('Invalid concept set')
    for row in concepts.values():
        annotation(row,POLARITY,text,row.get('label') in ('positive','negative','both','neutral'))
        if not isinstance(row.get('object'),str) or len(row['object'])>160:
            raise ValueError('Invalid concept object')
    annotation(value['valence'],('positive','negative','mixed','neutral','uncertain'),text)
    annotation(value['facts'],('faithful','unsupported','contradicted','uncertain'),text)
    if not isinstance(value['facts'].get('reason'),str) or len(value['facts']['reason'])>300:
        raise ValueError('Invalid factual explanation')
    for key,names in (('coherence',COHERENCE),('flags',FLAGS)):
        if not isinstance(value[key],dict) or set(value[key])!=set(names):
            raise ValueError('Invalid issue set')
        for name,row in value[key].items():
            annotation(row,('yes','no','uncertain'),context if name=='task_asks_feeling' else text,row.get('label')=='yes')
    guess=value['method_guess']
    if not isinstance(guess,dict) or not isinstance(guess.get('guess'),str) or not isinstance(guess.get('quote'),str) or len(guess['guess'])>160 or (guess['quote'] and guess['quote'] not in text):
        raise ValueError('Invalid blinded guess')
    return value
