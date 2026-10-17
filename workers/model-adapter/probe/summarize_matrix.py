"""Summarize raw matrix evidence without converting missing data into a pass."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import statistics


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("matrix", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("choose a new output")
    rows = []
    for case in sorted(args.matrix.iterdir()):
        if not case.is_dir():
            continue
        row: dict[str, object] = {"case": case.name}
        path_file = case / "path.json"
        if path_file.exists():
            path = json.loads(path_file.read_text(encoding="utf-8"))
            row.update(
                zero_prefill=path["zero_prefill_logits_max_abs_error"],
                zero_decode=path["zero_decode_logits_max_abs_error"],
                changed_vs_fresh=path["changed_vs_fresh_logits_max_abs_error"],
                cache_invalidated=path["changed_cache_discarded"],
                injection_max_abs_error=max(o["max_abs_error"] for o in path["injection_observations"]),
            )
        else:
            row["path_status"] = "missing; inspect path.log"
        perf_file = case / "concurrent.json"
        if perf_file.exists():
            data = json.loads(perf_file.read_text(encoding="utf-8"))
            samples = [s for s in data["samples"] if not s["warmup"]]
            complete = [s for s in samples if s["exactly_twenty_han"] and s["stop_reason"] == "sentence_boundary"]
            row.update(status=data["status"], samples=len(samples), completed_twenty=len(complete),
                       error=data.get("error"))
            if samples:
                row.update(
                    p50_all_ms=statistics.median(s["latency_ms"] for s in samples),
                    p50_ttft_ms=statistics.median(s["ttft_ms"] for s in samples),
                    p50_tokens_per_second=statistics.median(s["tokens_per_second"] for s in samples),
                    cuda_peak_allocated_bytes=max(s["cuda_peak_allocated_bytes"] for s in samples),
                    gpu_sample_max_mib=max(int(s["gpu"].split(",")[1]) for s in samples),
                )
                row["latency_requirement_met_in_this_fixture"] = (
                    len(samples) == 20 and len(complete) == 20
                    and statistics.median(s["latency_ms"] for s in samples) <= 600
                )
        else:
            row["performance_status"] = "missing; inspect concurrent.log"
        row["acceptance"] = "not automatically certified; VRAM peak attribution and calibrated tolerance separate"
        rows.append(row)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(rows, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


if __name__ == "__main__":
    main()
