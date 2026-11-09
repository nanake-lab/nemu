"""Process-only credential handoff; no network or key persistence in this module."""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent


def write(path: Path, data: object) -> None:
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')


def worker(session: Path) -> int:
    if not os.environ.get('DEEPSEEK_API_KEY'):
        write(session/'worker-status.json',{'status':'missing_process_key'})
        return 2
    write(session/'worker-status.json',{'status':'waiting_for_frozen_pilot','pid':os.getpid(),
        'model':'deepseek-flash','key_in_process_environment':True,'key_persisted':False,
        'utc':datetime.now(timezone.utc).isoformat()})
    deadline=time.monotonic()+3600  # One-hour handoff bound; no automatic restart.
    gate=session/'execute.json'
    while not gate.exists():
        if time.monotonic()>deadline:
            write(session/'worker-status.json',{'status':'preparation_timeout_no_call'})
            return 2
        time.sleep(1)
    request=json.loads(gate.read_text(encoding='utf-8-sig'))
    if request.get('model')!='deepseek-flash' or request.get('phase')!='pilot' or not re.fullmatch('[0-9a-f]{40}',request.get('head','')):
        write(session/'worker-status.json',{'status':'invalid_handoff_no_call'})
        return 2
    command=[sys.executable,'-B',str(HERE/'ds_pilot.py'),'--session',str(session),'--expected-head',request['head']]
    result=subprocess.run(command,check=False)
    write(session/'worker-status.json',{'status':'pilot_process_exited','exit_code':result.returncode,
        'utc':datetime.now(timezone.utc).isoformat()})
    return result.returncode


def main() -> int:
    parser=argparse.ArgumentParser()
    parser.add_argument('mode',choices=('pilot','worker'))
    parser.add_argument('--session',required=True,type=Path)
    parser.add_argument('--expected-head')  # Entry metadata only; worker waits for frozen execute.json.
    args=parser.parse_args()
    session=args.session.resolve()
    if session!=Path('D:/Dev/cache/R-STEER-01/ds-flash-pilot-01').resolve():
        raise ValueError('Use the designated one-shot session')
    if args.mode=='worker':
        try:
            return worker(session)
        except Exception:
            # Never print an exception that might contain provider/credential data.
            write(session/'worker-status.json',{'status':'worker_stopped_no_retry'})
            return 2
        finally:
            os.environ.pop('DEEPSEEK_API_KEY',None)
    if (session/'worker-started.json').exists():
        raise ValueError('Preserve existing worker; no automatic retry')
    with (session/'worker.stdout').open('xb') as stdout,(session/'worker.stderr').open('xb') as stderr:
        process=subprocess.Popen([sys.executable,'-B',str(Path(__file__).resolve()),'worker','--session',str(session)],
            stdout=stdout,stderr=stderr,stdin=subprocess.DEVNULL,
            creationflags=subprocess.DETACHED_PROCESS|subprocess.CREATE_NEW_PROCESS_GROUP)
    write(session/'worker-started.json',{'pid':process.pid,'model':'deepseek-flash','network_started':False})
    return 0


if __name__=='__main__':
    raise SystemExit(main())
