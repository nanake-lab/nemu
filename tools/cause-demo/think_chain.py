"""Run a solitary-thought chain, or interleave matched append/fresh chains."""
from __future__ import annotations

import argparse
from collections import deque
from datetime import datetime, timedelta, timezone
import json
import os
from pathlib import Path
import sys
from typing import Any

from chat import append_session_log, cargo_env, repo_root, run_assembler
from check_thought import check, complete_response, external_records
from think import (CACHE, CHECK_RUN_DIR, PLAN_RUN_DIR, TIME_FORMAT, append_thought,
                   deepseek_sample, projection_payload, thought_request)
from think_plan import ThoughtPlan

RUN_DIR = CACHE / "cause-demo" / "think-04"
STAMP = "%Y%m%dT%H%M%S.%fZ"


def recorded_sample(
    path: Path, prior: deque[dict[str, Any]], entry: dict[str, Any], api_key: str,
    plan: ThoughtPlan | None = None,
) -> dict[str, Any]:
    # Replay the saved prefix. A retry must use the identical request at this position.
    for _ in range(3):  # DEMO-THINK-04 recovery decision: stop after three consecutive failures.
        if prior:
            if prior.popleft() != entry or not prior:
                raise ValueError(f"旧记录与本轮请求不符，或缺少响应：{path}")
            response = prior.popleft()
            if response.get("kind") != "response" or response.get("round") != entry["round"]:
                raise ValueError(f"旧响应轮次不符：{path}")
            if plan and response["valid"]:
                # Historical points are local provenance. Their cites are absent from the HTTP body.
                replay = plan.verify({**response, "text": response["raw_text"]})
                if replay != response:
                    raise ValueError(f"旧要点与本轮外根核验不符：{path}")
        else:
            append_session_log(path, entry)
            sample = deepseek_sample(entry["request"], api_key)
            response = {**sample, "kind": "response", "round": entry["round"],
                        "valid": bool(sample["valid"] and sample["usage"])}
            if entry["kind"] == "check_request":
                response["valid"] = complete_response(sample)
            if plan:
                response = plan.verify(response)
            append_session_log(path, response)
        if response["valid"]:
            return response
    raise RuntimeError(f"第 {entry['round']} 轮连续三次响应无效，详见 {path}")


