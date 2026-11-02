"""Build fixed CPU-only inputs for the two-quantization native benchmark."""
from __future__ import annotations

import gzip
import hashlib
import importlib
import json
from pathlib import Path
from typing import Any

from compact_codec import compact_schema, legend
from http_probe import ROOT, sha256, write_json

REVISION = "cdbee75f17c01a7cc42f958dc650907174af0554"
SYSTEM = ("You are a small measurement assistant. Reply only to the supplied test input. "
          "Use the same language as the input. Do not call tools, browse, access files, "
          "or use external context. This is synthetic test data. "
          "Do not infer facts about the account owner.")
FIXED_SENTENCE = "今天的努力值得肯定请带着信心继续向前走吧。"
INPUTS = ["spikes/codex-app-server/fixtures/irl-cases.json",
          "spikes/codex-app-server/semantic-candidate.schema.json",
          "spikes/codex-app-server/semantic-candidate.wire.schema.json",
          "spikes/codex-app-server/src/local_irl.py", "spikes/codex-app-server/src/local_breakdown.py",
          "workers/model-adapter/probe/latency_probe.py", "spikes/carrier-engine/compact_codec.py",
          "spikes/carrier-engine/benchmark_plan.py", "spikes/carrier-engine/http_probe.py",
          "spikes/carrier-engine/benchmark_native.py", "spikes/carrier-engine/requirements-eval.lock",
          "spikes/carrier-engine/schema_validation.py", "spikes/carrier-engine/full_window.py",
          "spikes/carrier-engine/injection_probe.cpp", "spikes/carrier-engine/native_probe.hpp",
          "spikes/carrier-engine/activation_trace.hpp", "workers/model-adapter/probe/direction_probe.py",
          "workers/model-adapter/probe/memory_watch.ps1", "spikes/codex-app-server/src/decoder_adapter.py"]


def prepare(cache: Path, output: Path) -> None:
    # Transformers is imported only here and only from the CPU conversion environment.
    api = importlib.import_module("transformers")
    if output.exists():
        raise FileExistsError("Use a new prepared-input directory")
    model = cache / "models/Qwen--Qwen3-4B-Instruct-2507" / REVISION
    tokenizer = api.AutoTokenizer.from_pretrained(str(model), local_files_only=True, trust_remote_code=False)
    fixtures = json.loads((ROOT / INPUTS[0]).read_text(encoding="utf-8"))["cases"]
    if len(fixtures) != 200 or any(item.get("synthetic") is not True for item in fixtures):
        raise ValueError("Expected original 200 synthetic inputs")
    wire = json.loads((ROOT / INPUTS[2]).read_text(encoding="utf-8"))
    schemas = {"wire": wire, "compact": compact_schema()}
    jobs: list[dict[str, Any]] = []

    def job(identifier: str, arm: str, sample: dict[str, Any] | None, warmup: bool) -> dict[str, Any]:
        if arm == "speech":
            messages = [{"role": "user", "content": "请逐字输出下面这句话，不添加其他内容：" + FIXED_SENTENCE}]
            maximum = 64
        else:
            if sample is None:
                raise ValueError("IRL sample missing")
            schema = schemas["compact" if arm == "compact" else "wire"]
            system = SYSTEM + " JSON schema: " + json.dumps(schema)
            prompt = ("Convert this single synthetic user utterance into an experimental "
                      "SemanticCandidate JSON object using the supplied schema. "
                      "Preserve only what the sentence says. Use sourceRef "
                      f"{sample['sourceRef']}; sourceSpan is a half-open range of Unicode code "
                      "points in the utterance. Do not fill missing facts. "
                      f"Language={sample['lang']}; utterance={json.dumps(sample['text'], ensure_ascii=False)}")
            if arm == "compact":
                system += " Compact mapping (lossless, zero-based enum codes): " + json.dumps(legend(), ensure_ascii=False)
                prompt += " Return compact v=1; root s is the numeric part of sourceRef."
            messages = [{"role": "system", "content": system}, {"role": "user", "content": prompt}]
            maximum = 2048
        text = tokenizer.apply_chat_template(messages, tokenize=False, add_generation_prompt=True, enable_thinking=False)
        token_ids = tokenizer(text, add_special_tokens=True)["input_ids"]
        if len(token_ids) + maximum > 8192:
            raise ValueError("Registered context would truncate input/output")
        return {"id": identifier, "arm": arm, "warmup": warmup, "sample": sample,
                "prompt": text, "inputTokenIds": token_ids, "maxNewTokens": maximum,
                "promptSha256": hashlib.sha256(text.encode("utf-8")).hexdigest()}

    for arm in ["speech", "unconstrained", "wire", "compact"]:
        jobs.append(job("warmup-" + arm, arm, None if arm == "speech" else fixtures[0], True))
    for index in range(20):
        jobs.append(job(f"speech-{index + 1:02d}", "speech", None, False))
    for index, sample in enumerate(fixtures):
        arms = ["unconstrained", "wire", "compact"] if index < 50 else ["wire", "compact"]
        shift = index % len(arms)
        for arm in arms[shift:] + arms[:shift]:
            jobs.append(job(sample["id"] + "-" + arm, arm, sample, False))
    output.mkdir(parents=True)
    with gzip.open(output / "jobs.json.gz", "wt", encoding="utf-8", newline="\n") as stream:
        json.dump(jobs, stream, ensure_ascii=False)
    write_json(output / "schemas.json", schemas)
    write_json(output / "plan.json", {"revision": REVISION, "seed": 731, "sampling": "greedy",
        "context": 8192, "threads": 4, "batch": 512, "ubatch": 512, "flashAttention": "off",
        "promptCache": False, "maxIrlTokens": 2048, "m": 0, "jobsPerQuantization": len(jobs),
        "formalPerQuantization": {"speech": 20, "unconstrained": 50, "wire": 200, "compact": 200},
        "warmupsPerQuantization": 4, "bindings": [{"path": p, "sha256": sha256(ROOT / p)} for p in INPUTS],
        "jobsSha256": sha256(output / "jobs.json.gz"), "schemasSha256": sha256(output / "schemas.json"),
        "notes": ["Input text/template follows P0-07 fixed-twenty and P0-08 review 50/200 fixtures.",
                  "Input token IDs are prepared before timers, using the fixed tokenizer files.",
                  "Different quantization/kernel/HTTP observation prevent a pure engine-only causal claim.",
                  "Compact arm includes a decoding legend; report total speed, not only token savings."]})
    print(json.dumps({"prepared": str(output), "jobsPerQuantization": len(jobs), "gpuUsed": False}))
