"""Offline anonymous hosted-API packets; private truth is never sent to the provider."""
from __future__ import annotations

import json
from pathlib import Path
import random
from typing import cast
import uuid

from io_support import CACHE, digest, save
from judging import AUDIT_SYSTEM, JUDGE_SYSTEM, packets
from phases import verify_binding, verify_data
from worlds import SEEDS


def external(life: dict[str, object]) -> list[dict[str, object]]:
    return [fact for fact in cast(list[dict[str, object]], life["history"])
            if not str(fact["event_id"]).endswith("-action")]


def export_jobs(root: Path) -> dict[str, object]:
    root = root.resolve()
    if not root.is_relative_to(CACHE.resolve()) or root == CACHE.resolve():
        raise ValueError("Experiment cache only")
    verify_binding(root)
    verify_data(root)
    jobs_root = root / "judge-jobs"
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
    save(root / "judge-private-manifest.json", private)
    result: dict[str, object] = {"jobs": len(private), "status": "export_only_selection_and_approval_required", "model_calls": 0,
                                "restriction": "No model calls; choose hosted provider and approve separate phase-two cap."}
    save(root / "judge-export-status.json", result)
    return result
