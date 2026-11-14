"""Serial CPU execution of the complete frozen list; failures never retried."""
from __future__ import annotations
import argparse
import json
import subprocess
import shutil
from pathlib import Path
import q2

def main()->None:
    p=argparse.ArgumentParser();p.add_argument('package',type=Path);p.add_argument('results',type=Path);p.add_argument('binary',type=Path);args=p.parse_args()
    args.results.mkdir()
    runs=json.loads((args.package/'runs.json').read_text(encoding='utf-8'))
    for run in runs:
        if run['status']!='ready':continue
        # Engineering reserve: one unit's uncompressed upper estimate < 1.3 GiB.
        if shutil.disk_usage(args.results).free < 2*1024**3:
            with (args.results/'resource-stop.json').open('x') as f:json.dump({'status':'failed','reason':'less than 2 GiB evidence reserve','next_unrun':run['id']},f)
            break
        print(f'BEGIN {run["id"]}',flush=True)
        with (args.results/(run['id']+'.log')).open('x',encoding='utf-8') as log:
            result=subprocess.run([str(args.binary),'run',str(args.package),run['id'],str(args.results/run['id'])],stdout=log,stderr=subprocess.STDOUT,check=False)
        print(f'END {run["id"]}: exit={result.returncode}',flush=True)
        if result.returncode:
            with (args.results/(run['id']+'.failure.json')).open('x') as f:json.dump({'status':'failed','exit_code':result.returncode},f)
    q2.execute(args.package,args.results,args.binary)
    subprocess.run(['python',str(Path(__file__).with_name('analyze.py')),str(args.package),str(args.results),str(args.results/'summary.json')],check=True)
if __name__=='__main__':main()
