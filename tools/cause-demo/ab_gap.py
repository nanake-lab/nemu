from __future__ import annotations

import io
import json
import os
import sys
from contextlib import redirect_stdout
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from chat import (
    FIRST_DIALOGUE_H_POS,
    append_session_log,
    build_deepseek_body,
    call_deepseek_stream,
    cargo_env,
    estimate_cost,
    repo_root,
    run_assembler,
)

SCENARIO = Path(__file__).with_name("scenarios") / "thoughts.json"
LOG_DIR = Path(r"D:\Dev\cache\cause-demo\ab-gap")
QUESTIONS = {"Q1": "那天你是什么感觉？", "Q2": "阿杰骂你那天，你什么感觉？"}
CELLS = (("Q1", False), ("Q1", True), ("Q2", False), ("Q2", True))
ROUNDS = 10
Q2_ANCHOR = "关于 [2026-11-05 15:20] 那件事，你当时的想法没有记录。"


def prepare_requests() -> dict[tuple[str, bool], dict[str, Any]]:
    scenario = json.loads(SCENARIO.read_text(encoding="utf-8"))
    core_dir = repo_root() / "core"
    env = cargo_env()
    requests: dict[tuple[str, bool], dict[str, Any]] = {}
    for question, gap_facts in CELLS:
        latest = {
            "id": "d0",
            "h_pos": FIRST_DIALOGUE_H_POS,
            "time": "",
            "speaker": "竹",
            "text": QUESTIONS[question],
        }
        payload = {
            **scenario,
            "records": [*scenario["records"], latest],
            "dialogue": [],
            "latest": latest["id"],
            "gap_facts": gap_facts,
        }
        assembled = run_assembler(payload, core_dir, env)
        if assembled["outcome"] != "sent":
            raise RuntimeError(f"{question} gap_facts={gap_facts}: {assembled['reason']}")
        requests[question, gap_facts] = build_deepseek_body(assembled["messages"], no_think=False)
    return requests


def run_trial(
    log_path: Path,
    api_key: str,
    phase: str,
    round_no: int,
    question: str,
    condition: bool | str,
    body: dict[str, Any],
) -> bool:
    raw_stream: list[str] = []
    with redirect_stdout(io.StringIO()):
        response = call_deepseek_stream(body, api_key, raw_stream=raw_stream)
    valid = bool(response["done"] and not response["error"] and response["text"].strip())
    cost, cost_period = estimate_cost(response["usage"], response["started_utc"])
    field = "gap_facts" if isinstance(condition, bool) else "fact_mode"
    append_session_log(log_path, {
        "phase": phase,
        "round": round_no,
        "question": question,
        field: condition,
        "request": body,
        "response": response,
        "raw_stream": raw_stream,
        "valid": valid,
        "cost_usd": cost,
        "cost_period": cost_period,
    })
    print(f"{phase} {round_no:02d} {question} {field}={condition}: {'ok' if valid else 'failed'}", flush=True)
    return valid


def main() -> int:
    api_key = os.environ.get("DEEPSEEK_API_KEY")
    if not api_key:
        print("DEEPSEEK_API_KEY is not set", file=sys.stderr)
        return 1

    requests = prepare_requests()
    for gap_facts in (False, True):
        messages = requests["Q2", gap_facts]["messages"]
        if (Q2_ANCHOR in messages[-2]["content"].splitlines()) != gap_facts:
            raise RuntimeError(f"Q2 request has wrong gap fact for gap_facts={gap_facts}")

    stamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%S.%fZ")
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    log_path = LOG_DIR / f"{stamp}.jsonl"
    log_path.touch(exist_ok=False)
    print(log_path, flush=True)

    smoke = [
        run_trial(log_path, api_key, "smoke", 1, "Q2", gap_facts, requests["Q2", gap_facts])
        for gap_facts in (False, True)
    ]
    if not all(smoke):
        print("Smoke failed; formal trials not started", file=sys.stderr)
        return 1

    for round_no in range(1, ROUNDS + 1):
        for question, gap_facts in CELLS:
            run_trial(log_path, api_key, "formal", round_no, question, gap_facts, requests[question, gap_facts])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
