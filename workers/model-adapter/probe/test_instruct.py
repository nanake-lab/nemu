"""CPU checks for dose binding and the manual-only chat boundary."""
from __future__ import annotations

import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys

import pytest

from chat import read_chat_coefficient
from instruct_runtime import MODEL, PLAN, REVISION, coefficient_for_ratio


def test_dose_is_relative_to_actual_direction_norm() -> None:
    assert coefficient_for_ratio(-0.2, 100, 0.5) == -40
    assert coefficient_for_ratio(0, 100, 0.5) == 0
    for ratio, reference, direction in [(math.nan, 1, 1), (1, 0, 1), (1, 1, 0),
                                         (math.inf, 1, 1), (1, -1, 1)]:
        with pytest.raises(ValueError):
            coefficient_for_ratio(ratio, reference, direction)


def test_nonzero_chat_dose_requires_bound_measurement(tmp_path: Path) -> None:
    assert read_chat_coefficient(0, None, "u") == 0
    with pytest.raises(ValueError):
        read_chat_coefficient(.3, None, "u")
    with pytest.raises(ValueError):
        read_chat_coefficient(.1, None, "u")
    source = tmp_path / "norms.json"
    resource = dict(model=dict(id=MODEL, revision=REVISION), u_sha256="u",
                    plan_sha256=hashlib.sha256(PLAN.read_bytes()).hexdigest(),
                    status="measured-informational-only", reference_norm=100, direction_norm=.5)
    source.write_text(json.dumps(resource), encoding="utf-8")
    assert read_chat_coefficient(.1, source, "u") == 20
    resource["u_sha256"] = "wrong"
    source.write_text(json.dumps(resource), encoding="utf-8")
    with pytest.raises(ValueError, match="binding"):
        read_chat_coefficient(.1, source, "u")


def test_ci_refuses_chat_before_model_loading() -> None:
    result = subprocess.run([sys.executable, str(Path(__file__).with_name("chat.py"))],
                            env={**os.environ, "CI": "true"}, capture_output=True, text=True,
                            timeout=30, check=False)
    assert result.returncode == 2
    assert "manual only" in result.stderr
    assert "Loading checkpoint" not in result.stderr
