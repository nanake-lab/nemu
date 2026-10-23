"""A1 evidence: the resource regression fixtures have identical authority bytes in 3 languages."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(root / "core/contracts"))
from verify import probe  # noqa: E402

vectors = [v for v in json.loads((root / "core/contracts/golden/legal.json").read_text())
           if v["request"]["name"] == "ProactivityBoundaries"
           and any(c["kind"] == "__proto__" for c in v["request"]["value"]["cooldowns"])]
assert len(vectors) == 2
commands = {
    "Rust": [str(root / "core/target/debug/examples/contract_probe.exe")],
    "Python": [sys.executable, str(root / "core/contracts/python_probe.py")],
    "TypeScript": ["node", str(root / "packages/contracts/src/generated/.test-build/core/contracts/probe.js")],
}
evidence = {"working_tree_base": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
            "browser_sha256": hashlib.sha256((root / "packages/contracts/src/generated/echo/browser.ts").read_bytes()).hexdigest(),
            "languages": {}}
for name, command in commands.items():
    results = probe(command, [v["request"] for v in vectors])
    assert results == [v["expected"] for v in vectors], name
    evidence["languages"][name] = [dict(keys=[c["kind"] for c in row["value"]["cooldowns"]],
                                             canonical=row["canonical"], digest=row["digest"],
                                             mismatches=0) for row in results]
    print(f"{name}: 2 A1 vectors, canonical/digest/value/protobuf = frozen authority, 0 mismatches")
output = root / sys.argv[1]
assert not output.exists(), output
output.write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
