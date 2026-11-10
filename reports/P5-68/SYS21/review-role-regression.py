"""Run the review regression against the already-built three language probes.

The same requests enter normal CI through fixture_law_state and golden vectors.
"""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "core/contracts"))
from fixture_law_state import illegal_law_state, legal_law_state  # noqa: E402
from verify import probe  # noqa: E402

bad = [
    v["request"] for v in illegal_law_state() if v["label"] == "P5-68-internal-revisit"
]
good = next(
    q
    for q in legal_law_state()
    if q["name"] == "FastEffectRecord"
    and q["value"]["basis"]["kind"] == 2
    and q["value"]["role"] == 2
)
commands = {
    "Rust": [str(ROOT / "core/target/debug/examples/contract_probe.exe")],
    "Python": [sys.executable, str(ROOT / "core/contracts/python_probe.py")],
    "TypeScript": [
        "node",
        str(
            ROOT
            / "packages/contracts/src/generated/.test-build/core/contracts/probe.js"
        ),
    ],
}
failures = []
for language, command in commands.items():
    for request, result in zip(bad, probe(command, bad), strict=True):
        print(language, "internal + revisit", request["op"], "accepted=", result["ok"])
        if result["ok"]:
            failures.append((language, request["op"]))
    encoded = probe(command, [good])[0]
    assert encoded["ok"], (language, "internal + target update encode")
    for op in ("canonical", "protobuf"):
        decoded = probe(
            command,
            [dict(name=good["name"], op=op, hex=encoded[op], limits=good["limits"])],
        )[0]
        assert decoded == encoded, (language, "internal + target update", op)
    print(language, "internal + target update encode/canonical/protobuf PASS")
assert not failures, failures
print(
    "PASS: internal revisit rejected on 9 paths; internal target update accepted on 9 paths"
)
