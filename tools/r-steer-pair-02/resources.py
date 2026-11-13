"""Fresh mixed-phase quiet window with aggregate resource guarding."""
from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from typing import Any

def category(name: str, command: str, path: str) -> str | None:
    name, command, path = name.lower().removesuffix('.exe'), command.lower(), path.lower()
    if name in ('runner.listener', 'runner.worker'):
        return 'ci_runner'
    if name in ('cargo', 'rustc', 'clippy-driver', 'cl', 'link', 'cmake', 'ninja', 'msbuild', 'nvcc'):
        return 'compilation_or_build'
    if re.search(r'[/\\]target[/\\](debug|release)[/\\]deps[/\\]', path) or re.fullmatch(r'core_[a-z_]+-[0-9a-f]+', name):
        return 'native_unit_tests'
    if name.startswith(('python', 'node', 'electron')) and re.search(r'\b(pytest|unittest|vitest|playwright|jest)\b', command):
        return 'script_or_ui_tests'
    if (name in ('blender', 'ollama', 'llama-server', 'llama-cli', 'llama-bench')
            or re.fullmatch(r'c[0-9]_[a-z0-9_]*(probe|forward)', name)):
        return 'other_model_or_render_work'
    return None


def snapshot(owned_pid: int | None = None) -> dict[str, Any]:
    # Names/paths/command lines are transient classification inputs, never emitted or saved.
    script = "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); @(Get-CimInstance Win32_Process -ErrorAction Stop | Select-Object ProcessId,Name,CommandLine,ExecutablePath,WorkingSetSize) | ConvertTo-Json -Compress"
    data = subprocess.run(['powershell.exe','-NoProfile','-Command',script], capture_output=True,
        text=True, encoding='utf-8', errors='strict', timeout=20, check=True,
        creationflags=subprocess.CREATE_NO_WINDOW)
    rows = json.loads(data.stdout)
    if not isinstance(rows, list) or not rows:
        raise ValueError('Incomplete resource snapshot')
    counts: dict[str, int] = {}
    for row in rows:
        if owned_pid is not None and int(row['ProcessId']) == owned_pid:
            continue
        found = category(str(row['Name']), str(row.get('CommandLine') or ''), str(row.get('ExecutablePath') or ''))
        if found:
            counts[found] = counts.get(found, 0)+1
    return {'utc':datetime.now(timezone.utc).isoformat(), 'process_count':len(rows),
        'working_set_sum_bytes':sum(int(r.get('WorkingSetSize') or 0) for r in rows),
        'heavy_load_categories':counts,
        'coverage':'known build/test/model/render categories; user must also pause other heavy work'}


