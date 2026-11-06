"""Offline summary of saved observations; no GPU/package imports."""
import json
import math
import statistics
from pathlib import Path
base = Path(__file__).parent
events = [json.loads(line) for line in (base/'events.jsonl').read_text(encoding='utf-8-sig').splitlines()]
raw = [json.loads(line) for line in (base/'memory.jsonl').read_text(encoding='utf-8-sig').splitlines()]
def percentile(values, p):
    ordered = sorted(values)
    pos = (len(ordered)-1)*p
    low = math.floor(pos)
    high = math.ceil(pos)
    return ordered[low] + (ordered[high]-ordered[low])*(pos-low)
def latency(rows):
    return {'n':len(rows), 'first_playable_p50_s':percentile([r['first_playable_seconds'] for r in rows], .5), 'first_playable_p90_s':percentile([r['first_playable_seconds'] for r in rows], .9), 'first_playable_min_s':min(r['first_playable_seconds'] for r in rows), 'first_playable_max_s':max(r['first_playable_seconds'] for r in rows), 'total_s':sum(r['total_seconds'] for r in rows), 'audio_s':sum(r['audio_seconds'] for r in rows), 'aggregate_rtf':sum(r['total_seconds'] for r in rows)/sum(r['audio_seconds'] for r in rows), 'audio_seconds_per_wall_second':sum(r['audio_seconds'] for r in rows)/sum(r['total_seconds'] for r in rows)}
mem=[]
for row in raw:
    own=set(row['owned'])
    d=s=bd=bs=0
    for counter in row['counters']:
        pid=int(counter['Name'].split('_')[1])
        if pid in own:
            d+=counter['DedicatedUsage']; s+=counter['SharedUsage']
        else:
            bd+=counter['DedicatedUsage']; bs+=counter['SharedUsage']
    mem.append({'utc':row['utc'],'dedicated':d,'shared':s,'background_dedicated':bd,'background_shared':bs,'rss':row['rss'],'private':row['private']})
def peak(rows):
    result={'n':len(rows)}
    for key in ('dedicated','shared','background_dedicated','background_shared','rss','private'):
        winner=max(rows,key=lambda r:r[key])
        result[key+'_peak_bytes']=winner[key]
        result[key+'_peak_utc']=winner['utc']
    result['dedicated_plus_shared_independent_peaks_bytes']=result['dedicated_peak_bytes']+result['shared_peak_bytes']
    return result
loaded=next(r for r in events if r['event']=='model_loaded')
load_end=next(r for r in events if r['event']=='load_observation_end')
start=next(r for r in events if r['event']=='stability_start')
end=next(r for r in events if r['event']=='stability_end')
stable=[r for r in mem if start['utc']<=r['utc']<=end['utc']]
def trend(rows,key):
    xs=[r['utc']-rows[0]['utc'] for r in rows]
    ys=[r[key] for r in rows]
    xm=statistics.mean(xs); ym=statistics.mean(ys)
    slope=sum((x-xm)*(y-ym) for x,y in zip(xs,ys))/sum((x-xm)**2 for x in xs)
    return {'first_bytes':ys[0],'last_bytes':ys[-1],'delta_bytes':ys[-1]-ys[0],'min_bytes':min(ys),'max_bytes':max(ys),'ols_MiB_per_hour':slope*3600/1048576}
requests=[r for r in events if r['event']=='request']
summary={'complete':events[-1]['event']=='complete','scope':'standalone nonstreaming model.run to first 24k float32 PCM bytes; excludes websocket and sound device','percentile_method':'linear interpolation at (n-1)*p','load_event':loaded,'postload_observation':peak([r for r in mem if loaded['utc']<=r['utc']<=load_end['utc']]),'whole_run_peak':peak(mem),'cold':[r for r in requests if r['phase']=='cold'],'latency':{case:latency([r for r in requests if r['phase']=='latency' and r['case']==case]) for case in ('zh','mixed')},'stability':{'start_utc':start['utc'],'end_utc':end['utc'],'duration_seconds':end['duration_seconds'],'requests':end['requests'],'synthesis':latency([r for r in requests if r['phase']=='stability']),'trend':{key:trend(stable,key) for key in ('rss','private','dedicated','shared')},'last_10min_trend':{key:trend([r for r in stable if r['utc']>=end['utc']-600],key) for key in ('rss','private','dedicated','shared')}},'torch_final':events[-1], 'sample_interval_seconds':{'median':statistics.median(b['utc']-a['utc'] for a,b in zip(mem,mem[1:])),'max':max(b['utc']-a['utc'] for a,b in zip(mem,mem[1:]))},'network_attempt_log_exists':(base/'blocked-network.jsonl').exists()}
(base/'summary.json').write_text(json.dumps(summary,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
with (base/'memory-derived.jsonl').open('w',encoding='utf-8') as stream:
    for row in mem:stream.write(json.dumps(row)+'\n')
print(json.dumps(summary,indent=2,ensure_ascii=False))
