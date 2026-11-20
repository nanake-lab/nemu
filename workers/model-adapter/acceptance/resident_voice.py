"""Load the P4-02 local GPT-SoVITS V5 resident voice for the joint WDDM test."""
from __future__ import annotations

import argparse
import importlib
import json
import os
from pathlib import Path
import sys
import time


def main() -> None:
    parser = argparse.ArgumentParser()
    for key in ("source", "nltk", "ffmpeg", "ready", "stop"):
        parser.add_argument("--" + key, required=True, type=Path)
    a = parser.parse_args()
    os.environ.update(NLTK_DATA=str(a.nltk), HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1",
                      MPLBACKEND="Agg")
    def local_only(event: str, args: tuple[object, ...]) -> None:
        if event in ("socket.connect", "socket.getaddrinfo"):
            raise RuntimeError("resident voice requires offline assets")
    sys.addaudithook(local_only)
    root = a.source.resolve()
    ready = a.ready.resolve()
    stop = a.stop.resolve()
    os.chdir(root)
    sys.path[:0] = [str(root), str(root / "GPT_SoVITS")]
    with os.add_dll_directory(str(a.ffmpeg)):
        ort = importlib.import_module("onnxruntime")
        ort.get_available_providers = lambda: ["CPUExecutionProvider"]  # P4-02 g2pw configuration
        torch = importlib.import_module("torch")
        torch.set_num_threads(4)
        tts = importlib.import_module("TTS_infer_pack.TTS")
        weights = root / "GPT_SoVITS/pretrained_models"
        config = tts.TTS_Config({"custom": dict(version="v5turbo", device="cuda", is_half=True,
            t2s_weights_path=str(weights / "s1v3.ckpt"),
            vits_weights_path=str(weights / "gsv-v5-pretrained/s2Gv5turbo.pth"),
            bert_base_path=str(weights / "chinese-roberta-wwm-ext-large"),
            cnhuhbert_base_path=str(weights / "chinese-hubert-base"))})
        config.configs_path = str(ready.parent / "voice-runtime.yaml")
        model = tts.TTS(config)
        torch.cuda.synchronize()
        allocated = torch.cuda.memory_allocated()
        if not model or not allocated:
            raise RuntimeError("voice weights not resident")
        ready.write_text(json.dumps(dict(allocated=allocated, resident=True)), "utf-8")
        while not stop.exists():
            time.sleep(0.25)


if __name__ == "__main__":
    main()
