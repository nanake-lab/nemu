"""Read-only preregistered endpoint analysis. No parameter feedback or writes to inputs."""
from __future__ import annotations
import gzip
import argparse
import json
import struct
import statistics
from pathlib import Path
from typing import Any

def bits(x: float) -> str:
    return struct.pack('>d', x).hex()

def identity(v: Any) -> Any:
    if isinstance(v, float):
        return bits(v)
    if isinstance(v, list):
        return [identity(x) for x in v]
    if isinstance(v, dict):
        return {k: identity(x) for k, x in v.items()}
    return v

def visible(row: dict[str, Any]) -> Any:
    s = row['state']
    return identity({'r': [{k:r[k] for k in ('id','pi','r','active')} for r in s['r']], 'theta':s['theta'], 'probes':[{k:p[k] for k in ('id','delta','learned_delta')} for p in row['probes']]})

def endpoint(rows: list[dict[str, Any]], complete: bool) -> dict[str, Any]:
    if not rows:
        return {'status':'not_run','reason':'no daily rows'}
    genesis = {p['id']:p for p in rows[0]['probes']}
    streak: dict[tuple[str,str], tuple[int,int]] = {}
    first = None
    for row in rows[1:]:
        day = row['day']
        qualifying = set()
        for p in row['probes']:
            for m in p['marginal']:
                if any(x != 0 for x in m['f']) and (any(x != 0 for x in m['nature']) or m['learned'] != 0) and (identity(p['delta']) != identity(genesis[p['id']]['delta']) or any(x[1] != 0 for x in p['learned_delta'])):
                    qualifying.add((m['id'],p['id']))
        streak = {key:(streak[key][0],streak[key][1]+1) if key in streak else (day,1) for key in qualifying}
        for key,(start,length) in sorted(streak.items()):
            if length >= 30 and first is None:
                first = {'start_day':start,'confirmation_day':day,'tendency':key[0],'probe':key[1]}
    tail = [r for r in rows if 276 <= r['day'] <= 365]
    m2 = 'not_run'
    if complete and len(tail)==90:
        fixed = all(visible(r)==visible(tail[0]) for r in tail)
        m2 = '停在起点' if fixed and visible(tail[0])==visible(rows[0]) else '成长后平台' if fixed else '未形成持续区别' if first is None else '持续变化且达到主终点'
    magnitudes=[]
    for p in rows[0]['probes']:
        for coordinate in range(4):
            values=[next(q for q in row['probes'] if q['id']==p['id'])['delta'][coordinate]-p['delta'][coordinate] for row in rows[1:]]
            if values:magnitudes.append({'probe':p['id'],'coordinate':coordinate,'minimum':min(values),'maximum':max(values),'median':statistics.median(values)})
    return {'probe_effect_sizes':magnitudes,'M1':{'status':'passed' if first else 'failed' if complete else 'not_run','first':first,'right_censored': [{'tendency':k[0],'probe':k[1],'start_day':v[0],'observed_days':v[1]} for k,v in sorted(streak.items()) if v[1]<30]},'M2':m2,'complete':complete}

