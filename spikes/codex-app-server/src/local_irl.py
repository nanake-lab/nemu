"""P0-08 synthetic m=0 constrained IRL, isolated from all authoritative stores."""
from __future__ import annotations

import argparse
from collections.abc import Callable, Iterable
from datetime import datetime, timezone
import hashlib
import importlib
import json
import math
from pathlib import Path
import subprocess
import sys
import time
from typing import Protocol, cast

ROOT = Path(__file__).resolve().parents[3]
SPIKE = ROOT / "spikes/codex-app-server"
MODEL = "Qwen/Qwen3-4B-Instruct-2507"
REVISION = "cdbee75f17c01a7cc42f958dc650907174af0554"


def object_dict(value: object) -> dict[str, object]:
    if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
        raise TypeError("Expected JSON object")
    return cast(dict[str, object], value)


def read_json(path: Path) -> dict[str, object]:
    return object_dict(json.loads(path.read_text(encoding="utf-8-sig")))


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(8 * 1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def preflight() -> list[dict[str, object]]:
    command = (
        "Get-CimInstance Win32_Process | "
        "Select-Object Name,ProcessId,CommandLine | ConvertTo-Json -Compress"
    )
    result = subprocess.run(
        ["powershell.exe", "-NoProfile", "-Command", command],
        capture_output=True, text=True, check=True,
    )
    processes = json.loads(result.stdout)
    if not isinstance(processes, list):
        raise RuntimeError("Cannot establish process inventory")
    blocked: list[dict[str, object]] = []
    for raw in processes:
        process = object_dict(raw)
        name = str(process.get("Name", "")).casefold()
        command_line = str(process.get("CommandLine", "")).casefold()
        conflict = name in {"runner.listener.exe", "runner.worker.exe", "blender.exe"}
        if name in {"python.exe", "pythonw.exe", "uv.exe", "node.exe"}:
            conflict = conflict or any(marker in command_line for marker in (
                "p4-01", "p4-02", "asr_probe", "tts_probe", "benchmark",
                "latency_probe", "instruct_finish", "run_matrix", "qwen_tts",
            ))
        if conflict:
            blocked.append({"name": name, "pid": process["ProcessId"]})
    if blocked:
        raise RuntimeError(f"Quiet window conflict: {blocked}")
    return blocked


class SchemaValidator(Protocol):
    def iter_errors(self, value: object) -> Iterable[object]: ...


def source_valid(value: object, sample: dict[str, object]) -> bool:
    candidate = object_dict(value)
    if candidate.get("sourceRef") != sample["sourceRef"]:
        return False
    nodes = candidate.get("nodes")
    if not isinstance(nodes, list):
        return False
    ids: set[str] = set()
    for raw in nodes:
        node = object_dict(raw)
        node_id = str(node["id"])
        if node_id in ids:
            return False
        ids.add(node_id)
        span = node["sourceSpan"]
        refs = node["sourceRefs"]
        if not isinstance(span, list) or not isinstance(refs, list):
            return False
        if sample["sourceRef"] not in refs:
            return False
        if not (0 <= span[0] <= span[1] <= len(str(sample["text"]))):
            return False
    return True


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--runs", type=int, default=200)
    parser.add_argument("--quiet-window-confirmed", action="store_true")
    args = parser.parse_args()
    if not args.quiet_window_confirmed:
        parser.error("User-arranged GPU quiet window must be confirmed")
    if args.output.exists():
        parser.error("Output directory must be new; preserve all earlier runs")
    preflight()
    args.output.mkdir(parents=True)
    lock = object_dict(read_json(ROOT / "config/models.lock.json")["p0_07_probe"])
    selected = object_dict(lock["selected"])
    if selected["model_id"] != MODEL or selected["revision"] != REVISION:
        raise ValueError("Current selected carrier differs from preregistered carrier")
    model_root: Path = args.model_root.resolve()
    inventory = read_json(ROOT / "reports/P0-07/preparation/"
                          "Qwen--Qwen3-4B-Instruct-2507-weights-20261016T151833171127Z.json")
    weights = inventory["files"]
    if not isinstance(weights, list):
        raise TypeError("Missing pinned weight inventory")
    for raw in weights:
        entry = object_dict(raw)
        path = model_root / str(entry["path"])
        if path.stat().st_size != entry["bytes"] or sha256(path) != entry["sha256"]:
            raise ValueError(f"Weight mismatch: {path.name}")
    import torch
    from torch import nn

    if torch.__version__ != "2.10.0+cu128":
        raise RuntimeError("Carrier Torch version differs from selected configuration")
    api = importlib.import_module("transformers")
    quant = importlib.import_module("bitsandbytes")
    if api.__version__ != "4.57.3" or quant.__version__ != "0.50.2":
        raise RuntimeError("Carrier Transformers or bitsandbytes version differs")
    if model_root.name != REVISION:
        raise ValueError("Model cache path must end in pinned revision")
    torch.set_num_threads(4)
    torch.manual_seed(731)
    torch.backends.cuda.matmul.allow_tf32 = False
    torch.backends.cudnn.allow_tf32 = False
    model = cast(nn.Module, api.AutoModelForCausalLM.from_pretrained(
        str(model_root), local_files_only=True, trust_remote_code=False,
        dtype=torch.bfloat16, device_map={"": "cuda:0"}, attn_implementation="eager",
        quantization_config=api.BitsAndBytesConfig(
            load_in_4bit=True, bnb_4bit_quant_type="nf4",
            bnb_4bit_compute_dtype=torch.bfloat16, bnb_4bit_use_double_quant=False,
        ),
    )).eval().requires_grad_(False)
    generate_model = cast(Callable[..., torch.Tensor], getattr(model, "generate"))
    sys.path.insert(0, str(ROOT / "workers/model-adapter/probe"))
    native = importlib.import_module("native_norm")
    replace_norm = cast(Callable[[nn.Module], int], native.replace_rms_norms)
    norm_count = replace_norm(model)
    tokenizer = api.AutoTokenizer.from_pretrained(str(model_root), local_files_only=True)
    integration = importlib.import_module("lmformatenforcer.integrations.transformers")
    enforcer = importlib.import_module("lmformatenforcer")
    tokenizer_data = integration.build_token_enforcer_tokenizer_data(tokenizer)
    schema = read_json(SPIKE / "semantic-candidate.wire.schema.json")
    validation = importlib.import_module("jsonschema")
    validator = cast(SchemaValidator, validation.Draft202012Validator(
        read_json(SPIKE / "semantic-candidate.schema.json"),
    ))
    dataset = read_json(SPIKE / "fixtures/irl-cases.json")["cases"]
    if not isinstance(dataset, list) or not 1 <= args.runs <= 200:
        raise ValueError("Invalid fixture count")
    system = (
        "You are a small measurement assistant. Reply only to the supplied test input. "
        "Use the same language as the input. Do not call tools, browse, access files, "
        "or use external context. This is synthetic test data. "
        "Do not infer facts about the account owner."
    )
    binding = {
        "model": MODEL, "revision": REVISION, "m": 0, "hookInstalled": False,
        "zeroReference": "ADR 0012 section 2.1, same carrier without hook",
        "precision": "NF4", "computeDtype": "bfloat16", "attention": "eager",
        "doubleQuantization": False, "nativeRmsCount": norm_count,
        "kernel": "aten::_fused_rms_norm", "torch": torch.__version__,
        "transformers": api.__version__, "bitsandbytes": quant.__version__,
        "maxNewTokens": 2048, "sampling": "greedy", "seed": 731,
        "fixturesSha256": sha256(SPIKE / "fixtures/irl-cases.json"),
        "wireSchemaSha256": sha256(SPIKE / "semantic-candidate.wire.schema.json"),
        "sourceSha256": sha256(Path(__file__)),
        "startedUtc": datetime.now(timezone.utc).isoformat(),
    }
    write_json(args.output / "binding.json", binding)
    rows: list[dict[str, object]] = []
    (args.output / "candidates").mkdir()
    for index in range(-1, args.runs):
        preflight()
        sample = object_dict(dataset[max(index, 0)])
        prompt = (
            "Convert this single synthetic user utterance into an experimental "
            "SemanticCandidate JSON object using the supplied schema. "
            "Preserve only what the sentence says. Use sourceRef "
            f"{sample['sourceRef']}; sourceSpan is a half-open range of Unicode code "
            "points in the utterance. Do not fill missing facts. "
            f"Language={sample['lang']}; utterance={json.dumps(sample['text'], ensure_ascii=False)}"
        )
        messages = [
            {"role": "system", "content": system + " JSON schema: " + json.dumps(schema)},
            {"role": "user", "content": prompt},
        ]
        text = tokenizer.apply_chat_template(
            messages, tokenize=False, add_generation_prompt=True, enable_thinking=False,
        )
        tokens = tokenizer(text, return_tensors="pt")
        input_ids = cast(torch.Tensor, tokens["input_ids"]).to("cuda:0")
        attention_mask = cast(torch.Tensor, tokens["attention_mask"]).to("cuda:0")
        torch.cuda.synchronize()
        started = time.perf_counter()
        character_parser = enforcer.JsonSchemaParser(schema)
        allowed = integration.build_transformers_prefix_allowed_tokens_fn(
            tokenizer_data, character_parser,
        )
        with torch.inference_mode():
            result = generate_model(
                input_ids=input_ids, attention_mask=attention_mask, max_new_tokens=2048,
                do_sample=False, prefix_allowed_tokens_fn=allowed, use_cache=True,
                pad_token_id=tokenizer.eos_token_id,
            )
        torch.cuda.synchronize()
        elapsed_ms = (time.perf_counter() - started) * 1000
        output = str(tokenizer.decode(result[0, input_ids.shape[1]:], skip_special_tokens=True))
        candidate: object = None
        try:
            candidate = json.loads(output)
            errors = list(validator.iter_errors(candidate))
            schema_valid = not errors
            binding_valid = schema_valid and source_valid(candidate, sample)
        except (ValueError, TypeError, KeyError, IndexError):
            schema_valid = False
            binding_valid = False
        row: dict[str, object] = {
            "index": index + 1, "caseId": sample["id"], "lang": sample["lang"],
            "category": sample["category"], "model": MODEL, "warmup": index < 0,
            "completedMs": elapsed_ms, "schemaValid": schema_valid,
            "sourceBindingValid": binding_valid, "outputChars": len(output),
            "outputTokens": int(result.shape[1] - input_ids.shape[1]),
            "outputSha256": hashlib.sha256(output.encode("utf-8")).hexdigest(),
            "tokenLimitReached": int(result.shape[1] - input_ids.shape[1]) >= 2048,
        }
        write_json(args.output / "candidates" / ("warmup.json" if index < 0 else f"{sample['id']}.json"),
                   {"caseId": sample["id"], "candidate": candidate})
        if isinstance(candidate, dict) and isinstance(candidate.get("nodes"), list):
            row["nodeCount"] = len(candidate["nodes"])
        with (args.output / "samples.jsonl").open("a", encoding="utf-8") as stream:
            stream.write(json.dumps(row, ensure_ascii=False) + "\n")
        if index >= 0:
            rows.append(row)
        print(f"local IRL {index + 1}/{args.runs}: {elapsed_ms:.2f} ms, "
              f"schema={schema_valid}, binding={binding_valid}", flush=True)
    times = sorted(float(cast(float, row["completedMs"])) for row in rows)
    summary = {
        "count": len(rows), "p50Ms": times[math.ceil(.5 * len(times)) - 1],
        "p90Ms": times[math.ceil(.9 * len(times)) - 1],
        "schemaValidCount": sum(bool(row["schemaValid"]) for row in rows),
        "sourceBindingValidCount": sum(bool(row["sourceBindingValid"]) for row in rows),
        "completedUtc": datetime.now(timezone.utc).isoformat(),
    }
    write_json(args.output / "summary.json", summary)


if __name__ == "__main__":
    main()
