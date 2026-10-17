"""Offline, bounded generation with real resident TTS and an external render load.

Render readiness is supplied by render_load.mjs; the caller owns its lifetime.
All outputs are synthetic fixtures, never user conversations. No acceptance is
inferred from TTFT: first-sentence latency includes generation through punctuation.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import importlib
import json
from pathlib import Path
import statistics
import subprocess
import time
from typing import cast

import torch
from torch import nn

from injection import InjectedDecoder
from path_probe import forward
from runtime_guard import require_runner_offline
from stop_tokens import eos_ids

HERE = Path(__file__).resolve().parent
PROMPT = "请用一句恰好二十个汉字的话鼓励刚完成学习的人，句末加句号，不要解释。"
FIXED_SENTENCE = "今天的努力值得肯定请带着信心继续向前走吧。"


def gpu_memory() -> str:
    return subprocess.check_output(
        ["nvidia-smi", "--query-gpu=timestamp,memory.used,utilization.gpu",
         "--format=csv,noheader,nounits"], text=True,
    ).strip()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", required=True)
    parser.add_argument("--precision", choices=["bf16", "int8", "int4"], required=True)
    parser.add_argument("--render-ready", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repeats", type=int, default=20)
    parser.add_argument("--fixed-twenty", action="store_true",
                        help="controlled generation fixture; not a free-form fluency assessment")
    args = parser.parse_args()
    if args.output.exists() or args.repeats < 1:
        parser.error("new output and positive repeats required")
    require_runner_offline()
    prompt = ("请逐字输出下面这句话，不添加其他内容：" + FIXED_SENTENCE
              if args.fixed_twenty else PROMPT)
    render = json.loads(args.render_ready.read_text(encoding="utf-8"))
    manifest = json.loads((HERE / "manifest.json").read_text(encoding="utf-8"))

    def local(repo: str) -> Path:
        entry = next(m for m in manifest["models"] if m["id"] == repo)
        return HERE / ".cache/models" / repo.replace("/", "--") / entry["revision"]

    result: dict[str, object] = dict(
        timestamp=datetime.now(timezone.utc).isoformat(), model=args.model,
        precision=args.precision, render=render, prompt=prompt, samples=[],
        fixture="fixed-twenty-v1" if args.fixed_twenty else "free-form-v1",
        status="started", baseline_gpu=gpu_memory(),
        method="greedy; one warmup excluded; fresh KV; eager; max 96 new tokens; 30s cap",
        vram_scope="whole GPU samples, includes unrelated desktop allocations; not Nemu attribution",
        tts_scope="resident Base model only; voice/variant remains P4-03",
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    try:
        torch.set_num_threads(4)
        torch.manual_seed(731)
        torch.backends.cuda.matmul.allow_tf32 = False
        torch.backends.cudnn.allow_tf32 = False
        api = importlib.import_module("transformers")
        tts_api = importlib.import_module("qwen_tts")
        tts = tts_api.Qwen3TTSModel.from_pretrained(
            str(local("Qwen/Qwen3-TTS-12Hz-0.6B-Base")), device_map="cuda:0",
            dtype=torch.bfloat16, attn_implementation="eager", local_files_only=True,
        )
        tts_model = cast(nn.Module, tts.model).eval().requires_grad_(False)
        result["tts_parameter_devices"] = sorted({str(p.device) for p in tts_model.parameters()})
        if any(p.device.type != "cuda" for p in tts_model.parameters()):
            raise RuntimeError("TTS parameters not entirely GPU resident")
        result["after_tts_gpu"] = gpu_memory()
        quant = None if args.precision == "bf16" else api.BitsAndBytesConfig(
            load_in_8bit=args.precision == "int8", load_in_4bit=args.precision == "int4",
            bnb_4bit_quant_type="nf4", bnb_4bit_compute_dtype=torch.bfloat16,
            bnb_4bit_use_double_quant=False,
        )
        model = cast(nn.Module, api.AutoModelForCausalLM.from_pretrained(
            str(local(args.model)), local_files_only=True, trust_remote_code=False,
            dtype=torch.bfloat16, device_map={"": "cuda:0"}, quantization_config=quant,
            attn_implementation="eager",
        )).eval().requires_grad_(False)
        tokenizer = api.AutoTokenizer.from_pretrained(str(local(args.model)), local_files_only=True)
        generation = json.loads(
            (local(args.model) / "generation_config.json").read_text(encoding="utf-8")
        )
        stopping = eos_ids(generation["eos_token_id"])
        result["eos_token_ids"] = sorted(stopping)
        config = json.loads((local(args.model) / "config.json").read_text(encoding="utf-8"))
        layers = cast(nn.ModuleList, cast(nn.Module, model.model).layers)
        index = int(config["num_hidden_layers"]) // 2 - 1
        wrapper = InjectedDecoder(
            layers[index], torch.eye(int(config["hidden_size"]), 8, device="cuda:0",
                                     dtype=torch.bfloat16), beta=1, family=config["model_type"],
        )
        layers[index] = wrapper
        text = tokenizer.apply_chat_template(
            [{"role": "user", "content": prompt}], tokenize=False,
            add_generation_prompt=True, enable_thinking=False,
        )
        ids = tokenizer(text, return_tensors="pt")["input_ids"].to("cuda:0")
        result["input_token_ids"] = ids.tolist()
        samples: list[dict[str, object]] = []
        result["samples"] = samples
        with torch.inference_mode():
            for repeat in range(args.repeats + 1):
                require_runner_offline()
                wrapper.begin((0.0,) * 8)
                torch.cuda.reset_peak_memory_stats()
                torch.cuda.synchronize()
                started = time.perf_counter()
                kv: object = None
                current = ids
                tokens: list[int] = []
                ttft = 0.0
                elapsed = 0.0
                output_text = ""
                stop_reason = "token_limit"
                for _ in range(96):
                    out = forward(model, current, kv)
                    kv = out.past_key_values
                    current = out.logits[:, -1, :].argmax(-1, keepdim=True)
                    token = int(current.item())
                    tokens.append(token)
                    torch.cuda.synchronize()
                    elapsed = time.perf_counter() - started
                    if len(tokens) == 1:
                        ttft = elapsed
                    output_text = tokenizer.decode(tokens, skip_special_tokens=True)
                    if any(c in output_text for c in "。！？!?\n"):
                        stop_reason = "sentence_boundary"
                        break
                    if token in stopping:
                        stop_reason = "eos"
                        break
                    if elapsed > 30:
                        stop_reason = "time_limit"
                        break
                wrapper.end()
                han_count = sum("\u4e00" <= c <= "\u9fff" for c in output_text)
                sample = dict(
                    repeat=repeat, warmup=repeat == 0, output=output_text, token_ids=tokens,
                    han_count=han_count, exactly_twenty_han=han_count == 20,
                    matches_fixed_fixture=output_text == FIXED_SENTENCE,
                    stop_reason=stop_reason, latency_ms=elapsed * 1000, ttft_ms=ttft * 1000,
                    tokens_per_second=len(tokens) / elapsed,
                    cuda_peak_allocated_bytes=torch.cuda.max_memory_allocated(),
                    cuda_peak_reserved_bytes=torch.cuda.max_memory_reserved(), gpu=gpu_memory(),
                )
                samples.append(sample)
                del out, kv
        result["p50_latency_ms_all_nonwarmup"] = statistics.median(
            cast(float, s["latency_ms"]) for s in samples[1:]
        )
        result["status"] = "measured-not-accepted"
    except Exception as error:
        result["status"] = "failed"
        result["error_type"] = type(error).__name__
        result["error"] = str(error)
        raise
    finally:
        with args.output.open("x", encoding="utf-8") as stream:
            json.dump(result, stream, ensure_ascii=False, indent=2)
            stream.write("\n")


if __name__ == "__main__":
    main()
