"""P=0, all-support history followed by opposition; descriptive, no gates.

Call the existing old/new reference add methods unchanged, in group order.
No random draws, interpolation, closed-form shortcut or production state.
"""
from __future__ import annotations

import argparse
import copy
import csv
import hashlib
import json
import platform
import sys
from datetime import datetime, timezone
from pathlib import Path

from check_law import Tendency as Old
from law_v2_model import Tendency as New

HISTORIES = (10, 100, 1000, 10000, 100000)
CLAUDE: dict[tuple[str, str, int], int] = {
    ("fixed", "old", 100): 3, ("fixed", "old", 1000): 3,
    ("fixed", "old", 10000): 3,
    ("fixed", "new", 100): 11, ("fixed", "new", 1000): 28,
    ("fixed", "new", 10000): 49,
    ("adaptive", "old", 100): 16, ("adaptive", "old", 1000): 146,
    ("adaptive", "old", 10000): 1449,
    ("adaptive", "new", 100): 21, ("adaptive", "new", 1000): 197,
    ("adaptive", "new", 10000): 1953,
}


def snapshot(model: Old) -> dict[str, object]:
    """Endpoint evidence, including exact binary64 spellings for reproduction."""
    return {
        "pi": model.pi(), "pi_hex": model.pi().hex(),
        "a": model.a, "a_hex": model.a.hex(),
        "v": model.b, "v_hex": model.b.hex(),
        "E": model.consistent(), "C": model.contrary(),
        "next_nominal_h": 1.0 - model.rho(),
        "groups": model.groups, "flips": model.flips,
        "last_sign": model.last_sign,
    }


def run(out: Path) -> dict[str, object]:
    out.mkdir(parents=True, exist_ok=False)
    started = datetime.now(timezone.utc).isoformat()
    protocol: dict[str, object] = {
        "histories": HISTORIES, "n0": 1.0, "kappa": 0.0, "P": 0.0,
        "W": 1.0, "support_S": 1.0, "opposition_S": -1.0,
        "h0": 0.1, "nh": 10.0, "fixed_rho": 0.9,
        "threshold": "first post-opposition pi < 0.5 (strict); no delay offset",
        "update_order": "existing add: rho from pre-group counts, then evidence, then counts",
        "scope": "single tendency; no reshares; no random inputs; descriptive only",
        "parameter_note": "c=1 specified here; Claude's estimates did not specify c or nh",
    }
    print(json.dumps({"protocol": protocol}, ensure_ascii=False), flush=True)
    rows: list[dict[str, object]] = []
    witnesses: list[dict[str, object]] = []
    for mode in ("fixed", "adaptive"):
        for name, cls in (("old", Old), ("new", New)):
            model = cls(n0=1.0, kappa=0.0, prior=0.0, h0=0.1, nh=10.0,
                        fixed_rho=0.9 if mode == "fixed" else None)
            previous_n = 0
            for n in HISTORIES:
                for _ in range(previous_n, n):
                    model.add(1.0, 1.0)
                previous_n = n
                changed = copy.deepcopy(model)
                before = snapshot(changed)
                last = before
                count = 0
                while changed.pi() >= 0.5:
                    last = snapshot(changed)
                    changed.add(1.0, -1.0)
                    count += 1
                estimate = CLAUDE.get((mode, name, n))
                row: dict[str, object] = {
                    "mode": mode, "form": name, "N": n,
                    "opposition_groups_to_pi_lt_half": count,
                    "support_pi": model.pi(), "support_E": model.consistent(),
                    "last_pi": last["pi"], "crossing_pi": changed.pi(),
                    "first_opposition_h": before["next_nominal_h"],
                    "claude_estimate": estimate,
                    "difference_from_estimate": None if estimate is None else count - estimate,
                }
                rows.append(row)
                witnesses.append({"mode": mode, "form": name, "N": n,
                                  "before_opposition": before,
                                  "before_crossing": last, "after_crossing": snapshot(changed)})
                print(json.dumps(row, ensure_ascii=False), flush=True)
    with (out / "counts.csv").open("x", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    with (out / "endpoints.json").open("x", encoding="utf-8") as stream:
        json.dump(witnesses, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
    sources = out / "source"
    sources.mkdir()
    hashes: dict[str, str] = {}
    for name in (Path(__file__).name, "check_law.py", "law_v2_model.py"):
        data = (Path(__file__).parent / name).read_bytes()
        (sources / name).write_bytes(data)
        hashes[name] = hashlib.sha256(data).hexdigest()
    summary: dict[str, object] = {
        "protocol": protocol, "results": rows, "source_sha256": hashes,
        "python": sys.version, "platform": platform.platform(),
        "started_utc": started, "ended_utc": datetime.now(timezone.utc).isoformat(),
        "interpretation": "Measured finite histories only; not a law acceptance or future detector guarantee.",
    }
    with (out / "summary.json").open("x", encoding="utf-8") as stream:
        json.dump(summary, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    run(args.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
