"""Real GPU acceptance harness: serial variants, carrier+render resident, raw JSONL.

Run each variant in a fresh process. No simulated fallback is accepted as a result
for the requested variant. Pressure is at least 30 minutes; the approved retest criterion extends failures to two hours.
"""
from __future__ import annotations

import argparse
import asyncio
import ctypes
from ctypes import wintypes
from datetime import datetime, timezone
from functools import cache
import importlib
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import time
from typing import Any, cast
from collections.abc import Callable, Coroutine, Iterator
from contextlib import contextmanager

from pressure import assess, slope_mib_hour
from gpu_common import add_service_packages, load_carrier, quiet_gate

BASE = {
    "zh": ["今天的天气很好。", "我们慢慢来。", "窗边有一只小鸟。", "请记得喝水。", "我在这里。",
           "这本书很有意思。", "你今天感觉怎么样？", "我们先休息一下。", "晚饭已经准备好了。", "谢谢你的帮助。",
           "外面开始下雨了。", "花园里的花开了。", "明天一起去散步吧。", "我听到了你的声音。", "这个颜色很漂亮。",
           "请把窗户关上。", "今天辛苦了。", "我们从这里开始。", "小猫正在睡觉。", "祝你有个好梦。"],
    "en": ["The weather is nice today.", "Let us take our time.", "A bird is by the window.", "Please remember to drink water.", "I am right here.",
           "This book is interesting.", "How are you feeling today?", "Let us take a short break.", "Dinner is ready.", "Thank you for your help.",
           "It has started raining outside.", "The flowers are blooming.", "Let us walk together tomorrow.", "I heard your voice.", "That color is lovely.",
           "Please close the window.", "You worked hard today.", "Let us start from here.", "The kitten is sleeping.", "Have a peaceful dream."],
    "ja": ["今日はいい天気ですね。", "ゆっくり進めましょう。", "窓のそばに小鳥がいます。", "水を飲むのを忘れないでね。", "私はここにいます。",
           "この本は面白いですね。", "今日はどんな気分ですか？", "少し休憩しましょう。", "夕ご飯の準備ができました。", "手伝ってくれてありがとう。",
           "外は雨が降り始めました。", "庭の花が咲いています。", "明日は一緒に散歩しましょう。", "あなたの声が聞こえました。", "この色はとても綺麗ですね。",
           "窓を閉めてください。", "今日はお疲れさまでした。", "ここから始めましょう。", "子猫は眠っています。", "いい夢を見てね。"],
}


@cache
def _rss_reader() -> Callable[[], int]:
    """Bind the Windows API and its ctypes types once per measurement process."""
    if sys.platform != "win32":
        raise RuntimeError("formal RSS measurement requires Windows")

    class ProcessMemoryCountersEx(ctypes.Structure):
        _fields_ = [
            ("cb", wintypes.DWORD),
            ("PageFaultCount", wintypes.DWORD),
            ("PeakWorkingSetSize", ctypes.c_size_t),
            ("WorkingSetSize", ctypes.c_size_t),
            ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
            ("QuotaPagedPoolUsage", ctypes.c_size_t),
            ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
            ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
            ("PagefileUsage", ctypes.c_size_t),
            ("PeakPagefileUsage", ctypes.c_size_t),
            ("PrivateUsage", ctypes.c_size_t),
        ]

    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    psapi = ctypes.WinDLL("psapi", use_last_error=True)
    open_process = kernel32.OpenProcess
    open_process.argtypes = (wintypes.DWORD, wintypes.BOOL, wintypes.DWORD)
    open_process.restype = wintypes.HANDLE
    close_handle = kernel32.CloseHandle
    close_handle.argtypes = (wintypes.HANDLE,)
    close_handle.restype = wintypes.BOOL
    get_memory = psapi.GetProcessMemoryInfo
    get_memory.argtypes = (wintypes.HANDLE, ctypes.POINTER(ProcessMemoryCountersEx), wintypes.DWORD)
    get_memory.restype = wintypes.BOOL

    def read() -> int:
        handle = open_process(0x0400 | 0x0010, False, os.getpid())
        if not handle:
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            counters = ProcessMemoryCountersEx()
            counters.cb = ctypes.sizeof(counters)
            if not get_memory(handle, ctypes.byref(counters), counters.cb):
                raise ctypes.WinError(ctypes.get_last_error())
            return int(counters.WorkingSetSize)
        finally:
            close_handle(handle)

    return read


def rss() -> int:
    """Read physical working set without retaining a new ctypes type per sample."""
    if sys.platform != "win32":
        raise RuntimeError("formal RSS measurement requires Windows")
    return _rss_reader()()


