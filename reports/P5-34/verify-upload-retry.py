"""Exercise the actual Windows workflow upload conditions and PowerShell gate."""

from __future__ import annotations

import ast
import copy
import json
import os
import re
import subprocess
import tempfile
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/windows.yml"
workflow = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
baseline = yaml.safe_load(
    subprocess.check_output(
        ["git", "show", "origin/main:.github/workflows/windows.yml"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
    )
)
steps = workflow["jobs"]["windows-tests"]["steps"]
uploads = [step for step in steps if step.get("id", "").startswith("w02_upload_")]
gate = next(step for step in steps if step.get("name") == "Require W02 evidence upload")
assert len(uploads) == 3
assert len({step["with"]["name"] for step in uploads}) == 3
for step in uploads:
    assert step["uses"] == "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02"
    assert step["with"]["path"] == "${{ env.NEMU_W02_EVIDENCE }}/"
    assert step["with"]["if-no-files-found"] == "error"
    assert step["continue-on-error"] is True
    assert step["timeout-minutes"] == 6
    assert "overwrite" not in step["with"]
assert not gate.get("continue-on-error", False)
for number in range(1, 4):
    assert gate["env"][f"UPLOAD_{number}_OUTCOME"] == f"${{{{ steps.w02_upload_{number}.outcome }}}}"
    assert gate["env"][f"UPLOAD_{number}_ID"] == f"${{{{ steps.w02_upload_{number}.outputs.artifact-id }}}}"
assert workflow["jobs"]["windows-tests"]["timeout-minutes"] == 60
for number, delay in ((2, 20), (3, 40)):
    wait = next(step for step in steps if step.get("name") == f"Wait before W02 upload retry {number}")
    assert wait["if"] == uploads[number - 1]["if"]
    assert wait["run"] == f"Start-Sleep -Seconds {delay}"

# Everything before the old W02 upload, including every test, stays identical.
assert steps[: steps.index(uploads[0])] == baseline["jobs"]["windows-tests"]["steps"][:-1]
unchanged = copy.deepcopy(workflow)
unchanged["jobs"]["windows-tests"]["steps"] = baseline["jobs"]["windows-tests"]["steps"]
assert unchanged == baseline


def enabled(expression: str, outcomes: dict[str, str], evidence: bool, cancelled: bool) -> bool:
    expression = expression.removeprefix("${{").removesuffix("}}").strip()
    expression = expression.replace("!cancelled()", repr(not cancelled))
    expression = expression.replace("always()", "True")
    expression = expression.replace("env.NEMU_W02_EVIDENCE", repr("evidence" if evidence else ""))
    expression = re.sub(
        r"steps\.(w02_upload_[123])\.outcome",
        lambda match: repr(outcomes[match[1]]),
        expression,
    ).replace("&&", " and ")
    tree = ast.parse(expression, mode="eval")
    allowed = (ast.Expression, ast.BoolOp, ast.And, ast.Compare, ast.Eq, ast.NotEq, ast.Constant)
    assert all(isinstance(node, allowed) for node in ast.walk(tree)), expression
    return bool(eval(compile(tree, "<workflow-condition>", "eval"), {"__builtins__": {}}, {}))


cases = [
    ("first_success", ["success", "failure", "failure"], ["101", "", ""], True, False, [1], True),
    ("recover_second", ["failure", "success", "failure"], ["", "102", ""], True, False, [1, 2], True),
    ("recover_third", ["failure", "failure", "success"], ["", "", "103"], True, False, [1, 2, 3], True),
    ("all_fail", ["failure"] * 3, [""] * 3, True, False, [1, 2, 3], False),
    ("step_cancel_then_recover", ["cancelled", "success", "failure"], ["", "104", ""], True, False, [1, 2], True),
    ("all_steps_cancelled", ["cancelled"] * 3, [""] * 3, True, False, [1, 2, 3], False),
    ("success_without_id", ["success"] * 3, [""] * 3, True, False, [1], False),
    ("success_invalid_id", ["success"] * 3, ["invalid"] * 3, True, False, [1], False),
    ("success_zero_id", ["success"] * 3, ["0"] * 3, True, False, [1], False),
    ("failed_with_stale_id", ["failure"] * 3, ["105"] * 3, True, False, [1, 2, 3], False),
    ("skipped_with_evidence", ["skipped"] * 3, [""] * 3, True, False, [1, 2, 3], False),
    ("unknown_with_id", ["unknown"] * 3, ["106"] * 3, True, False, [1, 2, 3], False),
    ("cancelled_job", ["success"] * 3, ["107"] * 3, True, True, [], False),
    ("unselected_scope", ["success"] * 3, ["108"] * 3, False, False, [], None),
]
temp_base = Path(r"D:\Dev\cache\P5-34\tmp").resolve()
temp_base.mkdir(parents=True, exist_ok=True)
results = []
with tempfile.TemporaryDirectory(prefix="upload-gate-", dir=temp_base) as directory:
    temp_root = Path(directory).resolve()
    assert temp_root.is_relative_to(temp_base)
    script = temp_root / "gate.ps1"
    script.write_text(gate["run"], encoding="utf-8")
    for name, proposed, ids, evidence, cancelled, expected_attempts, expected_pass in cases:
        outcomes = {step["id"]: "skipped" for step in uploads}
        actual_ids = [""] * 3
        attempts = []
        for index, step in enumerate(uploads):
            if enabled(step["if"], outcomes, evidence, cancelled):
                attempts.append(index + 1)
                outcomes[step["id"]] = proposed[index]
                actual_ids[index] = ids[index]
        assert attempts == expected_attempts, (name, attempts)
        run_gate = enabled(gate["if"], outcomes, evidence, cancelled)
        assert run_gate == (expected_pass is not None), name
        if run_gate:
            environment = os.environ.copy()
            for index, step in enumerate(uploads, 1):
                environment[f"UPLOAD_{index}_OUTCOME"] = outcomes[step["id"]]
                environment[f"UPLOAD_{index}_ID"] = actual_ids[index - 1]
            environment["GITHUB_STEP_SUMMARY"] = str(temp_root / f"{name}.md")
            result = subprocess.run(
                ["pwsh", "-NoProfile", "-File", str(script)],
                env=environment,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
                timeout=15,
                check=False,
            )
            assert (result.returncode == 0) == expected_pass, (name, result.stdout, result.stderr)
        results.append({"case": name, "attempts": attempts, "gate_pass": expected_pass, "verified": True})
print(json.dumps({"cases": results, "yaml_version": yaml.__version__, "unchanged_tests_and_security_gate": True}, indent=2))
