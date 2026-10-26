"""Run the registered local native benchmark; GPU use requires a fresh quiet window."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import gzip
import importlib.util
import json
import math
import os
from pathlib import Path
import sys
from typing import Any

from schema_validation import ValidationError, validator
from benchmark_plan import INPUTS, REVISION, prepare
from compact_codec import expand
from http_probe import ROOT, Server, quiet, sha256, write_json


def reject_constant(value: str) -> None:
    raise ValueError("Non-JSON numeric constant: " + value)


def percentile(values: list[float], fraction: float) -> float | None:
    ordered = sorted(values)
    return ordered[max(0, math.ceil(fraction * len(ordered)) - 1)] if ordered else None


def run(cache: Path, prepared: Path, output: Path) -> None:
    quiet()
    if output.exists():
        raise FileExistsError("Preserve previous measurements")
    plan = json.loads((prepared / "plan.json").read_text(encoding="utf-8"))
    for binding in plan["bindings"]:
        if sha256(ROOT / binding["path"]) != binding["sha256"]:
            raise ValueError("Prepared-input source changed: " + binding["path"])
    for name, key in [("jobs.json.gz", "jobsSha256"), ("schemas.json", "schemasSha256")]:
        if sha256(prepared / name) != plan[key]:
            raise ValueError("Prepared inputs changed")
    with gzip.open(prepared / "jobs.json.gz", "rt", encoding="utf-8") as stream:
        jobs = json.load(stream)
    schemas = json.loads((prepared / "schemas.json").read_text(encoding="utf-8"))
    full = validator(json.loads((ROOT / INPUTS[1]).read_text(encoding="utf-8")))
    compact = validator(schemas["compact"])
    # Reuse the unchanged original source-reference check, not a weaker replacement.
    helpers = ROOT / "spikes/codex-app-server/src"
    sys.path.insert(0, str(helpers))
    spec = importlib.util.spec_from_file_location("p0_08_local_validation", helpers / "local_irl.py")
    if spec is None or spec.loader is None:
        raise RuntimeError("Original validation unavailable")
    original = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(original)
    inventory = json.loads((cache / "gguf/Qwen--Qwen3-4B-Instruct-2507" / REVISION / "inventory.json")
                           .read_text(encoding="utf-8-sig"))
    if inventory["modelRevision"] != REVISION or inventory["engineCommit"] != "620d2225cc436ab0b001e3200e590e3b286ef9ab":
        raise ValueError("Carrier/engine binding differs")
    extraction = json.loads((cache / "downloads/engine/b11669/extraction-20261025-01.json")
                            .read_text(encoding="utf-8"))
    for entry in extraction["files"]:
        if sha256(cache / "llama.cpp/b11669" / entry["path"]) != entry["sha256"]:
            raise ValueError("Engine artifact changed: " + entry["path"])
    output.mkdir(parents=True)
    write_json(output / "quiet-window.json", {"userConfirmedForThisRun": True,
        "checkedUtc": datetime.now(timezone.utc).isoformat(), "runnerProcesses": [], "scope": "two quantizations, carrier only"})
    write_json(output / "plan.json", plan)
    summaries = []
    for quant in ["Q4_K_M", "Q8_0"]:
        model_entry = next(item for item in inventory["files"] if item["quantization"] == quant)
        model = Path(model_entry["path"])
        if sha256(model) != model_entry["sha256"]:
            raise ValueError("Quantized model hash changed")
        destination = output / quant
        destination.mkdir()
        rows: list[dict[str, Any]] = []
        with Server(cache / "llama.cpp/b11669", model, destination / "server") as server:
            props = server.post("/tokenize", {"content": jobs[0]["prompt"], "add_special": False, "parse_special": True})
            write_json(destination / "tokenizer-diagnostic.json", {
                "native": props, "hfInputIds": jobs[0]["inputTokenIds"],
                "equal": props.get("tokens") == jobs[0]["inputTokenIds"],
                "executionUsesPreparedTokenIds": True})
            with (destination / "samples.jsonl").open("x", encoding="utf-8") as stream:
                for job in jobs:
                    payload: dict[str, Any] = {"prompt": job["inputTokenIds"], "n_predict": job["maxNewTokens"],
                        "stream": True, "return_tokens": True, "temperature": 0, "seed": 731,
                        "repeat_penalty": 1.0, "presence_penalty": 0.0, "frequency_penalty": 0.0,
                        "cache_prompt": False}
                    if job["arm"] in schemas:
                        payload["json_schema"] = schemas[job["arm"]]
                    raw = server.completion(payload, destination / "requests" / job["id"])
                    valid: bool | None = None
                    source_valid: bool | None = None
                    validation_error: str | None = None
                    candidate: object = None
                    if job["sample"] is not None:
                        valid, source_valid = False, False
                        try:
                            candidate = json.loads(raw["content"], parse_constant=reject_constant)
                            if job["arm"] == "compact":
                                compact.validate(candidate)
                                if not isinstance(candidate, dict):
                                    raise ValueError("Compact root must be an object")
                                candidate = expand(candidate)
                            full.validate(candidate)
                            valid = True
                            source_valid = original.source_valid(candidate, job["sample"])
                        except (ValueError, TypeError, KeyError, IndexError, ValidationError) as error:
                            validation_error = f"{type(error).__name__}: {error}"
                        write_json(destination / "requests" / job["id"] / "candidate.json", candidate)
                    final = raw["final"]
                    row = {"id": job["id"], "arm": job["arm"], "warmup": job["warmup"],
                           "sample": job["sample"], "promptSha256": job["promptSha256"],
                           "inputTokens": len(job["inputTokenIds"]), "ttftClientMs": raw["ttftClientMs"],
                           "completedClientMs": raw["completedClientMs"], "timings": final.get("timings"),
                           "firstSentenceClientMs": raw["firstSentenceClientMs"],
                           "returnedStreamTokenCount": len(raw["clientTokenTimesMs"]),
                           "stopType": final.get("stop_type"), "schemaValid": valid, "sourceBindingValid": source_valid,
                           "validationError": validation_error, "rawRequest": f"requests/{job['id']}/request.json",
                           "rawResponse": f"requests/{job['id']}/response.json"}
                    rows.append(row)
                    stream.write(json.dumps(row, ensure_ascii=False) + "\n")
                    stream.flush()
                    print(f"{quant} {job['id']} {row['completedClientMs']:.1f} ms schema={valid}", flush=True)
        arms: dict[str, dict[str, Any]] = {}
        for arm in ["speech", "unconstrained", "wire", "compact"]:
            group = [row for row in rows if row["arm"] == arm and not row["warmup"]]
            arms[arm] = {"attempts": len(group), "schemaValid": sum(row["schemaValid"] is True for row in group),
                         "sourceBindingValid": sum(row["sourceBindingValid"] is True for row in group),
                         "ttftObservable": sum(row["ttftClientMs"] is not None for row in group),
                         "ttftP50Ms": percentile([row["ttftClientMs"] for row in group if row["ttftClientMs"] is not None], .5),
                         "ttftP90Ms": percentile([row["ttftClientMs"] for row in group if row["ttftClientMs"] is not None], .9),
                         "completionP50Ms": percentile([row["completedClientMs"] for row in group], .5),
                         "completionP90Ms": percentile([row["completedClientMs"] for row in group], .9)}
            for key in ["prompt_n", "prompt_ms", "prompt_per_second", "predicted_n", "predicted_ms", "predicted_per_second"]:
                values = [float(row["timings"][key]) for row in group
                          if isinstance(row["timings"], dict) and isinstance(row["timings"].get(key), (int, float))]
                arms[arm][key] = {"coverage": len(values), "p50": percentile(values, .5), "p90": percentile(values, .9)}
        write_json(destination / "summary.json", {"quantization": quant, "arms": arms,
            "percentile": "nearest rank, all completed responses including invalid JSON/schema", "model": model_entry})
        summaries.append({"quantization": quant, "arms": arms})
    write_json(output / "summary.json", {"status": "measurements-completed-not-engine-selection", "summaries": summaries})


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, default=Path("D:/Dev/cache/P0-10"))
    parser.add_argument("--prepared", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--quiet-window-confirmed", action="store_true")
    args = parser.parse_args()
    os.environ.update(PYTHONDONTWRITEBYTECODE="1", HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1",
                      HF_HUB_DISABLE_TELEMETRY="1", HF_HOME=str(args.cache_root / "huggingface"))
    if args.prepare_only:
        prepare(args.cache_root.resolve(), args.prepared.resolve())
    else:
        if not args.quiet_window_confirmed or args.output is None:
            parser.error("A fresh user-confirmed quiet window and new output directory are required")
        run(args.cache_root.resolve(), args.prepared.resolve(), args.output.resolve())


if __name__ == "__main__":
    main()
