"""Sequential P0-07 supplementary evidence; new output only, no acceptance decisions."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from runtime_guard import require_runner_offline

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
MODELS = ["Qwen/Qwen3-4B", "Qwen/Qwen3-4B-Instruct-2507",
          "microsoft/Phi-4-mini-instruct", "Qwen/Qwen3-8B"]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--phase", choices=["all", "kernels", "memory", "direction"], default="all")
    args = parser.parse_args()
    require_runner_offline()
    os.environ["HF_HUB_OFFLINE"] = "1"
    os.environ["TRANSFORMERS_OFFLINE"] = "1"
    destination = args.output.resolve()
    destination.mkdir(parents=True, exist_ok=False)
    exits: list[dict[str, object]] = []

    def run(name: str, command: list[str]) -> None:
        require_runner_offline()
        with (destination / f"{name}.log").open("x", encoding="utf-8") as log:
            completed = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
                                       timeout=3600, check=False)
        exits.append(dict(name=name, command=command, exit=completed.returncode))
        (destination / "exit-codes.json").write_text(json.dumps(exits, indent=2), encoding="utf-8")
        print(f"{name}: exit {completed.returncode}", flush=True)
        if completed.returncode:
            raise RuntimeError(f"{name} failed; preserve evidence and diagnose")

    if args.phase in {"all", "kernels"}:
        for model in MODELS:
            name = model.replace("/", "--")
            run("kernel-" + name, [sys.executable, str(HERE / "kernel_probe.py"),
                                   "--model", model, "--output", str(destination / ("kernel-" + name))])
    if args.phase in {"all", "memory"}:
        stop = destination / "stop-memory"
        with (destination / "memory-watch.log").open("x", encoding="utf-8") as log:
            watcher = subprocess.Popen([
                "powershell.exe", "-NoProfile", "-File", str(HERE / "memory_watch.ps1"),
                "-OutputPath", str(destination / "memory.jsonl"), "-StopPath", str(stop),
                "-RootProcessIds", str(os.getpid()),
            ], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
                creationflags=subprocess.CREATE_NO_WINDOW)
            try:
                deadline = time.monotonic() + 30
                while not (destination / "memory.jsonl").exists():
                    if watcher.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError("memory watcher failed to start")
                    time.sleep(0.2)
                run("matrix", [sys.executable, str(HERE / "run_matrix.py"), "--precision", "int4",
                               "--native-rms", "--output", str(destination / "matrix")])
            finally:
                stop.touch(exist_ok=False)
                code = watcher.wait(timeout=30)
                exits.append(dict(name="memory-watch", exit=code))
                (destination / "exit-codes.json").write_text(json.dumps(exits, indent=2),
                                                             encoding="utf-8")
                if code:
                    raise RuntimeError("memory watcher failed; evidence incomplete")
        windows = []
        for case in sorted((destination / "matrix").iterdir()):
            if case.is_dir():
                windows.append(dict(
                    case=case.name,
                    startUtc=datetime.fromtimestamp((case / "path.log").stat().st_birthtime,
                                                    timezone.utc).isoformat(),
                    endUtc=datetime.fromtimestamp((case / "stop").stat().st_birthtime,
                                                  timezone.utc).isoformat(),
                    provenance="NTFS creation times of fresh path.log and stop; includes solo path",
                ))
        window_path = destination / "case-windows.json"
        window_path.write_text(json.dumps(windows, indent=2), encoding="utf-8")
        run("memory-summary", [sys.executable, str(HERE / "summarize_supplement.py"),
                               "--memory", str(destination / "memory.jsonl"),
                               "--windows", str(window_path),
                               "--output", str(destination / "memory-by-case.json")])
    if args.phase in {"all", "direction"}:
        inputs = []
        for model in MODELS[:3]:
            name = model.replace("/", "--")
            target = destination / (name + ".json")
            inputs.append(str(target))
            run(name, [sys.executable, str(HERE / "direction_probe.py"), "--model", model,
                       "--neutral-dose", "--native-rms", "--output", str(target)])
        run("html", [sys.executable, str(HERE / "direction_html.py"), *inputs,
                     "--output", str(destination / "comparison.html")])


if __name__ == "__main__":
    main()
