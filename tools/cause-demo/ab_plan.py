from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from ab_fact import build_question_payload
from ab_gap import QUESTIONS, ROUNDS, SCENARIO
from chat import (
    append_session_log,
    build_deepseek_body,
    cargo_env,
    repo_root,
    run_assembler,
)
from plan_speak import run_call, run_pipeline

TASK_QUESTIONS = {**QUESTIONS, "Q3": "我教过你一个上班偷懒的词，叫什么来着？"}
CELLS = tuple((question, method) for question in TASK_QUESTIONS for method in ("A", "B"))
LOG_DIR = Path(r"D:\Dev\cache\cause-demo\ab-plan")


def prepare(question: str, method: str) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    scenario = json.loads(SCENARIO.read_text(encoding="utf-8"))
    payload = build_question_payload(
        scenario, TASK_QUESTIONS[question], "per_event", with_dialogue=question == "Q1"
    )
    payload["show_ids"] = method == "B"
    assembled = run_assembler(payload, repo_root() / "core", cargo_env())
    if assembled["outcome"] != "sent":
        raise RuntimeError(f"{question}{method}: {assembled['reason']}")
    return assembled, payload["records"]


def run_trial(
    log_path: Path,
    api_key: str,
    phase: str,
    round_no: int,
    question: str,
    method: str,
    prepared: tuple[dict[str, Any], list[dict[str, Any]]],
) -> bool:
    assembled, records = prepared
    if method == "A":
        result = run_call(build_deepseek_body(assembled["messages"], no_think=False), api_key)
    else:
        result = run_pipeline(assembled, records, api_key)
    append_session_log(log_path, {
        "phase": phase, "round": round_no, "question": question, "method": method, **result,
    })
    print(f"{phase} {round_no:02d} {question}{method}: {'ok' if result['valid'] else 'failed'}", flush=True)
    return bool(result["valid"])


def main() -> int:
    parser = argparse.ArgumentParser()
    phase = parser.add_mutually_exclusive_group(required=True)
    phase.add_argument("--smoke", action="store_true", help="Run Q2B only for manual inspection")
    phase.add_argument("--formal", type=Path, metavar="SMOKE_JSONL", help="Append to the inspected smoke log")
    args = parser.parse_args()
    api_key = os.environ.get("DEEPSEEK_API_KEY")
    if not api_key:
        print("DEEPSEEK_API_KEY is not set", file=sys.stderr)
        return 1
    if args.smoke:
        prepared = prepare("Q2", "B")
        stamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%S.%fZ")
        LOG_DIR.mkdir(parents=True, exist_ok=True)
        log_path = LOG_DIR / f"{stamp}.jsonl"
        log_path.touch(exist_ok=False)
        print(log_path, flush=True)
        return 0 if run_trial(log_path, api_key, "smoke", 1, "Q2", "B", prepared) else 1

    log_path = args.formal.resolve()
    if log_path.parent != LOG_DIR.resolve():
        parser.error("smoke log must be in the ab-plan log directory")
    existing = log_path.read_text(encoding="utf-8").splitlines()
    if len(existing) != 1:
        parser.error("expected exactly one smoke row and no formal attempts")
    row = json.loads(existing[0])
    if (row["phase"], row["question"], row["method"], row["valid"]) != ("smoke", "Q2", "B", True):
        parser.error("expected a valid Q2B smoke row")
    prepared_cells = {(question, method): prepare(question, method) for question, method in CELLS}
    for round_no in range(1, ROUNDS + 1):
        for question, method in CELLS:
            run_trial(
                log_path, api_key, "formal", round_no, question, method,
                prepared_cells[question, method],
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
