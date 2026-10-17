"""One-command P0-07 measurements; requires verified weights and an idle Windows GPU.

This gathers evidence, including failures. It does not automatically approve a
carrier, a numerical tolerance or an unmeasured behavioral claim.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys

from runtime_guard import require_runner_offline

HERE = Path(__file__).resolve().parent


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--int4-only", action="store_true")
    parser.add_argument("--attention", choices=["eager", "sdpa"], default="eager")
    parser.add_argument("--lean", action="store_true")
    args = parser.parse_args()
    require_runner_offline()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    results: dict[str, int] = {}
    variant = ["--attention", args.attention] + (["--lean"] if args.lean else [])
    matrix = subprocess.run(
        [sys.executable, str(HERE / "run_matrix.py"), "--output", str(output / "matrix"),
         *variant, *(["--precision", "int4"] if args.int4_only else [])],
        check=False,
    )
    results["matrix_exit"] = matrix.returncode
    for model in ["Qwen/Qwen3-4B", "Qwen/Qwen3-4B-Instruct-2507", "microsoft/Phi-4-mini-instruct"]:
        require_runner_offline()
        name = model.replace("/", "--")
        with (output / f"{name}-direction.log").open("x", encoding="utf-8") as log:
            result = subprocess.run(
                [sys.executable, str(HERE / "direction_probe.py"), "--model", model,
                 "--output", str(output / f"{name}-direction.json"), *variant],
                stdout=log, stderr=subprocess.STDOUT, check=False,
            )
        results[name + "_direction_exit"] = result.returncode
    (output / "exit-codes.json").write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
    if any(results.values()):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
