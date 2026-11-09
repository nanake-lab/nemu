"""One high-thinking pass over the same twenty prompts, retaining the prior budget."""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from typing import Any

from ds_high_prepare import MAX_TOKENS, SESSION, high_request, reserve
from ds_pilot import ENDPOINT, NoRedirect, event
from ds_protocol import HERE, MODEL, SOFT_LIMIT, load, sha, validate_rating, write


def high_cost(usage: dict[str, Any]) -> int:
    names = ('prompt_tokens', 'completion_tokens', 'prompt_cache_hit_tokens',
             'prompt_cache_miss_tokens', 'total_tokens')
    if any(type(usage.get(k)) is not int or usage[k] < 0 for k in names):
        raise ValueError('Missing/noninteger usage')
    if (usage['prompt_tokens'] != usage['prompt_cache_hit_tokens'] + usage['prompt_cache_miss_tokens']
        or usage['total_tokens'] != usage['prompt_tokens'] + usage['completion_tokens']
        or usage['completion_tokens'] > MAX_TOKENS):
        raise ValueError('Usage totals differ')
    return int(usage['prompt_cache_hit_tokens'] * 6 + usage['prompt_cache_miss_tokens'] * 300
               + usage['completion_tokens'] * 1200)


def preflight(head: str) -> dict[str, Any]:
    root = HERE.parents[1]
    actual = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
    remote = subprocess.check_output(['git', 'ls-remote', 'origin', 'refs/heads/task/R-STEER-01'],
                                     cwd=root, text=True).split()[0]
    dirty = subprocess.check_output(['git', 'status', '--porcelain'], cwd=root, text=True).strip()
    if actual != head or remote != head or dirty:
        raise ValueError('Clean pushed registration head required')
    plan = load(HERE / 'inputs/ds-flash-high-plan-01.json')
    if sha(SESSION / 'plan.json') != sha(HERE / 'inputs/ds-flash-high-plan-01.json'):
        raise ValueError('Plan binding differs')
    old = load(HERE / 'inputs/ds-flash-plan-01.json')
    if [r['id'] for r in plan['requests']] != old['pilot_ids'] or len(old['pilot_ids']) != 20:
        raise ValueError('Same twenty and order required')
    for source in plan['sources']:
        if sha(HERE / source['path']) != source['sha256']:
            raise ValueError('Source changed')
    for row in plan['requests']:
        path = SESSION / 'requests' / (row['id'] + '.json')
        previous = HERE / 'ds-flash-review-01' / row['id'] / 'request.json'
        if (sha(path) != row['sha256'] or path.stat().st_size != row['bytes']
            or sha(previous) != row['old_request_sha256']
            or load(path) != high_request(load(previous))):
            raise ValueError('Frozen prompt/parameter binding differs')
    previous_run = load(SESSION.parent / 'ds-flash-pilot-01/pilot-run/result.json')
    if (previous_run['post_attempts'] != plan['prior_posts']
        or previous_run['cumulative_upper_nano_usd'] != plan['prior_upper_nano_usd']
        or plan['prior_posts'] != 20 or plan['prior_upper_nano_usd'] != 11447052):
        raise ValueError('Previous budget differs')
    price = load(SESSION / 'price-check.json')
    age = (datetime.now(timezone.utc) - datetime.fromisoformat(price['checked_utc'])).total_seconds()
    if (not 0 <= age <= 600 or price['model'] != MODEL or price['source'] != plan['price_source']
        or price['version'] != plan['documented_version']
        or price['peak_nano_usd_per_token'] != plan['peak_nano_usd_per_token']):
        raise ValueError('Fresh unchanged official price/version required')
    return dict(plan)


