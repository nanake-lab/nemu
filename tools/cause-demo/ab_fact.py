from __future__ import annotations

import json
import os
import sys
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from ab_gap import Q2_ANCHOR, QUESTIONS, ROUNDS, SCENARIO, run_trial
from chat import (
    FIRST_DIALOGUE_H_POS,
    build_deepseek_body,
    cargo_env,
    repo_root,
    run_assembler,
)

MODES = ("none", "per_event", "general")
CELLS = tuple((question, mode) for question in QUESTIONS for mode in MODES)
LOG_DIR = Path(r"D:\Dev\cache\cause-demo\ab-fact")
GENERAL_FACT = "你过去的想法和感受，只有说话人以 Nemu 开头的记录里写着。"
Q1_DIALOGUE = (
    ("竹", "你还记得我们约好打游戏那次吗？"),
    ("Nemu", "记得。11月1日晚上你约我九点一起打三角洲，后来23:40你说加班，没来得及上线。"),
)


def build_payload(scenario: dict[str, Any], question: str, mode: str) -> dict[str, Any]:
    prior = Q1_DIALOGUE if question == "Q1" else ()
    records = [
        {"id": f"d{index}", "h_pos": FIRST_DIALOGUE_H_POS + index, "speaker": speaker, "text": text}
        for index, (speaker, text) in enumerate((*prior, ("竹", QUESTIONS[question])))
    ]
    return {
        **scenario,
        "records": [*scenario["records"], *records],
        "dialogue": [{"id": rec["id"], "delivered": "full"} for rec in records[:-1]],
        "latest": records[-1]["id"],
        "fact_mode": mode,
    }


def prepare_requests() -> dict[tuple[str, str], dict[str, Any]]:
    scenario = json.loads(SCENARIO.read_text(encoding="utf-8"))
    core_dir = repo_root() / "core"
    env = cargo_env()
    requests: dict[tuple[str, str], dict[str, Any]] = {}
    for question, mode in CELLS:
        assembled = run_assembler(build_payload(scenario, question, mode), core_dir, env)
        if assembled["outcome"] != "sent":
            raise RuntimeError(f"{question} {mode}: {assembled['reason']}")
        messages = assembled["messages"]
        lines = messages[-2]["content"].splitlines()
        expected_dialogue = (
            [{"role": role, "content": text} for role, (_, text) in zip(
                ("user", "assistant"), Q1_DIALOGUE, strict=True
            )] if question == "Q1" else []
        )
        if (
            messages[1:-2] != expected_dialogue
            or messages[-2]["role"] != "system"
            or messages[-1] != {"role": "user", "content": QUESTIONS[question]}
            or (GENERAL_FACT in lines) != (mode == "general")
            or (Q2_ANCHOR in lines) != (mode == "per_event")
            or (mode == "general" and lines[2] != GENERAL_FACT)
        ):
            raise RuntimeError(f"{question} {mode}: unexpected messages")
        requests[question, mode] = build_deepseek_body(messages, no_think=False)
    return requests


def main() -> int:
    api_key = os.environ.get("DEEPSEEK_API_KEY")
    if not api_key:
        print("DEEPSEEK_API_KEY is not set", file=sys.stderr)
        return 1
    requests = prepare_requests()
    stamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%S.%fZ")
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    log_path = LOG_DIR / f"{stamp}.jsonl"
    log_path.touch(exist_ok=False)
    print(log_path, flush=True)
    smoke = [
        run_trial(log_path, api_key, "smoke", 1, question, "general", requests[question, "general"])
        for question in QUESTIONS
    ]
    if not all(smoke):
        print("Smoke failed; formal trials not started", file=sys.stderr)
        return 1
    for round_no in range(1, ROUNDS + 1):
        for question, mode in CELLS:
            run_trial(log_path, api_key, "formal", round_no, question, mode, requests[question, mode])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
