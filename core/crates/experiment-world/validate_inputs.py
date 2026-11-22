"""Independent preregistration v0.2 section 2.1 input audit; never advances a state."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


def check_person(world: str, event: dict[str, Any]) -> None:
    ordinal = event['ordinal']
    context = event['context']
    if context not in ('contact','task','explore','empathy','play','pattern','secret','punctual','feedback'):
        raise ValueError(f'unknown context: {context}')
    expected = 'U'
    if world not in ('W6','W7') and not world.startswith('O5-'):
        # Every seventh 12-opportunity block: only its two relation events use V.
        if ((ordinal - 1) // 12) % 7 == 6 and context in ('secret','punctual'):
            expected = 'V'
    if event['person'] != expected:
        raise ValueError(f'{world} ordinal {ordinal}: person {event["person"]}, expected {expected}')


def validate(package: Path, only: str | None = None) -> dict[str, int]:
    hashes = json.loads((package/'sha256.json').read_text(encoding='utf-8'))
    for name, digest in hashes.items():
        if only is not None and name.endswith('.jsonl') and name != only+'.jsonl':
            continue
        if hashlib.sha256((package/name).read_bytes()).hexdigest() != digest:
            raise ValueError(f'frozen byte mismatch: {name}')
    runs = json.loads((package/'runs.json').read_text(encoding='utf-8'))
    checked = count = v_count = 0
    for run in runs:
        if run['status'] != 'ready' or (only is not None and run['id'] != only):
            continue
        checked += 1
        previous = 0
        actual = 0
        with (package/(run['id']+'.jsonl')).open(encoding='utf-8') as stream:
            for actual, line in enumerate(stream, 1):
                event = json.loads(line)
                if event['ordinal'] != actual or event['day'] != (actual-1)//run['density']+1:
                    raise ValueError(f'{run["id"]}: event identity/day mismatch at {actual}')
                if event['time'] <= previous or event['hours'] != run['hours']:
                    raise ValueError(f'{run["id"]}: time/runtime mismatch at {actual}')
                previous = event['time']
                check_person(run['world'],event)
                v_count += int(event['person']=='V')
                count += 1
        expected = 0 if run['world']=='W5' else run['days']*run['density']
        if actual != expected:
            raise ValueError(f'{run["id"]}: {actual} events, expected {expected}')
    if only is not None and checked != 1:
        raise ValueError('run absent from ready list')
    return {'runs':checked,'events':count,'V_events':v_count}


def stopped(results: Path) -> bool:
    return any((results/name).exists() for name in ('STOP.json','structural-stop.json','resource-stop.json'))


if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('package',type=Path)
    print(json.dumps(validate(parser.parse_args().package)))