@contextmanager
def trace_phase(phase: str) -> Iterator[None]:
    """Collect step-level CUDA events for latency samples, not long stress runs."""
    configured = os.environ.get("NEMU_TTS_TRACE_DIR")
    if configured and phase not in {"warmup", "latency"}:
        os.environ.pop("NEMU_TTS_TRACE_DIR", None)
    try:
        yield
    finally:
        if configured is not None:
            os.environ["NEMU_TTS_TRACE_DIR"] = configured


async def measure(args: argparse.Namespace, engine: Any, destination: Path) -> dict[str, object]:
    assert engine.generation_limit == 512, "formal generation limit must remain unchanged"
    import httpx
    import uvicorn
    from websockets.asyncio.client import connect
    from nemu_voice_sidecar.app import create_app
    from nemu_voice_sidecar.voices import VoiceBank

    server = uvicorn.Server(uvicorn.Config(create_app(engine, VoiceBank(args.voice)),
                                         host="127.0.0.1", port=args.port,
                                         access_log=False, log_level="warning"))
    serving = asyncio.create_task(server.serve())
    while not server.started:
        if serving.done():
            await serving
            raise RuntimeError("TTS server did not start")
        await asyncio.sleep(.05)
    results: dict[str, object] = {}
    raw = (destination / "samples.jsonl").open("x", encoding="utf-8")

    async def utterance(lang: str, text: str, phase: str, index: int,
                        cancel_test: bool = False) -> dict[str, object]:
        quiet_gate()
        started = time.perf_counter()
        first: float | None = None
        total = 0
        last = 0.0
        cancel_at: float | None = None
        last_after_cancel = 0.0
        with trace_phase(phase):
            async with connect(f"ws://127.0.0.1:{args.port}/synthesize", max_queue=8,
                               open_timeout=10, close_timeout=5, proxy=None) as socket:
                await socket.send(json.dumps({"text": text, "lang": lang, "style": "neutral",
                                              "voiceId": "p4-02-synthetic-test", "sampleRate": 24000}))
                async for block in socket:
                    if not isinstance(block, bytes) or len(block) % 4:
                        raise RuntimeError("INVALID_PCM")
                    now = time.perf_counter()
                    if first is None:
                        first = (now - started) * 1000
                    last = now-started
                    total += len(block)
                    if cancel_test and cancel_at is None:
                        cancel_at = time.perf_counter()
                        await socket.send('{"cancel":true}')
                    elif cancel_at is not None:
                        last_after_cancel = (now - cancel_at) * 1000
            # Await actual generation termination before starting the next sample.
            async with httpx.AsyncClient(trust_env=False) as client:
                deadline = time.monotonic() + 120
                while (await client.get(f"http://127.0.0.1:{args.port}/health")).json()["busy"]:
                    if time.monotonic() > deadline:
                        raise RuntimeError("INFERENCE_DID_NOT_STOP")
                    await asyncio.sleep(.01)
        if first is None or total == 0:
            raise RuntimeError("NO_AUDIO")
        row = {"phase": phase, "lang": lang, "index": index, "first_packet_ms": first,
               "elapsed_s": time.perf_counter() - started, "bytes": total,
               "last_packet_ms": last*1000,
               "audio_per_stream_second": (total/4/24000)/(last-first/1000) if last > first/1000 else None,
               "cancel_last_block_ms": last_after_cancel if cancel_at is not None else None,
               "utc": datetime.now(timezone.utc).isoformat(), "rss_bytes": rss(),
               "timing": engine.last_timing, **engine.health()}
        raw.write(json.dumps(row) + "\n")
        raw.flush()
        print(f"{phase} {lang} {index}: {first:.2f} ms, {total} bytes", flush=True)
        return row

    try:
        async with httpx.AsyncClient(timeout=300, trust_env=False) as client:
            response = await client.post(f"http://127.0.0.1:{args.port}/warmup")
            response.raise_for_status()
            health = response.json()
            if not health["ready"]:
                raise RuntimeError("TTS_WARMUP_DEGRADED")
        if engine.variant != args.variant:
            raise RuntimeError("REQUESTED_VARIANT_NOT_MEASURED")
        for lang in BASE:
            await utterance(lang, BASE[lang][0], "warmup", 0)
        for lang, texts in BASE.items():
            times = []
            for index, text in enumerate(texts):
                row = await utterance(lang, text, "latency", index)
                times.append(cast(float, row["first_packet_ms"]))
            results[lang] = {"samples": len(times), "p50_ms": statistics.median(times),
                             "limit_ms": 250, "passed": statistics.median(times) <= 250}
        cancellation = []
        for lang in BASE:
            for index in range(5):
                row = await utterance(lang, BASE[lang][0], "cancel", index, True)
                cancellation.append(cast(float, row["cancel_last_block_ms"]))
        results["cancel"] = {"samples": len(cancellation), "limit_ms": 50,
                             "maximum_last_block_ms": max(cancellation),
                             "passed": max(cancellation) <= 50}
        stress_start = time.monotonic()
        index = 0
        pressure: list[tuple[float, int, int]] = []
        while time.monotonic() - stress_start < 1800:
            lang = list(BASE)[index % 3]
            row = await utterance(lang, BASE[lang][index % 20], "stress", index)
            pressure.append((time.monotonic()-stress_start, cast(int, row["rss_bytes"]),
                             int(engine.health()["vram"]["allocatedBytes"])))
            index += 1
        assessment = assess(pressure)
        if assessment["extension_required"]:
            while time.monotonic()-stress_start < 7200:
                lang = list(BASE)[index % 3]
                row = await utterance(lang, BASE[lang][index % 20], "stress", index)
                pressure.append((time.monotonic()-stress_start, cast(int, row["rss_bytes"]),
                                 int(engine.health()["vram"]["allocatedBytes"])))
                index += 1
            assessment["rss_110_120_min_slope_mib_hour"] = slope_mib_hour(pressure, 6600, 7200)
            assessment["allocated_at_two_hours_bytes"] = pressure[-1][2]
            assessment["rss_first_to_two_hours_delta_mib"] = (pressure[-1][1]-pressure[0][1])/2**20
            assessment["extended_to_two_hours"] = True
            assessment["decision"] = "requires user judgment; extension does not automatically pass"
        results["stress"] = {"seconds": time.monotonic() - stress_start, "iterations": index,
                             "assessment": assessment, "policy_samples": pressure}
        return results
    finally:
        raw.close()
        server.should_exit = True
        await serving


