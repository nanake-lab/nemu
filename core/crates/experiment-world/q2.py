"""Frozen Q2 enumeration and continuation schedule; no widening or resampling."""
from __future__ import annotations
import gzip
import itertools
import json
import struct
import subprocess
import shutil
from pathlib import Path
from typing import Any

def exact(a: float,b: float)->bool:
    return struct.pack('>d',a)==struct.pack('>d',b)

def fields(s: dict[str,Any])->dict[str,float]:
    result={f'x:{i}':float(v) for i,v in enumerate(s['x_nature'])}
    result.update({f'x:{i}':float(v) for i,v in s['x_learned']})
    for r in s['r']:
        result[f'pi:{r["id"]}']=r['pi'];result[f'r:{r["id"]}']=r['r']
    for key,prefix in [('baseline','b'),('gain','g'),('recovery_per_s','lambda')]:
        result.update({f'{prefix}:{k}':v for k,v in s['theta'][key].items() if k!='schema_version'})
    return result

def execute(package: Path,results: Path,binary: Path)->None:
    out=results/'Q2';out.mkdir()
    runs=json.loads((package/'runs.json').read_text(encoding='utf-8'))
    runs=[r for r in runs if r['world'] in ('W0','W1','W2','W3','W4','W6','W7')]
    daily={}
    histories={}
    for r in runs:
        f=results/r['id']/'daily.jsonl.gz'
        if f.exists():daily[r['id']]={v['day']:v for v in map(json.loads,gzip.open(f,'rt',encoding='utf-8')) if v['day'] in (30,90,180,365)}
        histories[r['id']]=[json.loads(l) for l in (package/(r['id']+'.jsonl')).read_text().splitlines()]
    records=[];branch_keys=set()
    for a,b in itertools.combinations(runs,2):
        if any(a[k]!=b[k] for k in ('seed','density','hours')):continue
        for day in (30,90,180,365):
            record={'left':a['id'],'right':b['id'],'day':day,'status':'non_identifiable','reason':''}
            if day not in daily.get(a['id'],{}) or day not in daily.get(b['id'],{}):
                record.update(status='not_run',reason='checkpoint unavailable');records.append(record);continue
            if [e for e in histories[a['id']] if e['day']<=day]==[e for e in histories[b['id']] if e['day']<=day]:
                record['reason']='identical experience prefix (lineage identity alone is not distinct experience)';records.append(record);continue
            sa=daily[a['id']][day]['state'];sb=daily[b['id']][day]['state']
            if [(r['id'],r['active']) for r in sa['r']]!=[(r['id'],r['active']) for r in sb['r']]:
                record['reason']='candidate or active identity mismatch';records.append(record);continue
            fa,fb=fields(sa),fields(sb)
            if fa.keys()!=fb.keys():record['reason']='coordinate mismatch';records.append(record);continue
            diffs={k:fa[k]-fb[k] for k in fa}
            widths={'r':1.,'g':2.8,'lambda':1.999}
            near=all(abs(d)<=widths.get(k.split(':')[0],1.)/64 for k,d in diffs.items())
            is_exact=all(exact(fa[k],fb[k]) for k in fa)
            record.update(differences=diffs,tier='exact' if is_exact else 'near' if near else 'excluded')
            if near:
                record.update(status='pending',reason='')
                for run in (a,b):branch_keys.add((run['id'],day,run['density'],run['hours'],run['seed']))
            else:record['reason']='outside frozen matching range'
            records.append(record)
    for run,day,density,hours,seed in sorted(branch_keys):
        path=out/f'{run}-d{day}.json'
        if shutil.disk_usage(out).free<2*1024**3:
            with path.open('x') as f:json.dump({'status':'not_run','reason':'less than 2 GiB evidence reserve'},f)
            continue
        with (out/f'{run}-d{day}.log').open('x') as log:
            result=subprocess.run([str(binary),'continue',str(results/run/f'checkpoint-{day}.json'),str(day),str(day*density),str(hours),str(seed),str(path)],stdout=log,stderr=subprocess.STDOUT,check=False)
        if result.returncode and not path.exists():path.write_text(json.dumps({'status':'failed','exit_code':result.returncode}))
    for record in records:
        if record['status']!='pending':continue
        states=[json.loads((out/f'{record[k]}-d{record["day"]}.json').read_text()) for k in ('left','right')]
        if any(s['status']!='passed' for s in states):record.update(status='not_run' if any(s['status']=='not_run' for s in states) else 'failed',reason='continuation incomplete or fault');continue
        changes=[]
        for ba,bb in zip(states[0]['branches'],states[1]['branches']):
            first=None;trajectory=[]
            for a,b in zip(ba['rows'],bb['rows']):
                fa,fb,ga,gb=map(fields,(a['with'],a['without'],b['with'],b['without']))
                keys=fa.keys()&fb.keys()&ga.keys()&gb.keys()
                delta={k:(fa[k]-fb[k])-(ga[k]-gb[k]) for k in sorted(keys) if not k.startswith('x:')}
                trajectory.append({'step':a['step'],'difference_of_increments':delta})
                if first is None and any(v!=0 for v in delta.values()):first=a['step']
            changes.append({'sign':ba['sign'],'first_different_update':first,'trajectory':trajectory})
        record.update(status='passed',changes=changes,interpretation='observed increments only; no strong causal attribution from near match')
    with (out/'pairs.json').open('x',encoding='utf-8') as f:json.dump(records,f,ensure_ascii=False)
