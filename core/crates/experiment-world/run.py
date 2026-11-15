"""Serial CPU execution of the complete frozen list; failures never retried."""
from __future__ import annotations
import argparse
import json
import subprocess
import shutil
import datetime
import sys
from validate_inputs import validate, stopped
from pathlib import Path
import q2

def main()->None:
    p=argparse.ArgumentParser();p.add_argument('package',type=Path);p.add_argument('results',type=Path);p.add_argument('binary',type=Path);args=p.parse_args()
    args.results.mkdir()
    with (args.results/'startup.json').open('x',encoding='utf-8') as f:json.dump({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary':str(args.binary),'package':str(args.package)},f)
    try:
        audit=validate(args.package)
    except (ValueError,OSError) as error:
        with (args.results/'structural-stop.json').open('x',encoding='utf-8') as f:json.dump({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'reason':str(error)},f)
        return
    with (args.results/'input-audit.json').open('x',encoding='utf-8') as f:json.dump(audit,f)
    runs=json.loads((args.package/'runs.json').read_text(encoding='utf-8'))
    for run in runs:
        if run['status']!='ready':continue
        if stopped(args.results):break
        try:
            validate(args.package,run['id'])
        except (ValueError,OSError) as error:
            with (args.results/'structural-stop.json').open('x',encoding='utf-8') as f:json.dump({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'next_unrun':run['id'],'reason':str(error)},f)
            break
        # Engineering reserve: one unit's uncompressed upper estimate < 1.3 GiB.
        if shutil.disk_usage(args.results).free < 2*1024**3:
            with (args.results/'resource-stop.json').open('x') as f:json.dump({'status':'failed','reason':'less than 2 GiB evidence reserve','next_unrun':run['id']},f)
            break
        print(f'{datetime.datetime.now(datetime.timezone.utc).isoformat()} BEGIN {run["id"]}',flush=True)
        with (args.results/(run['id']+'.log')).open('x',encoding='utf-8') as log:
            result=subprocess.run([str(args.binary),'run',str(args.package),run['id'],str(args.results/run['id'])],stdout=log,stderr=subprocess.STDOUT,check=False)
        print(f'{datetime.datetime.now(datetime.timezone.utc).isoformat()} END {run["id"]}: exit={result.returncode}',flush=True)
        if result.returncode:
            with (args.results/(run['id']+'.failure.json')).open('x') as f:json.dump({'status':'failed','exit_code':result.returncode},f)
            with (args.results/'structural-stop.json').open('x',encoding='utf-8') as f:json.dump({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'reason':'runner process failed; stop new dispatch pending diagnosis','run':run['id']},f)
            break
    if not stopped(args.results):q2.execute(args.package,args.results,args.binary)
    subprocess.run([sys.executable,'-X','utf8',str(Path(__file__).with_name('analyze.py')),str(args.package),str(args.results),str(args.results/'summary.json')],check=True)
if __name__=='__main__':main()
