"""Run the P0-07 path and concurrent performance matrix in isolated processes.

This command does not perform or certify the separate multilingual direction study.
Use a new output directory on every invocation. Failed configurations stay failed.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time

from runtime_guard import require_runner_offline

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--model", help="optional single model from the locked candidate list")
    parser.add_argument("--precision", choices=["bf16", "int8", "int4"])
    parser.add_argument("--attention", choices=["eager", "sdpa"], default="eager")
    parser.add_argument("--lean", action="store_true")
    parser.add_argument("--native-rms", action="store_true")
    args = parser.parse_args()
    if args.native_rms and args.precision != "int4":
        parser.error("native RMS experiment requires --precision int4")
    require_runner_offline()
    destination = args.output.resolve()
    destination.mkdir(parents=True, exist_ok=False)
    node = shutil.which("node")
    if node is None:
        raise RuntimeError("Node required for real rendering")
    models = ["Qwen/Qwen3-4B", "Qwen/Qwen3-4B-Instruct-2507",
              "microsoft/Phi-4-mini-instruct", "Qwen/Qwen3-8B"]
    if args.model and args.model not in models:
        parser.error("model must be a locked carrier candidate")
    rows: list[dict[str, object]] = []
    variant = (["--attention", args.attention] + (["--lean"] if args.lean else [])
               + (["--native-rms"] if args.native_rms else []))
    for model in models:
        if args.model and args.model != model:
            continue
        precisions = ["int4"] if model.endswith("-8B") else ["bf16", "int8", "int4"]
        for precision in precisions:
            if args.precision and args.precision != precision:
                continue
            require_runner_offline()
            case = destination / (model.replace("/", "--") + "-" + precision)
            case.mkdir()
            print(f"BEGIN {model} {precision}", flush=True)
            row: dict[str, object] = dict(model=model, precision=precision,
                                          attention=args.attention, lean=args.lean, native_rms=args.native_rms)
            rows.append(row)
            with (case / "path.log").open("x", encoding="utf-8") as log:
                path = subprocess.run(
                    [sys.executable, str(HERE / "path_probe.py"), "--model", model,
                     "--precision", precision, "--output", str(case / "path.json"), *variant],
                    stdout=log, stderr=subprocess.STDOUT, timeout=600, check=False,
                )
                row["path_exit"] = path.returncode
            ready, stop = case / "render.json", case / "stop"
            with (case / "render.log").open("x", encoding="utf-8") as render_log:
                render = subprocess.Popen(
                    [node, str(HERE / "render_load.mjs"), str(ready), str(stop)],
                    cwd=ROOT, stdout=render_log, stderr=subprocess.STDOUT,
                )
                try:
                    deadline = time.monotonic() + 90
                    while not ready.exists():
                        if render.poll() is not None or time.monotonic() > deadline:
                            raise RuntimeError("render did not become ready")
                        time.sleep(0.5)
                    with (case / "concurrent.log").open("x", encoding="utf-8") as log:
                        measured = subprocess.run(
                            [sys.executable, str(HERE / "latency_probe.py"), "--model", model,
                             "--precision", precision, "--fixed-twenty", "--render-ready",
                             str(ready), "--output", str(case / "concurrent.json"), *variant],
                            stdout=log, stderr=subprocess.STDOUT, timeout=900, check=False,
                        )
                        row["concurrent_exit"] = measured.returncode
                    if render.poll() is not None:
                        raise RuntimeError("render exited during measurement")
                finally:
                    stop.touch(exist_ok=False)
                    row["render_exit"] = render.wait(timeout=45)
            (destination / "matrix.json").write_text(
                json.dumps(rows, indent=2) + "\n", encoding="utf-8",
            )
            print(f"END {model} {precision}: {row}", flush=True)


if __name__ == "__main__":
    main()
