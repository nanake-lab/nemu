"""Serial carrier-only injection and IRL measurement in one fresh quiet window."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import subprocess

from benchmark_native import run
from benchmark_plan import REVISION
from http_probe import ROOT, quiet, sha256, write_json


def injection(cache: Path, build: Path, output: Path) -> None:
    quiet()
    binding = json.loads((build / "build.json").read_text(encoding="utf-8-sig"))
    if binding["exitCode"] != 0 or sha256(build / "injection_probe.exe") != binding["executableSha256"]:
        raise ValueError("Injection executable binding mismatch")
    for source in binding["sources"]:
        if sha256(ROOT / source["path"]) != source["sha256"]:
            raise ValueError("Injection source changed after compilation")
    inventory = json.loads((cache / "gguf/Qwen--Qwen3-4B-Instruct-2507" / REVISION / "inventory.json")
                           .read_text(encoding="utf-8-sig"))
    if inventory["modelRevision"] != REVISION or inventory["engineCommit"] != "620d2225cc436ab0b001e3200e590e3b286ef9ab":
        raise ValueError("Carrier/engine mismatch")
    model = next(entry for entry in inventory["files"] if entry["quantization"] == "Q4_K_M")
    if sha256(Path(model["path"])) != model["sha256"]:
        raise ValueError("Injection carrier hash mismatch")
    extraction = json.loads((cache / "downloads/engine/b11669/extraction-20261025-01.json").read_text(encoding="utf-8"))
    for entry in extraction["files"]:
        if sha256(cache / "llama.cpp/b11669" / entry["path"]) != entry["sha256"]:
            raise ValueError("Injection engine artifact changed")
    output.mkdir(parents=True, exist_ok=False)
    args = [str(build / "injection_probe.exe"), str(cache / "llama.cpp/b11669"), model["path"],
            str(output / "raw"), "--quiet-window-confirmed"]
    with (output / "native.stdout.log").open("xb") as stdout, (output / "native.stderr.log").open("xb") as stderr:
        process = subprocess.Popen(args, stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
        watcher: subprocess.Popen[bytes] | None = None
        try:
            watcher_args = ["pwsh", "-NoProfile", "-File", str(ROOT / "workers/model-adapter/probe/memory_watch.ps1"),
                "-OutputPath", str(output / "memory.jsonl"), "-StopPath", str(output / "stop"),
                "-RootProcessIds", str(process.pid)]
            write_json(output / "binding.json", {"build": binding, "model": model, "arguments": args,
                "startedUtc": datetime.now(timezone.utc).isoformat(), "pid": process.pid,
                "modes": "paired positions/masks, per-position zero, serial updates; invalid retained-KV arm separate",
                "strictCoreReceipt": False, "calibrationSource": "workers/model-adapter/probe/direction_probe.py"})
            with (output / "watch.stdout.log").open("xb") as watch_out, (output / "watch.stderr.log").open("xb") as watch_err:
                watcher = subprocess.Popen(watcher_args, stdout=watch_out, stderr=watch_err,
                                           creationflags=subprocess.CREATE_NO_WINDOW)
                status = process.wait(timeout=1200)
        finally:
            terminated = process.poll() is None
            if terminated:
                process.terminate()
                process.wait(timeout=20)
            if watcher:
                (output / "stop").write_text("completed\n", encoding="utf-8")
                watcher.wait(timeout=30)
            write_json(output / "exit.json", {"native": process.returncode, "watcher": watcher.returncode if watcher else None,
                "terminatedByHarness": terminated, "endedUtc": datetime.now(timezone.utc).isoformat()})
    if status != 0 or watcher is None or watcher.returncode != 0:
        raise RuntimeError("Injection/sampler failed; preserve evidence and stop")
    log = (output / "native.stderr.log").read_text(encoding="utf-8", errors="replace")
    offload = re.search(r"offloaded (\d+)/(\d+) layers to GPU", log)
    if "loaded CUDA backend" not in log or not offload or offload[1] != offload[2]:
        raise RuntimeError("Injection lacks full CUDA offload evidence")
    quiet()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, default=Path("D:/Dev/cache/P0-10"))
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--prepared", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--quiet-window-confirmed", action="store_true")
    args = parser.parse_args()
    if not args.quiet_window_confirmed:
        parser.error("A fresh user-confirmed full GPU window is required")
    quiet()
    args.output.mkdir(parents=True, exist_ok=False)
    os.environ.update(PYTHONDONTWRITEBYTECODE="1", HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1",
                      HF_HUB_DISABLE_TELEMETRY="1", HF_HOME=str(args.cache_root / "huggingface"))
    try:
        plan = json.loads((args.prepared / "plan.json").read_text(encoding="utf-8"))
        for source in plan["bindings"]:
            if sha256(ROOT / source["path"]) != source["sha256"]:
                raise ValueError("Prepared source changed before window start: " + source["path"])
        if (sha256(args.prepared / "jobs.json.gz") != plan["jobsSha256"]
                or sha256(args.prepared / "schemas.json") != plan["schemasSha256"]):
            raise ValueError("Prepared payload changed before window start")
        injection(args.cache_root.resolve(), args.build.resolve(), args.output / "injection")
        run(args.cache_root.resolve(), args.prepared.resolve(), args.output / "benchmark")
        write_json(args.output / "completed.json", {"endedUtc": datetime.now(timezone.utc).isoformat(),
            "status": "executed-needs-offline-analysis", "ttsResident": False, "rendererResident": False})
    except BaseException as error:
        write_json(args.output / "failure.json", {"type": type(error).__name__, "error": str(error),
            "endedUtc": datetime.now(timezone.utc).isoformat()})
        raise


if __name__ == "__main__":
    main()
