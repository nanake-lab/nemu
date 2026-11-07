"""One full-cap greedy codec check against preserved upstream development trace."""
from __future__ import annotations
import argparse
import hashlib
import importlib
import json
import os
from pathlib import Path
from threading import Event
from typing import Any
from benchmark import main
from gpu_common import quiet_gate


async def full_parity(args: argparse.Namespace, engine: Any, destination: Path) -> dict[str, object]:
    from nemu_voice_sidecar.backend import Request
    from nemu_voice_sidecar.voices import VoiceBank
    quiet_gate()
    reference = Path(os.environ["NEMU_TTS_FULL_PARITY_REFERENCE"])
    expected = json.loads(reference.read_text(encoding="utf-8"))["codec_frames"]
    engine.generation_limit = 512
    os.environ.update(NEMU_TTS_GREEDY="1", NEMU_TTS_SAVE_CODEC="1", NEMU_TTS_CUDA_GRAPH="1",
                      NEMU_TTS_ASYNC_DECODE="1", NEMU_TTS_FIRST_FRAMES="2",
                      NEMU_TTS_TRACE_DIR=str(destination / "full-trace"))
    engine.warmup()
    engine.torch.manual_seed(731)
    bank = VoiceBank(args.voice)
    packets: list[bytes] = []
    engine.stream(Request("我们慢慢来。", "zh", "neutral", "p4-02-synthetic-test"),
                  bank.select("p4-02-synthetic-test", "zh", "neutral"), Event(), packets.append)
    trace = next((destination / "full-trace").glob("*.json"))
    data = json.loads(trace.read_text(encoding="utf-8"))
    actual = data["codec_frames"]
    np, sf = importlib.import_module("numpy"), importlib.import_module("soundfile")
    sf.write(str(destination / "full.wav"), np.frombuffer(b"".join(packets), dtype="<f4"), 24000, subtype="FLOAT")
    def digest(codes: Any) -> str:
        return hashlib.sha256(json.dumps(codes, separators=(",", ":")).encode()).hexdigest()
    result = {"reference": str(reference), "reference_sha256": digest(expected), "actual_sha256": digest(actual),
              "expected_frames": len(expected), "actual_frames": len(actual), "codec_equal_upstream": actual == expected,
              "timing": engine.last_timing, "acceptance": False}
    (destination / "full-parity.json").write_text(json.dumps(result, indent=2)+"\n", encoding="utf-8")
    print(result, flush=True)
    if actual != expected:
        raise RuntimeError("FULL_CODEC_PARITY_FAILED")
    return result


if __name__ == "__main__":
    main(full_parity)
