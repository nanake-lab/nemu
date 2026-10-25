"""Read-only checks for the ADR 0026 documentation change; no runtime claims."""

import argparse
import ast
import hashlib
import json
import posixpath
import re
import subprocess
from datetime import datetime
from pathlib import Path
from zoneinfo import ZoneInfo


def git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root)


def blob_sha(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True)
    parser.add_argument("--source-manifest", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    register = "docs/traceability/diff-register.md"
    allowed = {
        "docs/adr/0026-continuous-cognition.md",
        "docs/traceability/continuous-cognition.md",
        register,
        "reports/continuous-cognition/REPORT.md",
        "reports/continuous-cognition/verify_docs.py",
        "reports/continuous-cognition/validation.json",
    }
    if args.source_manifest:
        manifest = json.loads(args.source_manifest.read_text(encoding="utf-8"))
        if manifest["base"]["sha"] != args.base:
            raise ValueError("source manifest base mismatch")
        base_paths = set(manifest["paths"])
        original = manifest["files"][register]["content"].encode("utf-8")
        expected_blob = manifest["files"][register]["sha"]
        compare_ref = "HEAD"  # local mirror commit, never described as upstream main
        source_mode = "github_connector_manifest_and_local_mirror"
    else:
        base_paths = set(git(root, "ls-tree", "-r", "--name-only", args.base).decode().splitlines())
        original = git(root, "show", f"{args.base}:{register}")
        expected_blob = git(root, "rev-parse", f"{args.base}:{register}").decode().strip()
        compare_ref = args.base
        source_mode = "git_checkout"
    changed = set(git(root, "diff", "--name-only", compare_ref).decode().splitlines())
    changed.update(git(root, "ls-files", "--others", "--exclude-standard").decode().splitlines())
    errors = []
    if blob_sha(original) != expected_blob:
        errors.append("baseline register bytes do not match GitHub blob SHA")
    unexpected = sorted(changed - allowed)
    if unexpected:
        errors.append(f"out-of-scope paths: {unexpected}")
    current = (root / register).read_bytes()
    if not current.startswith(original):
        errors.append("difference register does not preserve complete original byte prefix")
    added = current[len(original):].decode("utf-8")
    ids = re.findall(r"^\| (COG-\d+) \|", added, re.M)
    if ids != [f"COG-{i:02}" for i in range(1, 6)]:
        errors.append(f"unexpected new difference IDs: {ids}")
    markdown_paths = sorted(p for p in allowed if p.endswith(".md"))
    relative_links = 0
    for rel in markdown_paths:
        text = added if rel == register else (root / rel).read_text(encoding="utf-8")
        fences = [line for line in text.splitlines() if line.startswith("```")]
        if len(fences) % 2:
            errors.append(f"unbalanced code fences: {rel}")
        for number, line in enumerate(text.splitlines(), 1):
            if line != line.rstrip():
                errors.append(f"trailing whitespace: {rel}:{number}")
        for target in re.findall(r"\[[^\]]*\]\(([^)]+)\)", text):
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
                continue
            target_path = target.split("#", 1)[0]
            resolved = posixpath.normpath(posixpath.join(posixpath.dirname(rel), target_path))
            relative_links += 1
            if resolved.startswith("../") or resolved not in base_paths | allowed:
                errors.append(f"unresolved relative link: {rel} -> {target}")
    ast.parse(Path(__file__).read_text(encoding="utf-8"))
    check = subprocess.run(["git", "diff", "--check", compare_ref], cwd=root, capture_output=True, text=True)
    if check.returncode:
        errors.append(f"git diff --check exit {check.returncode}: {check.stdout}{check.stderr}")
    inspected = sorted(allowed - {"reports/continuous-cognition/validation.json"})
    result = {
        "status": "passed" if not errors else "failed",
        "scope": "documentation_only_not_runtime_or_behavior_acceptance",
        "base_commit": args.base,
        "source_mode": source_mode,
        "checked_at": datetime.now(ZoneInfo("Asia/Shanghai")).isoformat(),
        "checks": {
            "unexpected_paths": unexpected,
            "baseline_register_blob_sha": expected_blob,
            "baseline_register_blob_matches": blob_sha(original) == expected_blob,
            "register_original_prefix_preserved": current.startswith(original),
            "new_difference_ids": ids,
            "markdown_files_checked": len(markdown_paths),
            "relative_link_targets_checked": relative_links,
            "git_diff_check_exit_code": check.returncode,
        },
        "files_sha256": {rel: hashlib.sha256((root / rel).read_bytes()).hexdigest() for rel in inspected},
        "not_run": ["runtime implementation", "COG-A01..COG-A11", "GPU/model benchmarks", "full repository test suites", "independent cross-review"],
        "errors": errors,
    }
    output = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.write_text(output, encoding="utf-8")
    print(output, end="")
    return 0 if not errors else 1


if __name__ == "__main__":
    raise SystemExit(main())
