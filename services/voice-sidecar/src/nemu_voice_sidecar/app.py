"""Local-only HTTP health and bounded WebSocket PCM transport."""
from __future__ import annotations

import asyncio
from contextlib import suppress
import json
import os
import time
import uuid
from pathlib import Path
from threading import Event
from typing import cast

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from jsonschema import Draft202012Validator, ValidationError
import structlog

from .backend import Engine, Lane, Request
from .qwen import QwenEngine
from .voices import VoiceBank

SCHEMA = json.loads(Path(__file__).with_name("tts-request.schema.json").read_text(encoding="utf-8"))
VALIDATOR = Draft202012Validator(SCHEMA)
LOG = structlog.get_logger()


def create_app(engine: Engine | None = None, bank: VoiceBank | None = None) -> FastAPI:
    app = FastAPI(docs_url=None, redoc_url=None, openapi_url=None)
    lane = Lane(engine or QwenEngine())
    voices = bank or VoiceBank(Path(os.getenv("NEMU_TTS_VOICE", "voice.json")))

    @app.get("/health")
    async def health() -> dict[str, object]:
        state = lane.engine.health()
        reasons = list(cast(list[str], state.get("reasons", [])))
        if not lane.warmed:
            reasons.append("NOT_WARMED")
        if lane.failure:
            reasons.append(lane.failure)
        if not voices.path.is_file():
            reasons.append("VOICE_BANK_MISSING")
        return {**state, "ready": lane.warmed and voices.path.is_file() and state.get("modelLoaded", True) is True,
                "status": "degraded" if reasons else "ready", "reasons": reasons,
                "busy": lane.lock.locked()}

    @app.post("/warmup")
    async def warmup() -> dict[str, object]:
        try:
            await lane.warmup()
        except Exception as exc:
            LOG.warning("tts_warmup_degraded", reason=type(exc).__name__)
        return await health()

    @app.websocket("/synthesize")
    async def synthesize(ws: WebSocket) -> None:
        # Reject browser-origin cross-site requests. Clients use the native WS API.
        if ws.headers.get("origin"):
            await ws.close(code=1008)
            return
        await ws.accept()
        cancelled = Event()
        wire_start = time.perf_counter()
        wire: list[dict[str, float | int]] = []
        worker: asyncio.Task[None] | None = None
        receive: asyncio.Task[str] | None = None
        chunk: asyncio.Task[bytes | Exception | None] | None = None
        try:
            raw: object = await asyncio.wait_for(ws.receive_json(), timeout=10)
            VALIDATOR.validate(raw)
            obj = cast(dict[str, object], raw)
            if len(str(obj["text"])) > 4096:
                raise ValueError("TEXT_LIMIT")
            req = Request(str(obj["text"]), str(obj["lang"]), str(obj["style"]), str(obj["voiceId"]))
            ref = voices.select(req.voice_id, req.lang, req.style)
            queue, worker = lane.begin(req, ref, cancelled)
            receive = asyncio.create_task(ws.receive_text())
            chunk = asyncio.create_task(queue.get())
            while True:
                done, _ = await asyncio.wait((receive, chunk), return_when=asyncio.FIRST_COMPLETED)
                # Cancellation wins if a PCM block and control frame arrive together.
                if receive in done:
                    message = receive.result()
                    if json.loads(message) != {"cancel": True}:
                        raise ValueError("INVALID_CONTROL")
                    cancelled.set()
                    await ws.close(code=1000)
                    return
                item = chunk.result()
                if item is None:
                    await ws.close(code=1000)
                    return
                if isinstance(item, Exception):
                    raise item
                await asyncio.wait_for(ws.send_bytes(item), timeout=2)
                if os.getenv("NEMU_TTS_TRACE_DIR"):
                    wire.append({"wall_ms": (time.perf_counter()-wire_start)*1000, "bytes": len(item)})
                chunk = asyncio.create_task(queue.get())
        except WebSocketDisconnect:
            pass
        except (ValidationError, ValueError, KeyError):
            with suppress(RuntimeError, WebSocketDisconnect):
                await ws.close(code=1008)
        except Exception as exc:
            LOG.warning("tts_synthesis_failed", reason=type(exc).__name__)
            with suppress(RuntimeError, WebSocketDisconnect):
                await ws.close(code=1011)
        finally:
            if os.getenv("NEMU_TTS_TRACE_DIR"):
                folder = Path(os.environ["NEMU_TTS_TRACE_DIR"]) / "wire"
                folder.mkdir(parents=True, exist_ok=True)
                (folder / (uuid.uuid4().hex+".json")).write_text(
                    json.dumps({"emissions": wire, "clock": "perf_counter; after ws.send_bytes completes"})+"\n",
                    encoding="utf-8")
            cancelled.set()
            for task in (receive, chunk):
                if task is not None:
                    task.cancel()
                    with suppress(asyncio.CancelledError, WebSocketDisconnect):
                        await task
            # Keep lane locked until the actual inference thread exits. Cancellation
            # never permits a second generation to race the first CUDA invocation.
            if worker is not None:
                def consume(task: asyncio.Task[None]) -> None:
                    if not task.cancelled():
                        task.exception()
                worker.add_done_callback(consume)

    return app
