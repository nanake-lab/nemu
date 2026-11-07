"""Development-only component ablation; never a 30-minute acceptance run."""
from __future__ import annotations

import asyncio
import faulthandler
from functools import partial
import gc
import hashlib
import json
import os
from pathlib import Path
import statistics
import threading
import time
import tracemalloc
from typing import Any

from benchmark import BASE, main, rss
from gpu_common import quiet_gate


def wrapper_depth(value: Any) -> int:
    """Count explicit callable wrappers iteratively, without traversing models."""
    seen: set[int] = set()
    while id(value) not in seen:
        seen.add(id(value))
        if isinstance(value, partial):
            value = value.func
        elif hasattr(value, "__wrapped__"):
            value = value.__wrapped__
        else:
            break
    return len(seen)


async def probe(args: Any, engine: Any, destination: Path) -> dict[str, object]:
    import httpx
    import uvicorn
    from websockets.asyncio.client import connect
    from nemu_voice_sidecar.app import create_app
    from nemu_voice_sidecar.voices import VoiceBank
    from nemu_voice_sidecar.cuda_graph import Step

    case = os.environ["NEMU_TTS_RESOURCE_CASE"]
    engine.generation_limit = 64  # Identical development limit; formal benchmark stays 512.
    captures = 0
    original_step = Step.__init__

    def counted_step(self: Any, *a: Any, **kw: Any) -> None:
        nonlocal captures
        captures += 1
        original_step(self, *a, **kw)

    Step.__init__ = counted_step  # type: ignore[method-assign]
    server = uvicorn.Server(uvicorn.Config(create_app(engine, VoiceBank(args.voice)),
                            host="127.0.0.1", port=args.port, access_log=False, log_level="warning"))
    serving = asyncio.create_task(server.serve())
    while not server.started:
        if serving.done():
            await serving
            raise RuntimeError("SERVER_NOT_READY")
        await asyncio.sleep(.05)
    rows: list[dict[str, Any]] = []

    def state() -> dict[str, Any]:
        current, peak = tracemalloc.get_traced_memory()
        graphs = engine.graphs
        predictor = engine.model.model.talker.code_predictor
        inner = engine.model.model.talker.model
        host_stats = getattr(engine.torch.cuda.memory, "host_memory_stats", None)
        return {"rss": rss(), "python_current": current, "python_peak": peak,
                "tracer_metadata_bytes": tracemalloc.get_tracemalloc_memory(),
                "threads": threading.active_count(), "captures": captures,
                "talker_graphs": len(graphs.talker_steps) if graphs else 0,
                "predictor_graphs": len(graphs.predictor_steps) if graphs else 0,
                "pool": str(graphs.pool) if graphs else None,
                "decode_stream": engine.decode_stream.cuda_stream if engine.decode_stream else None,
                "talker_depth": wrapper_depth(inner.forward),
                "predictor_depth": wrapper_depth(predictor.forward),
                "decode_depth": wrapper_depth(engine.model.model.speech_tokenizer.decode),
                "hooks": len(engine.model.model.talker._forward_hooks),
                "host_allocator": host_stats() if host_stats else None,
                "torch": engine.health()["vram"]}

    async def utterance() -> dict[str, Any]:
        quiet_gate()
        engine.torch.manual_seed(731)
        started = time.perf_counter()
        first = last = None
        size = 0
        digest = hashlib.sha256()
        async with connect(f"ws://127.0.0.1:{args.port}/synthesize", proxy=None) as ws:
            await ws.send(json.dumps({"text": BASE["zh"][0], "lang": "zh", "style": "neutral",
                                      "voiceId": "p4-02-synthetic-test", "sampleRate": 24000}))
            async for block in ws:
                if not isinstance(block, bytes):
                    raise RuntimeError("INVALID_AUDIO")
                last = time.perf_counter()
                if first is None or last is None:
                    first = last
                size += len(block)
                digest.update(block)
        async with httpx.AsyncClient(trust_env=False) as client:
            while (await client.get(f"http://127.0.0.1:{args.port}/health")).json()["busy"]:
                await asyncio.sleep(.01)
        if first is None or last is None:
            raise RuntimeError("NO_AUDIO")
        return {"first_ms": (first-started)*1000, "last_ms": (last-started)*1000,
                "bytes": size, "audio_sha256": digest.hexdigest(), "elapsed": time.perf_counter()-started}

    try:
        async with httpx.AsyncClient(trust_env=False, timeout=300) as client:
            response = await client.post(f"http://127.0.0.1:{args.port}/warmup")
            response.raise_for_status()
            if not response.json()["ready"]:
                raise RuntimeError("WARMUP_FAILED")
        for _ in range(3):
            await utterance()
        gc.collect()  # Equal pre-run baseline only; never collect between requests.
        tracemalloc.start(1)
        before = tracemalloc.take_snapshot()
        start = time.monotonic()
        initial = state()
        with (destination / "samples.jsonl").open("x", encoding="utf-8") as raw:
            while time.monotonic()-start < 240:
                row = {"index": len(rows), **await utterance(), "seconds": time.monotonic()-start, **state()}
                raw.write(json.dumps(row)+"\n")
                raw.flush()
                rows.append(row)
                print(case, row["index"], row["rss"], row["python_current"], row["captures"], flush=True)
        after = tracemalloc.take_snapshot()
        differences = [{"bytes": s.size_diff, "count": s.count_diff,
                        "traceback": str(s.traceback)} for s in after.compare_to(before, "lineno")[:25]]
        pre_gc = state()
        collected = gc.collect()
        post_gc = state()
        tracemalloc.stop()
        times = [r["seconds"] for r in rows]
        return {"acceptance": False, "case": case, "seconds": times[-1], "requests": len(rows),
                "initial": initial, "final": rows[-1], "pre_gc": pre_gc, "post_gc": post_gc,
                "end_gc_collected": collected, "tracemalloc_differences": differences,
                "rss_slope_mib_hour": statistics.linear_regression(times, [r["rss"]/2**20 for r in rows]).slope*3600,
                "python_slope_mib_hour": statistics.linear_regression(times, [r["python_current"]/2**20 for r in rows]).slope*3600,
                "note": "Fixed seed/greedy and identical text; diagnostic tracer overhead applies equally. No 20-30 min criterion assessed."}
    finally:
        tracemalloc.stop()
        Step.__init__ = original_step  # type: ignore[method-assign]
        server.should_exit = True
        await serving


if __name__ == "__main__":
    faulthandler.enable(all_threads=True)
    case = os.environ["NEMU_TTS_RESOURCE_CASE"]
    graph, asynchronous, frames = {
        "eager": ("0", "0", "4"), "graph": ("1", "0", "4"),
        "async": ("0", "1", "4"), "first2": ("0", "0", "2"),
        "combined": ("1", "1", "2"),
    }[case]
    os.environ.update(NEMU_TTS_CUDA_GRAPH=graph, NEMU_TTS_ASYNC_DECODE=asynchronous,
                      NEMU_TTS_FIRST_FRAMES=frames, NEMU_TTS_GREEDY="1")
    os.environ.pop("NEMU_TTS_TRACE_DIR", None)
    main(probe)
