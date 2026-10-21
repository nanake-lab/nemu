import json, math, statistics, hashlib, struct, subprocess, re, sys
from pathlib import Path
from datetime import datetime, timezone
from collections import Counter
from html.parser import HTMLParser
from html import escape
ROOT=Path('reports/P0-07/instruct-adr0012-v1')
def readj(p): return json.loads(Path(p).read_text(encoding='utf-8'))
def jd(x): return json.dumps(x,ensure_ascii=True,sort_keys=True,separators=(',',':'))
def dt(s): return datetime.fromisoformat(s.replace('Z','+00:00'))
def qlinear(vals,q):
    a=sorted(float(x) for x in vals); h=(len(a)-1)*q; lo=math.floor(h); hi=math.ceil(h); return a[lo]+(a[hi]-a[lo])*(h-lo)
def han_count(s): return sum(1 for c in s if ('\u3400'<=c<='\u4dbf') or ('\u4e00'<=c<='\u9fff') or ('\uf900'<=c<='\ufaff'))
def levenshtein(a,b):
    prev=list(range(len(b)+1))
    for i,x in enumerate(a,1):
        cur=[i]
        for j,y in enumerate(b,1): cur.append(min(cur[-1]+1,prev[j]+1,prev[j-1]+(x!=y)))
        prev=cur
    return prev[-1]
def budget_stats(items,eos_ids):
    toks=[x.get('token_ids') for x in items]
    lengths=[len(x) for x in toks if isinstance(x,list)]
    elapsed=[float(x['elapsed_s']) for x in items if isinstance(x.get('elapsed_s'),(int,float))]
    eos_stop=[x for x in items if x.get('stop')=='eos']
    eos_match=sum(bool(x.get('token_ids')) and int(x['token_ids'][-1]) in eos_ids for x in eos_stop)
    return {'count':len(items),'token_ids_missing_or_nonlist':len(items)-len(lengths),'token_ids_min':min(lengths) if lengths else None,'token_ids_max':max(lengths) if lengths else None,'over_48_token_ids_count':sum(x>48 for x in lengths),'elapsed_s_missing_or_nonnumeric':len(items)-len(elapsed),'elapsed_max_s':max(elapsed) if elapsed else None,'over_25_seconds_count':sum(x>25 for x in elapsed),'stop_counts':dict(Counter(x.get('stop','<missing>') for x in items)),'eos_stop_count':len(eos_stop),'eos_stop_last_token_in_declared_eos_ids_count':eos_match}
