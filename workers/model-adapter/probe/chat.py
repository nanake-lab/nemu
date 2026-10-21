"""Manually launched CLI trial. RAM-only context; no transcript, logging or core storage."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path

from instruct_runtime import InstructRuntime, MODEL, PLAN, REVISION, coefficient_for_ratio


def read_chat_coefficient(ratio: float, norms_path: Path | None, u_sha256: str) -> float:
    plan = json.loads(PLAN.read_text(encoding="utf-8"))
    if not math.isfinite(ratio) or ratio not in plan["ratios"]:
        raise ValueError("Choose one of the preregistered dose ratios")
    if ratio == 0:
        return 0.0
    if norms_path is None:
        raise ValueError("A measured --norms norms.json is required for nonzero ratio")
    norms = json.loads(norms_path.read_text(encoding="utf-8"))
    if (norms["model"] != dict(id=MODEL, revision=REVISION)
            or norms["u_sha256"] != u_sha256
            or norms["plan_sha256"] != hashlib.sha256(PLAN.read_bytes()).hexdigest()
            or norms["status"] != "measured-informational-only"):
        raise ValueError("Norm resource binding mismatch")
    return coefficient_for_ratio(ratio, float(norms["reference_norm"]), float(norms["direction_norm"]))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ratio", type=float, default=0, help="0 for m=0; otherwise a registered ratio")
    parser.add_argument("--norms", type=Path, help="Measured norms.json from instruct_direction.py")
    args = parser.parse_args()
    if os.environ.get("CI"):
        parser.error("probe:chat is manual only and is disabled in CI")
    runtime = InstructRuntime()
    coefficient = read_chat_coefficient(args.ratio, args.norms, runtime.u_sha256)
    runtime.install(max(1, abs(coefficient)))
    messages: list[dict[str, str]] = []
    print("Instruct-2507 trial; RAM-only context. /reset clears context; /exit quits.")
    print(f"ratio={args.ratio}, a[0]={coefficient}; latency is informational, not acceptance.")
    try:
        while True:
            prompt = input("you> ").strip()
            if prompt == "/exit":
                break
            if prompt == "/reset":
                messages.clear()
                continue
            if not prompt:
                continue
            messages.append(dict(role="user", content=prompt))
            try:
                result = runtime.generate(messages, coefficient, 731 + len(messages))
            except Exception:
                messages.pop()
                print("Generation failed; no transcript saved. Reset context or restart the trial.")
                raise
            print("model> " + result.output)
            latency = (f"{result.first_sentence_ms:.2f} ms" if result.first_sentence_ms is not None
                       else "no sentence boundary within budget")
            print(f"first sentence: {latency}; stop={result.stop}; total={result.elapsed_s:.3f}s")
            messages.append(dict(role="assistant", content=result.output))
    except (EOFError, KeyboardInterrupt):
        print("\nTrial ended.")
    finally:
        messages.clear()


if __name__ == "__main__":
    main()
