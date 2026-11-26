"""Short development probe, not acceptance. Same carrier/render/memory harness."""
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


async def short_probe(args: argparse.Namespace, engine: Any, destination: Path) -> dict[str, object]:
    sf, np = importlib.import_module("soundfile"), importlib.import_module("numpy")
    from nemu_voice_sidecar.backend import Request
    from nemu_voice_sidecar.cuda_graph import ForwardGraphs
    from nemu_voice_sidecar.voices import VoiceBank
    bank = VoiceBank(args.voice)
    os.environ.update(NEMU_TTS_GREEDY="1", NEMU_TTS_SAVE_CODEC="1", NEMU_TTS_CUDA_GRAPH="0")
    engine.generation_limit = 64  # Development parity only; formal harness remains 512.
    engine.warmup()
    texts = {"zh": "我们慢慢来。", "en": "Let us take our time.", "ja": "ゆっくり進めましょう。"}
    results: dict[str, dict[str, Any]] = {}
    baseline = os.getenv("NEMU_TTS_PARITY_BASELINE")
    if baseline:
        prior = json.loads(Path(baseline).read_text(encoding="utf-8"))
        results = {mode: prior[mode] for mode in ("eager4", "eager2")}
    modes = (("eager4", False, False, "4"), ("eager2", False, False, "2"),
             ("predictor2", True, False, "2"), ("talker2", True, False, "2"),
             ("graph2", True, False, "2"), ("graph2-async", True, True, "2"))
    for name, graph, asynchronous, first_frames in modes:
        if os.getenv("NEMU_TTS_DIAGNOSE_COMPONENTS") == "1" and name not in ("predictor2", "talker2"):
            continue
        if not os.getenv("NEMU_TTS_DIAGNOSE_COMPONENTS") and name in ("predictor2", "talker2"):
            continue
        if baseline and not graph:
            continue
        quiet_gate()
        if graph and engine.graphs is None:
            with engine.torch.inference_mode():
                engine.graphs = ForwardGraphs(engine.model, engine.torch, generation_length=int(os.getenv("NEMU_TTS_GRAPH_CAPTURE_LENGTH", "512")))
        os.environ.update(NEMU_TTS_GRAPH_TALKER="0" if name == "predictor2" else "1",
                          NEMU_TTS_GRAPH_PREDICTOR="0" if name == "talker2" else "1")
        os.environ.update(NEMU_TTS_ASYNC_DECODE="1" if asynchronous else "0", NEMU_TTS_FIRST_FRAMES=first_frames)
        folder = destination / name
        os.environ["NEMU_TTS_TRACE_DIR"] = str(folder)
        results[name] = {}
        for lang, text in texts.items():
            quiet_gate()
            engine.torch.manual_seed(731)
            packets: list[bytes] = []
            engine.stream(Request(text, lang, "neutral", "p4-02-synthetic-test"),
                          bank.select("p4-02-synthetic-test", lang, "neutral"), Event(), packets.append)
            files = sorted(folder.glob("*.json"), key=lambda p: p.stat().st_mtime)
            data = json.loads(files[-1].read_text(encoding="utf-8"))
            audio = np.frombuffer(b"".join(packets), dtype="<f4")
            sf.write(str(folder / (lang+".wav")), audio, 24000, subtype="FLOAT")
            codec = json.dumps(data["codec_frames"], separators=(",", ":")).encode()
            seams = np.cumsum([len(packet)//4 for packet in packets])[:-1]
            results[name][lang] = {**engine.last_timing, "codec_sha256": hashlib.sha256(codec).hexdigest(),
                "frame_count": len(data["codec_frames"]), "trace": files[-1].name,
                "packet_boundary_jump_max": float(np.max(np.abs(audio[seams]-audio[seams-1]))) if len(seams) else 0,
                "all_sample_jump_max": float(np.max(np.abs(np.diff(audio)))),
                "torch": engine.health()["vram"],
                "captured_talker_positions": engine.health()["capturedTalkerPositions"],
                "graph_generation_length": engine.graphs.generation_length if engine.graphs else 0}
            if name != "eager4":
                results[name][lang]["codec_equal_upstream"] = results[name][lang]["codec_sha256"] == results["eager4"][lang]["codec_sha256"]
            print(name, lang, results[name][lang], flush=True)
            (destination / "development.json").write_text(json.dumps(results, indent=2)+"\n", encoding="utf-8")
    if not all(results[mode][lang]["codec_equal_upstream"] for mode in results if mode != "eager4" for lang in texts):
        raise RuntimeError("CODEC_PARITY_FAILED; development.json retains exact comparisons")
    return {"development": results, "reused_baseline": baseline, "acceptance": False, "note": "three greedy fixtures per mode capped identically at 64 tokens, not formal acceptance"}


if __name__ == "__main__":
    main(short_probe)
