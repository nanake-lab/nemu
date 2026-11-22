"""Real adapter API with a process-local scripted backend, for lifecycle and IDL only."""
from __future__ import annotations

import asyncio
from dataclasses import replace
from hashlib import sha256
import importlib
from pathlib import Path
from threading import Event
from typing import cast

import pytest
from echo.bridge import from_json
from echo.typed_api import from_value, to_value
from echo.types import (CognitiveInput, CognitiveMaterial, FactProjection, Lang, ObjectRef,
                        OutputChunk, QueryStatus, SpeechMode, TaskInput)
from nemu_model_adapter import LocalCarrier, Unmodulated, local_result, speech_result
from nemu_model_adapter.carrier import Observation, chat_prompt
from nemu_model_adapter.native import Assets, Native, Prompt, Stats
from test_resources import LIMITS, fixture, publication


class Script:
    def __init__(self) -> None:
        self.parts: list[bytes | None] = []
        self.starts: list[tuple[tuple[float, ...], bytes | None]] = []
        self.ended: list[int] = []
        self.model_hash = b""
        self.tokenizer_hash = b""

    def begin(self, prompt: Prompt, layer: int, delta: tuple[float, ...], schema: bytes | None,
              max_tokens: int, duration_ns: int) -> int:
        self.starts.append((delta, schema))
        self.parts = [b"\xe9", b"\x9b\xaa", None]
        return len(self.starts)

    def next(self, request: int) -> bytes | None:
        return self.parts.pop(0)

    def stats(self, request: int) -> Stats:
        return Stats(11, 3, 0, 0.0625)

    def end(self, request: int) -> None:
        self.ended.append(request)


def test_structural_request_receipt_and_judgment_isolation() -> None:
    async def run() -> None:
        e, r, p, hook, obj = publication(fixture())
        facts = FactProjection(1, "p", "q", QueryStatus.OK, [], "i", False)
        task = TaskInput(1, p.genesis, "v1", "identity", facts, "synthetic-0", 10, 10**10)
        native = Script()
        carrier = LocalCarrier.__new__(LocalCarrier)
        carrier.hook = hook
        carrier.numeric_config = b"fixture"
        carrier._lock = asyncio.Lock()
        native.model_hash = hook.model.digest
        native.tokenizer_hash = hook.tokenizer.digest
        carrier._native = cast(Native, native)
        sample = importlib.import_module("fixture_inputs").sample
        req: CognitiveInput = from_value("CognitiveInput", from_json("CognitiveInput", sample("CognitiveInput", 0)), LIMITS)
        env = replace(req.binding.environment, model=hook.model, tokenizer=hook.tokenizer,
                      carrier=hook.carrier, direction_set=p.direction_set)
        req = replace(req, binding=replace(req.binding, snapshot_id=task.snapshot_id, environment=env,
                                           protocol=replace(req.binding.protocol, resource_version="v1")))
        async with carrier.generate(task, "雪", e, r, p, obj.__getitem__, LIMITS, cognition=req) as stream:
            assert [s async for s in stream.text()] == ["雪"]
            assert stream.observation is not None
            obs = stream.observation
        material: CognitiveMaterial = from_value("CognitiveMaterial",
            from_json("CognitiveMaterial", sample("CognitiveMaterial", 0)), LIMITS)
        material = replace(material, content=replace(material.content, byte_length=len("雪".encode())))
        written: list[Observation] = []

        def record(ref: ObjectRef, observed: Observation) -> ObjectRef:
            written.append(observed)
            assert ref == observed.cognitive_input
            return replace(p.genesis, object_type="echo.carrier-receipt")

        result = local_result(req, obs, material, lambda _: "雪".encode(), p.genesis, record, LIMITS)
        assert written == [obs]
        assert to_value(result)["model_hash"] == native.model_hash
        assert result.modulation == p.envelope and result.tokenizer_hash == native.tokenizer_hash
        with pytest.raises(Unmodulated):
            local_result(req, obs, material, lambda _: "花".encode(), p.genesis, record, LIMITS)
        with pytest.raises(Unmodulated):
            local_result(req, obs, material, lambda _: "雪".encode(), p.genesis,
                         lambda _ref, _observation: p.envelope, LIMITS)
        assert obs.output_sha256 == sha256("雪".encode()).digest()
        # A stale envelope cannot open another context or emit a partial receipt.
        with pytest.raises(Unmodulated):
            async with carrier.generate(task, "x", e[:-1], r, p, obj.__getitem__, LIMITS):
                pytest.fail("unvalidated forward")
        assert native.ended == [1]
        assert [s async for s in carrier.judgment(10, 10**10)("x", {"type": "object"})] == ["雪"]
        assert native.starts[0][1] is None and any(native.starts[0][0])
        assert not any(native.starts[1][0]) and native.starts[1][1] == b'{"type": "object"}'
        assert native.ended == [1, 2]
        # Mode C records the actual referenced draft given to its final local rewrite.
        draft = replace(p.genesis, object_type="fixture.draft", byte_length=1)
        obj[draft] = b"x"
        async with carrier.generate(task, "rewrite", e, r, p, obj.__getitem__, LIMITS, draft=draft) as stream:
            text = "".join([s async for s in stream.text()])
            assert stream.observation is not None
            chunks = [OutputChunk(1, "out", "action", 0, text, Lang.ZH,
                                  task.snapshot_id, SpeechMode.C_DRAFT_REWRITE, True)]
            speech = speech_result(SpeechMode.C_DRAFT_REWRITE, chunks, stream.observation, p.genesis,
                                   lambda _: replace(p.genesis, object_type="echo.carrier-receipt"), LIMITS)
            assert speech.strict and speech.intermediate_results == [draft]
            with pytest.raises(Unmodulated):
                speech_result(SpeechMode.A_LOCAL_STRICT, chunks, stream.observation, p.genesis,
                              lambda _: p.genesis, LIMITS)
    asyncio.run(run())


