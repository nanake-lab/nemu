"""Separate int4 CPU/CUDA diagnostic. Never use profiler timings for acceptance.

Solo carrier isolates dispatch/kernel costs; formal measurements use run_matrix.py
with the unchanged TTS/render concurrent protocol. Uses only synthetic prompts.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import importlib
import json
from pathlib import Path
import time
from typing import cast

import torch
from torch import nn

from injection import InjectedDecoder
from latency_probe import FIXED_SENTENCE
from path_probe import forward
from native_norm import replace_rms_norms
from runtime_guard import require_runner_offline

HERE = Path(__file__).resolve().parent


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--attention", choices=["eager", "sdpa"], default="eager")
    parser.add_argument("--lean", action="store_true")
    parser.add_argument("--native-rms", action="store_true")
    args = parser.parse_args()
    require_runner_offline()
    args.output.mkdir(parents=True, exist_ok=False)
    torch.manual_seed(731)
    torch.set_num_threads(4)
    torch.backends.cuda.matmul.allow_tf32 = False
    torch.backends.cudnn.allow_tf32 = False
    manifest = json.loads((HERE / "manifest.json").read_text(encoding="utf-8"))
    entry = next(m for m in manifest["models"] if m["id"] == args.model)
    local = HERE / ".cache/models" / args.model.replace("/", "--") / entry["revision"]
    config = json.loads((local / "config.json").read_text(encoding="utf-8"))
    api = importlib.import_module("transformers")
    model = cast(nn.Module, api.AutoModelForCausalLM.from_pretrained(
        str(local), local_files_only=True, trust_remote_code=False,
        dtype=torch.bfloat16, device_map={"": "cuda:0"},
        attn_implementation=args.attention,
        quantization_config=api.BitsAndBytesConfig(
            load_in_4bit=True, bnb_4bit_quant_type="nf4",
            bnb_4bit_compute_dtype=torch.bfloat16, bnb_4bit_use_double_quant=False,
        ),
    )).eval().requires_grad_(False)
    tokenizer = api.AutoTokenizer.from_pretrained(str(local), local_files_only=True)
    text = tokenizer.apply_chat_template(
        [{"role": "user", "content": "请逐字输出下面这句话，不添加其他内容：" + FIXED_SENTENCE}],
        tokenize=False, add_generation_prompt=True, enable_thinking=False,
    )
    ids = tokenizer(text, return_tensors="pt")["input_ids"].to("cuda:0")
    norm_count = replace_rms_norms(model) if args.native_rms else 0
    layers = cast(nn.ModuleList, cast(nn.Module, model.model).layers)
    layer_index = int(config["num_hidden_layers"]) // 2 - 1
    wrapper = InjectedDecoder(
        layers[layer_index], torch.eye(int(config["hidden_size"]), 8,
                                      device="cuda:0", dtype=torch.bfloat16),
        beta=1, family=config["model_type"], cache_delta=args.lean,
    )
    layers[layer_index] = wrapper

    def request() -> list[dict[str, object]]:
        wrapper.begin((0.0,) * 8)
        current = ids
        kv: object = None
        rows: list[dict[str, object]] = []
        try:
            for step in range(9):
                torch.cuda.synchronize()
                started = time.perf_counter()
                with torch.profiler.record_function("prefill" if step == 0 else "decode"):
                    out = forward(model, current, kv, last_logits=args.lean)
                    kv = out.past_key_values
                    current = out.logits[:, -1, :].argmax(-1, keepdim=True)
                    token = int(current.item())
                    torch.cuda.synchronize()
                rows.append(dict(step=step, token=token,
                                 wall_ms=(time.perf_counter() - started) * 1000))
        finally:
            wrapper.end()
        return rows

    with torch.inference_mode():
        request()  # Unreported warmup; no reuse of its KV or modulation cache.
        with torch.profiler.profile(
            activities=[torch.profiler.ProfilerActivity.CPU, torch.profiler.ProfilerActivity.CUDA],
            record_shapes=True,
        ) as measured:
            rows = request()
    measured.export_chrome_trace(str(args.output / "trace.json"))
    events = [dict(name=e.key, count=e.count, self_cpu_us=e.self_cpu_time_total,
                   cpu_total_us=e.cpu_time_total, self_device_us=e.self_device_time_total,
                   device_total_us=e.device_time_total) for e in measured.key_averages()]
    (args.output / "profile.json").write_text(json.dumps(dict(
        timestamp=datetime.now(timezone.utc).isoformat(), model=entry,
        attention=args.attention, lean=args.lean, native_rms_count=norm_count, precision="NF4 bf16-compute no-double-quant",
        layer_index=layer_index, rank=8, steps=rows, events=events,
        scope="diagnostic-only; solo carrier; fixed 1 prefill + 8 decode; not sentence acceptance",
        limitation="profiler perturbs timings; self CPU/device event sums are not wall-time attribution",
    ), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
