"""Explicit development operation; NEVER invoked by acceptance or CI.

Run only when intentionally reviewing a new encoding fixture set. Rust is the
authority, and verify.py independently checks Python/TS plus the manual oracle.
"""

from __future__ import annotations
import json
import subprocess
from pathlib import Path
from fixture_inputs import requests

ROOT = Path(__file__).resolve().parents[2]
queries = requests()
result = subprocess.run(
    [str(ROOT / "core/target/debug/examples/contract_probe")],
    input="".join(json.dumps(q) + "\n" for q in queries),
    text=True,
    capture_output=True,
    check=True,
    encoding="utf-8",
)
values = [json.loads(s) for s in result.stdout.splitlines()]
assert len(values) == len(queries)
for q, v in zip(queries, values, strict=True):
    assert v["ok"], (q["name"], v)
target = Path(__file__).parent / "golden"
target.mkdir(exist_ok=True)
(target / "legal.json").write_text(
    json.dumps(
        [dict(request=q, expected=v) for q, v in zip(queries, values, strict=True)],
        ensure_ascii=False,
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
illegal = []
for q in queries[::3]:
    for label, value in [
        (
            "missing-version",
            {k: v for k, v in q["value"].items() if k != "schema_version"},
        ),
        ("unknown-version", {**q["value"], "schema_version": 2}),
        ("unknown-field", {**q["value"], "undeclared": 0}),
    ]:
        illegal.append(dict(label=label, request={**q, "value": value}))
(target / "illegal.json").write_text(
    json.dumps(illegal, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
)
print(
    f"Frozen {len(values)} legal and {len(illegal)} illegal vectors. Review before commit."
)