def run(head: str) -> int:
    plan = preflight(head)
    key = os.environ.get('DEEPSEEK_API_KEY')
    if not key:
        raise ValueError('Process credential missing')
    output = SESSION / 'run'
    output.mkdir(exist_ok=False)
    write(output / 'binding.json', {'head': head, 'plan_sha256': sha(SESSION / 'plan.json'),
          'started_utc': datetime.now(timezone.utc).isoformat(), 'key_source': 'process environment only'})
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
    total = int(plan['prior_upper_nano_usd'])
    attempts = completed = 0
    model: str | None = None
    status = 'high_stopped'
    start = time.monotonic()
    ledger = output / 'ledger.jsonl'
    try:
        for row in plan['requests']:
            if attempts >= 20 or time.monotonic() - start > 1800:
                raise ValueError('Registered time/call bound')
            body = (SESSION / 'requests' / (row['id'] + '.json')).read_bytes()
            held = reserve(body)
            if total + held >= SOFT_LIMIT:
                status = 'budget_stop'
                break
            folder = output / row['id']
            folder.mkdir()
            (folder / 'request.json').write_bytes(body)
            attempts += 1
            total += held
            event(ledger, {'event': 'reserve_before_post', 'id': row['id'], 'ordinal': attempts,
                  'overall_ordinal': 20 + attempts, 'reserved_nano_usd': held,
                  'cumulative_upper_nano_usd': total, 'request_sha256': row['sha256']})
            request = urllib.request.Request(ENDPOINT, data=body,
                headers={'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json',
                         'User-Agent': 'R-STEER-01-frozen-high-twenty'}, method='POST')
            try:
                with opener.open(request, timeout=180) as response:
                    raw = response.read(2_000_001)
                    response_status = response.status
            except urllib.error.HTTPError as error:
                raw = error.read(2_000_001)
                if key.encode() not in raw:
                    (folder / 'http-error.body').write_bytes(raw)
                event(ledger, {'event': 'http_error_no_retry', 'id': row['id'], 'http_status': error.code})
                raise ValueError('HTTP failed; retain reservation') from None
            if len(raw) > 2_000_000 or key.encode() in raw:
                raise ValueError('Unsafe/oversized response not persisted')
            (folder / 'response.json').write_bytes(raw)
            value = json.loads(raw)
            cost = high_cost(value['usage'])
            total += cost - held
            event(ledger, {'event': 'settle_from_usage_peak_upper', 'id': row['id'],
                  'http_status': response_status, 'usage': value['usage'], 'upper_nano_usd': cost,
                  'cumulative_upper_nano_usd': total, 'response_model': value.get('model'),
                  'system_fingerprint': value.get('system_fingerprint'), 'response_id': value.get('id'),
                  'response_created': value.get('created')})
            if cost > held:
                raise ValueError('Usage exceeded pre-reservation')
            if value.get('model') not in (MODEL, plan['documented_version']) or (model is not None and value['model'] != model):
                raise ValueError('Unexpected/changed model')
            model = value['model']
            choices = value['choices']
            if len(choices) != 1 or choices[0]['finish_reason'] != 'stop':
                raise ValueError('Nonstop/invalid completion')
            message = choices[0]['message']
            if message.get('tool_calls') or not isinstance(message.get('reasoning_content'), str) or not message['reasoning_content'].strip():
                raise ValueError('Expected thinking without tools')
            content = message['content']
            if not isinstance(content, str) or not content.strip():
                raise ValueError('Blank JSON; no repair')
            presented = json.loads(load(folder / 'request.json')['messages'][1]['content'])
            rating = validate_rating(json.loads(content), presented['待评输出'], presented['评分事实参照'])
            write(folder / 'rating.json', rating)
            completed += 1
        if completed == 20:
            status = 'high_complete_awaiting_user_trust'
    except Exception as error:
        event(ledger, {'event': 'stop_no_retry', 'error_class': type(error).__name__, 'completed': completed})
    finally:
        write(output / 'result.json', {'status': status, 'post_attempts': attempts,
              'completed_ratings': completed, 'prior_post_attempts': 20,
              'overall_post_attempts': 20 + attempts, 'full_post_attempts': 0,
              'prior_upper_nano_usd': plan['prior_upper_nano_usd'],
              'new_upper_nano_usd': total - plan['prior_upper_nano_usd'],
              'cumulative_upper_nano_usd': total, 'response_model': model,
              'ended_utc': datetime.now(timezone.utc).isoformat(),
              'fee_method': 'usage at peak rates; unresolved reservations retained; not invoice'})
        os.environ.pop('DEEPSEEK_API_KEY', None)
    return 0 if completed == 20 else 2


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--expected-head', required=True)
    args = parser.parse_args()
    try:
        raise SystemExit(run(args.expected_head))
    except Exception:
        write(SESSION / 'preflight-stop.json', {'status': 'preflight_stopped_no_retry',
              'utc': datetime.now(timezone.utc).isoformat()})
        raise SystemExit(2) from None