def main() -> None:
    parser=argparse.ArgumentParser()
    parser.add_argument('package',type=Path);parser.add_argument('results',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args()
    manifest=json.loads((args.package/'runs.json').read_text(encoding='utf-8-sig'))
    reports=[]; loaded={}
    for run in manifest:
        root=args.results/run['id']; result=root/'result.json'
        if not result.exists():
            reports.append({'id':run['id'],'status':'not_run','reason':run['reason'] or 'unfinished execution'});continue
        r=json.loads(result.read_text());rows=[json.loads(line) for line in gzip.open(root/'daily.jsonl.gz','rt',encoding='utf-8')]
        complete=r['status']=='passed' and rows[-1]['day']==run['days']
        r.update(endpoint(rows,complete))
        first_entry=None; zero_coverage=[]
        with gzip.open(root/'events.jsonl.gz','rt',encoding='utf-8') as events:
            for line in events:
                event=json.loads(line)
                if event['event']['a']==[0,0,0,0]:zero_coverage.append({'ordinal':event['event']['ordinal'],'context':event['event']['context']})
                active=[t['id'] for t in event['state']['r'] if t['active']]
                if active and first_entry is None:
                    e=event['event'];first_entry={'calendar_days':e['time']/86400000000000,'running_days':((e['day']-1)*e['hours']*3600000000000+e['time']%86400000000000)/86400000000000,'events':e['ordinal'],'independent_groups':e['ordinal'],'ids':active}
        r['T_entry']=first_entry
        r['four_nature_zero_coverage']=zero_coverage
        reports.append(r);loaded[run['id']]=rows
    pairs=[]
    for run in manifest:
        if run['world'] not in ('W0','W6'):continue
        for other in (['W1','W2','W3','W4'] if run['world']=='W0' else ['W7']):
            left=run['id'];right=left.replace(run['world'],other,1)
            if left not in loaded or right not in loaded:continue
            streak: dict[tuple[str,str,int],tuple[int,int]]={};first=None; q1=[]; trajectory=[]
            for a,b in zip(loaded[left],loaded[right]):
                day=a['day']
                if day!=b['day']:raise ValueError('checkpoint misalignment')
                if run['world']=='W6' and day in (30,90,180,365):
                    same=identity(a['state']['theta'])==identity(b['state']['theta']) and identity(a['state']['x_nature'])==identity(b['state']['x_nature']) and identity(a['state']['x_learned'])==identity(b['state']['x_learned'])
                    q1.append({'day':day,'strict_current_match':same,'status':'passed' if same else 'non_identifiable','probe_differences': [{'id':x['id'],'delta':[u-v for u,v in zip(x['delta'],y['delta'])]} for x,y in zip(a['probes'],b['probes'])]})
                ra={r['id']:r for r in a['state']['r']};rb={r['id']:r for r in b['state']['r']}
                trajectory.append({'day':day,'active_symmetric_difference':sorted({k for k,v in ra.items() if v['active']} ^ {k for k,v in rb.items() if v['active']}),'r_differences':{k:{f:ra[k][f]-rb[k][f] for f in ('pi','r')} for k in sorted(ra.keys()&rb.keys())},'theta_left':a['state']['theta'],'theta_right':b['state']['theta'],'probe_delta_differences':[{'id':x['id'],'nature':[u-v for u,v in zip(x['delta'],y['delta'])]} for x,y in zip(a['probes'],b['probes'])]})
                current={}
                for x,y in zip(a['probes'],b['probes']):
                    for i,(u,v) in enumerate(zip(x['delta'],y['delta'])):
                        d=u-v
                        if d: current[(x['id'],f'nature:{i}',1 if d>0 else -1)]=d
                    lx=dict(x['learned_delta']);ly=dict(y['learned_delta'])
                    for identifier in lx.keys()&ly.keys():
                        d=lx[identifier]-ly[identifier]
                        if d:current[(x['id'],identifier,1 if d>0 else -1)]=d
                streak={key:(streak[key][0],streak[key][1]+1) if key in streak else (day,1) for key in current}
                for key,(start,n) in sorted(streak.items()):
                    if n>=30 and first is None:first={'probe':key[0],'coordinate':key[1],'sign':key[2],'start_day':start,'confirmation_day':day}
            pairs.append({'left':left,'right':right,'M3_first_sustained':first,'Q1':q1,'trajectory':trajectory})
    feedback=[]
    for run in manifest:
        if not run['world'].startswith('O5-') or '-none' in run['world']:continue
        reference=run['id'].replace('-temporary-','-none-').replace('-permanent-','-none-')
        if run['id'] not in loaded or reference not in loaded:continue
        feedback.append({'id':run['id'],'reference':reference,'formal_AC2_25':'not_run: no calibrated life reference/alarm acceptance','trajectory':[{'day':a['day'],'baseline_difference':{k:a['state']['theta']['baseline'][k]-b['state']['theta']['baseline'][k] for k in ('connection','empathy','curiosity','achievement')},'distance_to_baseline_bound':{k:1-abs(a['state']['theta']['baseline'][k]) for k in ('connection','empathy','curiosity','achievement')}} for a,b in zip(loaded[run['id']],loaded[reference])]})
    cross=[]
    by_id={r['id']:r for r in reports}
    for run in manifest:
        if run['seed']!=53001:continue
        a=by_id[run['id']];b=by_id.get(run['id'].replace('53001','53002'))
        if not b:continue
        sa=a.get('M1',{}).get('status','not_run');sb=b.get('M1',{}).get('status','not_run')
        cross.append({'seed1':a['id'],'seed2':b['id'],'M1_statuses':[sa,sb],'agreement':sa==sb,'growth_requirement':'passed' if sa==sb=='passed' else 'not_run' if 'not_run' in (sa,sb) else 'failed'})
    output={'label':'第一版对照 / 非正式结论','runs':reports,'pairs':pairs,'feedback':feedback,'Q2':json.loads((args.results/'Q2'/'pairs.json').read_text()) if (args.results/'Q2'/'pairs.json').exists() else {'status':'not_run','reason':'continuations unfinished'},'cross_seed':cross}
    with args.output.open('x',encoding='utf-8') as f:json.dump(output,f,ensure_ascii=False,indent=2)
if __name__=='__main__':main()
