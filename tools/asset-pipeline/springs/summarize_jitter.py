"""Compress raw diagnostic traces and summarize each physical region without acceptance thresholds."""
from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import math
from pathlib import Path
from typing import Any


def category(name: str) -> str:
    if name.startswith("NemuTail"):
        return "tail"
    if name.startswith("J_Sec_Hair") and name.endswith(("_23", "_24")):
        return "ear"
    if "Hair" in name:
        return "hair"
    if "Skirt" in name:
        return "skirt"
    return "other-preserved"


def summarize(directory: Path) -> None:
    summary: list[dict[str, Any]] = []
    for path in sorted(directory.glob("jitter-*.json")):
        raw = path.read_bytes()
        data = json.loads(raw)
        samples = data["angularSpeedSamples"]
        hz = data["hz"]
        regions = {}
        for region in ("tail", "ear", "hair", "skirt", "other-preserved"):
            indexes = [i for i, name in enumerate(data["jointNames"]) if category(name) == region]
            windows = [math.sqrt(sum(frame[i] ** 2 for frame in samples[s * hz:(s + 1) * hz]
                                     for i in indexes) / (hz * len(indexes))) for s in range(35)]
            regions[region] = {"joints": len(indexes), "peakRadPerS": max(frame[i] for frame in samples for i in indexes),
                               "rmsBySecond": windows, "lastSecondRms": windows[-1]}
        archive = path.with_suffix(".json.gz")
        archive.write_bytes(gzip.compress(raw, mtime=0))
        summary.append({"variant": data["variant"], "finite": data["finite"], "renderer": data["renderer"],
                        "hz": hz, "drivenSeconds": 30, "settlingSeconds": 5, "regions": regions,
                        "rawArchive": archive.name, "uncompressedSha256": hashlib.sha256(raw).hexdigest(),
                        "acceptance": "Diagnostic only: no high-frequency or user-feel acceptance issued."})
    (directory / "jitter-summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("directory", type=Path)
    summarize(parser.parse_args().directory)
