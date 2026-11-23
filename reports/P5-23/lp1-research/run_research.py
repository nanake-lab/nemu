"""Run once into a new output folder. A completed study with failed properties exits 1."""

from __future__ import annotations

import argparse
import contextlib
import gzip
import hashlib
import io
import json
import platform
import sys
from collections.abc import Iterator
from datetime import datetime, timezone
from pathlib import Path
from typing import TextIO

from original_suite import run_original
from properties import edge_checks, lp1_checks, lp7_checks, replay_checks
from worlds import run_worlds, summarize


@contextlib.contextmanager
def compressed(path: Path) -> Iterator[TextIO]:
    with path.open("xb") as raw:
        with gzip.GzipFile(filename="", fileobj=raw, mode="wb", mtime=0) as zipped:
            with io.TextIOWrapper(zipped, encoding="utf-8", newline="\n") as text:
                yield text


def write(path: Path, value: object) -> None:
    with path.open("x", encoding="utf-8", newline="\n") as out:
        json.dump(value, out, ensure_ascii=False, allow_nan=False, indent=2)
        out.write("\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    destination: Path = args.output
    destination.mkdir(parents=True, exist_ok=False)
    here = Path(__file__).resolve().parent
    source = here.parents[1] / "echo-core-v2-draft" / "check_law.py"
    files = [source, *sorted(here.glob("*.py")), here / "PLAN.md"]
    write(destination / "manifest.json", {
        "started_utc": datetime.now(timezone.utc).isoformat(), "python": sys.version,
        "platform": platform.platform(), "command": sys.argv,
        "sources": {str(p.relative_to(here.parents[2])): hashlib.sha256(p.read_bytes()).hexdigest()
                    for p in files},
    })
    original = run_original(source)
    write(destination / "original-law-results.json", original)
    with compressed(destination / "failures.jsonl.gz") as failed:
        extra = lp1_checks(failed)
        print("LP1", extra, flush=True)
        extra.extend(lp7_checks(failed))
        extra.extend(replay_checks(failed))
    extra.extend(edge_checks())
    write(destination / "extra-results.json", extra)
    print("EXTRA", extra, flush=True)
    with compressed(destination / "world-trajectories.jsonl.gz") as trace:
        worlds = run_worlds(trace)
    write(destination / "world-seeds.json", worlds)
    summary = summarize(worlds)
    write(destination / "world-summary.json", summary)
    print("WORLDS", summary, flush=True)
    failed_count = sum(not bool(row["ok"]) for row in original + extra + worlds)
    write(destination / "completion.json", {
        "finished_utc": datetime.now(timezone.utc).isoformat(), "study_completed": True,
        "failed_checks_or_worlds": failed_count, "formula_approved": False,
        "production_acceptance": "blocked", "exit_code": int(failed_count > 0),
    })
    return int(failed_count > 0)


if __name__ == "__main__":
    raise SystemExit(main())
