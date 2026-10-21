"""CPU doubles exercise the real adapter's hook, cancellation and admission logic."""
from __future__ import annotations

from contextlib import nullcontext
from pathlib import Path
import struct
from threading import Event
from types import SimpleNamespace
from typing import Any

import pytest

from nemu_voice_sidecar.backend import Request
from nemu_voice_sidecar.qwen import QwenEngine
from nemu_voice_sidecar.voices import Reference


class Wave:
    def __init__(self, samples: int) -> None:
        self.samples = samples

    def __len__(self) -> int:
        return self.samples

    def __getitem__(self, key: slice) -> Wave:
        return Wave(self.samples - (key.start or 0))

    def astype(self, dtype: str) -> Wave:
        assert dtype == '<f4'
        return self

    def tobytes(self) -> bytes:
        return struct.pack('<f', .25) * self.samples


class Row:
    def detach(self) -> Row:
        return self


class Frame:
    shape = (1, 16)

    def __getitem__(self, key: int | tuple[int, int]) -> Row | int:
        return Row() if isinstance(key, int) else 1


class Model:
    def __init__(self) -> None:
        self.callback: Any = None
        self.removed = False
        self.finished = False
        self.kwargs: dict[str, object] = {}
        self.prompt_calls = 0
        self.decode_lengths: list[int] = []
        self.model = SimpleNamespace(
            talker=SimpleNamespace(register_forward_hook=self.register),
            speech_tokenizer=SimpleNamespace(decode=self.decode),
            config=SimpleNamespace(talker_config=SimpleNamespace(codec_eos_token_id=2048)))

    def register(self, callback: Any) -> Any:
        self.callback = callback
        return SimpleNamespace(remove=lambda: setattr(self, 'removed', True))

    def decode(self, codes: list[dict[str, list[Row]]]) -> tuple[list[Wave], int]:
        count = len(codes[0]['audio_codes'])
        self.decode_lengths.append(count)
        return [Wave(count * 1920)], 24000

    def create_voice_clone_prompt(self, **kwargs: object) -> object:
        self.prompt_calls += 1
        return object()

    def generate_voice_clone(self, **kwargs: object) -> None:
        self.kwargs = kwargs
        self.callback(None, None, SimpleNamespace(hidden_states=(None, None)))
        for _ in range(45):
            self.callback(None, None, SimpleNamespace(hidden_states=(None, Frame())))
        self.finished = True


def engine_and_model() -> tuple[QwenEngine, Model]:
    engine = QwenEngine()
    model = Model()
    engine.model = model
    engine.torch = SimpleNamespace(stack=lambda rows, dim: rows, inference_mode=nullcontext)
    return engine, model


def test_incremental_decode_before_generation_finishes_and_hook_cleanup() -> None:
    engine, model = engine_and_model()
    packets: list[bytes] = []

    def emit(packet: bytes) -> None:
        assert not model.finished or len(model.decode_lengths) > 1
        packets.append(packet)

    engine.stream(Request('hello', 'en', 'happy', 'test'), Reference(Path('synthetic.wav'), 'ref', 'a' * 64), Event(), emit)
    assert packets and model.removed
    assert model.decode_lengths[0] == 4
    assert max(model.decode_lengths) <= 29
    assert sum(len(p) for p in packets) == 45 * 1920 * 4
    assert model.kwargs['language'] == 'English'
    assert 'instruct' not in model.kwargs and 'style' not in model.kwargs


def test_cancellation_unhooks_and_stops_generation() -> None:
    engine, model = engine_and_model()
    cancel = Event()
    packets: list[bytes] = []

    def emit(packet: bytes) -> None:
        packets.append(packet)
        cancel.set()

    engine.stream(Request('test', 'ja', 'neutral', 'test'), Reference(Path('synthetic.wav'), 'ref', 'a' * 64), cancel, emit)
    assert len(packets) == 1
    assert model.removed and not model.finished


def test_low_vram_switches_to_small_base(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    monkeypatch.setenv('NEMU_TTS_VARIANT', '1.7B')
    monkeypatch.setenv('NEMU_TTS_CACHE', str(tmp_path))
    engine = QwenEngine()
    monkeypatch.setattr('nemu_voice_sidecar.qwen.importlib.metadata.version', lambda _: '0.1.1')
    cuda = SimpleNamespace(is_available=lambda: True, mem_get_info=lambda: (4_000_000_000, 10_000_000_000),
                           OutOfMemoryError=MemoryError)
    torch = SimpleNamespace(cuda=cuda, bfloat16=object())
    captured = []

    def load(path: str, **kwargs: object) -> object:
        captured.append(path)
        return object()

    api = SimpleNamespace(Qwen3TTSModel=SimpleNamespace(from_pretrained=load))
    monkeypatch.setattr('nemu_voice_sidecar.qwen.verified_model', lambda cache, variant: tmp_path / variant)
    monkeypatch.setattr('nemu_voice_sidecar.qwen.importlib.import_module', lambda name: torch if name == 'torch' else api)
    engine.warmup()
    assert engine.variant == '0.6B'
    assert captured == [str(tmp_path / '0.6B')]
    assert 'LOW_VRAM_FALLBACK_0_6B' in engine.reasons


@pytest.mark.parametrize('partial', [False, True])
def test_runtime_oom_downgrades_without_replaying_partial_output(
    monkeypatch: pytest.MonkeyPatch, partial: bool,
) -> None:
    monkeypatch.setenv('NEMU_TTS_VARIANT', '1.7B')
    engine = QwenEngine()
    engine.torch = SimpleNamespace(cuda=SimpleNamespace(OutOfMemoryError=MemoryError,
                                                      empty_cache=lambda: None))
    calls = []
    packets: list[bytes] = []

    def generate(request: Request, reference: Reference, cancel: Event,
                 emit: Any) -> None:
        calls.append(engine.variant)
        if engine.variant == '1.7B':
            if partial:
                emit(b'partial')
            raise MemoryError('mock CUDA OOM')
        emit(b'small')

    monkeypatch.setattr(engine, '_stream', generate)
    monkeypatch.setattr(engine, 'warmup', lambda: None)
    args = (Request('test', 'en', 'neutral', 'test'),
            Reference(Path('synthetic.wav'), 'ref', 'a' * 64), Event(), packets.append)
    if partial:
        with pytest.raises(RuntimeError, match='PARTIAL'):
            engine.stream(*args)
        assert packets == [b'partial'] and calls == ['1.7B']
    else:
        engine.stream(*args)
        assert packets == [b'small'] and calls == ['1.7B', '0.6B']
    assert engine.variant == '0.6B'
