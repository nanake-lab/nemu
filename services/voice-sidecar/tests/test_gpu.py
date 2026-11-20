"""Opt-in smoke through the real supervised V5 entry; CI runs offline fixtures."""
from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys

import pytest


def test_real_gpu_warmup_when_explicitly_enabled() -> None:
    if os.getenv("NEMU_RUN_TTS_GPU_TESTS") != "1":
        pytest.skip("GPU smoke is opt-in")
    root = Path(__file__).resolve().parents[1]
    # quiet_gate checks the confirmed Windows window before importing the model.
    result = subprocess.run(
        [sys.executable, "-c",
         "from gpu_common import quiet_gate; quiet_gate(); "
         "from nemu_voice_sidecar.gsv import GsvEngine; "
         "e = GsvEngine(); e.warmup(); "
         "assert e.health()['modelLoaded'] and e.health()['vram']; e.close()"],
        cwd=root, capture_output=True, text=True,
    )
    assert result.returncode == 0, result.stderr
