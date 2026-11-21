"""Unit tests for the preflight gate used by the long-running GPU harness."""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
import time
from types import SimpleNamespace

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import gpu_common


def _process_list(stdout: str = '"python.exe","123","Console","1","1,000 K"') -> SimpleNamespace:
    return SimpleNamespace(stdout=stdout)


def test_quiet_gate_checks_once_per_minute(monkeypatch: pytest.MonkeyPatch) -> None:
    now = [100.0]
    calls = 0

    def run(*args: object, **kwargs: object) -> SimpleNamespace:
        nonlocal calls
        calls += 1
        return _process_list()

    monkeypatch.setattr(os, "name", "nt")
    monkeypatch.setenv("NEMU_P4_02_QUIET_WINDOW", "user-confirmed")
    monkeypatch.setattr(time, "monotonic", lambda: now[0])
    monkeypatch.setattr(subprocess, "run", run)
    monkeypatch.setattr(gpu_common, "_last_quiet_gate_check", None)

    gpu_common.quiet_gate()
    gpu_common.quiet_gate()
    now[0] += 59.9
    gpu_common.quiet_gate()
    assert calls == 1

    now[0] += 0.1
    gpu_common.quiet_gate()
    assert calls == 2


def test_quiet_gate_retries_enumeration_but_still_fails_closed(monkeypatch: pytest.MonkeyPatch) -> None:
    calls = 0
    sleeps: list[float] = []

    def run(*args: object, **kwargs: object) -> SimpleNamespace:
        nonlocal calls
        calls += 1
        if calls == 1:
            raise subprocess.CalledProcessError(1, ["tasklist"])
        return _process_list()

    monkeypatch.setattr(os, "name", "nt")
    monkeypatch.setenv("NEMU_P4_02_QUIET_WINDOW", "user-confirmed")
    monkeypatch.setattr(subprocess, "run", run)
    monkeypatch.setattr(time, "sleep", sleeps.append)
    monkeypatch.setattr(gpu_common, "_last_quiet_gate_check", None)

    gpu_common.quiet_gate()

    assert calls == 2
    assert sleeps == [0.1]


def test_quiet_gate_rejects_forbidden_process(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(os, "name", "nt")
    monkeypatch.setenv("NEMU_P4_02_QUIET_WINDOW", "user-confirmed")
    monkeypatch.setattr(subprocess, "run", lambda *args, **kwargs: _process_list(
        '"Runner.Worker.exe","7","Services","0","10,000 K"'))
    monkeypatch.setattr(gpu_common, "_last_quiet_gate_check", None)

    with pytest.raises(RuntimeError, match="Quiet window unavailable"):
        gpu_common.quiet_gate()