def run_chains(
    scenario: dict[str, Any], start: datetime, step_min: int, rounds: int,
    modes: tuple[str, ...], chains: int, api_key: str, output: Path, resume: str | None = None,
    *, plan2: bool = False, check_thought: bool = False,
    checker: str = "v1", checker_think: str = "off",
) -> None:
    stamp = datetime.strptime(resume, STAMP).strftime(STAMP) if resume else datetime.now(timezone.utc).strftime(STAMP)
    output.mkdir(parents=True, exist_ok=True)
    states = {(number, mode): scenario for number in range(1, chains + 1) for mode in modes}
    logs = {key: output / f"{stamp}-{key[1]}-{key[0]}.jsonl" for key in states}
    if resume and set(output.glob(f"{stamp}-*.jsonl")) != set(logs.values()):
        raise ValueError("续跑时的链号和模式须与原批次相同")
    prior: dict[Path, deque[dict[str, Any]]] = {}
    for path in logs.values():
        if resume:
            prior[path] = deque(map(json.loads, path.read_text(encoding="utf-8").splitlines()))
            recorded_rounds = max((row["round"] for row in prior[path]), default=0)
            if recorded_rounds > rounds or (path.with_suffix(".final.json").exists() and rounds != recorded_rounds):
                raise ValueError(f"--rounds 与已保存的进度不符：{path}")
        else:
            path.touch(exist_ok=False)
            prior[path] = deque()
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
            request, plan = thought_request(assembled, payload, no_think=True, plan2=plan2)
            entry = dict(kind="request", round=index + 1, at=at,
                         mode=mode, chain=number, binding=binding, request=request)
            if check_thought:
                entry["check"] = True
                if checker != "v1":
                    entry["checker"] = checker
                if checker_think != "off":
                    entry["checker_think"] = checker_think
            sample = recorded_sample(path, prior[path], entry, api_key, plan)
            blocked = False
            if check_thought and not sample.get("empty"):
                def send_check(body: dict[str, Any]) -> dict[str, Any]:
                    check_entry = dict(kind="check_request", round=index + 1, request=body)
                    return recorded_sample(path, prior[path], check_entry, api_key)

                result = check(sample["text"], external_records(payload["records"][:-1], binding),
                               checker=checker, checker_think=checker_think, send=send_check)
                blocked = result["blocked"]
                decision = dict(kind="decision", round=index + 1, blocked=blocked, claims=result["claims"])
                if checker == "v2":
                    decision["out_of_bounds"] = result["out_of_bounds"]
                    decision["voided_claims"] = result["voided_claims"]
                if prior[path]:
                    if prior[path].popleft() != decision:
                        raise ValueError(f"旧核对结论与重放不符：{path}")
                else:
                    append_session_log(path, decision)
            if mode == "append" and not sample.get("empty") and not blocked:
                points = sample["verification"]["kept"] if plan else None
                states[number, mode] = append_thought(current, binding, at, sample["text"], points)
            if index + 1 == rounds:
                final = path.with_suffix(".final.json")
                if resume and final.exists():
                    if json.loads(final.read_text(encoding="utf-8")) != states[number, mode]:
                        raise ValueError(f"最终场景与重放不符：{final}")
                    continue
                with final.open("x", encoding="utf-8", newline="\n") as stream:
                    json.dump(states[number, mode], stream, ensure_ascii=False, indent=2)
                    stream.write("\n")
        print(f"{index + 1}/{rounds} 轮 · {len(states)} 条链", flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", required=True, type=Path)
    parser.add_argument("--start", required=True, help="YYYY-MM-DD HH:MM (first round)")
    parser.add_argument("--step-min", type=int, default=10)
    parser.add_argument("--rounds", required=True, type=int)
    parser.add_argument("--resume", metavar="UTC_STAMP", help="replay the recorded prefix and continue")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--mode", choices=("append", "fresh"))
    group.add_argument("--paired-chains", type=int, metavar="N", help="alternate N append/fresh pairs")
    parser.add_argument("--thinker", choices=("deepseek",), default="deepseek")
    parser.add_argument("--ds-think", choices=("off",), default="off")
    parser.add_argument("--check", action="store_true", help="独立核对整条念头，通过后才写回")
    parser.add_argument("--checker", choices=("v1", "v2"), default="v1", help="独立核对口径")
    parser.add_argument("--checker-think", choices=("off", "high"), default="off", help="独立核对思考档位")
    parser.add_argument("--plan2", action="store_true")
    parser.add_argument("--chains", type=int, help="同一 --mode 下的独立链数")
    args = parser.parse_args()
    if args.chains is not None and args.mode is None:
        parser.error("--chains 需要 --mode")
    chains = args.chains if args.mode else args.paired_chains
    if chains is None:
        chains = 1
    if min(args.rounds, args.step_min, chains) < 1:
        parser.error("--rounds、--step-min 和链数必须为正整数")
    api_key = os.environ.get("DEEPSEEK_API_KEY", "").strip()
    if not api_key:
        parser.error("环境变量 DEEPSEEK_API_KEY 未设置或为空")
    modes = (args.mode,) if args.mode else ("append", "fresh")
    try:
        output = CHECK_RUN_DIR if args.check else (PLAN_RUN_DIR if args.plan2 else RUN_DIR)
        run_chains(json.loads(args.scenario.read_text(encoding="utf-8")),
                   datetime.strptime(args.start, TIME_FORMAT), args.step_min, args.rounds,
                   modes, chains, api_key, output, args.resume,
                   plan2=args.plan2, check_thought=args.check,
                   checker=args.checker, checker_think=args.checker_think)
    except (KeyError, OSError, RuntimeError, ValueError, TypeError, OverflowError) as exc:
        print(f"错误：{exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
