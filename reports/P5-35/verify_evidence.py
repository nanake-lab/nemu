"""Verify SHA256 over exact Git blob bytes, without worktree newline conversion.

Usage from this checkout: python reports/P5-35/verify_evidence.py [commit]
The manifest and every evidence file are read from the same commit (default HEAD).
This only hashes existing evidence; it never runs or rewrites the original tests.
"""

import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[2]
revision = sys.argv[1] if len(sys.argv) > 1 else "HEAD"
commit = subprocess.check_output(
    ["git", "rev-parse", "--verify", revision + "^{commit}"], cwd=root, text=True
).strip()


def blob(path: str) -> bytes:
    return subprocess.check_output(
        ["git", "cat-file", "blob", f"{commit}:reports/P5-35/{path}"], cwd=root
    )


manifest = json.loads(blob("EVIDENCE-SHA256.json"))
failures = [
    path
    for path, expected in manifest.items()
    if hashlib.sha256(blob(path)).hexdigest() != expected
]
print(
    json.dumps(
        dict(
            commit=commit,
            byte_source="git-blob",
            checked=len(manifest),
            failures=failures,
        )
    )
)
sys.exit(bool(failures))
