"""Hardware smoke only; benchmark.py is the separate carrier-resident acceptance."""
from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys

import pytest


def test_real_gpu_warmup_when_explicitly_enabled() -> None:
    if os.getenv('NEMU_RUN_TTS_GPU_TESTS') != '1':
        pytest.skip('GPU test not requested; CI never downloads models or allocates CUDA')
    if os.name != 'nt' or os.getenv('NEMU_P4_02_QUIET_WINDOW') != 'user-confirmed':
        pytest.skip('Windows GPU quiet window not confirmed')
    if not os.getenv('NEMU_TTS_CACHE'):
        pytest.skip('Hash-verified local TTS model cache is not configured')
    root = Path(__file__).resolve().parents[1]
    # This imports no torch before the same hard quiet gate as the benchmark.
    result = subprocess.run([sys.executable, '-c',
                             'from gpu_common import quiet_gate; quiet_gate(); '
                             'from nemu_voice_sidecar.qwen import QwenEngine; QwenEngine().warmup()'],
                            cwd=root, capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
