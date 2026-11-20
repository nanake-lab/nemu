"""Bounded local A/C streams and the m=0 structured-decoding callable for IRL/A/M."""
from __future__ import annotations

import asyncio
from collections.abc import AsyncGenerator, AsyncIterator, Callable
from contextlib import asynccontextmanager
from dataclasses import dataclass
from hashlib import sha256
import codecs
import json
from time import monotonic_ns

from echo.runtime import SCHEMA, Limits, normalize
from echo.cognition import check_cognition_message
from echo.typed_api import to_value
from echo.types import (
    CognitiveCompletion, CognitiveInput, JActivationDtype, JDecodeMask, JHookBinding, JLayerKind,
    JPrefillMask, JResidualStage, ObjectRef, TaskInput,
)

from .native import Assets, Native, Prompt
from .resources import CacheKey, Publication, Unmodulated, canonical_ref, receive

type Json = None | bool | int | float | str | list[Json] | dict[str, Json]


def chat_prompt(system: str | None, user: str) -> Prompt:
    # Text-only, single-turn branch of the frozen Qwen3.5 chat_template.jinja.
    # Data containing Qwen control markers must not introduce another role at tokenization.
    prefix = bytearray()
    spans: list[int] = []
    for role, field in (("system", system), ("user", user)):
        if field is None:
            continue
        prefix.extend(f"<|im_start|>{role}\n".encode())
        start = len(prefix)
        prefix.extend(field.strip().encode())
        if "<|" in field or "<think>" in field or "</think>" in field:
            spans.extend((start, len(prefix)))
        prefix.extend(b"<|im_end|>\n")
    prefix.extend(b"<|im_start|>assistant\n<think>\n\n</think>\n\n")
    return Prompt(bytes(prefix), tuple(spans))


@dataclass(frozen=True)
class Observation:
    key: CacheKey
    cognitive_input: ObjectRef | None
    draft: ObjectRef | None
    snapshot_id: str
    model_hash: bytes
    tokenizer_hash: bytes
    completion: CognitiveCompletion
    tokens: int
    positions: int
    forward_batches: int
    max_abs_error: float
    max_delta_l2: float
    output_sha256: bytes

    def record(self, writer: Callable[[Observation], ObjectRef], limits: Limits) -> ObjectRef:
        ref = writer(self)
        normalize("ObjectRef", to_value(ref), limits)
        # The trusted post-execution writer owns the object format. A pre-execution
        # J resource cannot also be evidence of this forward pass.
        if (not ref.byte_length or ref.object_type in {
                SCHEMA["messages"][name]["tag"] for name in
                ("JEffectReceipt", "ModulationEnvelopeV2", "JDirectionSet", "GenesisManifestV3")}):
            raise Unmodulated("missing post-execution carrier evidence")
        return ref


