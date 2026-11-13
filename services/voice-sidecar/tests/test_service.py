from __future__ import annotations

from collections.abc import Callable
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
from threading import Event
import time

from fastapi.testclient import TestClient
import pytest
from starlette.websockets import WebSocketDisconnect

from nemu_voice_sidecar.app import create_app
from nemu_voice_sidecar.backend import Request
from nemu_voice_sidecar.voices import Reference, VoiceBank


class FakeEngine:
    def __init__(self, *, fail: bool = False, slow: bool = False) -> None:
        self.fail = fail
        self.slow = slow
        self.stopped = Event()
        self.calls: list[Request] = []

    def health(self) -> dict[str, object]:
        return {"vram": None, "reasons": []}

    def warmup(self) -> None:
        if self.fail:
            raise RuntimeError("mock warmup failure")

    def stream(self, request: Request, reference: Reference, cancel: Event,
               emit: Callable[[bytes], None]) -> None:
        self.calls.append(request)
        try:
            for _ in range(10000 if self.slow else 3):
                if cancel.is_set():
                    return
                emit(struct.pack("<ff", 0.25, -0.25))
                if self.slow:
                    time.sleep(0.005)
        finally:
            self.stopped.set()


@pytest.fixture
def bank(tmp_path: Path) -> VoiceBank:
    audio = tmp_path / "synthetic.wav"
    # Transport fake only, never loaded by a real cloning model.
    audio.write_bytes(b"fixture-placeholder")
    ref = {"path": audio.name, "text": "synthetic fixture", "provenance": "VoiceDesign",
           "source": "test-double; not real audio", "license": "CC0-1.0",
           "cloneAllowed": True, "sha256": hashlib.sha256(audio.read_bytes()).hexdigest()}
    path = tmp_path / "voice.json"
    path.write_text(json.dumps({"id": "test", "references": {
        lang: {"neutral": ref, "happy": ref} for lang in ("zh", "en", "ja")}}), encoding="utf-8")
    return VoiceBank(path)


def payload(lang: str = "zh", style: str = "neutral") -> dict[str, object]:
    return {"text": "test", "lang": lang, "style": style, "voiceId": "test", "sampleRate": 24000}


def test_degraded_startup_and_failed_warmup_survive(tmp_path: Path) -> None:
    with TestClient(create_app(FakeEngine(fail=True), VoiceBank(tmp_path / "missing.json"))) as client:
        assert client.get("/health").status_code == 200
        assert client.post("/warmup").json()["status"] == "degraded"
        assert client.get("/health").json()["ready"] is False


def test_default_backend_missing_configuration_stays_alive(monkeypatch: pytest.MonkeyPatch,
                                                           tmp_path: Path) -> None:
    monkeypatch.delenv("NEMU_TTS_CACHE", raising=False)
    with TestClient(create_app(bank=VoiceBank(tmp_path / "missing.json"))) as client:
        assert client.get("/health").json()["status"] == "degraded"
        assert "MODEL_CACHE_NOT_CONFIGURED" in client.post("/warmup").json()["reasons"]
        assert client.get("/health").status_code == 200


@pytest.mark.parametrize("lang", ["zh", "en", "ja"])
@pytest.mark.parametrize("style", ["neutral", "happy"])
def test_three_languages_pcm_and_reference_selection(bank: VoiceBank, lang: str, style: str) -> None:
    engine = FakeEngine()
    with TestClient(create_app(engine, bank)) as client:
        assert client.post("/warmup").json()["ready"] is True
        with client.websocket_connect("/synthesize") as socket:
            socket.send_json(payload(lang, style))
            for _ in range(3):
                assert struct.unpack("<ff", socket.receive_bytes()) == (0.25, -0.25)
            with pytest.raises(WebSocketDisconnect) as closed:
                socket.receive_bytes()
            assert closed.value.code == 1000
    assert engine.calls[0].lang == lang
    assert engine.calls[0].style == style


def test_cancel_suppresses_emission_and_releases_lane(bank: VoiceBank) -> None:
    engine = FakeEngine(slow=True)
    with TestClient(create_app(engine, bank)) as client:
        client.post("/warmup")
        with client.websocket_connect("/synthesize") as socket:
            socket.send_json(payload())
            socket.receive_bytes()
            start = time.perf_counter()
            socket.send_json({"cancel": True})
            # Some frames can already be in flight at send time; bound last arrival.
            while True:
                try:
                    socket.receive_bytes()
                    assert (time.perf_counter() - start) * 1000 <= 50
                except WebSocketDisconnect as exc:
                    assert exc.code == 1000
                    break
            assert (time.perf_counter() - start) * 1000 <= 50
        assert engine.stopped.wait(1)
        assert client.get("/health").json()["busy"] is False


@pytest.mark.parametrize("bad", [{"lang": "fr"}, {"sampleRate": 16000}, {"text": ""},
                                  {"reference_audio": "user.wav"}, {"style": "unknown"}])
def test_invalid_input_rejected(bank: VoiceBank, bad: dict[str, object]) -> None:
    with TestClient(create_app(FakeEngine(), bank)) as client:
        client.post("/warmup")
        with client.websocket_connect("/synthesize") as socket:
            socket.send_json({**payload(), **bad})
            with pytest.raises(WebSocketDisconnect) as closed:
                socket.receive_bytes()
            assert closed.value.code == 1008


def test_reference_hash_and_provenance_enforced(bank: VoiceBank) -> None:
    bank.path.with_name("synthetic.wav").write_bytes(b"tampered")
    with pytest.raises(ValueError, match="HASH"):
        bank.select("test", "zh", "neutral")
    data = json.loads(bank.path.read_text())
    data["references"]["zh"]["neutral"]["provenance"] = "human-recording"
    bank.path.write_text(json.dumps(data))
    with pytest.raises(ValueError, match="PROVENANCE"):
        bank.select("test", "zh", "neutral")


def test_stdlib_fallback_without_site_packages() -> None:
    # -S excludes all installed dependencies. The exact supervised entry stays alive.
    import os
    import socket
    import urllib.request
    with socket.socket() as available:
        available.bind(("127.0.0.1", 0))
        port = available.getsockname()[1]
    entry = Path(__file__).resolve().parents[1] / "src/main.py"
    process = subprocess.Popen([sys.executable, "-S", str(entry)],
                               env={**os.environ, "NEMU_TTS_PORT": str(port)},
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        deadline = time.monotonic() + 5
        while True:
            try:
                with urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=1) as response:
                    health = json.loads(response.read())
                break
            except OSError:
                if time.monotonic() > deadline:
                    raise
                time.sleep(.02)
        assert process.poll() is None
        assert health["reasons"] == ["PYTHON_DEPENDENCIES_MISSING"]
    finally:
        process.terminate()
        process.communicate(timeout=5)
