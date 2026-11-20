"""GPT-SoVITS is imported only in this child. The upstream prints request text."""
from __future__ import annotations

import hashlib
import importlib
import json
import os
import sys
from pathlib import Path

if sys.platform == "win32":
    from multiprocessing.connection import PipeConnection as ModelPipe
else:
    from multiprocessing.connection import Connection as ModelPipe


__all__ = ["ModelPipe", "run_worker", "verify_files"]


def _offline(event: str, args: tuple[object, ...]) -> None:
    if event in {"socket.connect", "socket.getaddrinfo", "socket.sendto",
                 "socket.gethostbyname", "socket.gethostbyaddr", "socket.sendmsg"}:
        raise PermissionError("OFFLINE_RUNTIME")


def verify_files(root: Path) -> None:
    inventory = Path(__file__).with_name("gsv-v5-files.json")
    for name, expected in json.loads(inventory.read_text(encoding="utf-8")).items():
        target = root / name
        if not target.is_file():
            raise RuntimeError("MODEL_FILES_MISSING")
        with target.open("rb") as stream:
            if hashlib.file_digest(stream, "sha256").hexdigest() != expected:
                raise RuntimeError("MODEL_HASH_MISMATCH")


def run_worker(pipe: ModelPipe, root: Path) -> None:
    # Silence both Python and native output, keeping the inference pipe separate.
    with open(os.devnull, "wb") as sink:
        os.dup2(sink.fileno(), 1)
        os.dup2(sink.fileno(), 2)
    dll = None
    try:
        verify_files(root)
        sys.addaudithook(_offline)
        os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", MPLBACKEND="Agg")
        dll_dir = os.getenv("NEMU_GSV_FFMPEG_BIN")
        # TorchCodec resolves its FFmpeg shared libraries through the Windows loader.
        if sys.platform == "win32" and dll_dir:
            dll = os.add_dll_directory(dll_dir)
        os.chdir(root)
        sys.path[:0] = [str(root), str(root / "GPT_SoVITS")]
        ort = importlib.import_module("onnxruntime")
        torch = importlib.import_module("torch")
        if not torch.cuda.is_available():
            raise RuntimeError("CUDA_UNAVAILABLE")
        torch.set_num_threads(4)  # Same thread pool as the qualified V5 configuration.
        providers = ort.get_available_providers
        try:
            setattr(ort, "get_available_providers", lambda: ["CPUExecutionProvider"])
            upstream = importlib.import_module("TTS_infer_pack.TTS")
            frontend = importlib.import_module("text.chinese2")
        finally:
            setattr(ort, "get_available_providers", providers)
        if frontend.g2pw._g2pw.session_g2pw.get_providers() != ["CPUExecutionProvider"]:
            raise RuntimeError("G2PW_PROVIDER_MISMATCH")
        # Load the English dictionaries at warmup, before a user waits for speech.
        importlib.import_module("text.english")
        weights = root / "GPT_SoVITS" / "pretrained_models"
        config = upstream.TTS_Config({"custom": {
            "version": "v5turbo", "device": "cuda", "is_half": True,
            "t2s_weights_path": str(weights / "s1v3.ckpt"),
            "vits_weights_path": str(weights / "gsv-v5-pretrained/s2Gv5turbo.pth"),
            "bert_base_path": str(weights / "chinese-roberta-wwm-ext-large"),
            "cnhuhbert_base_path": str(weights / "chinese-hubert-base"),
        }})
        np = importlib.import_module("numpy")
        signal = importlib.import_module("scipy.signal")

        def vram() -> dict[str, int]:
            return {"allocatedBytes": int(torch.cuda.memory_allocated()),
                    "reservedBytes": int(torch.cuda.memory_reserved()),
                    "peakAllocatedBytes": int(torch.cuda.max_memory_allocated()),
                    "peakReservedBytes": int(torch.cuda.max_memory_reserved())}

        # The fixed model has no user-editable runtime configuration to persist.
        config.configs_path = os.devnull
        model = upstream.TTS(config)
        pipe.send(("ready", vram()))
        while True:
            request, reference = pipe.recv()
            emitted = False
            for rate, audio in model.run({
                "text": request.text, "text_lang": request.lang,
                "ref_audio_path": str(reference.path), "prompt_text": reference.text,
                "prompt_lang": "all_zh" if request.lang == "zh" else "en",
                "speed_factor": 1.0, "parallel_infer": False, "streaming_mode": False,
                "return_fragment": False, "text_split_method": "cut0", "seed": 42,
            }):
                if rate != 48000 or audio.ndim != 1 or audio.dtype != np.int16 or not np.any(audio):
                    raise RuntimeError("INVALID_MODEL_PCM")
                pcm = signal.resample_poly(audio.astype(np.float32) / 32768, 1, 2).astype("<f4")
                # 64 KiB stays below the client's per-message limit.
                for offset in range(0, len(pcm), 16384):
                    pipe.send(("pcm", pcm[offset:offset + 16384].tobytes()))
                emitted = True
            if not emitted:
                raise RuntimeError("INVALID_MODEL_PCM")
            pipe.send(("done", vram()))
    except (EOFError, BrokenPipeError):
        pass
    except Exception as exc:
        known = {"MODEL_FILES_MISSING", "MODEL_HASH_MISMATCH", "CUDA_UNAVAILABLE",
                 "G2PW_PROVIDER_MISMATCH", "INVALID_MODEL_PCM"}
        reason = str(exc) if str(exc) in known else type(exc).__name__
        pipe.send(("error", reason))
    finally:
        if dll is not None:
            dll.close()
        pipe.close()
