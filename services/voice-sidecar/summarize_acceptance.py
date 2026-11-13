"""Reproduce P4-02 acceptance summaries without CUDA or modifying raw data."""
from __future__ import annotations
import argparse
from datetime import datetime
import hashlib
import json
from pathlib import Path
import statistics
from typing import Any


def summarize(folder: Path) -> dict[str, Any]:
    result = json.loads((folder / "result.json").read_text(encoding="utf-8"))
    rows = [json.loads(line) for line in (folder / "samples.jsonl").read_text(encoding="utf-8").splitlines()]
    for lang in ("zh", "en", "ja"):
        samples = [row["first_packet_ms"] for row in rows if row["phase"] == "latency" and row["lang"] == lang]
        assert len(samples) == 20 and statistics.median(samples) == result[lang]["p50_ms"]
    assert {row["variant"] for row in rows} == {result["variant"]}
    stress = [row for row in rows if row["phase"] == "stress"]
    assert len(stress) == result["stress"]["iterations"] and result["stress"]["seconds"] >= 1800
    start = datetime.fromisoformat(stress[0]["utc"])
    bins = []
    for index in range(6):
        batch = [row for row in stress if min(5, int((datetime.fromisoformat(row["utc"]) - start).total_seconds() / 300)) == index]
        bins.append({"minutes": [index * 5, (index + 1) * 5], "samples": len(batch),
                     "rss_median_mib": statistics.median(row["rss_bytes"] / 2**20 for row in batch)})
    return {"status": result["status"], "variant": result["variant"],
            "latency": {lang: result[lang] for lang in ("zh", "en", "ja")}, "cancel": result["cancel"],
            "stress": result["stress"], "rss_first_mib": stress[0]["rss_bytes"] / 2**20,
            "rss_last_mib": stress[-1]["rss_bytes"] / 2**20,
            "rss_delta_mib": (stress[-1]["rss_bytes"] - stress[0]["rss_bytes"]) / 2**20,
            "rss_five_minute_bins": bins,
            "pressure_assessment": "completed; RSS bin medians rise, stability not accepted; no growth tolerance invented",
            "torch_max_bytes": {key: max(row["vram"][key] for row in rows) for key in (
                "allocatedBytes", "reservedBytes", "peakAllocatedBytes", "peakReservedBytes")},
            "stress_torch_current_ranges": {key: [min(row["vram"][key] for row in stress), max(row["vram"][key] for row in stress)]
                                            for key in ("allocatedBytes", "reservedBytes")},
            "memory_sampled_peaks": result["memory"]["sampled_peaks"],
            "memory_upper_bound_bytes": result["nemu_memory_upper_bound_bytes"],
            "memory_observations": result["memory"]["observations"],
            "memory_max_gap_s": result["memory"]["maximum_start_interval_s"],
            "memory_errors": result["memory"]["errors"], "render_exit": result["render_exit"],
            "memory_watch_exit": result["memory_watch_exit"]}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reports", type=Path, required=True)
    args = parser.parse_args()
    cases = ("gpu-0.6B-v2", "gpu-1.7B-v1")
    summary = {case: summarize(args.reports / case) for case in cases}
    summary["failed_startup_attempt"] = json.loads((args.reports / "gpu-0.6B-v1/result.json").read_text(encoding="utf-8"))
    (args.reports / "acceptance-summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    inventory = []
    for case in (*cases, "gpu-0.6B-v1"):
        for file in sorted((args.reports / case).iterdir()):
            if file.is_file():
                with file.open("rb") as stream:
                    digest = hashlib.file_digest(stream, "sha256").hexdigest()
                inventory.append({"path": file.relative_to(args.reports).as_posix(), "bytes": file.stat().st_size, "sha256": digest})
    (args.reports / "gpu-evidence-sha256.json").write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
    print("Validated both 60-sentence runs, exact variants, complete 30-minute pressure logs and evidence hashes")


if __name__ == "__main__":
    main()
