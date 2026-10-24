"""Reproduce development timing, seam and byte-level audio comparisons; no CUDA."""
from __future__ import annotations
from collections import defaultdict
import hashlib
import importlib
import json
from pathlib import Path
import statistics
from typing import Any


def summarize(root: Path) -> dict[str, Any]:
    sf, np = importlib.import_module("soundfile"), importlib.import_module("numpy")
    result: dict[str, Any] = {}
    for case, baseline in (("optimization-0.6B-v9", "optimization-0.6B-v3"),
                           ("optimization-1.7B-v4", "optimization-1.7B-v1")):
        folder = root/case
        data = json.loads((folder/"development.json").read_text(encoding="utf-8"))
        memory = json.loads((folder/"result.json").read_text(encoding="utf-8"))
        result[case] = {"memory_upper_bound_bytes": memory["nemu_memory_upper_bound_bytes"],
                        "memory": memory["memory"], "langs": {}}
        for lang in ("zh", "en", "ja"):
            entry = data["graph2-async"][lang]
            trace = json.loads((folder/"graph2-async"/entry["trace"]).read_text(encoding="utf-8"))
            grouped: dict[str, list[float]] = defaultdict(list)
            for span in trace["spans"]:
                grouped[span["name"]].append(span["cuda_ms"])
            waveform, rate = sf.read(str(folder/"graph2-async"/(lang+".wav")), dtype="float32")
            baseline4, _ = sf.read(str(root/baseline/"eager4"/(lang+".wav")), dtype="float32")
            baseline2, _ = sf.read(str(root/baseline/"eager2"/(lang+".wav")), dtype="float32")
            sync, _ = sf.read(str(folder/"graph2"/(lang+".wav")), dtype="float32")
            def seam(first: int, wave: Any) -> dict[str, object]:
                positions = list(range(first*1920, len(wave), 4*1920))
                jumps = [float(abs(wave[i]-wave[i-1])) for i in positions]
                return {"count": len(jumps), "maximum_jump": max(jumps, default=0), "positions_samples": positions}
            result[case]["langs"][lang] = {
                "first_emit_ms": entry["first_emit_ms"], "last_emit_ms": entry["last_emit_ms"],
                "audio_seconds": entry["audio_seconds"], "realtime_ratio": entry["audio_per_stream_second"],
                "codec_equal_upstream": entry["codec_equal_upstream"], "codec_frames": entry["frame_count"],
                "cuda_stages": {name: {"count":len(values), "sum_ms":sum(values), "median_ms":statistics.median(values)}
                                for name,values in grouped.items()},
                "seams_2_frame_first": seam(2, waveform),
                "seams_original_4_frame_first": seam(4, baseline4),
                "two_vs_four_frame_pcm_max_abs_diff": float(np.max(np.abs(waveform-baseline4))) if len(waveform)==len(baseline4) else None,
                "same_partition_eager_pcm_equal": bool(np.array_equal(waveform, baseline2)),
                "same_partition_eager_pcm_max_abs_diff": float(np.max(np.abs(waveform-baseline2))) if len(waveform)==len(baseline2) else None,
                "sync_vs_async_pcm_equal": bool(np.array_equal(waveform,sync)),
                "sync_vs_async_pcm_max_abs_diff": float(np.max(np.abs(waveform-sync))) if len(waveform)==len(sync) else None,
                "sample_rate": rate, "torch": entry["torch"]}
    return result


def main() -> None:
    root = Path(__file__).resolve().parents[2]/"reports/P4-02"
    result = summarize(root)
    (root/"optimization-summary.json").write_text(json.dumps(result,indent=2)+"\n",encoding="utf-8")
    assets=[]
    refs=json.loads((root/"test-voice-v1/voice.json").read_text(encoding="utf-8"))["references"]
    for wav in sorted(root.glob("optimization-*/**/*.wav")):
        lang=wav.stem if wav.stem in refs else "zh"
        assets.append({"path":wav.relative_to(root).as_posix(), "sha256":hashlib.sha256(wav.read_bytes()).hexdigest(),
            "source":"P4-02 Qwen3-TTS Base synthesis using fictional VoiceDesign reference",
            "reference_sha256":refs[lang]["neutral"]["sha256"], "license":"Apache-2.0",
            "cloneAllowed":True,"redistributable":True,"purpose":"development test, not final Nemu voice"})
    (root/"optimization-audio-manifest.json").write_text(json.dumps({"assets":assets},indent=2)+"\n",encoding="utf-8")
    inventory=[]
    for file in sorted(root.glob("optimization-*/**/*")):
        if file.is_file():
            with file.open("rb") as stream:
                digest=hashlib.file_digest(stream,"sha256").hexdigest()
            inventory.append({"path":file.relative_to(root).as_posix(),"bytes":file.stat().st_size,"sha256":digest})
    (root/"optimization-evidence-sha256.json").write_text(json.dumps(inventory,indent=2)+"\n",encoding="utf-8")
    print("Recomputed development CUDA stages, exact codec comparisons, PCM/seams and evidence hashes")


if __name__ == "__main__":
    main()
