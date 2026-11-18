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
from plan_speak import run_call, run_pipeline, run_plan2

TASK_QUESTIONS = {
    **QUESTIONS, "Q3": "我教过你一个上班偷懒的词，叫什么来着？", "Q4": "今天加班好累啊",
}
CELLS = tuple((question, method) for question in ("Q1", "Q2", "Q3") for method in ("A", "B"))
LOG_DIR = Path(r"D:\Dev\cache\cause-demo\ab-plan")


def prepare(question: str, method: str) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    scenario = json.loads(SCENARIO.read_text(encoding="utf-8"))
    payload = build_question_payload(
        scenario, TASK_QUESTIONS[question], "per_event", with_dialogue=question == "Q1"
    )
    payload["show_ids"] = method != "A"
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
    elif method == "B":
        result = run_pipeline(assembled, records, api_key)
    else:
        result = run_plan2(
            assembled, records, api_key, self_quote="strict" if method == "D" else "loose",
            semantic_review=method == "E",
        )
    append_session_log(log_path, {
        "phase": phase, "round": round_no, "question": question, "method": method, **result,
    })
    print(f"{phase} {round_no:02d} {question}{method}: {'ok' if result['valid'] else 'failed'}", flush=True)
    return bool(result["valid"])


def main(
    cells: tuple[tuple[str, str], ...] = CELLS,
    smoke_cell: tuple[str, str] = ("Q2", "B"),
    log_dir: Path = LOG_DIR,
    *,
    smoke_routes: dict[tuple[str, str], str] | None = None,
) -> int:
    parser = argparse.ArgumentParser()
    phase = parser.add_mutually_exclusive_group(required=True)
    phase.add_argument("--smoke", action="store_true", help="Run only the smoke cell for manual inspection")
    phase.add_argument("--formal", type=Path, metavar="SMOKE_JSONL", help="Append to the inspected smoke log")
    args = parser.parse_args()
    expected_routes = smoke_routes or {}
    smoke_cells = tuple(expected_routes) or (smoke_cell,)
    api_key = os.environ.get("DEEPSEEK_API_KEY")
    if not api_key:
        print("DEEPSEEK_API_KEY is not set", file=sys.stderr)
        return 1
    if args.smoke:
        prepared_smokes = [prepare(*cell) for cell in smoke_cells]
        stamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%S.%fZ")
        log_dir.mkdir(parents=True, exist_ok=True)
        log_path = log_dir / f"{stamp}.jsonl"
        log_path.touch(exist_ok=False)
        print(log_path, flush=True)
        successes = [
            run_trial(log_path, api_key, "smoke", 1, *cell, prepared)
            for cell, prepared in zip(smoke_cells, prepared_smokes, strict=True)
        ]
        return 0 if all(successes) else 1

    log_path = args.formal.resolve()
    if log_path.parent != log_dir.resolve():
        parser.error("smoke log must be in the experiment log directory")
    existing = log_path.read_text(encoding="utf-8").splitlines()
    if len(existing) != len(smoke_cells):
        parser.error("expected only the required smoke rows and no formal attempts")
    for line, cell in zip(existing, smoke_cells, strict=True):
        row = json.loads(line)
        if (row["phase"], row["question"], row["method"], row["valid"]) != ("smoke", *cell, True):
            parser.error("expected a valid smoke row")
        if cell in expected_routes and row.get("route") != expected_routes[cell]:
            parser.error(f"{cell}: expected {expected_routes[cell]} route")
    prepared_cells = {(question, method): prepare(question, method) for question, method in cells}
    for round_no in range(1, ROUNDS + 1):
        for question, method in cells:
            run_trial(
                log_path, api_key, "formal", round_no, question, method,
                prepared_cells[question, method],
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
