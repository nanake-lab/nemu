"""Single-owner inference lane; cancellation stops emission and cooperative generation."""
from __future__ import annotations

import asyncio
from collections.abc import Callable
from dataclasses import dataclass
from threading import Event, Lock
from typing import Protocol

from .voices import Reference


@dataclass(frozen=True)
class Request:
    text: str
    lang: str
    style: str
    voice_id: str


class Engine(Protocol):
    def warmup(self) -> None: ...
    def stream(self, request: Request, reference: Reference, cancel: Event,
               emit: Callable[[bytes], None]) -> None: ...
    def health(self) -> dict[str, object]: ...


class Lane:
    def __init__(self, engine: Engine) -> None:
        self.engine = engine
        self.lock = Lock()
        self.warmed = False
        self.failure: str | None = None

    async def warmup(self) -> None:
        if not self.lock.acquire(blocking=False):
            raise RuntimeError("BUSY")
        try:
            await asyncio.to_thread(self.engine.warmup)
            self.warmed = True
            self.failure = None
        except Exception as exc:
            message = str(exc)
            self.failure = message if message in {
                "MODEL_CACHE_NOT_CONFIGURED", "MODEL_FILES_MISSING", "MODEL_HASH_MISMATCH",
                "CUDA_UNAVAILABLE", "QWEN_API_VERSION_MISMATCH", "INVALID_BASE_VARIANT",
                "GRAPH_API_VERSION_MISMATCH", "SOX_ALREADY_IMPORTED_UNCONTROLLED",
            } else type(exc).__name__
            raise
        finally:
            self.lock.release()

    def begin(self, request: Request, reference: Reference,
              cancel: Event) -> tuple[asyncio.Queue[bytes | Exception | None], asyncio.Task[None]]:
        if not self.warmed:
            raise RuntimeError("NOT_WARMED")
        if not self.lock.acquire(blocking=False):
            raise RuntimeError("BUSY")
        queue: asyncio.Queue[bytes | Exception | None] = asyncio.Queue(maxsize=8)
        loop = asyncio.get_running_loop()

        def put(item: bytes | Exception | None) -> None:
            # Backpressure; a stalled or cancelled consumer cannot grow RAM unboundedly.
            future = asyncio.run_coroutine_threadsafe(queue.put(item), loop)
            while not cancel.is_set():
                try:
                    future.result(timeout=0.02)
                    return
                except TimeoutError:
                    continue
            future.cancel()

        def run() -> None:
            try:
                self.engine.stream(request, reference, cancel, put)
            except Exception as exc:
                put(exc)
            finally:
                self.lock.release()
                put(None)

        async def worker() -> None:
            await asyncio.to_thread(run)

        return queue, asyncio.create_task(worker())
