"""Single supervised, offline GPT-SoVITS V5 inference process."""
from __future__ import annotations

import multiprocessing as mp
import os
from collections.abc import Callable
from multiprocessing.process import BaseProcess
from pathlib import Path
from threading import Event

from .backend import Request
from .gsv_worker import ModelPipe, run_worker
from .voices import Reference


class GsvEngine:
    def __init__(self) -> None:
        self.process: BaseProcess | None = None
        self.pipe: ModelPipe | None = None
        self.vram: dict[str, int] | None = None
        self.failure: str | None = None

    def health(self) -> dict[str, object]:
        alive = self.process is not None and self.process.is_alive()
        reason = self.failure or ("WORKER_EXITED" if self.process and not alive else None)
        return {"model": "gpt-sovits-v5turbo", "modelLoaded": alive and self.vram is not None,
                "languages": ["zh", "en"], "vram": self.vram if alive else None,
                "reasons": [reason] if reason else []}

    def close(self) -> None:
        pipe, self.pipe = self.pipe, None
        process, self.process = self.process, None
        if pipe is not None:
            pipe.close()
        if process is not None:
            process.join(timeout=1)  # An idle worker exits on pipe EOF.
            if process.is_alive():
                process.terminate()
                process.join()
        self.vram = None

    def warmup(self) -> None:
        if self.health()["modelLoaded"]:
            return
        self.close()
        root = os.getenv("NEMU_GSV_ROOT")
        if not root:
            raise RuntimeError("MODEL_CACHE_NOT_CONFIGURED")
        ctx = mp.get_context("spawn")
        pipe, child = ctx.Pipe()
        process = ctx.Process(target=run_worker, args=(child, Path(root).resolve()), daemon=True)
        try:
            process.start()
            self.pipe, self.process = pipe, process
        except BaseException:
            pipe.close()
            raise
        finally:
            child.close()
        try:
            status, data = self._receive(pipe)
            if status != "ready" or not isinstance(data, dict):
                raise RuntimeError(str(data))
            self.vram = data
            self.failure = None
        except Exception as exc:
            self.failure = str(exc) if isinstance(exc, RuntimeError) else type(exc).__name__
            self.close()
            raise

    @staticmethod
    def _receive(pipe: ModelPipe) -> tuple[str, object]:
        # Match the local client's 120 second timeout and reclaim a stalled worker.
        if not pipe.poll(120):
            raise RuntimeError("MODEL_TIMEOUT")
        try:
            kind, data = pipe.recv()
        except EOFError as exc:
            raise RuntimeError("WORKER_EXITED") from exc
        return kind, data

    def stream(self, request: Request, reference: Reference, cancel: Event,
               emit: Callable[[bytes], None]) -> None:
        if request.lang not in ("zh", "en"):
            raise ValueError("UNSUPPORTED_VOICE_LANGUAGE")
        pipe = self.pipe
        if pipe is None:
            raise RuntimeError("NOT_WARMED")
        try:
            pipe.send((request, reference))
            while True:
                kind, data = self._receive(pipe)
                if kind == "pcm" and isinstance(data, bytes):
                    if not cancel.is_set():
                        emit(data)
                elif kind == "done" and isinstance(data, dict):
                    self.vram = data
                    return
                else:
                    raise RuntimeError("MODEL_INFERENCE_FAILED")
        except Exception:
            self.failure = "MODEL_INFERENCE_FAILED"
            self.close()
            raise
