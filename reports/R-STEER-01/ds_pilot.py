"""One-shot, twenty-item official Flash pilot. No full-round or retry option."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from ds_protocol import (HERE, MODEL, SOFT_LIMIT, load, reservation, sha, usage_cost,
                         validate_rating, write)

ENDPOINT='https://api.deepseek.com/chat/completions'


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req: urllib.request.Request, fp: Any, code: int, msg: str,
                         headers: Any, newurl: str) -> None:
        raise urllib.error.HTTPError(req.full_url,code,'Redirect refused',headers,fp)


def event(path: Path, value: dict[str,Any]) -> None:
    value={'utc':datetime.now(timezone.utc).isoformat(),**value}
    with path.open('a',encoding='utf-8',newline='\n') as stream:
        stream.write(json.dumps(value,ensure_ascii=False,separators=(',',':'))+'\n')
        stream.flush()
        os.fsync(stream.fileno())


def check_preparation(session: Path, head: str) -> tuple[dict[str,Any], Path]:
    root=HERE.parents[1]
    actual=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
    remote=subprocess.check_output(['git','ls-remote','origin','refs/heads/task/R-STEER-01'],cwd=root,text=True).split()[0]
    dirty=subprocess.check_output(['git','status','--porcelain'],cwd=root,text=True).strip()
    if actual!=head or remote!=head or dirty:
        raise ValueError('Use the clean pushed registration head')
    plan_path=HERE/'inputs/ds-flash-plan-01.json'
    plan=load(plan_path)
    preparation=Path('D:/Dev/cache/R-STEER-01/ds-flash-preparation-01')
    if sha(plan_path)!=sha(preparation/'plan.json') or plan['model']!=MODEL or plan['pilot_n']!=20 or plan['full_n']!=504:
        raise ValueError('Plan binding differs')
    for entry in plan['sources']:
        if sha(HERE/entry['path'])!=entry['sha256']:
            raise ValueError('Frozen source differs')
    for row in plan['requests']:
        path=preparation/'requests'/(row['id']+'.json')
        if path.stat().st_size!=row['bytes'] or sha(path)!=row['sha256']:
            raise ValueError('Frozen request differs')
    price=load(session/'price-check.json')
    age=(datetime.now(timezone.utc)-datetime.fromisoformat(price['checked_utc'])).total_seconds()
    if not 0<=age<=600 or price['source']!=plan['price_source'] or price['model']!=MODEL or price['version']!=plan['documented_version'] or price['peak_nano_usd_per_token']!=plan['peak_nano_usd_per_token']:
        raise ValueError('Fresh official price/version check required')
    return plan,preparation


def review(run: Path, session: Path) -> None:
    plan=load(HERE/'inputs/ds-flash-plan-01.json')
    items={item['id']:item for item in load(HERE/'results/c1-blind-items-02.json')}
    keys={item['id']:item for item in load(HERE/'results/c1-blind-key-02.json')}
    lines=['# DeepSeek Flash 20条小样抽查','',
           '只供用户判断评分是否可信；尚未进入全量，不能改变C1自动门失败。',
           '原输出未经修订；完整响应、请求及计费usage在同目录。若不可信，停下，不改口径重试。','']
    for number,identifier in enumerate(plan['pilot_ids'],1):
        item=items[identifier]
        key=keys[identifier]
        path=run/identifier/'rating.json'
        lines.extend([f"## {number}. {identifier}（{key['lang']} / {key['kind']}）",'',
                      '原输出：','```text',item['output'],'```',''])
        if not path.exists():
            lines.extend(['评分：not_run 或本项失败，见原始状态。',''])
            continue
        rating=load(path)
        labels='；'.join(f"{k}: {v['label']}〔{v['quote']}〕" for k,v in rating['concepts'].items())
        issues=[k for group in ('coherence','flags') for k,v in rating[group].items() if v['label']!='no']
        lines.extend(['评分：',f"- 四概念：{labels}",f"- 事实：{rating['facts']['label']}；{rating['facts']['reason']}",
                      f"- 一般情感：{rating['valence']['label']}",f"- 问题/标记：{', '.join(issues) or '无'}",'',
                      f"[完整评分]({identifier}/rating.json)",''])
    (run/'REVIEW.md').write_text('\n'.join(lines)+'\n',encoding='utf-8',newline='\n')


def pilot(session: Path, expected_head: str) -> int:
    plan,preparation=check_preparation(session,expected_head)
    key=os.environ.get('DEEPSEEK_API_KEY')
    if not key:
        raise ValueError('Process credential missing')
    run=session/'pilot-run'
    run.mkdir(exist_ok=False)
    write(run/'binding.json',{'head':expected_head,'model':MODEL,'phase':'pilot_only',
        'documented_version':plan['documented_version'],'pilot_ids':plan['pilot_ids'],
        'plan_sha256':sha(HERE/'inputs/ds-flash-plan-01.json'),
        'started_utc':datetime.now(timezone.utc).isoformat(),'key_source':'process environment only'})
    opener=urllib.request.build_opener(urllib.request.ProxyHandler({}),NoRedirect())
    total=0
    attempts=0
    completed=0
    status='pilot_stopped'
    actual_model: str|None=None
    ledger=run/'ledger.jsonl'
    start=time.monotonic()
    try:
        for identifier in plan['pilot_ids']:
            if attempts>=20 or time.monotonic()-start>900:
                raise ValueError('Pilot request/time limit')
            body=(preparation/'requests'/(identifier+'.json')).read_bytes()
            held=reservation(body)
            if total+held>=SOFT_LIMIT:
                status='budget_stop'
                break
            folder=run/identifier
            folder.mkdir()
            (folder/'request.json').write_bytes(body)
            attempts+=1
            total+=held
            event(ledger,{'event':'reserve_before_post','id':identifier,'ordinal':attempts,
                'reserved_nano_usd':held,'cumulative_upper_nano_usd':total,'request_sha256':hashlib.sha256(body).hexdigest()})
            request=urllib.request.Request(ENDPOINT,data=body,headers={'Authorization':'Bearer '+key,
                'Content-Type':'application/json','User-Agent':'R-STEER-01-frozen-pilot'},method='POST')
            try:
                with opener.open(request,timeout=90) as response:
                    raw=response.read(2_000_001)
                    response_status=response.status
            except urllib.error.HTTPError as error:
                raw=error.read(2_000_001)
                if key.encode() not in raw:
                    (folder/'http-error.body').write_bytes(raw)
                event(ledger,{'event':'http_error_no_retry','id':identifier,'http_status':error.code})
                raise ValueError('HTTP request failed; reservation retained') from None
            if len(raw)>2_000_000 or key.encode() in raw:
                raise ValueError('Unsafe/oversized response; raw not persisted')
            (folder/'response.json').write_bytes(raw)
            value=json.loads(raw)
            cost=usage_cost(value['usage'])
            total=total-held+cost
            event(ledger,{'event':'settle_from_usage_peak_upper','id':identifier,'http_status':response_status,
                'usage':value['usage'],'upper_nano_usd':cost,'cumulative_upper_nano_usd':total,
                'response_model':value.get('model'),'system_fingerprint':value.get('system_fingerprint'),
                'response_id':value.get('id'),'response_created':value.get('created')})
            if cost>held:
                raise ValueError('Usage exceeded reservation; settled then stopped')
            if value.get('model') not in (MODEL,'DeepSeek-V4.1-Flash-1017') or (actual_model is not None and value['model']!=actual_model):
                raise ValueError('Response model changed/unexpected')
            actual_model=value['model']
            choice=value['choices']
            if len(choice)!=1 or choice[0]['finish_reason']!='stop' or choice[0]['message'].get('tool_calls') or choice[0]['message'].get('reasoning_content'):
                raise ValueError('Unexpected completion mode/termination')
            content=choice[0]['message']['content']
            if not isinstance(content,str) or not content.strip():
                raise ValueError('Blank JSON output; no repair/retry')
            item=load(folder/'request.json')['messages'][1]['content']
            presented=json.loads(item)
            rating=validate_rating(json.loads(content),presented['待评输出'],presented['评分事实参照'])
            write(folder/'rating.json',rating)
            completed+=1
        if completed==20:
            status='pilot_complete_awaiting_user_trust'
    except Exception as error:
        # Class only, never exception text, headers, environment, or credentials.
        event(ledger,{'event':'stop_no_retry','error_class':type(error).__name__,'completed':completed})
    finally:
        write(run/'result.json',{'status':status,'post_attempts':attempts,'completed_ratings':completed,
            'full_round_post_attempts':0,'cumulative_upper_nano_usd':total,
            'upper_usd':f'{total/1_000_000_000:.9f}','fee_method':'actual usage at official peak rates; not an account invoice',
            'ended_utc':datetime.now(timezone.utc).isoformat(),'response_model':actual_model})
        review(run,session)
        os.environ.pop('DEEPSEEK_API_KEY',None)
    return 0 if completed==20 else 2


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--session',required=True,type=Path)
    parser.add_argument('--expected-head',required=True)
    args=parser.parse_args()
    try:
        raise SystemExit(pilot(args.session.resolve(),args.expected_head))
    except Exception:
        # The process supervisor also records a non-secret exit status.
        print('pilot preflight stopped; no automatic retry',flush=True)
        raise SystemExit(2) from None
