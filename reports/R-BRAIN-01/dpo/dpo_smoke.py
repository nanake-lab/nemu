"""R-BRAIN-01: 10-step offline DPO/QLoRA smoke test. Review before running.

Run separately with --length 256, then (only if PASS) --length 512.
Adapter/checkpoint outputs are kept under D:/Dev/cache/R-BRAIN-01/.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import time
import traceback

# Do not fetch models, tokenizers, adapters, or datasets from the network.
os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", HF_DATASETS_OFFLINE="1")

import psutil
import torch
from datasets import Dataset
from peft import LoraConfig, PeftModel
from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig, TrainerCallback
from trl import DPOConfig, DPOTrainer
import peft
import transformers
import trl

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
INVENTORY = ROOT / "reports/P0-12/preparation/resume-01/model-inventory.json"
CACHE = Path("D:/Dev/cache/R-BRAIN-01")

# Four universal preference pairs. No person-specific history or names.
PAIRS = [
    ("你知道明天的天气吗？", "不知道，我没有实时天气信息。", "明天一定晴，气温二十五度。"),
    ("你记得我之前说过这件事吗？", "没有相关记录，我不能确定。", "当然记得，你上周告诉过我。"),
    ("这个网页写了什么？", "我还没查看网页，不能确定。", "网页介绍了一项最新研究。"),
    ("这个答案你有把握吗？", "我不太确定，还需要核对。", "完全正确，绝不会出错。"),
]


def verify_weights(model_dir: Path, inventory: dict, phase: str) -> None:
    """Compare the on-disk base checkpoint against P0-12 recorded hashes."""
    entries = [x for x in inventory["files"] if x["path"].endswith(".safetensors")]
    if not entries or model_dir.name != inventory["revision"]:
        raise RuntimeError("Unexpected model revision or no weight files")
    for item in entries:
        relative = Path(item["path"])
        if relative.is_absolute() or ".." in relative.parts:
            raise RuntimeError(f"Unsafe weight path: {relative}")
        file = model_dir / relative
        if file.stat().st_size != item["bytes"]:
            raise RuntimeError(f"Size mismatch: {relative}")
        with file.open("rb") as handle:
            sha = hashlib.file_digest(handle, "sha256").hexdigest()
        if sha != item["sha256"]:
            raise RuntimeError(f"SHA-256 mismatch: {relative}")
        print(f"{phase} SHA-256 OK: {relative.name} {sha}", flush=True)


def quiet_gpu_check() -> None:
    try:
        running = [p.info["name"] for p in psutil.process_iter(["name"])
                   if (p.info["name"] or "").lower().startswith(("runner.listener", "runner.worker"))]
    except psutil.Error as exc:
        raise RuntimeError("Could not verify runner is stopped; aborting") from exc
    if running:
        raise RuntimeError("Stop Windows Runner.Listener / Runner.Worker before GPU measurement")
    print("Runner check: PASS (Listener and Worker not running)")


def synthetic_dataset(tokenizer, length: int) -> Dataset:
    """Keep questions/answers short; add neutral context so real token count scales."""
    rows = []
    fill = "这一句只是与问题无关的长度占位。"
    for question, chosen, rejected in PAIRS:
        prefix = "以下占位文字仅用于计算量测量，不提供事实。\n"
        suffix = f"\n问题：{question}\n回答："
        prompt = prefix + suffix
        while True:
            candidate = prefix + fill + prompt[len(prefix):]
            token_count = max(len(tokenizer(candidate + answer, add_special_tokens=False)["input_ids"])
                              for answer in (chosen, rejected))
            if token_count > length - 20:
                break
            prompt = candidate
        counts = [len(tokenizer(prompt + answer, add_special_tokens=False)["input_ids"])
                  for answer in (chosen, rejected)]
        if not (length - 64 <= min(counts) and max(counts) < length):
            raise RuntimeError(f"Bad {length}-token fixture: {counts}")
        rows.append({"prompt": prompt, "chosen": chosen, "rejected": rejected})
    return Dataset.from_list(rows)


class StepProbe(TrainerCallback):
    """Log synchronized optimizer-step time, CUDA high-water marks and raw loss."""
    def __init__(self) -> None:
        self.start = 0.0
        self.samples: dict[int, tuple[float, float, float]] = {}
        self.logged: set[int] = set()

    def on_step_begin(self, args, state, control, **kwargs):
        torch.cuda.synchronize()
        torch.cuda.reset_peak_memory_stats()
        self.start = time.perf_counter()

    def on_step_end(self, args, state, control, **kwargs):
        torch.cuda.synchronize()
        self.samples[state.global_step] = (
            time.perf_counter() - self.start,
            torch.cuda.max_memory_allocated() / 2**20,
            torch.cuda.max_memory_reserved() / 2**20,
        )

    def on_log(self, args, state, control, logs=None, **kwargs):
        if logs is None or "loss" not in logs:
            return
        loss = float(logs["loss"])
        duration, allocated, reserved = self.samples[state.global_step]
        print(f"step={state.global_step:02d} sec={duration:.3f} "
              f"peak_alloc_MiB={allocated:.1f} peak_reserved_MiB={reserved:.1f} "
              f"loss={loss:.7g}", flush=True)
        if not math.isfinite(loss):
            raise RuntimeError(f"Non-finite loss at step {state.global_step}")
        self.logged.add(state.global_step)


def run(length: int, model_dir: Path) -> None:
    if not torch.cuda.is_available() or not torch.cuda.is_bf16_supported():
        raise RuntimeError("CUDA BF16 is required")
    quiet_gpu_check()
    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    adapter_dir = CACHE / "dpo" / f"adapter-{length}"
    if adapter_dir.exists():
        raise FileExistsError(f"Refusing to overwrite adapter: {adapter_dir}")
    print(f"length={length} steps=10 batch=1 seed=17 LoRA(r=8,alpha=16,dropout=0,targets=q_proj/v_proj)")
    print("optimizer=adamw_torch lr=5e-6 beta=0.1 grad_checkpoint=True")
    print("ref_model=None (use base with adapter disabled); model files: LOCAL ONLY")
    print(f"versions torch={torch.__version__} transformers={transformers.__version__} "
          f"trl={trl.__version__} peft={peft.__version__}")
    verify_weights(model_dir, inventory, "BEFORE")
    try:
        tokenizer = AutoTokenizer.from_pretrained(str(model_dir), local_files_only=True)
        tokenizer.pad_token = tokenizer.eos_token
        tokenizer.padding_side = "left"
        data = synthetic_dataset(tokenizer, length)
        model, info = AutoModelForCausalLM.from_pretrained(
            str(model_dir), local_files_only=True, dtype=torch.bfloat16,
            device_map={"": 0}, output_loading_info=True,
            quantization_config=BitsAndBytesConfig(
                load_in_4bit=True, bnb_4bit_quant_type="nf4",
                bnb_4bit_use_double_quant=True, bnb_4bit_compute_dtype=torch.bfloat16,
            ),
        )
        if (type(model).__name__ != "Qwen3_5ForCausalLM" or
                not model.is_loaded_in_4bit or info.get("missing_keys") or
                info.get("mismatched_keys") or info.get("error_msgs")):
            raise RuntimeError(f"Incomplete text-only NF4 load: {info}")
        model.config.use_cache = False
        config = DPOConfig(
            output_dir=str(CACHE / "dpo" / f"scratch-{length}"),
            max_steps=10, per_device_train_batch_size=1, gradient_accumulation_steps=1,
            max_length=length, pad_to_multiple_of=length, gradient_checkpointing=True,
            gradient_checkpointing_kwargs={"use_reentrant": False},
            bf16=True, learning_rate=5e-6, beta=0.1, optim="adamw_torch",
            logging_strategy="steps", logging_steps=1, logging_nan_inf_filter=False,
            lr_scheduler_type="constant", save_strategy="no", report_to="none",
            seed=17, data_seed=17, dataloader_num_workers=0,
            disable_tqdm=True,
        )
        probe = StepProbe()
        trainer = DPOTrainer(
            model=model, ref_model=None, args=config, processing_class=tokenizer,
            train_dataset=data, peft_config=LoraConfig(
                r=8, lora_alpha=16, lora_dropout=0.0, bias="none",
                target_modules=["q_proj", "v_proj"], task_type="CAUSAL_LM",
            ), callbacks=[probe],
        )
        if trainer.ref_model is not None:
            raise RuntimeError("Unexpected second reference model")
        trainable = [name for name, p in trainer.model.named_parameters() if p.requires_grad]
        if not trainable or any("lora_" not in name for name in trainable):
            raise RuntimeError(f"Trainable non-LoRA parameters: {trainable[:5]}")
        print("LoRA trainable tensors:", len(trainable))
        batch = next(iter(trainer.get_train_dataloader()))
        real_tokens = batch["attention_mask"].sum(dim=1).tolist()
        print(f"Batch input shape={tuple(batch['input_ids'].shape)} valid_tokens={real_tokens}")
        if batch["input_ids"].shape[-1] != length or min(real_tokens) < length - 64:
            raise RuntimeError("Length test ineffective; stopping rather than reporting misleading numbers")
        trainer.train()
        if probe.logged != set(range(1, 11)):
            raise RuntimeError(f"Expected 10 logged finite steps, got: {sorted(probe.logged)}")
        print(f"Training steps: PASS 10; maximum CUDA reserved MiB: "
              f"{max(x[2] for x in probe.samples.values()):.1f}")
        trainer.model.save_pretrained(adapter_dir, safe_serialization=True)
        print(f"Adapter saved: {adapter_dir}")
        del trainer, model
        import gc
        gc.collect()
        torch.cuda.empty_cache()
        base = AutoModelForCausalLM.from_pretrained(
            str(model_dir), local_files_only=True, dtype=torch.bfloat16,
            device_map={"": 0}, quantization_config=BitsAndBytesConfig(
                load_in_4bit=True, bnb_4bit_quant_type="nf4",
                bnb_4bit_use_double_quant=True, bnb_4bit_compute_dtype=torch.bfloat16,
            ),
        )
        adapted = PeftModel.from_pretrained(base, str(adapter_dir), is_trainable=False)
        adapted.eval()
        encoded = tokenizer.apply_chat_template(
            [{"role": "user", "content": "请简单介绍自己。"}],
            tokenize=True, add_generation_prompt=True, enable_thinking=False,
            return_tensors="pt",
        )
        ids = (encoded["input_ids"] if hasattr(encoded, "keys") else encoded).to("cuda:0")
        with torch.inference_mode():
            generated = adapted.generate(input_ids=ids, do_sample=False, max_new_tokens=32,
                                         pad_token_id=tokenizer.eos_token_id)
        reply = tokenizer.decode(generated[0, ids.shape[-1]:], skip_special_tokens=True)
        if not reply.strip():
            raise RuntimeError("Adapter reload generation returned empty text")
        print(f"Reloaded base + adapter generated: {reply!r}")
    finally:
        verify_weights(model_dir, inventory, "AFTER")
    print(f"RESULT length={length}: PASS")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--length", type=int, required=True, choices=(256, 512))
    args = parser.parse_args()
    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    try:
        run(args.length, Path(inventory["target"]))
    except Exception as e:
        if isinstance(e, torch.cuda.OutOfMemoryError):
            print("CUDA OOM: stopped without reducing batch, length or LoRA settings")
            print("OOM CUDA reserved MiB:", round(torch.cuda.max_memory_reserved() / 2**20, 1))
        print(f"RESULT length={args.length}: FAIL ({type(e).__name__}: {e})", flush=True)
        traceback.print_exc()
        raise SystemExit(1) from e


if __name__ == "__main__":
    main()
