"""Run a solitary-thought chain, or interleave matched append/fresh chains."""
from __future__ import annotations

import argparse
from datetime import datetime, timedelta, timezone
import json
import os
from pathlib import Path
import sys
from typing import Any

from chat import append_session_log, build_deepseek_body, cargo_env, repo_root, run_assembler
from think import CACHE, TIME_FORMAT, append_thought, deepseek_sample, projection_payload, thought_messages

RUN_DIR = CACHE / "cause-demo" / "think-04"


def run_chains(
    scenario: dict[str, Any], start: datetime, step_min: int, rounds: int,
    modes: tuple[str, ...], chains: int, api_key: str, output: Path,
) -> None:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    output.mkdir(parents=True, exist_ok=True)
    states = {(number, mode): scenario for number in range(1, chains + 1) for mode in modes}
    logs = {key: output / f"{stamp}-{key[1]}-{key[0]}.jsonl" for key in states}
    for path in logs.values():
        path.touch(exist_ok=False)
    core = repo_root() / "core"
    env = cargo_env()
    for index in range(rounds):
        at = (start + timedelta(minutes=index * step_min)).strftime(TIME_FORMAT)
        for (number, mode), current in states.items():
            path = logs[number, mode]
            payload = projection_payload(current, at)
            assembled = run_assembler(payload, core, env)
            if assembled["outcome"] != "sent":
                append_session_log(path, dict(round=index + 1, at=at, assembly=assembled))
                raise RuntimeError(f"组包失败，详见 {path}")
            binding = assembled["binding"]
            if mode == "append" and not binding["accepted"]:
                raise ValueError(f"链 {number} 第 {index + 1} 轮没有接纳任何槽。")
            request = build_deepseek_body(thought_messages(assembled["messages"]), no_think=True)
            append_session_log(path, dict(kind="request", round=index + 1, at=at,
                                          mode=mode, chain=number, binding=binding, request=request))
            sample = deepseek_sample(request, api_key)
            valid = bool(sample["valid"] and sample["usage"])
            append_session_log(path, {**sample, "kind": "response", "round": index + 1, "valid": valid})
            if not valid:
                raise RuntimeError(f"第 {index + 1} 轮响应无效，详见 {path}")
            if mode == "append":
                states[number, mode] = append_thought(current, binding, at, sample["text"])
            if index + 1 == rounds:
                with path.with_suffix(".final.json").open("x", encoding="utf-8", newline="\n") as stream:
                    json.dump(states[number, mode], stream, ensure_ascii=False, indent=2)
                    stream.write("\n")
        print(f"{index + 1}/{rounds} 轮 · {len(states)} 条链", flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", required=True, type=Path)
    parser.add_argument("--start", required=True, help="YYYY-MM-DD HH:MM (first round)")
    parser.add_argument("--step-min", type=int, default=10)
    parser.add_argument("--rounds", required=True, type=int)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--mode", choices=("append", "fresh"))
    group.add_argument("--paired-chains", type=int, metavar="N", help="alternate N append/fresh pairs")
    parser.add_argument("--thinker", choices=("deepseek",), default="deepseek")
    parser.add_argument("--ds-think", choices=("off",), default="off")
    args = parser.parse_args()
    chains = 1 if args.paired_chains is None else args.paired_chains
    if min(args.rounds, args.step_min, chains) < 1:
        parser.error("--rounds、--step-min 和 --paired-chains 必须为正整数")
    api_key = os.environ.get("DEEPSEEK_API_KEY", "").strip()
    if not api_key:
        parser.error("环境变量 DEEPSEEK_API_KEY 未设置或为空")
    modes = (args.mode,) if args.mode else ("append", "fresh")
    try:
        run_chains(json.loads(args.scenario.read_text(encoding="utf-8")),
                   datetime.strptime(args.start, TIME_FORMAT), args.step_min, args.rounds,
                   modes, chains, api_key, RUN_DIR)
    except (KeyError, OSError, RuntimeError, ValueError, TypeError, OverflowError) as exc:
        print(f"错误：{exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
