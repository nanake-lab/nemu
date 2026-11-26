"""Development-only graph pool/shape comparison, with codec and PCM hashes."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
from threading import Event
import time
from typing import Any

from benchmark import main


async def measure_memory(args: Any, engine: Any, destination: Path) -> dict[str, object]:
    from nemu_voice_sidecar.backend import Request
    from nemu_voice_sidecar.cuda_graph import Step
    from nemu_voice_sidecar.voices import VoiceBank

    original = Step.__init__
    separate = os.getenv("NEMU_TTS_PROBE_POOL") == "separate"

    def capture(self: Any, *a: Any, **kw: Any) -> None:
        if separate:
            kw["pool"] = None
        original(self, *a, **kw)

    Step.__init__ = capture  # type: ignore[method-assign]
    try:
        engine.generation_limit = 64
        engine.warmup()
        if engine.graphs:
            engine.graphs.frames_per_request = int(os.getenv("NEMU_TTS_PROBE_GRAPH_FRAMES", "16"))
        bank = VoiceBank(args.voice)
        outputs = {}
        for lang, text in {"zh": "我们慢慢来。", "en": "Let us take our time.", "ja": "ゆっくり進めましょう。"}.items():
            engine.torch.manual_seed(731)
            folder = destination / lang
            os.environ["NEMU_TTS_TRACE_DIR"] = str(folder)
            os.environ["NEMU_TTS_SAVE_CODEC"] = "1"
            packets: list[bytes] = []
            start = time.perf_counter()
            engine.stream(Request(text, lang, "neutral", "p4-02-synthetic-test"),
                          bank.select("p4-02-synthetic-test", lang, "neutral"), Event(), packets.append)
            trace = json.loads((folder / "traces.jsonl").read_text().splitlines()[-1])
            outputs[lang] = {"codec_sha256": hashlib.sha256(json.dumps(trace["codec_frames"]).encode()).hexdigest(),
                "pcm_sha256": hashlib.sha256(b"".join(packets)).hexdigest(),
                "seconds": time.perf_counter()-start, "first_ms": trace["first_emit_ms"],
                "graph_count": len(engine.graphs.talker_steps) if engine.graphs else 0,
                "torch": engine.health()["vram"]}
        return {"acceptance": False, "pool": "separate" if separate else "shared",
                "frames_per_request": engine.graphs.frames_per_request if engine.graphs else 0,
                "outputs": outputs, "note": "identical 64-frame greedy fixtures; formal limit unchanged"}
    finally:
        Step.__init__ = original  # type: ignore[method-assign]


if __name__ == "__main__":
    os.environ.update(NEMU_TTS_GREEDY="1", NEMU_TTS_ASYNC_DECODE=os.getenv("NEMU_TTS_PROBE_ASYNC", "0"),
                      NEMU_TTS_FIRST_FRAMES=os.getenv("NEMU_TTS_PROBE_FIRST_FRAMES", "4"))
    main(measure_memory)
