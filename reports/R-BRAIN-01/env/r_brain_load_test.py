"""R-BRAIN-01: verify P0-12 inventory and smoke-test text-only NF4, offline.

Usage (Windows):
  & "D:\\Dev\\cache\\R-BRAIN-01\\.venv\\Scripts\\python.exe" reports/R-BRAIN-01/env/r_brain_load_test.py
An optional --model-dir overrides the inventory's machine-specific target path.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import sys
import traceback

# Process-local only; no model files or libraries downloaded by this script.
os.environ["HF_HUB_OFFLINE"] = "1"
os.environ["TRANSFORMERS_OFFLINE"] = "1"

REPORT_DIR = Path(__file__).resolve().parent
REPO_ROOT = REPORT_DIR.parents[2]
INVENTORY_FILE = REPO_ROOT / "reports/P0-12/preparation/resume-01/model-inventory.json"
LOG_FILE = REPORT_DIR / "QWEN-4BIT-CHECK-03.txt"
FREEZE_FILE = REPORT_DIR / "pip-freeze.txt"
log: list[str] = []


def record(*parts: object) -> None:
    line = " ".join(map(str, parts))
    print(line, flush=True)
    log.append(line)


def verify_inventory(model_dir: Path, inventory: dict) -> None:
    if inventory.get("repository") != "Qwen/Qwen3.5-4B":
        raise ValueError("Unexpected inventory repository")
    if inventory.get("revision") != model_dir.name:
        raise ValueError("Inventory revision does not match model directory")
    entries = inventory["files"]
    if not entries or not any(e["path"].endswith(".safetensors") for e in entries):
        raise ValueError("Inventory has no weights")
    for item in entries:
        relative = PurePosixPath(item["path"])
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError(f"Unsafe inventory path: {relative}")
        file = model_dir.joinpath(*relative.parts)
        if not file.resolve().is_relative_to(model_dir.resolve()) or not file.is_file():
            raise FileNotFoundError(f"Missing/outside model directory: {relative}")
        measured_size = file.stat().st_size
        if measured_size != item["bytes"]:
            raise ValueError(f"Size mismatch {relative}: {measured_size} != {item['bytes']}")
        with file.open("rb") as source:
            measured_sha = hashlib.file_digest(source, "sha256").hexdigest()
        if measured_sha.lower() != item["sha256"].lower():
            raise ValueError(f"SHA-256 mismatch: {relative}")
        if relative.suffix == ".safetensors":
            record("Verified weight:", relative, "bytes:", measured_size, "sha256:", measured_sha)
    record("Inventory verified:", f"{len(entries)}/{len(entries)}", "files, size + SHA-256 PASS")


def run(model_dir: Path, inventory: dict) -> None:
    record("R-BRAIN-01 offline 4-bit integrity test")
    record("Python:", sys.version.split()[0], "Executable:", sys.executable)
    record("Model:", model_dir)
    record("Inventory:", INVENTORY_FILE)
    verify_inventory(model_dir, inventory)  # Fail before loading if any byte differs.

    import torch
    import transformers
    import bitsandbytes
    from transformers import AutoConfig, AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig

    record("torch:", torch.__version__, "CUDA:", torch.version.cuda)
    record("transformers:", transformers.__version__, "bitsandbytes:", bitsandbytes.__version__)
    if not torch.cuda.is_available():
        raise RuntimeError("CUDA unavailable")
    record("GPU:", torch.cuda.get_device_name(0))
    config = AutoConfig.from_pretrained(str(model_dir), local_files_only=True)
    record("Checkpoint model_type:", config.model_type)
    record("Has vision_config:", hasattr(config, "vision_config"))

    tokenizer = AutoTokenizer.from_pretrained(str(model_dir), local_files_only=True)
    quant = BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_quant_type="nf4",
        bnb_4bit_use_double_quant=True,
        bnb_4bit_compute_dtype=torch.bfloat16,
    )
    torch.cuda.empty_cache()
    torch.cuda.reset_peak_memory_stats(0)
    record("Loading text-only NF4 model; requesting weight loading diagnostics...")
    model, info = AutoModelForCausalLM.from_pretrained(
        str(model_dir),
        local_files_only=True,
        quantization_config=quant,
        dtype=torch.bfloat16,
        device_map={"": 0},
        output_loading_info=True,
    )
    record("Loaded class:", type(model).__name__, "config:", model.config.model_type)
    if type(model).__name__ != "Qwen3_5ForCausalLM" or not model.is_loaded_in_4bit:
        raise RuntimeError("Wrong architecture or not quantized to 4-bit")
    if next(model.parameters()).device.type != "cuda":
        raise RuntimeError("Model parameters are not on GPU")

    missing = list(info.get("missing_keys") or [])
    mismatched = list(info.get("mismatched_keys") or [])
    errors = list(info.get("error_msgs") or [])
    unexpected = list(info.get("unexpected_keys") or [])
    # Only the image encoder is deliberately omitted from the CausalLM model.
    unexpected_text = [key for key in unexpected if not (
        key.startswith("visual.") or key.startswith("model.visual.")
    )]
    record("Missing keys count:", len(missing))
    record("Missing keys:", missing)
    record("Mismatched keys:", mismatched)
    record("Loader errors:", errors)
    record("Unexpected visual keys:", len(unexpected) - len(unexpected_text))
    record("Unexpected non-visual keys:", unexpected_text)
    if missing or mismatched or errors or unexpected_text:
        raise RuntimeError("Weight loading diagnostics failed; no training allowed")

    model.eval()
    encoded = tokenizer.apply_chat_template(
        [{"role": "user", "content": "请用一句简短的中文介绍你自己。"}],
        tokenize=True, add_generation_prompt=True, enable_thinking=False,
        return_tensors="pt",
    ).to("cuda:0")
    input_ids = encoded["input_ids"] if hasattr(encoded, "keys") else encoded
    with torch.inference_mode():
        output = model.generate(
            input_ids=input_ids,
            max_new_tokens=64,
            do_sample=False,
            pad_token_id=tokenizer.eos_token_id,
        )
    generated = tokenizer.decode(output[0, input_ids.shape[-1]:], skip_special_tokens=True)
    record("Generated text:", repr(generated))
    if not generated.strip():
        raise RuntimeError("Empty generated text")


def main() -> int:
    failure = False
    try:
        inventory = json.loads(INVENTORY_FILE.read_text(encoding="utf-8-sig"))
        parser = argparse.ArgumentParser(description=__doc__)
        parser.add_argument("--model-dir", type=Path, default=Path(inventory["target"]))
        model_dir = parser.parse_args().model_dir
        run(model_dir, inventory)
    except Exception:
        failure = True
        record("ERROR:", traceback.format_exc())
    finally:
        if "torch" in sys.modules:
            try:
                import torch
                if torch.cuda.is_available():
                    torch.cuda.synchronize()
                    record("Peak CUDA allocated MiB:", round(torch.cuda.max_memory_allocated(0) / 2**20, 1))
                    record("Peak CUDA reserved MiB:", round(torch.cuda.max_memory_reserved(0) / 2**20, 1))
            except Exception:
                record("Peak CUDA measurement error:", traceback.format_exc())
                failure = True
        try:
            with FREEZE_FILE.open("w", encoding="utf-8", newline="\n") as dest:
                result = subprocess.run([sys.executable, "-m", "pip", "freeze"],
                                        stdout=dest, capture_output=False, check=False)
            record("pip freeze exit code:", result.returncode)
            if result.returncode:
                failure = True
        except Exception:
            record("pip freeze error:", traceback.format_exc())
            failure = True
        record("RESULT:", "FAIL" if failure else "PASS")
        LOG_FILE.write_text("\n".join(log) + "\n", encoding="utf-8")
        record("Log:", LOG_FILE)
    return 1 if failure else 0


if __name__ == "__main__":
    sys.exit(main())
