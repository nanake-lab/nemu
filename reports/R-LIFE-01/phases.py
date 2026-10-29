"""Separate immutable behavior protocol from later, explicitly chosen judge protocol."""
from __future__ import annotations

import json
from pathlib import Path

from io_support import digest, save

HERE = Path(__file__).resolve().parent
PHASE1_FILES = ("PHASE1.md", "phases.py", "phase1.py", "run.py", "worlds.py", "carrier.py", "carrier.cpp",
                "build.ps1", "io_support.py", "analysis.py", "probability_analysis.py", "diagnostics.py")
COMPLETE = "C_phase1_behavior_complete_pending_local_audit"


def binding() -> dict[str, str]:
    return {name: digest(HERE / name) for name in PHASE1_FILES}


def verify_binding(root: Path) -> None:
    if json.loads((root / "phase1-binding.json").read_text(encoding="utf-8")) != binding():
        raise ValueError("Phase-one protocol/source changed; do not mix or silently rebind")
    if json.loads((root / "status.json").read_text(encoding="utf-8")).get("status") != COMPLETE:
        raise ValueError("Complete phase-one behavior required")


def data_binding(root: Path) -> dict[str, str]:
    names = ("life-*.json", "step-*.json", "ability-pre-*.json", "prompt-*.txt", "decision-*.json",
             "api-*-request.json", "api-*-response.json", "carrier-binding.json", "smoke.json")
    return {p.name: digest(p) for pattern in names for p in sorted(root.glob(pattern))}


def freeze_data(root: Path) -> None:
    save(root / "phase1-data-binding.json", data_binding(root))


def verify_data(root: Path) -> None:
    if json.loads((root / "phase1-data-binding.json").read_text(encoding="utf-8")) != data_binding(root):
        raise ValueError("Frozen phase-one data changed")
