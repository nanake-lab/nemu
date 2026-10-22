"""P0-08 review: matched 50-case constrained/unconstrained timing, temporary data only."""
from __future__ import annotations

import argparse
from collections.abc import Callable
from datetime import datetime, timezone
import hashlib
import importlib
import json
from pathlib import Path
import sys
import time
from typing import Protocol, cast

from decoder_adapter import harden_tokenizer_data
from local_irl import MODEL, REVISION, ROOT, SPIKE, SchemaValidator
from local_irl import object_dict, preflight, read_json, sha256, source_valid, write_json
from phase_timing import TokenClock, arm_order, decode_rate


class CudaTimingEvent(Protocol):
    def record(self) -> None: ...
    def synchronize(self) -> None: ...
    def elapsed_time(self, other: CudaTimingEvent) -> float: ...


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--quiet-window-confirmed", action="store_true")
    args = parser.parse_args()
    if not args.quiet_window_confirmed or args.output.exists():
        parser.error("Requires an approved quiet window and a new output directory")
    preflight()
    args.output.mkdir(parents=True)
    write_json(args.output / "quiet-window.json", {
        "confirmedByUser": True, "checkedUtc": datetime.now(timezone.utc).isoformat(),
        "blockedProcesses": [], "recheckedBeforeEveryArm": True,
    })
    selected = object_dict(object_dict(read_json(ROOT / "config/models.lock.json")["p0_07_probe"])["selected"])
    if selected["model_id"] != MODEL or selected["revision"] != REVISION:
        raise RuntimeError("Carrier differs from the preregistered selection")
    model_root: Path = args.model_root.resolve()
    if model_root.name != REVISION:
        raise RuntimeError("Pinned model revision required")
    inventory = read_json(ROOT / "reports/P0-07/preparation/"
                          "Qwen--Qwen3-4B-Instruct-2507-weights-20261016T151833171127Z.json")
    weights = inventory["files"]
    if not isinstance(weights, list):
        raise TypeError("Missing pinned weights")
    for raw in weights:
        entry = object_dict(raw)
        weight = model_root / str(entry["path"])
        if weight.stat().st_size != entry["bytes"] or sha256(weight) != entry["sha256"]:
            raise RuntimeError("Weight hash mismatch")
    import torch
    from torch import nn

    event_factory = cast(Callable[..., CudaTimingEvent], torch.cuda.Event)
    api = importlib.import_module("transformers")
    quant = importlib.import_module("bitsandbytes")
    if (torch.__version__, api.__version__, quant.__version__) != (
        "2.10.0+cu128", "4.57.3", "0.50.2",
    ):
        raise RuntimeError("Pinned CUDA library versions required")
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
    generate = cast(Callable[..., torch.Tensor], getattr(model, "generate"))
    sys.path.insert(0, str(ROOT / "workers/model-adapter/probe"))
    native = importlib.import_module("native_norm")
    norms = cast(Callable[[nn.Module], int], native.replace_rms_norms)(model)
    if norms != 145 or native.__file__ is None:
        raise RuntimeError("Selected native RMS binding missing")
    tokenizer = api.AutoTokenizer.from_pretrained(str(model_root), local_files_only=True)
    integration = importlib.import_module("lmformatenforcer.integrations.transformers")
    enforcer = importlib.import_module("lmformatenforcer")
    tokenizer_data, decoder = harden_tokenizer_data(
        integration.build_token_enforcer_tokenizer_data(tokenizer),
    )
    schema = read_json(SPIKE / "semantic-candidate.wire.schema.json")
    validation = importlib.import_module("jsonschema")
    validator = cast(SchemaValidator, validation.Draft202012Validator(
        read_json(SPIKE / "semantic-candidate.schema.json"),
    ))
    dataset = read_json(SPIKE / "fixtures/irl-cases.json")["cases"]
    if not isinstance(dataset, list) or len(dataset) != 200:
        raise RuntimeError("Original 200-case dataset required")
    write_json(args.output / "binding.json", {
        "model": MODEL, "revision": REVISION, "m": 0, "modulationHookInstalled": False,
        "precision": "NF4", "computeDtype": "bfloat16", "doubleQuantization": False,
        "attention": "eager", "nativeRmsCount": norms, "kernel": "aten::_fused_rms_norm",
        "torch": torch.__version__, "transformers": api.__version__, "bitsandbytes": quant.__version__,
        "decoder": decoder, "maxNewTokens": 2048, "seed": 731, "sampling": "greedy",
        "sourceSha256": sha256(Path(__file__)),
        "helperSourceSha256": sha256(SPIKE / "src/local_irl.py"),
        "timingSourceSha256": sha256(SPIKE / "src/phase_timing.py"),
        "adapterSourceSha256": sha256(SPIKE / "src/decoder_adapter.py"),
        "nativeNormSourceSha256": sha256(Path(native.__file__)),
        "fixturesSha256": sha256(SPIKE / "fixtures/irl-cases.json"),
        "wireSchemaSha256": sha256(SPIKE / "semantic-candidate.wire.schema.json"),
        "caseIndices": list(range(1, 51)), "warmupPerArm": 1,
        "gpu": torch.cuda.get_device_properties(0).name,
        "startedUtc": datetime.now(timezone.utc).isoformat(),
        "instrumentation": "Both arms use first-forward CUDA events/wall hooks and a token CPU streamer. This adds synchronization; compare matched arms, not instrumentation-free latency as identical.",
    })
    system = (
        "You are a small measurement assistant. Reply only to the supplied test input. "
        "Use the same language as the input. Do not call tools, browse, access files, "
        "or use external context. This is synthetic test data. "
        "Do not infer facts about the account owner."
    )
    (args.output / "outputs").mkdir()
    for index in range(-1, 50):
        sample = object_dict(dataset[max(index, 0)])
        prompt = (
            "Convert this single synthetic user utterance into an experimental "
            "SemanticCandidate JSON object using the supplied schema. "
            "Preserve only what the sentence says. Use sourceRef "
            f"{sample['sourceRef']}; sourceSpan is a half-open range of Unicode code "
            "points in the utterance. Do not fill missing facts. "
            f"Language={sample['lang']}; utterance={json.dumps(sample['text'], ensure_ascii=False)}"
        )
        messages = [{"role": "system", "content": system + " JSON schema: " + json.dumps(schema)},
                    {"role": "user", "content": prompt}]
        text = tokenizer.apply_chat_template(
            messages, tokenize=False, add_generation_prompt=True, enable_thinking=False,
        )
        tokens = tokenizer(text, return_tensors="pt")
        input_ids = cast(torch.Tensor, tokens["input_ids"]).to("cuda:0")
        mask = cast(torch.Tensor, tokens["attention_mask"]).to("cuda:0")
        for arm in arm_order(max(index, 0)):
            preflight()
            torch.manual_seed(731)
            first_start = event_factory(enable_timing=True)
            first_end = event_factory(enable_timing=True)
            forward_count = 0
            first_forward_start: float | None = None
            prefill_wall: float | None = None

            def before_forward(module: nn.Module, values: tuple[object, ...]) -> None:
                nonlocal first_forward_start
                if forward_count == 0:
                    first_forward_start = time.perf_counter()
                    first_start.record()

            def after_forward(module: nn.Module, values: tuple[object, ...], output: object) -> None:
                nonlocal forward_count, prefill_wall
                if forward_count == 0:
                    first_end.record()
                    first_end.synchronize()
                    if first_forward_start is not None:
                        prefill_wall = (time.perf_counter() - first_forward_start) * 1000
                forward_count += 1

            before_handle = model.register_forward_pre_hook(before_forward)
            after_handle = model.register_forward_hook(after_forward)
            torch.cuda.synchronize()
            started = time.perf_counter()
            clock = TokenClock(started)
            callback_ms = 0.
            callback_count = 0
            parser_started = time.perf_counter()
            allowed: Callable[..., object] | None = None
            if arm == "constrained":
                character_parser = enforcer.JsonSchemaParser(
                    schema, config=enforcer.CharacterLevelParserConfig(
                        alphabet=tokenizer_data.tokenizer_alphabet,
                    ),
                )
                constraint = integration.build_transformers_prefix_allowed_tokens_fn(
                    tokenizer_data, character_parser,
                )

                def measured_allowed(batch_id: int, sentence: torch.Tensor) -> object:
                    nonlocal callback_ms, callback_count
                    callback_start = time.perf_counter()
                    result: object = constraint(batch_id, sentence)
                    callback_ms += (time.perf_counter() - callback_start) * 1000
                    callback_count += 1
                    return result

                allowed = measured_allowed
            parser_ms = (time.perf_counter() - parser_started) * 1000
            try:
                with torch.inference_mode():
                    result = generate(
                        input_ids=input_ids, attention_mask=mask, max_new_tokens=2048,
                        do_sample=False, prefix_allowed_tokens_fn=allowed, use_cache=True,
                        pad_token_id=tokenizer.eos_token_id, streamer=clock,
                    )
                torch.cuda.synchronize()
                completed_ms = (time.perf_counter() - started) * 1000
            finally:
                before_handle.remove()
                after_handle.remove()
            count = int(result.shape[1] - input_ids.shape[1])
            if count != clock.generated or forward_count == 0 or clock.first_ms is None:
                raise RuntimeError("Timing instrumentation did not observe every generated token")
            output_text = str(tokenizer.decode(result[0, input_ids.shape[1]:], skip_special_tokens=True))
            candidate: object = None
            schema_valid = False
            binding_valid = False
            try:
                candidate = json.loads(output_text)
                schema_valid = not list(validator.iter_errors(candidate))
                binding_valid = schema_valid and source_valid(candidate, sample)
            except (ValueError, TypeError, KeyError, IndexError):
                pass
            row = {
                "index": index + 1, "caseId": sample["id"], "lang": sample["lang"],
                "warmup": index < 0, "arm": arm, "inputTokens": int(input_ids.shape[1]),
                "outputTokens": count, "completedMs": completed_ms,
                "firstTokenMs": clock.first_ms, "lastTokenMs": clock.last_ms,
                "tokenTimesMs": clock.token_ms,
                "prefillWallMs": prefill_wall, "prefillCudaElapsedMs": first_start.elapsed_time(first_end),
                "parserBuildMs": parser_ms if arm == "constrained" else None,
                "constraintCallbackWallMs": callback_ms if arm == "constrained" else None,
                "constraintCallbackCount": callback_count,
                "decodeTokensPerSecond": decode_rate(count, clock.first_ms, clock.last_ms),
                "totalTokensPerSecond": count * 1000 / completed_ms,
                "schemaValid": schema_valid, "sourceBindingValid": binding_valid,
                "tokenLimitReached": count >= 2048,
                "outputSha256": hashlib.sha256(output_text.encode("utf-8")).hexdigest(),
                "measuredUtc": datetime.now(timezone.utc).isoformat(),
            }
            stem = "warmup" if index < 0 else str(sample["id"])
            write_json(args.output / "outputs" / f"{stem}-{arm}.json", {
                "caseId": sample["id"], "candidate": candidate, "outputText": output_text,
            })
            with (args.output / "samples.jsonl").open("a", encoding="utf-8") as stream:
                stream.write(json.dumps(row, ensure_ascii=False) + "\n")
            print(f"local timing {index + 1}/50 {arm}: {completed_ms:.2f} ms, tokens={count}", flush=True)
    write_json(args.output / "completed.json", {"formalPairs": 50, "warmupPairs": 1,
                                               "completedUtc": datetime.now(timezone.utc).isoformat()})


if __name__ == "__main__":
    main()
