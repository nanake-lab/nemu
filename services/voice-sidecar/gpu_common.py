"""Explicit Windows quiet-window gate and pinned carrier residency for P4-02 only."""
from __future__ import annotations

import csv
import hashlib
import importlib
import importlib.metadata
import json
import os
from pathlib import Path
import subprocess
import sys
from typing import Any

ROOT = Path(__file__).resolve().parents[2]


def quiet_gate() -> None:
    if os.name != "nt" or os.getenv("NEMU_P4_02_QUIET_WINDOW") != "user-confirmed":
        raise RuntimeError("GPU run requires Windows and explicit user-confirmed quiet window")
    forbidden = {"runner.listener.exe", "runner.worker.exe", "blender.exe"}
    output = subprocess.run(["tasklist", "/FO", "CSV", "/NH"], check=True,
                            capture_output=True, text=True)
    running = {row[0].casefold() for row in csv.reader(output.stdout.splitlines()) if row}
    if forbidden & running:
        raise RuntimeError("Quiet window unavailable: " + ", ".join(sorted(forbidden & running)))


def add_service_packages() -> None:
    # Run under the existing P0-07 probe Python, without modifying that environment.
    sys.path.append(str(Path(__file__).parent / ".venv/Lib/site-packages"))
    sys.path.insert(0, str(Path(__file__).parent / "src"))


def load_carrier(probe: Path) -> Any:
    quiet_gate()
    lock = json.loads((ROOT / "config/models.lock.json").read_text(encoding="utf-8-sig"))
    selected = lock["p0_07_probe"]["selected"]
    entry = next(item for item in lock["p0_07_probe"]["models"] if item["id"] == selected["model_id"])
    config = entry["carrier_config"]
    for package, expected in (("torch", config["torch_version"]),
                              ("transformers", config["transformers_version"]),
                              ("bitsandbytes", config["bitsandbytes_version"])):
        if importlib.metadata.version(package) != expected:
            raise RuntimeError(f"Carrier environment mismatch: {package}")
    local = probe / ".cache/models" / entry["id"].replace("/", "--") / selected["revision"]
    # Local P0-07 inventory is checked against immutable HF metadata before reuse.
    metadata = json.loads((ROOT / "reports/P4-02/carrier-file-lock.json").read_text(encoding="utf-8-sig"))
    if metadata["revision"] != selected["revision"] or metadata["id"] != selected["model_id"]:
        raise RuntimeError("Carrier metadata revision mismatch")
    for item in metadata["files"]:
        with (local / item["path"]).open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        if digest != item["sha256"]:
            raise RuntimeError("Carrier file hash mismatch")
    torch = importlib.import_module("torch")
    api = importlib.import_module("transformers")
    torch.set_num_threads(4)
    torch.backends.cuda.matmul.allow_tf32 = False
    torch.backends.cudnn.allow_tf32 = False
    torch.manual_seed(731)
    model = api.AutoModelForCausalLM.from_pretrained(
        str(local), local_files_only=True, trust_remote_code=False, dtype=torch.bfloat16,
        device_map={"": "cuda:0"}, attn_implementation=config["attention"],
        quantization_config=api.BitsAndBytesConfig(
            load_in_4bit=True, bnb_4bit_quant_type="nf4", bnb_4bit_compute_dtype=torch.bfloat16,
            bnb_4bit_use_double_quant=config["double_quantization"]))
    sys.path.insert(0, str(probe))
    norms = importlib.import_module("native_norm")
    norms.replace_rms_norms(model)
    # Rank-8 hook with m=0, at the selected attention-residual boundary.
    wrapper_api = importlib.import_module("injection")
    layers = model.model.layers
    index = config["layer_index_zero_based"]
    basis = torch.eye(model.config.hidden_size, config["rank"], device="cuda:0")
    wrapper = wrapper_api.InjectedDecoder(layers[index], basis, beta=1.0)
    wrapper.begin((0.0,) * config["rank"])
    layers[index] = wrapper
    model.eval().requires_grad_(False)
    tokenizer = api.AutoTokenizer.from_pretrained(str(local), local_files_only=True)
    fixture = "The window is open. 今天的天气很好。今日はいい天気ですね。"
    ids = tokenizer(fixture, return_tensors="pt")["input_ids"].to("cuda:0")
    prefill_tokens = int(ids.shape[1])
    cache = None
    with torch.inference_mode():
        for _ in range(48):
            out = model(input_ids=ids, past_key_values=cache, use_cache=True)
            cache = out.past_key_values
            ids = out.logits[:, -1, :].argmax(-1, keepdim=True)
    # Preserve the real KV allocation alongside weights throughout the TTS run.
    return {"model": model, "cache": cache, "prefill_tokens": prefill_tokens,
            "decode_steps": 48, "m": [0.0] * config["rank"], "carrier_config": config}