def stage1():
    began=datetime.now(timezone.utc).isoformat()
    out={'stage':'run-latency-memory-path','evidence_root':str(ROOT.resolve()),'runs':[]}
    for i in range(1,4):
        d=ROOT/f'run-{i}'; summary=readj(d/'summary.json'); cp=next((d/'matrix').glob('*/concurrent.json')); concurrent=readj(cp); path=readj(cp.parent/'path.json'); render=readj(cp.parent/'render.json')
        samples=concurrent.get('samples',[]); warm=[x for x in samples if x.get('warmup') is True]; nw=[x for x in samples if x.get('warmup') is not True]
        lat=[x['latency_ms'] for x in nw if isinstance(x.get('latency_ms'),(int,float))]
        han=[han_count(x.get('output','')) for x in nw]; fixture=[x.get('matches_fixed_fixture') is True for x in nw]
        starts=summary.get('started_utc'); ends=summary.get('ended_utc')
        memfile=d/'memory.jsonl'; memrows=[json.loads(x) for x in memfile.read_text(encoding='utf-8').splitlines() if x.strip()]
        mem=[]; intervals=[]
        for j,rec in enumerate(memrows):
            owned={int(x['ProcessId']) for x in rec.get('ownedProcesses',[]) if 'ProcessId' in x}; ded=shared=count=0
            for c in rec.get('counters',[]):
                m=re.match(r'pid_(\d+)_',str(c.get('Name','')))
                if m and int(m.group(1)) in owned:
                    ded+=int(c.get('DedicatedUsage') or 0); shared+=int(c.get('SharedUsage') or 0); count+=1
            if count: mem.append({'line':j+1,'startedUtc':rec.get('startedUtc'),'endedUtc':rec.get('endedUtc'),'dedicated_bytes':ded,'shared_bytes':shared,'sum_bytes':ded+shared,'counter_rows':count,'owned_process_ids':sorted(owned)})
            if j:
                intervals.append({'seconds':(dt(rec['startedUtc'])-dt(memrows[j-1]['startedUtc'])).total_seconds(),'previous_line':j,'previous_start':memrows[j-1]['startedUtc'],'current_line':j+1,'current_start':rec['startedUtc']})
        dpk=max(mem,key=lambda x:x['dedicated_bytes']) if mem else None; spk=max(mem,key=lambda x:x['shared_bytes']) if mem else None; gap=max(intervals,key=lambda x:x['seconds']) if intervals else None
        alloc=[x['cuda_peak_allocated_bytes'] for x in samples if isinstance(x.get('cuda_peak_allocated_bytes'),(int,float))]
        reserv=[x['cuda_peak_reserved_bytes'] for x in samples if isinstance(x.get('cuda_peak_reserved_bytes'),(int,float))]
        injections=[{k:x.get(k) for k in ('phase','sequence_length','injected_position','max_abs_error','observed_l2')} for x in path.get('injection_observations',[])]
        anchors={'summary_start':starts,'concurrent_timestamp':concurrent.get('timestamp'),'path_timestamp':path.get('timestamp'),'render_startedAt':render.get('startedAt')}
        anchor_diffs={}
        if gap:
            cur=dt(gap['current_start']); prev=dt(gap['previous_start'])
            anchor_diffs={k:{'current_minus_anchor_s':(cur-dt(v)).total_seconds(),'previous_minus_anchor_s':(prev-dt(v)).total_seconds()} for k,v in anchors.items() if v}
        nonwarmup_lat=lat
        out['runs'].append({'run':i,'files':{'summary':str((d/'summary.json').resolve()),'concurrent':str(cp.resolve()),'path':str((cp.parent/'path.json').resolve()),'render':str((cp.parent/'render.json').resolve()),'memory':str(memfile.resolve())},
          'recorded_times':{'summary_start_utc':starts,'summary_end_utc':ends,'concurrent_timestamp':anchors['concurrent_timestamp'],'path_timestamp':anchors['path_timestamp'],'render_startedAt':anchors['render_startedAt']},
          'subprocess_exit_codes':{'matrix':summary.get('matrix_exit'),'memory_watch':summary.get('memory_watch_exit')},'sample_counts':{'all':len(samples),'warmup':len(warm),'nonwarmup':len(nw),'numeric_latency':len(lat)},
          'latency_ms':{'p50_recomputed':statistics.median(nonwarmup_lat) if nonwarmup_lat else None,'p90_linear_recomputed':qlinear(nonwarmup_lat,.9) if nonwarmup_lat else None,'p50_summary':summary.get('p50_ms'),'p90_summary':summary.get('p90_ms'),'summary_method':summary.get('percentile_method')},
          'fixture':{'nonwarmup_han_count_histogram':dict(Counter(han)),'exactly_20_han_count':sum(x==20 for x in han),'matches_fixed_fixture_true_count':sum(fixture),'stop_reason_counts':dict(Counter(x.get('stop_reason','<missing>') for x in nw))},
          'wddm':{'jsonl_line_count':len(memrows),'owned_gpu_counter_observation_count':len(mem),'no_owned_gpu_counter_samples':len(memrows)-len(mem),'dedicated_peak_sample':dpk,'shared_peak_sample':spk,'recomputed_bound_bytes':dpk['dedicated_bytes']+spk['shared_bytes'] if dpk and spk else None,'summary_bound_bytes':summary.get('memory_upper_bound_bytes'),'summary_sampled_peaks':summary.get('memory',{}).get('sampled_peaks'),'summary_errors':summary.get('memory',{}).get('errors'),'maximum_sample_start_interval':gap,'summary_maximum_sample_start_interval_s':summary.get('memory',{}).get('maximum_start_interval_s'),'gap_relative_to_recorded_events_s':anchor_diffs},
          'torch_allocator':{'allocated_sample_count':len(alloc),'allocated_peak_recomputed_bytes':max(alloc) if alloc else None,'allocated_peak_summary_bytes':summary.get('torch_peak_allocated_bytes'),'reserved_sample_count':len(reserv),'reserved_peak_recomputed_bytes':max(reserv) if reserv else None,'reserved_peak_summary_bytes':summary.get('torch_peak_reserved_bytes')},
          'path':{'torch':path.get('torch'),'dtype':path.get('dtype'),'attention':path.get('attention'),'native_rms_count':path.get('native_rms_count'),'native_rms_vs_original_prefill_max_abs_error':path.get('native_rms_vs_original_prefill_max_abs_error'),'native_rms_vs_original_decode_max_abs_error':path.get('native_rms_vs_original_decode_max_abs_error'),'layer_index_zero_based':path.get('layer_index_zero_based'),'rank':path.get('rank'),'zero_prefill_logits_max_abs_error':path.get('zero_prefill_logits_max_abs_error'),'zero_decode_logits_max_abs_error':path.get('zero_decode_logits_max_abs_error'),'injection_observations':injections,'changed_cache_discarded':path.get('changed_cache_discarded'),'changed_vs_fresh_logits_max_abs_error':path.get('changed_vs_fresh_logits_max_abs_error')},
          'render_tts':{'render_json':{'browser':render.get('browser'),'frame_start':render.get('frameStart'),'frame_end':render.get('frameEnd'),'mode':render.get('mode'),'renderer':render.get('state',{}).get('renderer'),'vendor':render.get('state',{}).get('vendor'),'canvas':[render.get('state',{}).get('canvasWidth'),render.get('state',{}).get('canvasHeight')],'gpu_devices':render.get('gpu',{}).get('devices')},'concurrent_render':concurrent.get('render'),'tts_scope':concurrent.get('tts_scope'),'tts_parameter_devices':concurrent.get('tts_parameter_devices'),'baseline_gpu':concurrent.get('baseline_gpu'),'after_tts_gpu':concurrent.get('after_tts_gpu')}})
        x=out['runs'][-1]
        x['summary_consistency']={'p50_exact_match':x['latency_ms']['p50_recomputed']==x['latency_ms']['p50_summary'],'p90_abs_diff':abs(x['latency_ms']['p90_linear_recomputed']-x['latency_ms']['p90_summary']) if x['latency_ms']['p90_linear_recomputed'] is not None and x['latency_ms']['p90_summary'] is not None else None,'wddm_bound_exact_match':x['wddm']['recomputed_bound_bytes']==x['wddm']['summary_bound_bytes'],'allocator_allocated_exact_match':x['torch_allocator']['allocated_peak_recomputed_bytes']==x['torch_allocator']['allocated_peak_summary_bytes'],'allocator_reserved_exact_match':x['torch_allocator']['reserved_peak_recomputed_bytes']==x['torch_allocator']['reserved_peak_summary_bytes']}
    out['ended_utc']=datetime.now(timezone.utc).isoformat()
    return out
