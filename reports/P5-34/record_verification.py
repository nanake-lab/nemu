"""Record existing local evidence and code hashes; does not run acceptance."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).parent
RUN = HERE / "verification" / "idle-20261024"


def git(*args: str) -> str:
    return subprocess.check_output(
        ["git", *args], cwd=ROOT, encoding="utf-8"
    ).strip()


def main() -> None:
    code = git("rev-parse", "HEAD")
    paths = git(
        "diff", "--name-only", "origin/main", "--", "core", "packages", "workers"
    ).splitlines()
    source = {
        "implementation_commit": code,
        "main": git("rev-parse", "origin/main"),
        "files": {
            path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
            for path in paths
        },
        "git_blob_oids": {path: git("rev-parse", f"{code}:{path}") for path in paths},
    }
    (RUN / "tested-source-sha256.json").write_text(
        json.dumps(source, indent=2) + "\n", encoding="utf-8"
    )
    entries = []
    for path in sorted(RUN.glob("attempt-*/*.json")):
        item = json.loads(path.read_text(encoding="utf-8-sig"))
        if "step" not in item:
            continue
        log = path.with_suffix(".log")
        actual = hashlib.sha256(log.read_bytes()).hexdigest()
        if actual != item["logSha256"]:
            raise ValueError(f"log hash mismatch: {path}")
        entries.append({"metadata": path.relative_to(HERE).as_posix(), **item})
    (RUN / "command-index.json").write_text(
        json.dumps(entries, indent=2) + "\n", encoding="utf-8"
    )
    checks = {entry["step"]: entry for entry in entries}
    for acceptance, scope, commands in (
        ("SYS21", "P5-34 q/V codec and Rust semantic rejection cases", ("contracts", "types")),
        ("D10.8", "P5-34 IDL generation, compatibility and three-language bytes", ("contracts", "compat", "bytes")),
    ):
        if not all(checks[name]["exitCode"] == 0 for name in commands):
            raise ValueError(f"acceptance commands failed: {acceptance}")
        folder = HERE / acceptance
        folder.mkdir(exist_ok=True)
        (folder / "result.json").write_text(
            json.dumps({
                "status": "passed", "scope": scope,
                "implementation_commit": code,
                "evidence": [checks[name]["metadata"] for name in commands],
                "whole_system_acceptance": False,
            }, indent=2) + "\n", encoding="utf-8"
        )
    for acceptance in ("I23", "SYS17", "AC2-08", "AC2-19", "AC2-21"):
        folder = HERE / acceptance
        folder.mkdir(exist_ok=True)
        (folder / "result.json").write_text(
            json.dumps({
                "status": "not_run",
                "reason": "Real effective-H G replay implementation is unavailable on main; storage seam tests do not prove production migration or law behavior.",
                "implementation_commit": code,
                "evidence": "BLOCKED.md",
            }, indent=2) + "\n", encoding="utf-8"
        )
    print(json.dumps({"source_files": len(paths), "commands": len(entries)}))


if __name__ == "__main__":
    main()
