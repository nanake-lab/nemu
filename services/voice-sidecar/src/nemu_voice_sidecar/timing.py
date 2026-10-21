"""Optional per-request CUDA-event timing; never synchronize each model step."""
from __future__ import annotations
from collections.abc import Iterator
from contextlib import contextmanager
import json
import os
from pathlib import Path
from threading import Lock
import time
from typing import Any
import uuid


class Trace:
    def __init__(self, torch: Any, lang: str) -> None:
        self.torch = torch
        self.folder = os.getenv("NEMU_TTS_TRACE_DIR")
        self.lang = lang
        self.start = time.perf_counter()
        self.lock = Lock()
        self.spans: list[tuple[str, int, float, float, Any, Any]] = []
        self.frames: list[dict[str, object]] = []
        self.sent: list[dict[str, float | int]] = []
        self.codecs: list[Any] = []

    @contextmanager
    def span(self, name: str, index: int = 0) -> Iterator[None]:
        if not self.folder:
            yield
            return
        start, end = (self.torch.cuda.Event(enable_timing=True) for _ in range(2))
        wall = time.perf_counter()
        start.record()
        try:
            yield
        finally:
            end.record()
            with self.lock:
                self.spans.append((name, index, wall, time.perf_counter(), start, end))

    def frame(self, index: int) -> None:
        if self.folder:
            self.frames.append({"index": index, "wall_ms": (time.perf_counter()-self.start)*1000})

    def packet(self, size: int) -> None:
        if self.folder:
            self.sent.append({"wall_ms": (time.perf_counter()-self.start)*1000, "bytes": size})

    def finish(self, *, cancelled: bool, failed: bool) -> dict[str, object]:
        if not self.folder:
            return {}
        self.torch.cuda.synchronize()
        spans = [{"name": n, "index": i, "start_ms": (s-self.start)*1000,
                  "host_ms": (e-s)*1000, "cuda_ms": a.elapsed_time(b)}
                 for n, i, s, e, a, b in self.spans]
        first = self.sent[0]["wall_ms"] if self.sent else None
        last = self.sent[-1]["wall_ms"] if self.sent else None
        duration = sum(p["bytes"] for p in self.sent) / 4 / 24000
        interval = (last-first)/1000 if first is not None and last is not None else 0
        data: dict[str, object] = {"lang": self.lang, "cancelled": cancelled, "failed": failed,
            "spans": spans, "frames": self.frames, "emissions": self.sent,
            "first_emit_ms": first, "last_emit_ms": last, "audio_seconds": duration,
            "first_to_last_seconds": interval, "audio_per_stream_second": duration/interval if interval > 0 else None,
            "metric_note": "audio duration / first-to-last engine emission; report only; WS arrival measured separately"}
        if self.codecs:
            data["codec_frames"] = self.torch.stack(self.codecs).cpu().tolist()
        folder = Path(self.folder)
        folder.mkdir(parents=True, exist_ok=True)
        file = folder / (uuid.uuid4().hex + ".json")
        file.write_text(json.dumps(data, indent=2)+"\n", encoding="utf-8")
        return data
