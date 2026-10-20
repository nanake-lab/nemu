"""Three independent Instruct-only concurrent runs, then the preregistered dose study."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
import time

import numpy as np

from instruct_direction import read_committed_plan
from instruct_runtime import MODEL
from runtime_guard import require_runner_offline
from summarize_supplement import summarize_memory

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def run_trial(destination: Path) -> dict[str, object]:
    require_runner_offline()
    destination.mkdir()
    stop = destination / "stop-memory"
    source = destination / "memory.jsonl"
    with (destination / "memory-watch.log").open("x", encoding="utf-8") as log:
        watcher = subprocess.Popen([
            "powershell.exe", "-NoProfile", "-File", str(HERE / "memory_watch.ps1"),
            "-OutputPath", str(source), "-StopPath", str(stop), "-RootProcessIds", str(os.getpid()),
        ], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW)
        try:
            deadline = time.monotonic() + 30
            while not source.exists() or source.stat().st_size == 0:
                if watcher.poll() is not None or time.monotonic() > deadline:
                    raise RuntimeError("Memory watcher did not produce its first sample")
                time.sleep(0.2)
            started = datetime.now(timezone.utc).isoformat()
            with (destination / "matrix.log").open("x", encoding="utf-8") as matrix_log:
                process = subprocess.run([
                    sys.executable, str(HERE / "run_matrix.py"), "--model", MODEL,
                    "--precision", "int4", "--native-rms", "--output", str(destination / "matrix"),
                ], cwd=ROOT, stdout=matrix_log, stderr=subprocess.STDOUT, timeout=1800, check=False)
            ended = datetime.now(timezone.utc).isoformat()
        finally:
            stop.touch(exist_ok=False)
            watch_exit = watcher.wait(timeout=30)
    memory = summarize_memory(source)
    peaks = memory["sampled_peaks"]
    if not isinstance(peaks, dict):
        raise TypeError("Memory peak fields missing")
    bound = int(peaks["dedicated_bytes"]) + int(peaks["shared_bytes"])
    evidence = destination / "matrix" / (MODEL.replace("/", "--") + "-int4")
    concurrent = json.loads((evidence / "concurrent.json").read_text(encoding="utf-8"))
    path = json.loads((evidence / "path.json").read_text(encoding="utf-8"))
    matrix = json.loads((destination / "matrix/matrix.json").read_text(encoding="utf-8"))
    samples = concurrent["samples"]
    measured = [s for s in samples if not s["warmup"]]
    times = [float(s["latency_ms"]) for s in measured]
    p50, p90 = statistics.median(times), float(np.quantile(times, .9, method="linear"))
    exits_ok = all(row[k] == 0 for row in matrix
                   for k in ("path_exit", "concurrent_exit", "render_exit"))
    complete = sum(bool(s["matches_fixed_fixture"]) for s in measured)
    path_ok = (path["zero_prefill_logits_max_abs_error"] == 0
               and path["zero_decode_logits_max_abs_error"] == 0
               and path["changed_vs_fresh_logits_max_abs_error"] == 0
               and path["changed_cache_discarded"]
               and all(o["max_abs_error"] == 0 for o in path["injection_observations"]))
    result = dict(
        started_utc=started, ended_utc=ended, matrix_exit=process.returncode,
        memory_watch_exit=watch_exit, memory=memory,
        memory_upper_bound_bytes=bound, memory_limit_bytes=8_000_000_000,
        torch_peak_allocated_bytes=max(s["cuda_peak_allocated_bytes"] for s in samples),
        torch_peak_reserved_bytes=max(s["cuda_peak_reserved_bytes"] for s in samples),
        p50_ms=p50, p90_ms=p90, percentile_method="p50 median; p90 numpy linear",
        nonwarmup_samples=len(times), fixed_sentence_matches=complete, path_ok=path_ok,
        latency_ok=len(times) == 20 and complete == 20 and p50 <= 600,
        memory_ok=bool(memory["observations"]) and not memory["errors"] and bound <= 8_000_000_000,
        subprocesses_ok=process.returncode == 0 and watch_exit == 0 and exits_ok,
        interpretation="ADR 0012 sampled conservative upper bound, not a continuous peak",
    )
    (destination / "summary.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    read_committed_plan()
    require_runner_offline()
    os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1")
    destination = args.output.resolve()
    destination.mkdir(parents=True, exist_ok=False)
    trials: list[dict[str, object]] = []
    summary: dict[str, object] = dict(status="started", trials=trials, selected=False)
    try:
        for index in range(1, 4):
            trial = run_trial(destination / f"run-{index}")
            trials.append(trial)
            print(f"run-{index}: p50={trial['p50_ms']}, p90={trial['p90_ms']}", flush=True)
        with (destination / "direction.log").open("x", encoding="utf-8") as log:
            direction = subprocess.run([
                sys.executable, str(HERE / "instruct_direction.py"), "--output",
                str(destination / "direction"),
            ], stdout=log, stderr=subprocess.STDOUT, cwd=ROOT, timeout=3600, check=False)
        summary["direction_exit"] = direction.returncode
        if direction.returncode == 0:
            html = subprocess.run([
                sys.executable, str(HERE / "direction_html.py"),
                str(destination / "direction/direction.json"), "--output",
                str(destination / "direction/comparison.html"),
            ], cwd=ROOT, check=False)
            summary["html_exit"] = html.returncode
        summary["status"] = "measured-not-selected"
    except Exception as error:
        summary.update(status="failed", error=str(error))
        raise
    finally:
        (destination / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
