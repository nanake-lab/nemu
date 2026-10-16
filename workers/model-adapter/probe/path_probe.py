"""Measure explicit injection on a local pinned Qwen3, or a tiny CPU fixture.

No pretrained fallback, offload, network loading or production storage access.
Raw observations are written even when the measured path differs from baseline.
Numerical errors are measurements; production tolerance is not assigned here.
"""

from __future__ import annotations

import argparse
from dataclasses import asdict
from datetime import datetime, timezone
import importlib
import json
from pathlib import Path
from typing import Protocol, cast

import torch
from torch import nn

from injection import CacheKey, InjectedDecoder, RequestCache, modulation_digest
from runtime_guard import require_runner_offline

HERE = Path(__file__).resolve().parent


class Output(Protocol):
    logits: torch.Tensor
    past_key_values: object


def forward(model: nn.Module, ids: torch.Tensor, kv: object = None) -> Output:
    return cast(Output, model(input_ids=ids, past_key_values=kv, use_cache=True))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tiny-cpu", action="store_true")
    parser.add_argument("--model", default="Qwen/Qwen3-4B")
    parser.add_argument("--precision", choices=["bf16", "int8", "int4"], default="bf16")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("output must be new; preserve existing measurements")
    torch.manual_seed(731)
    torch.set_num_threads(4)
    torch.backends.cuda.matmul.allow_tf32 = False
    torch.backends.cudnn.allow_tf32 = False
    api = importlib.import_module("transformers")
    if args.tiny_cpu:
        config = api.Qwen3Config(
            vocab_size=64,
            hidden_size=32,
            intermediate_size=64,
            num_hidden_layers=2,
            num_attention_heads=4,
            num_key_value_heads=2,
            head_dim=8,
        )
        model = cast(nn.Module, api.Qwen3ForCausalLM(config)).eval()
        model_id, revision = "random-cpu-fixture", "seed-731"
        width, layer_index, device = 32, 0, "cpu"
    else:
        require_runner_offline()
        manifest = json.loads((HERE / "manifest.json").read_text(encoding="utf-8"))
        entry = next(m for m in manifest["models"] if m["id"] == args.model)
        model_id, revision = entry["id"], entry["revision"]
        local = HERE / ".cache/models" / model_id.replace("/", "--") / revision
        config_data = json.loads((local / "config.json").read_text(encoding="utf-8"))
        if config_data["model_type"] != "qwen3":
            raise ValueError("only pinned Qwen3 decoder candidates are supported")
        width = int(config_data["hidden_size"])
        layer_index = int(config_data["num_hidden_layers"]) // 2 - 1
        device = "cuda:0"
        quantization = None
        if args.precision != "bf16":
            quantization = api.BitsAndBytesConfig(
                load_in_8bit=args.precision == "int8",
                load_in_4bit=args.precision == "int4",
                bnb_4bit_quant_type="nf4",
                bnb_4bit_compute_dtype=torch.bfloat16,
                bnb_4bit_use_double_quant=False,
            )
        model = cast(
            nn.Module,
            api.AutoModelForCausalLM.from_pretrained(
                str(local),
                local_files_only=True,
                trust_remote_code=False,
                torch_dtype=torch.bfloat16,
                device_map={"": device},
                quantization_config=quantization,
                attn_implementation="eager",
            ),
        ).eval()
    model.requires_grad_(False)
    layers = cast(nn.ModuleList, cast(nn.Module, model.model).layers)
    dtype = next(model.parameters()).dtype
    rank = 8
    u = torch.eye(width, rank, device=device, dtype=dtype)
    wrapper = InjectedDecoder(layers[layer_index], u, beta=1.0)
    ids = torch.tensor([[1, 3, 5, 7, 9]], dtype=torch.long, device=device)
    base_key = CacheKey(
        model_id + "@" + revision,
        "token-ids-fixture-v1",
        "1,3,5,7,9",
        f"qwen3-attention-residual-layer-{layer_index}-rank8-basis-v1",
        modulation_digest((0.0,) * rank),
        "last-prefill-each-decode-v1",
        "float32" if args.tiny_cpu else args.precision,
    )
    with torch.inference_mode():
        baseline = forward(model, ids)
        baseline_logits = baseline.logits.detach().clone()
        token = baseline_logits[:, -1, :].argmax(-1, keepdim=True)
        baseline_decode = forward(model, token, baseline.past_key_values).logits.detach().clone()
        del baseline
        layers[layer_index] = wrapper
        wrapper.begin((0.0,) * rank, observe=True)
        zero = forward(model, ids)
        zero_prefill_error = float((zero.logits - baseline_logits).abs().max().item())
        zero_decode = forward(model, token, zero.past_key_values)
        zero_decode_error = float((zero_decode.logits - baseline_decode).abs().max().item())
        cache = RequestCache()
        cache.begin(base_key)
        cache.put(base_key, zero.past_key_values)
        wrapper.end()
        amplitude = (0.5,) + (0.0,) * (rank - 1)
        changed_key = CacheKey(**{**asdict(base_key), "modulation": modulation_digest(amplitude)})
        changed_kv = cache.get(changed_key)
        if changed_kv is not None:
            raise AssertionError("old modulation cache survived")
        wrapper.begin(amplitude, observe=True)
        changed = forward(model, ids, changed_kv)
        cache.put(changed_key, changed.past_key_values)
        changed_logits = changed.logits.detach().clone()
        forward(model, token, cache.get(changed_key))
        observations = wrapper.observations.copy()
        wrapper.end()
        wrapper.begin(amplitude)
        fresh = forward(model, ids)
        fresh_error = float((fresh.logits - changed_logits).abs().max().item())
        wrapper.end()
    result = dict(
        status="measured-not-accepted",
        tiny_cpu_fixture=args.tiny_cpu,
        timestamp=datetime.now(timezone.utc).isoformat(),
        key=asdict(changed_key),
        torch=torch.__version__,
        dtype=str(dtype),
        layer_index_zero_based=layer_index,
        rank=rank,
        u="first 8 columns of identity",
        u_operator_norm=1.0,
        beta=1.0,
        zero_prefill_logits_max_abs_error=zero_prefill_error,
        zero_decode_logits_max_abs_error=zero_decode_error,
        changed_cache_discarded=changed_kv is None,
        changed_vs_fresh_logits_max_abs_error=fresh_error,
        injection_observations=observations,
        limitation="No real text, concurrent TTS/render, latency or direction acceptance.",
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
    print(args.output)


if __name__ == "__main__":
    main()
