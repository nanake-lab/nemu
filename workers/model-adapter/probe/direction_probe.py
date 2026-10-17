"""Informational activation-direction study, not M7 or a psychological certificate.

Extract one mean excited-minus-calm residual direction from three synthetic pairs.
Freeze it before generation. Ten matched seeds times two conditions give twenty
generations per language. No prompt style instruction is changed between conditions.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
from functools import partial
import hashlib
import importlib
import json
from pathlib import Path
import time
from typing import cast

import torch
from torch import nn

from injection import InjectedDecoder
from path_probe import forward
from runtime_guard import require_runner_offline
from stop_tokens import eos_ids

HERE = Path(__file__).resolve().parent
PAIRS = [
    ("请平静地说：我们完成了今天的任务。", "请兴奋地说：我们完成了今天的任务。"),
    ("Say calmly: We finished today's task.", "Say excitedly: We finished today's task."),
    ("落ち着いて言ってください：今日の作業が終わりました。",
     "興奮して言ってください：今日の作業が終わりました。"),
]
PROMPTS = {
    "zh": "朋友告诉你他第一次成功烤好了面包。用中文简短回应。",
    "en": "A friend tells you they successfully baked bread for the first time. Reply briefly in English.",
    "ja": "友達が初めてパンを上手に焼けたと教えてくれました。日本語で短く返事をしてください。",
}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--attention", choices=["eager", "sdpa"], default="eager")
    parser.add_argument("--lean", action="store_true")
    args = parser.parse_args()
    if args.output.exists():
        parser.error("preserve previous measurements; choose a new output")
    require_runner_offline()
    torch.manual_seed(731)
    torch.set_num_threads(4)
    torch.backends.cuda.matmul.allow_tf32 = False
    manifest = json.loads((HERE / "manifest.json").read_text(encoding="utf-8"))
    entry = next(m for m in manifest["models"] if m["id"] == args.model)
    local = HERE / ".cache/models" / args.model.replace("/", "--") / entry["revision"]
    config = json.loads((local / "config.json").read_text(encoding="utf-8"))
    result: dict[str, object] = dict(
        status="started", timestamp=datetime.now(timezone.utc).isoformat(), model=entry,
        calibration_pairs=PAIRS, generation_prompts=PROMPTS,
        precision=f"int4 NF4; bf16 compute; no double quantization; {args.attention}",
        attention=args.attention, lean=args.lean,
        sampling="temperature 0.7; full vocabulary multinomial; matched seeds 731..740",
        budget="48 tokens and 25 seconds per request; fresh KV; immutable coefficient",
        limitation="exploratory single scenario per language; not M7, no psychological coordinate claim",
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    try:
        api = importlib.import_module("transformers")
        model = cast(nn.Module, api.AutoModelForCausalLM.from_pretrained(
            str(local), local_files_only=True, trust_remote_code=False,
            dtype=torch.bfloat16, device_map={"": "cuda:0"}, attn_implementation=args.attention,
            quantization_config=api.BitsAndBytesConfig(
                load_in_4bit=True, bnb_4bit_quant_type="nf4",
                bnb_4bit_compute_dtype=torch.bfloat16, bnb_4bit_use_double_quant=False,
            ),
        )).eval().requires_grad_(False)
        tokenizer = api.AutoTokenizer.from_pretrained(str(local), local_files_only=True)
        generation = json.loads((local / "generation_config.json").read_text(encoding="utf-8"))
        stopping = eos_ids(generation["eos_token_id"])
        result["eos_token_ids"] = sorted(stopping)
        layers = cast(nn.ModuleList, cast(nn.Module, model.model).layers)
        index = int(config["num_hidden_layers"]) // 2 - 1
        width = int(config["hidden_size"])
        original = layers[index]
        recorder = InjectedDecoder(original, torch.zeros(width, 8, device="cuda:0"),
                                   beta=1, family=config["model_type"], cache_delta=args.lean)
        run_forward = partial(forward, last_logits=args.lean)
        layers[index] = recorder

        def encode(prompt: str) -> torch.Tensor:
            text = tokenizer.apply_chat_template(
                [{"role": "user", "content": prompt}], tokenize=False,
                add_generation_prompt=True, enable_thinking=False,
            )
            return cast(torch.Tensor, tokenizer(text, return_tensors="pt")["input_ids"]).to("cuda:0")

        with torch.inference_mode():
            differences = []
            calibration: list[dict[str, object]] = []
            for calm, excited in PAIRS:
                vectors = []
                for prompt in (calm, excited):
                    recorder.begin((0.0,) * 8, observe=True)
                    out = run_forward(model, encode(prompt))
                    vector = torch.tensor(cast(list[list[float]], recorder.observations[0]["before"]),
                                          dtype=torch.float64).flatten()
                    vectors.append(vector)
                    calibration.append(dict(prompt=prompt, residual=vector.tolist()))
                    recorder.end()
                    del out
                differences.append(vectors[1] - vectors[0])
            direction = torch.stack(differences).mean(dim=0)
            norm = float(torch.linalg.vector_norm(direction).item())
            if not norm > 0 or not torch.isfinite(direction).all():
                raise ValueError("invalid extracted direction")
            u = torch.zeros(width, 8, dtype=torch.float32)
            u[:, 0] = (direction / norm * 0.999).float()
            wrapper = InjectedDecoder(original, u.to("cuda:0"), beta=1,
                                      family=config["model_type"], cache_delta=args.lean)
            layers[index] = wrapper
            result.update(
                calibration=calibration, unnormalized_direction_l2=norm,
                u=u.tolist(), u_sha256=hashlib.sha256(u.numpy().tobytes()).hexdigest(),
                u_operator_norm=float(torch.linalg.matrix_norm(u.double(), ord=2).item()),
                coefficient_slots=8, active_directions=1, beta=1, layer_index_zero_based=index,
            )
            samples: list[dict[str, object]] = []
            result["samples"] = samples
            for language, prompt in PROMPTS.items():
                ids = encode(prompt)
                for seed in range(731, 741):
                    for coefficient in (0.0, 1.0):
                        require_runner_offline()
                        torch.manual_seed(seed)
                        wrapper.begin((coefficient,) + (0.0,) * 7)
                        current = ids
                        kv: object = None
                        tokens: list[int] = []
                        started = time.perf_counter()
                        stop = "token_limit"
                        for _ in range(48):
                            out = run_forward(model, current, kv)
                            kv = out.past_key_values
                            probabilities = torch.softmax(out.logits[:, -1, :].float() / 0.7, dim=-1)
                            current = torch.multinomial(probabilities, 1)
                            token = int(current.item())
                            tokens.append(token)
                            if token in stopping:
                                stop = "eos"
                                break
                            if time.perf_counter() - started >= 25:
                                stop = "time_limit"
                                break
                        wrapper.end()
                        samples.append(dict(language=language, seed=seed, coefficient=coefficient,
                                            token_ids=tokens, output=tokenizer.decode(tokens, skip_special_tokens=True),
                                            stop=stop, elapsed_s=time.perf_counter() - started))
                        del out, kv
                        print(f"{args.model} {language} {seed} coefficient={coefficient}", flush=True)
        result["status"] = "measured-informational-only"
    except Exception as error:
        result.update(status="failed", error_type=type(error).__name__, error=str(error))
        raise
    finally:
        with args.output.open("x", encoding="utf-8") as stream:
            json.dump(result, stream, ensure_ascii=False, indent=2)
            stream.write("\n")


if __name__ == "__main__":
    main()
