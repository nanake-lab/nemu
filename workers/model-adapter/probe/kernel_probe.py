"""Offline cross-kernel evidence on frozen multilingual prompts, not a tolerance approval."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import importlib
import json
from pathlib import Path
from typing import cast

import numpy as np
import torch
from torch import nn

from native_norm import replace_rms_norms
from path_probe import forward
from runtime_guard import require_runner_offline

HERE = Path(__file__).resolve().parent
PROMPTS = {
    "zh": "桌上有一本书和一杯水。请用中文简短描述这个场景。",
    "en": "There is a book and a glass of water on the table. Briefly describe the scene in English.",
    "ja": "机の上に本と水の入ったコップがあります。日本語で短く描写してください。",
}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    require_runner_offline()
    args.output.mkdir(parents=True, exist_ok=False)
    manifest = json.loads((HERE / "manifest.json").read_text(encoding="utf-8"))
    entry = next(m for m in manifest["models"] if m["id"] == args.model)
    local = HERE / ".cache/models" / args.model.replace("/", "--") / entry["revision"]
    torch.manual_seed(731)
    torch.set_num_threads(4)
    torch.backends.cuda.matmul.allow_tf32 = False
    api = importlib.import_module("transformers")
    model = cast(nn.Module, api.AutoModelForCausalLM.from_pretrained(
        str(local), local_files_only=True, trust_remote_code=False, dtype=torch.bfloat16,
        device_map={"": "cuda:0"}, attn_implementation="eager",
        quantization_config=api.BitsAndBytesConfig(
            load_in_4bit=True, bnb_4bit_quant_type="nf4",
            bnb_4bit_compute_dtype=torch.bfloat16, bnb_4bit_use_double_quant=False,
        ),
    )).eval().requires_grad_(False)
    tokenizer = api.AutoTokenizer.from_pretrained(str(local), local_files_only=True)
    # Keep original modules so each case starts with the same reference kernel.
    original_norms = [(parent, name, child) for parent in model.modules()
                      for name, child in parent.named_children()
                      if type(child).__name__ in {"Qwen3RMSNorm", "Phi3RMSNorm"}]
    result: dict[str, object] = dict(
        model=entry, timestamp=datetime.now(timezone.utc).isoformat(),
        status="started", precision="NF4/bf16 eager", torch=torch.__version__,
        protocol="one full prefill + eight teacher-forced decode steps; independent KV per kernel",
        limitation="Three synthetic prompts; empirical evidence, no production tolerance frozen",
        cases=[],
    )
    cases: list[dict[str, object]] = []
    result["cases"] = cases
    try:
        with torch.inference_mode():
            for language, prompt in PROMPTS.items():
                require_runner_offline()
                for parent, name, child in original_norms:
                    setattr(parent, name, child)
                text = tokenizer.apply_chat_template(
                    [{"role": "user", "content": prompt}], tokenize=False,
                    add_generation_prompt=True, enable_thinking=False,
                )
                ids = tokenizer(text, return_tensors="pt")["input_ids"].to("cuda:0")
                baseline: list[torch.Tensor] = []
                teacher: list[int] = []
                current, kv = ids, None
                for _ in range(9):
                    out = forward(model, current, kv, last_logits=True)
                    kv = out.past_key_values
                    logits = out.logits[0, -1].float().cpu()
                    baseline.append(logits)
                    token = int(logits.argmax().item())
                    teacher.append(token)
                    current = torch.tensor([[token]], device="cuda:0")
                del out, kv
                count = replace_rms_norms(model)
                current, kv = ids, None
                native: list[torch.Tensor] = []
                metrics: list[dict[str, object]] = []
                for step in range(9):
                    out = forward(model, current, kv, last_logits=True)
                    kv = out.past_key_values
                    logits = out.logits[0, -1].float().cpu()
                    native.append(logits)
                    difference = (logits - baseline[step]).abs()
                    metrics.append(dict(
                        step=step, phase="prefill" if step == 0 else "decode",
                        max_abs=float(difference.max()), mean_abs=float(difference.mean()),
                        p99_abs=float(torch.quantile(difference, 0.99)),
                        reference_argmax=teacher[step], native_argmax=int(logits.argmax()),
                        argmax_equal=teacher[step] == int(logits.argmax()),
                    ))
                    current = torch.tensor([[teacher[step]]], device="cuda:0")
                del out, kv
                np.savez_compressed(args.output / f"{language}-logits.npz",
                                    reference=torch.stack(baseline).numpy(),
                                    native=torch.stack(native).numpy())
                cases.append(dict(language=language, prompt=prompt, input_ids=ids.cpu().tolist(),
                                  teacher_tokens=teacher, native_rms_count=count, metrics=metrics))
                maximum = max(cast(float, m["max_abs"]) for m in metrics)
                print(f"{args.model} {language}: max={maximum}",
                      flush=True)
        result["status"] = "measured-not-accepted"
    except Exception as error:
        result.update(status="failed", error=str(error))
        raise
    finally:
        (args.output / "result.json").write_text(
            json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8",
        )


if __name__ == "__main__":
    main()
