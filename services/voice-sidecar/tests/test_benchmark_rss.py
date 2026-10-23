from pathlib import Path
import sys
import os
import subprocess

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import benchmark  # noqa: E402


def test_rss_requires_windows_without_spawning_a_process(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(os, "name", "posix")
    monkeypatch.setattr(subprocess, "run", lambda *args, **kwargs: pytest.fail("spawned subprocess"))
    with pytest.raises(RuntimeError, match="requires Windows"):
        benchmark.rss()


def test_step_traces_are_disabled_for_pressure_and_restored(monkeypatch: pytest.MonkeyPatch) -> None:
    target = "D:/evidence/traces"
    monkeypatch.setenv("NEMU_TTS_TRACE_DIR", target)
    with benchmark.trace_phase("stress"):
        assert "NEMU_TTS_TRACE_DIR" not in os.environ
    assert os.environ["NEMU_TTS_TRACE_DIR"] == target


def test_latency_keeps_step_traces_enabled(monkeypatch: pytest.MonkeyPatch) -> None:
    target = "D:/evidence/traces"
    monkeypatch.setenv("NEMU_TTS_TRACE_DIR", target)
    with benchmark.trace_phase("latency"):
        assert os.environ["NEMU_TTS_TRACE_DIR"] == target