def stage2():
    began=datetime.now(timezone.utc).isoformat(); planp=Path('workers/model-adapter/probe/instruct-dose-plan-v1.json'); planbytes=planp.read_bytes(); plan=json.loads(planbytes.decode('utf-8'))
    src=readj(plan['direction_source']); norms=readj(ROOT/'direction/norms.json'); direction=readj(ROOT/'direction/direction.json')
    show=subprocess.run(['git','show','03071ae101d39f9a90d0de7e0bd4c5e52509c496:workers/model-adapter/probe/instruct-dose-plan-v1.json'],capture_output=True,check=False)
    committed=show.stdout; gitsha=hashlib.sha256(committed).hexdigest() if committed else None; committed_plan=json.loads(committed.decode('utf-8')) if show.returncode==0 and committed else None
    u=src.get('u',[]); u_bytes=b''.join(struct.pack('<f',float(v)) for row in u for v in row); u_sha=hashlib.sha256(u_bytes).hexdigest()
    languages=['zh','en','ja']; phases=['prefill','decode']; seedset=[731,732,733]; ratios=[-.4,-.2,-.1,-.05,0,.05,.1,.2,.4]
    median_rows=[]
    for lang in languages:
        for phase in phases:
            vals=[float(p['h_l2']) for s in norms.get('samples',[]) if s.get('language')==lang for p in s.get('positions',[]) if p.get('phase')==phase and isinstance(p.get('h_l2'),(int,float))]
            median_rows.append({'language':lang,'phase':phase,'count':len(vals),'median':statistics.median(vals) if vals else None})
    six=[x['median'] for x in median_rows if x['median'] is not None]; reference=statistics.median(six) if len(six)==6 else None
    declared_medians=norms.get('h_l2_medians'); declared_direction_medians=direction.get('h_l2_medians')
    def declared(d,lang,phase):
        x=d.get(lang) if isinstance(d,dict) else None
        return x.get(phase) if isinstance(x,dict) else None
    med_check=[]
    for x in median_rows:
        lang=x['language']; phase=x['phase']; a=declared(declared_medians,lang,phase); b=declared(declared_direction_medians,lang,phase)
        med_check.append({'language':lang,'phase':phase,'recomputed':x['median'],'norms_declared':a,'direction_declared':b,'matches_norms':a==x['median'],'matches_direction':b==x['median']})
    norm_samples=norms.get('samples',[]); norm_generations=[x.get('generation',{}) for x in norm_samples]
    dose_samples=direction.get('samples',[]); eos=[int(x) for x in direction.get('eos_token_ids',[])]
    combos=Counter((x.get('language'),x.get('seed'),x.get('ratio')) for x in dose_samples); missing=[]; duplicate=[]
    for lang in languages:
        for seed in seedset:
            for ratio in ratios:
                count=combos[(lang,seed,ratio)]
                if count==0: missing.append([lang,seed,ratio])
                elif count>1: duplicate.append([lang,seed,ratio,count])
    bad_coeff=[]; coeff_diffs=[]
    dn=direction.get('direction_norm'); ref=direction.get('reference_norm')
    for idx,x in enumerate(dose_samples):
        r=x.get('ratio'); c=x.get('coefficient')
        if not isinstance(r,(int,float)) or not isinstance(c,(int,float)) or not isinstance(dn,(int,float)) or not isinstance(ref,(int,float)) or dn==0:
            bad_coeff.append({'sample_index':idx,'language':x.get('language'),'seed':x.get('seed'),'ratio':r,'coefficient':c})
        else: coeff_diffs.append(abs(float(c)-(float(r)*float(ref)/float(dn))))
    expected_coeff=[r*float(ref)/float(dn) for r in ratios] if isinstance(ref,(int,float)) and isinstance(dn,(int,float)) and dn else []
    declared_coeff=direction.get('coefficients',[])
    coeff_top={'declared_length':len(declared_coeff) if isinstance(declared_coeff,list) else None,'expected_length':len(expected_coeff),'max_abs_diff':max((abs(float(a)-b) for a,b in zip(declared_coeff,expected_coeff) if isinstance(a,(int,float))),default=None),'non_numeric_count':sum(not isinstance(a,(int,float)) for a in declared_coeff) if isinstance(declared_coeff,list) else None}
    baseline={(x.get('language'),x.get('seed')):x.get('token_ids',[]) for x in dose_samples if x.get('ratio')==0}
    changes=[]
    for ratio in ratios:
        subset=[x for x in dose_samples if x.get('ratio')==ratio]; changed=edits=basecount=pairs=0
        for x in subset:
            b=baseline.get((x.get('language'),x.get('seed'))); a=x.get('token_ids')
            if not isinstance(b,list) or not isinstance(a,list): continue
            pairs+=1; changed+=a!=b; edits+=levenshtein(a,b); basecount+=len(b)
        changes.append({'ratio':ratio,'paired_sequence_count':pairs,'sequence_token_ids_changed_count':changed,'sequence_token_ids_changed_fraction':changed/pairs if pairs else None,'levenshtein_token_edit_count':edits,'baseline_token_id_count':basecount,'edit_count_over_baseline_token_ids':edits/basecount if basecount else None})
    result={'stage':'direction-dose-norm-audit','evidence_files':{'plan':str(planp.resolve()),'archived_direction':str(Path(plan['direction_source']).resolve()),'norms':str((ROOT/'direction/norms.json').resolve()),'direction':str((ROOT/'direction/direction.json').resolve())},
      'plan':{'model':plan.get('model'),'revision':plan.get('revision'),'ratios':plan.get('ratios'),'seeds':plan.get('seeds'),'generation_budget':plan.get('generation'),'sha256_worktree_bytes':hashlib.sha256(planbytes).hexdigest(),'plan_commit':'03071ae101d39f9a90d0de7e0bd4c5e52509c496','git_show_exit':show.returncode,'sha256_committed_bytes':gitsha,'worktree_equals_committed_bytes':planbytes==committed,'declared_plan_hash_norms':norms.get('plan_sha256'),'declared_plan_hash_direction':direction.get('plan_sha256'),'norms_hash_equals_committed_bytes':norms.get('plan_sha256')==gitsha,'direction_hash_equals_committed_bytes':direction.get('plan_sha256')==gitsha,'committed_plan_json_equals_worktree_plan':committed_plan==plan},
      'u_integrity':{'shape':[len(u),len(u[0]) if u else 0],'float32_little_endian_sha256_recomputed':u_sha,'source_declared_sha256':src.get('u_sha256'),'norms_declared_sha256':norms.get('u_sha256'),'direction_declared_sha256':direction.get('u_sha256'),'all_equal':u_sha==src.get('u_sha256')==norms.get('u_sha256')==direction.get('u_sha256')},
      'norms':{'sample_count':len(norm_samples),'language_seed_counts':dict(Counter((x.get('language'),x.get('seed')) for x in norm_samples)),'phase_medians':median_rows,'declared_median_matches':med_check,'six_median_reference_recomputed':reference,'norms_reference_declared':norms.get('reference_norm'),'direction_reference_declared':direction.get('reference_norm'),'reference_matches':reference is not None and math.isclose(reference,float(norms.get('reference_norm')),rel_tol=0,abs_tol=1e-12) and math.isclose(reference,float(direction.get('reference_norm')),rel_tol=0,abs_tol=1e-12),'budget_and_stops':budget_stats(norm_generations,eos),'stop_counts':dict(Counter(x.get('stop','<missing>') for x in norm_generations))},
      'dose':{'sample_count':len(dose_samples),'unique_language_seed_ratio_combinations':len(combos),'missing_combinations':missing,'duplicate_combinations':duplicate,'languages':dict(Counter(x.get('language') for x in dose_samples)),'seeds':dict(Counter(x.get('seed') for x in dose_samples)),'ratios':dict(Counter(x.get('ratio') for x in dose_samples)),'stop_counts':dict(Counter(x.get('stop','<missing>') for x in dose_samples)),'budget_and_eos':budget_stats(dose_samples,eos),'declared_eos_token_ids':eos,'prompt_text_equals_archived_source':direction.get('generation_prompts')==src.get('generation_prompts'),'embedded_plan_equals_current_plan':direction.get('dose_plan')==plan,'coefficients':{'bad_numeric_fields':bad_coeff,'per_sample_max_abs_formula_difference':max(coeff_diffs) if coeff_diffs else None,'top_level_coefficients':coeff_top},'same_language_seed_ratio_vs_zero_token_changes':changes}}
    result['ended_utc']=datetime.now(timezone.utc).isoformat(); return result
