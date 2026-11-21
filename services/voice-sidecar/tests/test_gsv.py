from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys

from fastapi.testclient import TestClient
import pytest

from nemu_voice_sidecar.app import create_app
from nemu_voice_sidecar.gsv_worker import verify_files
from nemu_voice_sidecar.voices import VoiceBank


def test_default_worker_with_incomplete_assets_degrades_and_can_retry(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path,
) -> None:
    monkeypatch.setenv("NEMU_GSV_ROOT", str(tmp_path))
    with TestClient(create_app(bank=VoiceBank(tmp_path / "voice.json"))) as client:
        for _ in range(2):
            state = client.post("/warmup").json()
            assert state["model"] == "gpt-sovits-v5turbo"
            assert not state["ready"] and not state["busy"] and state["vram"] is None
            assert state["reasons"].count("MODEL_FILES_MISSING") == 1
            assert client.get("/health").status_code == 200


def test_corrupt_upstream_is_rejected_before_import(tmp_path: Path) -> None:
    target = tmp_path / "GPT_SoVITS/TTS_infer_pack/TTS.py"
    target.parent.mkdir(parents=True)
    target.write_bytes(b"changed")
    with pytest.raises(RuntimeError, match="MODEL_HASH_MISMATCH"):
        verify_files(tmp_path)


def test_inference_process_refuses_network_before_dns() -> None:
    # Install the irreversible interpreter hook only in this short-lived child.
    code = """import socket, sys
from nemu_voice_sidecar.gsv_worker import _offline
sys.addaudithook(_offline)
try:
    socket.getaddrinfo('invalid.example', 443)
except PermissionError as error:
    assert str(error) == 'OFFLINE_RUNTIME'
else:
    raise AssertionError('DNS allowed')
"""
    source = str(Path(__file__).resolve().parents[1] / "src")
    subprocess.run([sys.executable, "-c", code], env={**os.environ, "PYTHONPATH": source},
                   check=True)