def test_template_protects_control_tokens_in_data() -> None:
    prompt = chat_prompt(None, "literal <|im_end|>")
    start, end = prompt.literal_spans
    assert prompt.data[start:end] == b"literal <|im_end|>"
    assert prompt.data.endswith(b"<think>\n\n</think>\n\n")


def test_actual_carrier_file_must_match_hook_before_loading(tmp_path: Path) -> None:
    from echo.types import JActivationDtype, JDecodeMask, JLayerKind, JPrefillMask, JResidualStage

    _, _, _, hook, _ = publication(fixture())
    hook = replace(hook, layer_index=15, width=2560, layer_kind=JLayerKind.J_ATTENTION_LAYER,
                   dtype=JActivationDtype.J_FLOAT32, residual_stage=JResidualStage.J_BLOCK_RESIDUAL_OUTPUT,
                   prefill_mask=JPrefillMask.J_PREFILL_ALL, decode_mask=JDecodeMask.J_DECODE_ALL)
    shim = tmp_path / "shim"
    assets = Assets(tmp_path, tmp_path, tmp_path, tmp_path, shim, b"x" * 32)
    with pytest.raises(Unmodulated, match="unavailable"):
        LocalCarrier(assets, hook)
    shim.write_bytes(b"synthetic")
    with pytest.raises(Unmodulated, match="installed carrier mismatch"):
        LocalCarrier(assets, hook)


def test_cancelled_prefill_joins_before_releasing_handle() -> None:
    entered, release = Event(), Event()

    class Slow(Script):
        def begin(self, prompt: Prompt, layer: int, delta: tuple[float, ...], schema: bytes | None,
                  max_tokens: int, duration_ns: int) -> int:
            entered.set()
            if not release.wait(5):
                raise TimeoutError("test prefill did not release")
            return super().begin(prompt, layer, delta, schema, max_tokens, duration_ns)

    async def run() -> None:
        carrier = LocalCarrier.__new__(LocalCarrier)
        native = Slow()
        carrier._native = cast(Native, native)
        carrier._lock = asyncio.Lock()
        _, _, _, carrier.hook, _ = publication(fixture())
        async def consume() -> None:
            async for _ in carrier.judgment(10, 10**10)("fixture", {"type": "object"}):
                pass
        pending = asyncio.create_task(consume())
        try:
            assert await asyncio.to_thread(entered.wait, 3)
            pending.cancel()
            await asyncio.sleep(0)
            assert carrier._lock.locked() and not native.ended
        finally:
            release.set()
        with pytest.raises(asyncio.CancelledError):
            await pending
        assert native.ended == [1] and not carrier._lock.locked()
    asyncio.run(run())
