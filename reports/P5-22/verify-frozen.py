"""Re-encode/decode every pre-P5-22 vector and record comparable SHA-256 evidence."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'core/contracts'))
from verify import probe  # noqa: E402

BASE = sys.argv[2] if len(sys.argv) > 2 else '91b31ed'

def stable(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, ensure_ascii=True, separators=(',', ':')).encode('ascii')

def sha(value: object) -> str:
    return hashlib.sha256(stable(value)).hexdigest()

def load_at(name: str) -> list[dict[str, object]]:
    return json.loads(subprocess.check_output(['git', 'show', f'{BASE}:core/contracts/golden/{name}.json'], cwd=ROOT))

legal, illegal, unicode = (load_at(name) for name in ('legal', 'illegal', 'unicode'))
queries, expected = [], []
for vector in legal + unicode:
    request, want = vector['request'], vector['expected']
    queries.append(request)
    expected.append(want)
    if want['ok']:
        for op in ('canonical', 'protobuf'):
            queries.append(dict(name=request['name'], op=op, hex=want[op], limits=request['limits']))
            expected.append(want)
for vector in illegal:
    queries.append(vector['request'])
    expected.append(dict(ok=False))
summary = dict(baseline=subprocess.check_output(['git','rev-parse',BASE],cwd=ROOT,text=True).strip(), files={}, languages={})
for name, rows in [('legal', legal), ('illegal', illegal), ('unicode', unicode)]:
    current = json.loads((ROOT / f'core/contracts/golden/{name}.json').read_text(encoding='utf-8'))
    assert rows == current[:len(rows)], name
    summary['files'][name] = dict(old_count=len(rows), current_count=len(current), prefix_exact=True,
                                  baseline_sha256=sha(rows), current_old_prefix_sha256=sha(current[:len(rows)]))
commands = {
    'Rust': [str(ROOT / 'core/target/debug/examples/contract_probe.exe')],
    'Python': [sys.executable, str(ROOT / 'core/contracts/python_probe.py')],
    'TypeScript': ['node', str(ROOT / 'packages/contracts/src/generated/.test-build/core/contracts/probe.js')],
}
for language, command in commands.items():
    actual = probe(command, queries)
    for index, (got, want) in enumerate(zip(actual, expected, strict=True)):
        assert got['ok'] == want['ok'], (language,index)
        if want['ok']:
            assert got == want, (language,index)
    accepted = [row for row in actual if row['ok']]
    reference = [row for row in expected if row['ok']]
    evidence = dict(cases=len(actual), rejected=sum(not row['ok'] for row in actual), mismatches=0)
    for key in ('canonical', 'protobuf', 'digest'):
        evidence[key+'_sha256'] = sha([row[key] for row in accepted])
        evidence[key+'_baseline_sha256'] = sha([row[key] for row in reference])
        assert evidence[key+'_sha256'] == evidence[key+'_baseline_sha256']
    summary['languages'][language] = evidence
    print(language, 'frozen encode/decode/hash comparisons: 0 mismatches', flush=True)
output=ROOT / sys.argv[1]
assert not output.exists(), output
output.write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