def main(measurement: Callable[[argparse.Namespace, Any, Path], Coroutine[Any, Any, dict[str, object]]] = measure) -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--variant", choices=["0.6B", "1.7B"], required=True)
    parser.add_argument("--probe", type=Path, required=True)
    parser.add_argument("--cache", type=Path, required=True)
    parser.add_argument("--voice", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--port", type=int, default=17864)
    args = parser.parse_args()
    quiet_gate()
    add_service_packages()
    destination = args.output.resolve()
    destination.mkdir(parents=True, exist_ok=False)
    os.environ.update(NEMU_TTS_VARIANT=args.variant, NEMU_TTS_CACHE=str(args.cache.resolve()),
                      HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1")
    render_stop = destination / "render.stop"
    memory_stop = destination / "memory.stop"
    ready = destination / "render.json"
    with (destination / "render.log").open("x", encoding="utf-8") as rlog, \
         (destination / "memory-watch.log").open("x", encoding="utf-8") as mlog:
        watcher = subprocess.Popen(["powershell.exe", "-NoProfile", "-File",
                                    str(Path(__file__).with_name("bounded_memory_watch.ps1")),
                                    "-OutputPath", str(destination / "memory.jsonl"),
                                    "-StopPath", str(memory_stop), "-RootProcessIds", str(os.getpid())],
                                   stdout=mlog, stderr=subprocess.STDOUT, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        render = subprocess.Popen([shutil.which("node") or "node", str(args.probe / "render_load.mjs"),
                                   str(ready), str(render_stop)], stdout=rlog, stderr=subprocess.STDOUT,
                                  creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        result: dict[str, object] = {"variant": args.variant, "status": "failed",
                                     "rss_sampler": "GetProcessMemoryInfo-cached-types-v2"}
        try:
            deadline = time.monotonic() + 90
            while not ready.is_file():
                if render.poll() is not None or time.monotonic() > deadline:
                    raise RuntimeError("RENDER_NOT_READY")
                time.sleep(.1)
            carrier = load_carrier(args.probe)
            from nemu_voice_sidecar.qwen import QwenEngine
            engine = QwenEngine(measure_exact_variant=True)
            result.update(asyncio.run(measurement(args, engine, destination)))
            result["status"] = "measured"
            result["carrier"] = {key: value for key, value in carrier.items()
                                 if key not in ("model", "cache")}
        except Exception as exc:
            result["error"] = str(exc)
            raise
        finally:
            render_stop.touch()
            memory_stop.touch()
            result["render_exit"] = render.wait(timeout=45)
            result["memory_watch_exit"] = watcher.wait(timeout=30)
            sys_path = importlib.import_module("sys").path
            sys_path.insert(0, str(args.probe))
            summary = importlib.import_module("summarize_supplement").summarize_memory(destination / "memory.jsonl")
            result["memory"] = summary
            peaks = summary["sampled_peaks"]
            result["nemu_memory_upper_bound_bytes"] = peaks["dedicated_bytes"] + peaks["shared_bytes"]
            result["adr0012_limit_bytes"] = 8_000_000_000
            result["memory_evidence_available"] = (
                summary["observations"] > summary["no_owned_gpu_counter_samples"]
                and not summary["errors"])
            result["vram_within_limit"] = (result["memory_evidence_available"] and
                                           cast(int, result["nemu_memory_upper_bound_bytes"]) <= 8_000_000_000)
            (destination / "result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
