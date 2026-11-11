"""Execute the six frozen §8 histories once after a committed run manifest.

This entry point cannot launch the full matrix. It records a cost projection
and leaves the >4h user gate to the main agent. No hidden retries or samples.
"""
import argparse
from copy import deepcopy
import ctypes
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import time

from arithmetic import Q, bits, check_hand_vectors
from c_oracle import C, bit_tree, first_difference
from f_oracle import F
from prepare import binding, write


class Bridge:
    def __init__(self, binary, log):
        env=dict(os.environ,P571_RPC='1')
        self.log=log
        self.p=subprocess.Popen([str(binary),'--exact','p571_public_bridge','--nocapture'],
            stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=log,text=True,encoding='utf-8',env=env)
        self.cache={}; self.primitive_misses=0

    def call(self, request):
        self.p.stdin.write(json.dumps(request)+'\n'); self.p.stdin.flush()
        while True:
            line=self.p.stdout.readline()
            if not line: raise RuntimeError(f'bridge terminated: {self.p.poll()}')
            if line.startswith('P571:'): return json.loads(line[5:])
            self.log.write(line); self.log.flush()

    def primitive(self, name, value):
        key=(name,bits(value))
        if key not in self.cache:
            self.primitive_misses+=1
            self.cache[key]=Q(self.call(dict(op='libm',function=name,x=float(value)))['value'])
        return self.cache[key]

    def close(self):
        self.p.stdin.close()
        self.log.write(self.p.stdout.read()); self.log.flush()
        assert self.p.wait()==0


def peak_bytes():
    # Windows process-owned working-set high water, no background process list.
    class Counters(ctypes.Structure):
        _fields_=[('cb',ctypes.c_ulong),('PageFaultCount',ctypes.c_ulong)]+[(k,ctypes.c_size_t) for k in
            ('PeakWorkingSetSize','WorkingSetSize','QuotaPeakPagedPoolUsage','QuotaPagedPoolUsage',
             'QuotaPeakNonPagedPoolUsage','QuotaNonPagedPoolUsage','PagefileUsage','PeakPagefileUsage')]
    counters=Counters(); counters.cb=ctypes.sizeof(counters)
    kernel=ctypes.windll.kernel32; kernel.GetCurrentProcess.restype=ctypes.c_void_p
    get=ctypes.windll.psapi.GetProcessMemoryInfo
    get.argtypes=[ctypes.c_void_p,ctypes.c_void_p,ctypes.c_ulong]
    if not get(kernel.GetCurrentProcess(),ctypes.byref(counters),counters.cb): raise ctypes.WinError()
    return counters.PeakWorkingSetSize


def check_c(model, row):
    actual=row['snapshot']; actual={k:actual[k] for k in model.state()}
    return first_difference(bit_tree(model.state()),bit_tree(actual))


def check_f(model,row):
    names=('connection','empathy','curiosity','achievement')
    if model.version=='old':
        actual=[f'{x:016x}' for x in row['old_x_bits']]
        return first_difference([bits(x) for x in model.x],actual,'old.x')
    snapshot=row['snapshot']
    actual=dict(x=[snapshot['nature'][k] for k in names],anchor=int(snapshot['anchor_time_ns']),h={},targets={})
    for slot in snapshot['slots']:
        root=slot['basis']['record_id']
        actual['h'][root]=[slot['current']['nature'][k] for k in names]
        actual['targets'][root]=[slot['target']['nature'][k] for k in names]
    return first_difference(bit_tree(model.state()),bit_tree(actual))


def run(case, resource, bridge, dut_rows, destination):
    family=case['family']; cls=C if family=='C' else F
    def initial():
        return cls(resource,case if family=='C' else case['version'],bridge.primitive)
    model=initial(); rows=[]; differences=[]
    for end in range(len(case['events'])+1):
        start=time.perf_counter_ns()
        if end:
            if family=='C': model.step(case['events'][end-1])
            else: model.step(case['events'][end-1],end-1)
        increment=time.perf_counter_ns()-start
        start=time.perf_counter_ns(); replay=initial()
        for j,event in enumerate(case['events'][:end]):
            if family=='C': replay.step(event)
            else: replay.step(event,j)
        replay_ns=time.perf_counter_ns()-start
        expected=bit_tree(model.state()); actual=bit_tree(replay.state())
        difference=first_difference(expected,actual,'independent-I23')
        if difference is None: difference=(check_c if family=='C' else check_f)(model,dut_rows[end])
        if difference: differences.append(dict(prefix=end,difference=difference))
        item=dict(prefix=end,incremental_ns=increment,replay_ns=replay_ns,state=deepcopy(expected),difference=difference)
        if family=='C':
            item['exact_baseline']={k:str(v) for k,v in model.exact.items()}
        rows.append(item)
    write(destination,dict(id=case['id'],purpose='resource_probe_not_mechanism_result',rows=rows,differences=differences))
    return dict(id=case['id'],family=family,version=case['version'],N=len(case['events']),
        independent_incremental_ns=sum(x['incremental_ns'] for x in rows),
        independent_replay_ns=sum(x['replay_ns'] for x in rows),
        dut_incremental_ns=sum(x['incremental_ns'] for x in dut_rows),dut_replay_ns=sum(x['replay_ns'] for x in dut_rows),
        dut_snapshot_ns=sum(x.get('snapshot_ns',0) for x in dut_rows),
        first_difference=differences[0] if differences else None,bytes=destination.stat().st_size)


