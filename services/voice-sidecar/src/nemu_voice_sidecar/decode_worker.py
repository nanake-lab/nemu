"""Bounded decode worker on an independent CUDA stream, preserving 25 left frames."""
from __future__ import annotations
from collections.abc import Callable
from queue import Queue, Full
from threading import Event, Thread
from typing import Any


class DecodeWorker:
    def __init__(self, torch: Any, decode: Callable[[Any], None], cancel: Event, *, stream: Any = None) -> None:
        self.torch = torch
        self.decode = decode
        self.cancel = cancel
        self.queue: Queue[tuple[Any, Any] | None] = Queue(maxsize=8)
        self.error: BaseException | None = None
        self.stream = stream if stream is not None else torch.cuda.Stream()
        self.thread = Thread(target=self.run, name="tts-codec-decode", daemon=True)
        self.thread.start()

    def submit(self, job: Any) -> None:
        ready = self.torch.cuda.Event()
        ready.record()
        while not self.cancel.is_set():
            if self.error:
                raise self.error
            try:
                self.queue.put((job, ready), timeout=.02)
                return
            except Full:
                continue

    def run(self) -> None:
        try:
            with self.torch.inference_mode(), self.torch.cuda.stream(self.stream):
                while True:
                    item = self.queue.get()
                    if item is None:
                        return
                    job, ready = item
                    self.stream.wait_event(ready)
                    if not self.cancel.is_set():
                        self.decode(job)
        except BaseException as exc:
            self.error = exc

    def finish(self) -> None:
        while self.thread.is_alive():
            try:
                self.queue.put(None, timeout=.02)
                break
            except Full:
                continue
        self.thread.join()
        self.stream.synchronize()
        if self.error:
            raise self.error
