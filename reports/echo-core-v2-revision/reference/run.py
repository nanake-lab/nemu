"""Run once into a NEW output directory: python -B run.py --out <path>."""
import argparse
import csv
import hashlib
import json
import math
import platform
import statistics
import sys
import time
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from experiments import Row, e1, e2, e3, e4


def stats(values: list[float]) -> dict[str, float | int | list[float]]:
    mean = statistics.mean(values)
    sd = statistics.stdev(values)
    error = 1.96*sd/math.sqrt(len(values))
    return dict(n=len(values), mean=mean, sd=sd, ci95_mc=[mean-error, mean+error])


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    output: Path = args.out
    output.mkdir(parents=True, exist_ok=False)
    source = Path(__file__).resolve().parent
    paths = sorted(source.glob("*.py")) + [source.parent/"PROTOCOL.md"]
    hashes = {p.relative_to(source.parent).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    start = time.perf_counter()
    provenance: dict[str, object] = dict(start_utc=datetime.now(timezone.utc).isoformat(),
                                       python=sys.version, platform=platform.platform(), source_sha256=hashes)
    (output/"provenance.json").write_text(json.dumps(provenance, indent=2)+"\n", encoding="utf-8")
    rows: list[Row] = []
    traces: list[Row] = []
    for index in range(256):
        rows.extend(e1(31000+index))
        rows.extend(e3(51000+index))
        rows.extend(e4(61000+index))
        if index < 128:
            r, t = e2(41000+index)
            rows.extend(r)
            traces.extend(t)
        if (index+1) % 32 == 0:
            print(f"seeds={index+1} seconds={time.perf_counter()-start:.3f}", flush=True)
    grouped: dict[str, list[float]] = defaultdict(list)
    for row in rows:
        for key, value in row.items():
            if key not in ("seed", "experiment", "arm") and isinstance(value, (int, float)):
                grouped[f'{row["experiment"]}/{row["arm"]}/{key}'].append(float(value))
    summary = {key: stats(values) for key, values in sorted(grouped.items())}
    lookup = {(row["experiment"], row["arm"], row["seed"]): row for row in rows}
    for world in ("stable", "abrupt", "gradual", "cycle"):
        for comparator in ("cumulative", "window16", "window64", "fixed_change"):
            differences = []
            for seed in range(41000, 41128):
                a = lookup[(f"E2_{world}", "adaptive", seed)]["mse"]
                b = lookup[(f"E2_{world}", comparator, seed)]["mse"]
                assert isinstance(a, float) and isinstance(b, float)
                differences.append(a-b)
            summary[f"paired/{world}/adaptive-minus-{comparator}/mse"] = stats(differences)
    artifacts: list[tuple[str, object]] = [("raw.json", rows), ("summary.json", summary), ("sample_traces.json", traces)]
    for name, payload in artifacts:
        (output/name).write_text(json.dumps(payload, indent=2)+"\n", encoding="utf-8")
    fields = ["experiment", "arm", "seed"]+sorted({key for row in rows for key in row}-{ "experiment", "arm", "seed"})
    with (output/"raw.csv").open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=fields, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    unchanged = all(hashlib.sha256((source.parent/name).read_bytes()).hexdigest() == digest for name, digest in hashes.items())
    assert unchanged
    provenance.update(end_utc=datetime.now(timezone.utc).isoformat(), duration_seconds=time.perf_counter()-start,
                      rows=len(rows), sources_unchanged=unchanged)
    (output/"provenance.json").write_text(json.dumps(provenance, indent=2)+"\n", encoding="utf-8")
    print(json.dumps(provenance, indent=2))


if __name__ == "__main__":
    main()