def main():
    ap=argparse.ArgumentParser(); ap.add_argument('prepared',type=Path); ap.add_argument('binary',type=Path);ap.add_argument('output',type=Path);args=ap.parse_args()
    manifest=json.loads((args.prepared/'RUN-MANIFEST.json').read_text(encoding='utf-8'))
    for name,expected in manifest['prepared_files'].items():
        assert binding(args.prepared/name)==expected, name
    for name,expected in manifest['code_files'].items():
        assert binding(Path(__file__).parent/name)==expected, name
    assert binding(args.binary)==manifest['binary']
    args.output.mkdir(parents=True,exist_ok=False)
    write(args.output/'arithmetic-hand-vectors.json',check_hand_vectors())
    cases=json.loads((args.prepared/'probe-inputs.json').read_text(encoding='utf-8'))
    resources={k:json.loads((args.prepared/(k+'.json')).read_text(encoding='utf-8')) for k in ('C','F')}
    resources['F']['lambda']=struct.unpack('>d',bytes.fromhex(resources['F']['lambda_bits']))[0]
    summaries=[]; dut_f=None; start=time.perf_counter_ns()
    with (args.output/'bridge.log').open('w',encoding='utf-8') as log:
        bridge=Bridge(args.binary,log)
        try:
            for case in cases:
                family=case['family']
                if family=='C' or dut_f is None:
                    response=bridge.call(dict(op=family,resource=resources[family],case=case,
                        resource_ref=binding(args.prepared/(family+'.json')),input_ref=binding(args.prepared/'probe-inputs.json')))
                    write(args.output/(case['id']+'-dut.json'),response)
                    if family=='F': dut_f=response
                else: response=dut_f
                summaries.append(run(case,resources[family],bridge,response['rows'],args.output/(case['id']+'-independent.json')))
            write(args.output/'primitive-cache.json',{f'{name}:{bits_}':bits(v) for (name,bits_),v in bridge.cache.items()})
        finally: bridge.close()
    elapsed=time.perf_counter_ns()-start
    totals=json.loads((args.prepared/'plan-summary.json').read_text(encoding='utf-8'))
    rates={}
    for layer in ('dut','independent'):
        for family in ('C','F'):
            sample=[x for x in summaries if x['family']==family]
            if family=='F' and layer=='dut': sample=sample[:1] # adapter ran the pair once
            operations=sum(x['N']*(x['N']+1)//2 for x in sample)
            # F DUT timing is the pair; divide only when assigning per-reference
            # rate. Counts contain one row per version, so otherwise double cost.
            if family=='F' and layer=='dut': operations*=2
            rates[layer+'/'+family]=sum(x[layer+'_replay_ns'] for x in sample)/operations
    estimates={}
    for key,total in totals.items():
        layer,family=key.split('/'); prefix='dut' if layer=='DUT' else 'independent'
        rate=rates.get(prefix+'/'+family)
        if family=='CF': rate=rates[prefix+'/C']+rates[prefix+'/F']
        if rate is not None:
            estimates[key]=(total['sum_N']+total['prefix_recompute'])*rate/1e9
    projected=sum(estimates.values())
    write(args.output/'SUMMARY.json',dict(purpose='cost_only',manifest=binding(args.prepared/'RUN-MANIFEST.json'),
        rows=summaries,wall_seconds=elapsed/1e9,python_peak_working_set_bytes=peak_bytes(),
        primitive_cache_misses=bridge.primitive_misses,rates_ns_per_replayed_operation=rates,
        estimated_seconds_by_layer=estimates,projected_seconds_known_components=projected,
        gate='stop_over_4h' if projected>14400 else 'within_4h_known_components_only',
        unestimated_components=['G','candidate sensitivity','arithmetic-only','long-history rational numerator growth','full evidence serialization'],
        note='Projection is an estimate, not elapsed full-run time. Unestimated costs are additional, not zero.'))


if __name__=='__main__': main()
