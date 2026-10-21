"""Summarize retained supplementary evidence without assigning acceptance."""
from __future__ import annotations

import argparse
from datetime import datetime
import json
from pathlib import Path
import re


def summarize_memory(source: Path, start: datetime | None = None,
                     end: datetime | None = None) -> dict[str, object]:
    peaks = dict(dedicated_bytes=0, shared_bytes=0, total_committed_bytes=0)
    fields = dict(dedicated_bytes="DedicatedUsage", shared_bytes="SharedUsage",
                  total_committed_bytes="TotalCommitted")
    observations = 0
    missing = 0
    errors = []
    max_gap = 0.0
    max_acquisition = 0.0
    previous: datetime | None = None
    processes: dict[int, dict[str, object]] = {}
    adapters: set[str] = set()
    with source.open(encoding="utf-8-sig") as stream:
        for line in stream:
            sample = json.loads(line)
            if "error" in sample:
                errors.append(sample)
                continue
            observations += 1
            started = datetime.fromisoformat(sample["startedUtc"])
            ended = datetime.fromisoformat(sample["endedUtc"])
            if (start is not None and started < start) or (end is not None and started > end):
                observations -= 1
                continue
            max_acquisition = max(max_acquisition, (ended - started).total_seconds())
            if previous is not None:
                max_gap = max(max_gap, (started - previous).total_seconds())
            previous = started
            owned = {int(p["ProcessId"]) for p in sample["ownedProcesses"]}
            for process in sample["ownedProcesses"]:
                processes[int(process["ProcessId"])] = process
            totals = dict.fromkeys(fields, 0)
            found = 0
            for counter in sample["counters"]:
                match = re.match(r"pid_(\d+)_(.+)", counter["Name"])
                if match is None or int(match[1]) not in owned:
                    continue
                found += 1
                adapters.add(match[2])
                for key, field in fields.items():
                    value = counter[field]
                    if value is None:
                        raise ValueError(f"missing counter {field}")
                    totals[key] += int(value)
            missing += int(found == 0)
            for key in fields:
                peaks[key] = max(peaks[key], totals[key])
    return dict(
        status="sampled-not-accepted", observations=observations,
        maximum_start_interval_s=max_gap, maximum_acquisition_s=max_acquisition,
        no_owned_gpu_counter_samples=missing, errors=errors, sampled_peaks=peaks,
        processes=list(processes.values()), adapters=sorted(adapters),
        limitation="CIM/provider sampling; unknown refresh, transient allocations and driver scope; "
                   "sum across owned process instances and adapters, not an exact continuous peak",
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--memory", type=Path)
    parser.add_argument("--windows", type=Path,
                        help="recorded case windows; retain timestamp provenance in this JSON")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if not args.memory:
        parser.error("--memory is required")
    result = summarize_memory(args.memory)
    if args.windows:
        windows = json.loads(args.windows.read_text(encoding="utf-8-sig"))
        result["windows"] = [dict(
            window=window, summary=summarize_memory(
                args.memory, datetime.fromisoformat(window["startUtc"]),
                datetime.fromisoformat(window["endUtc"]),
            ),
        ) for window in windows]
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


if __name__ == "__main__":
    main()