def stage3():
    began=datetime.now(timezone.utc).isoformat(); f=ROOT/'direction/comparison.html'; html=f.read_text(encoding='utf-8'); direction=readj(ROOT/'direction/direction.json'); samples=direction.get('samples',[])
    class Parser(HTMLParser):
        def __init__(self): super().__init__(convert_charrefs=True); self.rows=[]; self.row=None; self.cell=None; self.tables=0
        def handle_starttag(self,tag,attrs):
            if tag=='table': self.tables+=1
            elif tag=='tr': self.row=[]
            elif tag in ('td','th') and self.row is not None: self.cell=[]
        def handle_data(self,data):
            if self.cell is not None: self.cell.append(data)
        def handle_endtag(self,tag):
            if tag in ('td','th') and self.cell is not None and self.row is not None: self.row.append(''.join(self.cell).strip()); self.cell=None
            elif tag=='tr' and self.row is not None: self.rows.append(self.row); self.row=None
    parser=Parser(); parser.feed(html); cells=Counter(c for row in parser.rows for c in row); outputs=Counter(str(x.get('output','')) for x in samples)
    matched=sum(min(n,cells[s]) for s,n in outputs.items()); special=[s for s in outputs if any(c in s for c in '<>&')]; escaped=sum(cells[s]>0 and escape(s,quote=False) in html for s in special)
    result={'stage':'html-audit','file':str(f.resolve()),'dose_sample_count':len(samples),'html_table_count':parser.tables,'html_tr_count':len(parser.rows),'html_rows_with_cells':dict(Counter(len(x) for x in parser.rows)),'first_rows':parser.rows[:4],'exact_output_cell_occurrences_matched':matched,'sample_output_occurrences_expected':len(samples),'unique_sample_output_strings':len(outputs),'outputs_containing_html_special_chars':len(special),'special_char_output_occurrences_with_exact_parsed_text_and_escaped_markup':escaped,'escaping_cases_exercised':len(special),'escaping_testability':'No tested output strings contain HTML special characters.' if not special else 'Escaped markup and parsed text were checked for generated outputs containing <, >, or &.'}
    result['ended_utc']=datetime.now(timezone.utc).isoformat(); return result
if __name__=='__main__':
    stage=sys.argv[1]; outdir=Path(sys.argv[2]); began=datetime.now(timezone.utc).isoformat()
    result={'1':stage1,'2':stage2,'3':stage3}[stage](); result['started_utc']=began
    dest=outdir/f'stage{stage}-checkpoint.json'
    with dest.open('x',encoding='utf-8',newline='\n') as fh: json.dump(result,fh,ensure_ascii=True,indent=2); fh.write('\n')
    print(jd(result)); print('CHECKPOINT_PATH='+str(dest.resolve()))