class LocalCarrier:
    def __init__(self, assets: Assets, hook: JHookBinding, *, load_mmproj: bool = True) -> None:
        # P0-12 Qwen3.5 config, b11669 native block output broadcast on all positions.
        kind = (JLayerKind.J_ATTENTION_LAYER if hook.layer_index % 4 == 3
                else JLayerKind.J_RECURRENT_LAYER)
        if (not 0 < hook.layer_index < 31 or hook.layer_kind != kind or hook.width != 2560
                or hook.residual_stage != JResidualStage.J_BLOCK_RESIDUAL_OUTPUT
                or hook.dtype != JActivationDtype.J_FLOAT32
                or hook.prefill_mask != JPrefillMask.J_PREFILL_ALL
                or hook.decode_mask != JDecodeMask.J_DECODE_ALL):
            raise Unmodulated("unsupported native hook")
        try:
            if (hook.carrier.digest, hook.carrier.byte_length) != (assets.shim_sha256, assets.shim.stat().st_size):
                raise Unmodulated("installed carrier mismatch")
            self._native = Native(assets, load_mmproj=load_mmproj)
        except OSError as exc:
            raise Unmodulated("carrier asset unavailable") from exc
        if self._native.lib.carrier_measurement_control():
            self._native.close()
            raise Unmodulated("measurement-only carrier")
        if ((hook.model.digest, hook.model.byte_length) !=
                (self._native.model_hash, self._native.model_bytes)
                or (hook.tokenizer.digest, hook.tokenizer.byte_length) !=
                (self._native.tokenizer_hash, self._native.tokenizer_bytes)):
            self._native.close()
            raise Unmodulated("installed model/tokenizer mismatch")
        self.hook = hook
        self.numeric_config = sha256(b"b11669/Q4_K_M/F32/8192/512/512/4/FA-off/greedy/"
                                     + assets.shim_sha256).digest()
        self._lock = asyncio.Lock()

    async def close(self) -> None:
        async with self._lock:
            self._native.close()

    def judgment(self, max_tokens: int, duration_ns: int
                 ) -> Callable[[str, dict[str, Json]], AsyncGenerator[str, None]]:
        """Same callable as P5-07 LocalInference. The worker supplies its original schema."""
        async def infer(prompt: str, schema: dict[str, Json]) -> AsyncGenerator[str, None]:
            if not schema:
                raise Unmodulated("missing judgment schema")
            pending: list[Json] = [schema]
            while pending:
                node = pending.pop()
                if isinstance(node, dict):
                    ref = node.get("$ref")
                    if ref is not None and (not isinstance(ref, str) or not ref.startswith("#")):
                        raise Unmodulated("nonlocal schema reference")
                    pending.extend(node.values())
                elif isinstance(node, list):
                    pending.extend(node)
            wire = json.dumps(schema, ensure_ascii=False, allow_nan=False).encode()
            async with self._request(chat_prompt(None, prompt), (0.0,) * self.hook.width,
                                     wire, max_tokens, duration_ns) as stream:
                async for chunk in stream.text():
                    yield chunk
                if stream.completion != CognitiveCompletion.COG_COMPLETED:
                    raise Unmodulated("judgment output truncated")
        return infer

    @asynccontextmanager
    async def generate(self, task: TaskInput, material: str, envelope: bytes, receipt: bytes,
                       committed: Publication, read: Callable[[ObjectRef], bytes],
                       limits: Limits, *, cognition: CognitiveInput | None = None,
                       draft: ObjectRef | None = None) -> AsyncIterator[Stream]:
        """Task layer supplies the authorized, rendered o/c_task (and C draft) as material."""
        deadline = monotonic_ns() + task.max_duration_ns
        normalize("TaskInput", to_value(task), limits)
        if draft is not None:
            if cognition is not None:
                raise Unmodulated("speech draft supplied to cognition")
            # Object reader resolves the task layer's immutable, recorded Mode C draft.
            raw_draft = read(draft)
            if len(raw_draft) != draft.byte_length:
                raise Unmodulated("draft length")
            material = material + "\n\n" + raw_draft.decode("utf-8")
        prefix = chat_prompt(task.protocol_content or None, material)
        m = receive(envelope, receipt, committed, self.hook, read, prefix.cache_prefix(), self.numeric_config, limits)
        if m.snapshot_id != task.snapshot_id:
            raise Unmodulated("task/envelope snapshot")
        input_ref = None
        if cognition is not None:
            cv = to_value(cognition)
            check_cognition_message("CognitiveInput", cv, limits)
            b = cognition.binding
            if (b.snapshot_id != m.snapshot_id or b.protocol.resource_version != task.protocol_version
                    or b.environment.model != self.hook.model or b.environment.tokenizer != self.hook.tokenizer
                    or b.environment.carrier != self.hook.carrier or b.environment.direction_set != m.key.direction_set):
                raise Unmodulated("cognitive input binding")
            input_ref = canonical_ref("CognitiveInput", cv, limits)
        async with self._request(prefix, m.delta, None, task.max_tokens, deadline - monotonic_ns()) as stream:
            stream.key = m.key
            stream.cognitive_input = input_ref
            stream.draft = draft
            stream.snapshot_id = m.snapshot_id
            yield stream

    @asynccontextmanager
    async def _request(self, prompt: Prompt, delta: tuple[float, ...], schema: bytes | None,
                       max_tokens: int, duration_ns: int) -> AsyncIterator[Stream]:
        deadline = monotonic_ns() + duration_ns
        if not 0 < duration_ns < 2**63:
            raise Unmodulated("request deadline")
        async with asyncio.timeout(duration_ns / 1e9), self._lock:
            # A cancelled to_thread keeps running. Join it before freeing the handle or
            # handing the model to another request, including cancellation during prefill.
            start = asyncio.create_task(asyncio.to_thread(
                self._native.begin, prompt, self.hook.layer_index, delta, schema, max_tokens,
                deadline - monotonic_ns()))
            try:
                handle = await asyncio.shield(start)
            except asyncio.CancelledError:
                handle = await start
                self._native.end(handle)
                raise
            stream = Stream(self._native, handle, max_tokens)
            try:
                yield stream
            finally:
                stream.closed = True
                async with stream.reading:
                    self._native.end(handle)


class Stream:
    def __init__(self, native: Native, handle: int, max_tokens: int) -> None:
        self.native = native
        self.handle = handle
        self.max_tokens = max_tokens
        self.closed = False
        self.started = False
        self.reading = asyncio.Lock()
        self.key: CacheKey | None = None
        self.cognitive_input: ObjectRef | None = None
        self.draft: ObjectRef | None = None
        self.snapshot_id = ""
        self.observation: Observation | None = None
        self.completion: CognitiveCompletion | None = None

    async def text(self) -> AsyncIterator[str]:
        if self.closed or self.started:
            raise Unmodulated("stream is single-use")
        self.started = True
        decoder = codecs.getincrementaldecoder("utf-8")("strict")
        output = sha256()
        count = 0
        completion = CognitiveCompletion.COG_TRUNCATED
        while count < self.max_tokens:
            async with self.reading:
                if self.closed:
                    raise Unmodulated("stream closed")
                call = asyncio.create_task(asyncio.to_thread(self.native.next, self.handle))
                try:
                    piece = await asyncio.shield(call)
                except asyncio.CancelledError:
                    await call
                    raise
            if piece is None:
                completion = CognitiveCompletion.COG_COMPLETED
                break
            count += 1
            chunk = decoder.decode(piece)
            if chunk:
                output.update(chunk.encode())
                yield chunk
        if completion == CognitiveCompletion.COG_COMPLETED:
            decoder.decode(b"", final=True)
        if self.closed:
            raise Unmodulated("stream closed")
        self.completion = completion
        if self.key is not None:
            s = self.native.stats(self.handle)
            self.observation = Observation(self.key, self.cognitive_input, self.draft, self.snapshot_id, self.native.model_hash,
                                           self.native.tokenizer_hash, completion, count, s.positions,
                                           s.batches, s.max_abs_error, s.max_delta_l2, output.digest())
