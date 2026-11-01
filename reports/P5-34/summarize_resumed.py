"""Record current P5-34 evidence without rewriting earlier runs."""
from __future__ import annotations

import hashlib
import json
import subprocess
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "reports/P5-34/verification/resumed-20261026/summary.json"


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT).decode("utf-8").strip()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    if OUT.exists():
        raise SystemExit("Refusing to replace existing evidence")
    checks = []
    for path in sorted(OUT.parent.glob("attempt-*/*.json")):
        item = json.loads(path.read_text(encoding="utf-8-sig"))
        log = path.with_suffix(".log")
        assert digest(log) == item["logSha256"], str(log)
        checks.append({"metadata": path.relative_to(ROOT).as_posix(), **item})
    names = ("EvidenceAllocationV3", "TendencyV3", "EvidenceGroupV3", "RSnapshotV3")
    vectors = {}
    for kind in ("legal", "illegal", "unicode"):
        path = ROOT / f"core/contracts/golden/{kind}.json"
        rows = json.loads(path.read_text(encoding="utf-8"))
        counts = Counter(row["request"]["name"] for row in rows)
        vectors[kind] = {"total": len(rows), "v3": {n: counts[n] for n in names}, "sha256": digest(path)}
    baseline = {}
    for path in sorted((ROOT / "core/contracts/baselines/p5-32").rglob("*")):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT).as_posix()
        actual = digest(path)
        published = hashlib.sha256(subprocess.check_output(["git", "show", f"origin/main:{relative}"], cwd=ROOT)).hexdigest()
        assert actual == published, relative
        baseline[relative] = actual
    changed = git("diff", "--name-only", "origin/main").splitlines()
    allowed = ("core/contracts/", "core/crates/core-types/", "packages/contracts/src/generated/", "workers/irl/generated/", "workers/drive/generated/", "workers/model-adapter/generated/")
    source = {p: digest(ROOT / p) for p in changed if p.startswith(allowed)}
    assert not any(p.startswith("core/crates/core-store/") for p in changed)
    generated = {}
    for name in ("__init__.py", "bridge.py", "runtime.py", "typed_api.py", "types.py", "schema.json"):
        hashes = {worker: digest(ROOT / f"workers/{worker}/generated/echo/{name}") for worker in ("irl", "drive", "model-adapter")}
        assert len(set(hashes.values())) == 1, name
        generated[name] = hashes
    summary = {
        "recordedUtc": datetime.now(timezone.utc).isoformat(),
        "head": git("rev-parse", "HEAD"),
        "main": git("rev-parse", "origin/main"),
        "checks": checks,
        "goldenVectors": vectors,
        "unchangedP532Baseline": baseline,
        "sourceSha256": source,
        "generatedParity": generated,
        "coreStoreDiff": [],
        "supplementaryChecks": {"remaining": "not_run: global resource guard", "doctests": "not_run: global resource guard; full workspace already attempted and failed"},
        "codeChangesSinceWorkspaceStarted": git("diff", "17ef44ddf2d794bfc77373bc1349c91a12b959bf", "HEAD", "--name-only", "--", "core", "packages", "workers").splitlines(),
        "historicalEvidence": "All other verification directories and top-level acceptance result.json files describe closed PR #72, not current acceptance.",
        "runtimeAcceptance": {name: "not_run; outside contract scope" for name in ("AC2-08", "AC2-19", "AC2-21", "I23", "SYS17")},
    }
    OUT.write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"Recorded {len(checks)} checks; failures: {[c['step'] for c in checks if c['exitCode'] != 0]}")


if __name__ == "__main__":
    main()
