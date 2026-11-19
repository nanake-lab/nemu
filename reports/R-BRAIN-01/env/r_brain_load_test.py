"""Verify P0-12 hashes, then load local Qwen3.5 text-only NF4 and generate offline."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

os.environ["HF_HUB_OFFLINE"] = "1"
os.environ["TRANSFORMERS_OFFLINE"] = "1"

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
inventory = json.loads(
    (ROOT / "reports/P0-12/preparation/resume-01/model-inventory.json").read_text(encoding="utf-8")
)
model_dir = Path(sys.argv[1] if len(sys.argv) > 1 else inventory["target"])
if model_dir.name != inventory["revision"]:
    raise RuntimeError("Model revision does not match P0-12 inventory")

# Fail before loading if any inventoried file differs from the verified download.
for item in inventory["files"]:
    relative = Path(item["path"])
    if relative.is_absolute() or ".." in relative.parts:
        raise RuntimeError(f"Unsafe inventory path: {relative}")
    file = model_dir / relative
    if file.stat().st_size != item["bytes"]:
        raise RuntimeError(f"Size mismatch: {relative}")
    with file.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    if digest != item["sha256"]:
        raise RuntimeError(f"SHA-256 mismatch: {relative}")
print(f"Inventory SHA-256 + size: PASS ({len(inventory['files'])} files)")

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig

if not torch.cuda.is_available():
    raise RuntimeError("CUDA unavailable")
print("PyTorch:", torch.__version__, "CUDA:", torch.version.cuda)
print("GPU:", torch.cuda.get_device_name(0))

tokenizer = AutoTokenizer.from_pretrained(str(model_dir), local_files_only=True)
torch.cuda.reset_peak_memory_stats()
model, info = AutoModelForCausalLM.from_pretrained(
    str(model_dir),
    local_files_only=True,
    dtype=torch.bfloat16,
    device_map={"": 0},
    quantization_config=BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_quant_type="nf4",
        bnb_4bit_use_double_quant=True,
        bnb_4bit_compute_dtype=torch.bfloat16,
    ),
    output_loading_info=True,
)
print("Model:", type(model).__name__, "4-bit:", model.is_loaded_in_4bit)
missing = list(info.get("missing_keys") or [])
mismatched = list(info.get("mismatched_keys") or [])
errors = list(info.get("error_msgs") or [])
extra = list(info.get("unexpected_keys") or [])
print("missing_keys:", missing)
print("mismatched_keys:", mismatched)
print("loader_errors:", errors)
print("unexpected_keys (visual weights can be skipped):", len(extra), extra[:5])
if missing or mismatched or errors:
    raise RuntimeError("Incomplete language model load")
if type(model).__name__ != "Qwen3_5ForCausalLM" or not model.is_loaded_in_4bit:
    raise RuntimeError("Wrong model type or quantization")

inputs = tokenizer.apply_chat_template(
    [{"role": "user", "content": "请用一句中文介绍你自己。"}],
    tokenize=True, add_generation_prompt=True, enable_thinking=False, return_tensors="pt",
)
input_ids = (inputs["input_ids"] if hasattr(inputs, "keys") else inputs).to("cuda:0")
with torch.inference_mode():
    generated = model.generate(
        input_ids=input_ids, max_new_tokens=64, do_sample=False,
        pad_token_id=tokenizer.eos_token_id,
    )
answer = tokenizer.decode(generated[0, input_ids.shape[-1]:], skip_special_tokens=True)
print("Generated:", repr(answer))
if not answer.strip():
    raise RuntimeError("Empty generation")
print("Peak CUDA allocated MiB:", round(torch.cuda.max_memory_allocated() / 2**20, 1))
print("Peak CUDA reserved MiB:", round(torch.cuda.max_memory_reserved() / 2**20, 1))
with (HERE / "pip-freeze.txt").open("w", encoding="utf-8") as output:
    subprocess.run([sys.executable, "-m", "pip", "freeze"], stdout=output, check=True)
print("RESULT: PASS; pip-freeze.txt saved")
