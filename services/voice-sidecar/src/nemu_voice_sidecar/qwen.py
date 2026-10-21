"""qwen-tts 0.1.1 adapter; scoped talker hook exposes codec frames before EOS.

Uses the tokenizer's causal left-context convention (25 frames). No upstream
source is patched. The private output layout is pinned and checked at runtime.
"""
from __future__ import annotations

from collections.abc import Callable
import gc
import importlib
import importlib.metadata
import os
from pathlib import Path
from threading import Event
from typing import Any

from .backend import Request
from .models import verified_model
from .qwen_import import load_qwen
from .voices import Reference

LANGUAGES = {"zh": "Chinese", "en": "English", "ja": "Japanese"}


class Cancelled(Exception):
    pass


class QwenEngine:
    def __init__(self, *, measure_exact_variant: bool = False) -> None:
        self.model: Any = None
        self.torch: Any = None
        self.variant = os.getenv("NEMU_TTS_VARIANT", "0.6B")
        self.requested = self.variant
        self.reasons: list[str] = []
        self.prompts: dict[str, Any] = {}
        self.measure_exact_variant = measure_exact_variant

    def health(self) -> dict[str, object]:
        vram = None
        reasons = list(self.reasons)
        if self.model is None:
            try:
                importlib.metadata.version("qwen-tts")
            except importlib.metadata.PackageNotFoundError:
                reasons.append("QWEN_DEPENDENCY_MISSING")
            if not os.getenv("NEMU_TTS_CACHE"):
                reasons.append("MODEL_CACHE_NOT_CONFIGURED")
        if self.torch is not None and self.torch.cuda.is_available():
            vram = {"allocatedBytes": self.torch.cuda.memory_allocated(),
                    "reservedBytes": self.torch.cuda.memory_reserved(),
                    "peakAllocatedBytes": self.torch.cuda.max_memory_allocated(),
                    "peakReservedBytes": self.torch.cuda.max_memory_reserved()}
        return {"variant": self.variant, "requestedVariant": self.requested,
                "reasons": reasons, "vram": vram,
                "streaming": "talker-hook-4-frames-left-context-25",
                "modelLoaded": self.model is not None}

    def warmup(self) -> None:
        if self.model is not None:
            return
        configured_cache = os.getenv("NEMU_TTS_CACHE")
        if not configured_cache:
            raise RuntimeError("MODEL_CACHE_NOT_CONFIGURED")
        cache = Path(configured_cache)
        if importlib.metadata.version("qwen-tts") != "0.1.1":
            raise RuntimeError("QWEN_API_VERSION_MISMATCH")
        self.torch = importlib.import_module("torch")
        if not self.torch.cuda.is_available():
            raise RuntimeError("CUDA_UNAVAILABLE")
        free, _ = self.torch.cuda.mem_get_info()
        # Admission reserve is conservative, not the ADR 0012 acceptance metric.
        if self.variant == "1.7B" and free < 6_000_000_000 and not self.measure_exact_variant:
            self.variant = "0.6B"
            self.reasons.append("LOW_VRAM_FALLBACK_0_6B")
        if self.variant not in ("0.6B", "1.7B"):
            raise ValueError("INVALID_BASE_VARIANT")
        api = load_qwen()
        folder = verified_model(cache, self.variant)
        fallback = False
        try:
            self.model = api.Qwen3TTSModel.from_pretrained(
                str(folder), device_map="cuda:0", dtype=self.torch.bfloat16,
                attn_implementation="eager", local_files_only=True)
        except self.torch.cuda.OutOfMemoryError:
            if self.variant != "1.7B" or self.measure_exact_variant:
                raise
            self.model = None
            fallback = True
        if fallback:
            # Exit the exception scope first, releasing its CUDA allocation traceback.
            gc.collect()
            self.torch.cuda.empty_cache()
            self.variant = "0.6B"
            self.reasons.append("CUDA_OOM_FALLBACK_0_6B")
            self.warmup()

    def stream(self, request: Request, reference: Reference, cancel: Event,
               emit: Callable[[bytes], None]) -> None:
        emitted = False

        def counted(packet: bytes) -> None:
            nonlocal emitted
            emitted = True
            emit(packet)

        oom = getattr(getattr(self.torch, "cuda", None), "OutOfMemoryError", MemoryError)
        try:
            self._stream(request, reference, cancel, counted)
            return
        except Exception as exc:
            if not isinstance(exc, oom) or self.variant != "1.7B" or self.measure_exact_variant:
                raise
        # The failed generation frame (and its CUDA traceback) is gone here.
        self.model = None
        self.prompts.clear()
        gc.collect()
        self.torch.cuda.empty_cache()
        self.variant = "0.6B"
        self.reasons.append("SYNTHESIS_OOM_FALLBACK_0_6B")
        self.warmup()
        if emitted:
            # Never replay a sentence whose beginning has already been emitted.
            raise RuntimeError("TTS_PARTIAL_CUDA_OOM")
        self._stream(request, reference, cancel, emit)

    def _stream(self, request: Request, reference: Reference, cancel: Event,
                emit: Callable[[bytes], None]) -> None:
        if cancel.is_set():
            return
        key = reference.sha256 + reference.text
        if key not in self.prompts:
            # Bounded ephemeral acceleration only; reference remains authoritative.
            if len(self.prompts) >= 32:
                self.prompts.clear()
            self.prompts[key] = self.model.create_voice_clone_prompt(
                ref_audio=str(reference.path), ref_text=reference.text,
                x_vector_only_mode=False)
        prompt = self.prompts[key]
        codes: list[Any] = []
        emitted_frames = 0
        torch = self.torch
        tokenizer = self.model.model.speech_tokenizer
        eos = self.model.model.config.talker_config.codec_eos_token_id

        def flush() -> None:
            nonlocal emitted_frames
            if cancel.is_set():
                raise Cancelled()
            if len(codes) == emitted_frames:
                return
            start = max(0, emitted_frames - 25)
            context = emitted_frames - start
            frames = torch.stack(codes[start:], dim=0)
            wavs, rate = tokenizer.decode([{"audio_codes": frames}])
            if rate != 24000:
                raise RuntimeError("TOKENIZER_SAMPLE_RATE_MISMATCH")
            wav = wavs[0]
            # 12 Hz tokenizer actually uses 1920 samples/frame at 24 kHz (12.5 Hz).
            samples_per_frame = len(wav) // len(frames)
            pcm = wav[context * samples_per_frame:].astype("<f4").tobytes()
            emitted_frames = len(codes)
            for offset in range(0, len(pcm), 960 * 4):
                if cancel.is_set():
                    raise Cancelled()
                emit(pcm[offset:offset + 960 * 4])
            # Generation is capped at 512 frames; decode retains only left context.

        def hook(module: Any, args: Any, output: Any) -> None:
            if cancel.is_set():
                raise Cancelled()
            if not isinstance(output.hidden_states, tuple) or len(output.hidden_states) != 2:
                raise RuntimeError("QWEN_CODEC_LAYOUT_CHANGED")
            frame = output.hidden_states[-1]
            if frame is None:
                return
            if frame.shape[0] != 1:
                raise RuntimeError("QWEN_BATCH_UNSUPPORTED")
            if int(frame[0, 0]) == eos:
                return
            codes.append(frame[0].detach())
            if len(codes) - emitted_frames >= 4:
                flush()

        handle = self.model.model.talker.register_forward_hook(hook)
        try:
            with torch.inference_mode():
                # No instruct/style kwarg: Base receives the selected reference only.
                self.model.generate_voice_clone(
                    text=request.text, language=LANGUAGES[request.lang],
                    voice_clone_prompt=prompt, non_streaming_mode=False,
                    max_new_tokens=512)
                flush()
        except Cancelled:
            pass
        finally:
            handle.remove()
