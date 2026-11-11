"""Normal CI checks the probe's adapters/oracles on one-step hand fixtures.

These checks are tooling validation, not a second formal §8 resource probe.
"""
import hashlib
import io
import json
import struct
import sys
from pathlib import Path
from c_oracle import C
from f_oracle import F
from prepare import resources
from probe import Bridge, check_c, check_f


def main():
    c,f,_=resources()
    f['lambda']=struct.unpack('>d',bytes.fromhex(f['lambda_bits']))[0]
    bridge=Bridge(Path(sys.argv[1]),io.StringIO())
    try:
        for family in ('C','F'):
            resource=c if family=='C' else f
            case=dict(version='A',p0=2,b0=0.25,events=[dict(id='ci-one',ordinal=1,time_ns=10**9,a=0.75,root='r1',first_position=1,target=0.25)])
            encoded=json.dumps(dict(resource=resource,case=case),sort_keys=True).encode()
            ref=dict(sha256=hashlib.sha256(encoded).hexdigest(),length=len(encoded))
            response=bridge.call(dict(op=family,resource=resource,case=case,resource_ref=ref,input_ref=ref))
            if family=='C':
                model=C(c,case,bridge.primitive)
                assert check_c(model,response['rows'][0]) is None
                model.step(case['events'][0])
                assert check_c(model,response['rows'][1]) is None
            else:
                for version in ('old','A1'):
                    model=F(f,version,bridge.primitive)
                    assert check_f(model,response['rows'][0]) is None
                    model.step(case['events'][0],0)
                    assert check_f(model,response['rows'][1]) is None
    finally: bridge.close()
    print('P5-71 tooling smoke: C genesis/one step, old F/A1 genesis/one step matched')


if __name__=='__main__': main()
