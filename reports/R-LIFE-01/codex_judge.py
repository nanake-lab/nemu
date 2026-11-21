"""RETIRED historical prototype. Subscription judging is disabled by user decision."""
from __future__ import annotations

import json
from pathlib import Path
import random
from typing import cast
import uuid

from io_support import CACHE, digest, save
from judging import AUDIT_SYSTEM, JUDGE_SYSTEM, packets
from worlds import SEEDS

BLOCK_REASON = ("CODEX_JUDGE_UNVERIFIED / RETIRED: user disabled subscription judging; "
                "subscription batch terms applicability is unverified. See CODEX-JUDGE.md. No process launched.")


class CodexJudge:
    """Fail closed; no boolean/config flag can turn a proposed sandbox into evidence."""
    def __init__(self, root: Path) -> None:
        raise RuntimeError(BLOCK_REASON)

    def call(self, role: str, system: str, payload: object) -> str:
        raise RuntimeError(BLOCK_REASON)

    def close(self) -> None:
        pass


def check_events(events: list[dict[str, object]]) -> str:
    """Necessary post-run check only; cannot prove absent tools or filesystem isolation."""
    threads = [event for event in events if event.get("type") == "thread.started"]
    if len(threads) != 1 or not isinstance(threads[0].get("thread_id"), str):
        raise ValueError("One new thread required")
    if sum(event.get("type") == "turn.started" for event in events) != 1:
        raise ValueError("One fresh turn required")
    if sum(event.get("type") == "turn.completed" for event in events) != 1:
        raise ValueError("Completed turn required")
    for event in events:
        kind = event.get("type")
        if kind in ("item.started", "item.updated", "item.completed"):
            item = event.get("item")
            if not isinstance(item, dict) or item.get("type") not in ("agent_message", "reasoning"):
                raise ValueError("Tool or unknown item; invalidate, never retry")
        elif kind not in ("thread.started", "turn.started", "turn.completed"):
            raise ValueError("Unknown/failure event; invalidate")
    return str(threads[0]["thread_id"])


def check_model_receipt(receipt: dict[str, object], pinned: tuple[str, str]) -> None:
    """Proposed import contract, not an assertion that exec emits these fields."""
    if (receipt.get("source") != "provider_metadata" or not receipt.get("raw_evidence_sha256")
            or (receipt.get("actual_model"), receipt.get("actual_model_revision")) != pinned
            or not all(pinned)):
        raise ValueError("Actual provider identity absent/changed; CLI version or self-report is insufficient")


def external(life: dict[str, object]) -> list[dict[str, object]]:
    return [fact for fact in cast(list[dict[str, object]], life["history"])
            if not str(fact["event_id"]).endswith("-action")]


def export_jobs(root: Path) -> dict[str, object]:
    root = root.resolve()
    if not root.is_relative_to(CACHE.resolve()) or root == CACHE.resolve():
        raise ValueError("Experiment cache only")
    if json.loads((root / "status.json").read_text(encoding="utf-8"))["status"] != "C_behavior_complete_pending_blind_audit":
        raise ValueError("Complete C behavior required; preparation is not data")
    jobs_root = root / "codex-jobs"
    jobs_root.mkdir(exist_ok=False)
    private = []

    def write(system: str, payload: object, identity: dict[str, object]) -> None:
        if len((system + json.dumps(payload, ensure_ascii=False)).encode("utf-8")) + 1024 > 12000:
            raise ValueError("Registered material size exceeded; never truncate")
        identifier = uuid.uuid4().hex
        cwd = jobs_root / identifier
        cwd.mkdir()
        # This is the only file in this job's cwd. Truth/indices stay in the parent manifest.
        save(cwd / "anonymous.json", {"instructions": system, "material": payload})
        private.append({"job": identifier, "request_sha256": digest(cwd / "anonymous.json"), **identity})

    for seed in SEEDS:
        lives = [cast(dict[str, object], json.loads((root / f"life-{seed}-{world}.json").read_text(encoding="utf-8")))
                 for world in (0, 1)]
        for world, life in enumerate(lives):
            write(AUDIT_SYSTEM, {"events": external(life)}, {"role": "audit", "seed": seed, "world": world})
        histories: list[object] = [external(life) for life in lives]
        behaviors = [life["behavior"] for life in lives]
        for layout, (payload, truth) in enumerate(packets(histories, behaviors)):
            for repeat in range(2):
                write(JUDGE_SYSTEM, payload, {"role": "main", "seed": seed, "layout": layout, "repeat": repeat, "truth": truth})
        random.Random(seed + 91000).shuffle(behaviors)
        for layout, (payload, truth) in enumerate(packets([histories[0], histories[0]], behaviors)):
            write(JUDGE_SYSTEM, payload, {"role": "null", "seed": seed, "layout": layout, "truth": truth})
    for index in range(8):
        histories = [[{"event_id": "e0", "at": "2040-01-01", "text": f"清单写明可用数量为{value}。"}]
                     for value in (index + 1, index + 11)]
        behaviors = [[{"action": f"登记可用数量{value}"}] for value in (index + 1, index + 11)]
        for layout, (payload, truth) in enumerate(packets(histories, behaviors)):
            write(JUDGE_SYSTEM, payload, {"role": "anchor", "index": index, "layout": layout, "truth": truth})
    save(root / "codex-private-manifest.json", private)
    result: dict[str, object] = {"jobs": len(private), "status": "export_only_live_blocked", "model_calls": 0,
                                "restriction": BLOCK_REASON}
    save(root / "codex-export-status.json", result)
    return result
