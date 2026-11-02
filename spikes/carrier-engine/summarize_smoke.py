"""Analyze saved smoke evidence without loading any inference library."""
from __future__ import annotations

from array import array
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import re
import sys


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or sys.byteorder != "little":
        raise ValueError("Need a fresh output path and little-endian host")
    with (args.run / "requests.tsv").open(encoding="utf-8") as stream:
        requests = list(csv.DictReader(stream, delimiter="\t"))
    if len(requests) != 21:
        raise ValueError("Unexpected request count")
    languages = []
    modes = ["plain", "zero", "A", "B", "clear", "A-again", "fresh-A"]
    for lang in ["zh", "en", "ja"]:
        rows = {row["mode"]: row for row in requests if row["language"] == lang}
        if set(rows) != set(modes):
            raise ValueError("Unexpected request modes")
        raw = {mode: (args.run / f"{lang}-{mode}.f32").read_bytes() for mode in modes}
        values = {}
        for mode, data in raw.items():
            if len(data) != 151936 * 4:
                raise ValueError("Logit dimensions changed")
            vector = array("f")
            vector.frombytes(data)
            if not all(math.isfinite(value) for value in vector):
                raise ValueError("Non-finite logits")
            values[mode] = vector
        def maximum(a: str, b: str) -> float:
            return max(abs(float(x) - float(y)) for x, y in zip(values[a], values[b], strict=True))
        languages.append({
            "language": lang, "promptTokens": int(rows["plain"]["prompt_tokens"]),
            "zeroLogitsByteIdentical": raw["zero"] == raw["plain"],
            "clearLogitsByteIdentical": raw["clear"] == raw["plain"],
            "zeroGreedyTokensEqual": rows["zero"]["greedy_tokens"] == rows["plain"]["greedy_tokens"],
            "clearGreedyTokensEqual": rows["clear"]["greedy_tokens"] == rows["plain"]["greedy_tokens"],
            "generatedTokensIncludingEos": len([x for x in rows["plain"]["greedy_tokens"].split(",") if x]),
            "zeroMaxAbs": maximum("zero", "plain"), "clearMaxAbs": maximum("clear", "plain"),
            "repeatAMaxAbs": maximum("A-again", "A"), "freshAMaxAbs": maximum("fresh-A", "A"),
            "aVsPlainMaxAbs": maximum("A", "plain"), "bVsAMaxAbs": maximum("B", "A"),
        })
    process = json.loads((args.metadata / "processes.json").read_text(encoding="utf-8-sig"))
    pid = int(process["nativePid"])
    memory = []
    for line in (args.metadata / "memory.jsonl").read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        if "error" in row:
            raise ValueError("Memory sampler error")
        counters = [x for x in row["counters"] if re.search(r"pid_" + str(pid) + "_", x["Name"], re.I)]
        memory.append({"dedicated": sum(int(x["DedicatedUsage"]) for x in counters),
                       "shared": sum(int(x["SharedUsage"]) for x in counters), "matches": len(counters)})
    dedicated = max(row["dedicated"] for row in memory)
    shared = max(row["shared"] for row in memory)
    log = (args.metadata / "stderr.log").read_text(encoding="utf-8", errors="replace")
    result = {
        "status": "minimal-execution-analyzed-not-full-acceptance", "requests": len(requests),
        "apiUpdates": sum(row["api_called"] == "1" for row in requests),
        "apiNonzeroReturns": sum(row["api_called"] == "1" and row["set_return"] != "0" for row in requests),
        "languages": languages, "cudaBackendReported": "loaded CUDA backend" in log,
        "offloadLogLines": [line for line in log.splitlines() if "offloaded" in line],
        "memory": {"scope": "native carrier PID, sampled short inputs; no TTS or renderer",
                   "samples": len(memory), "samplesWithPid": sum(x["matches"] > 0 for x in memory),
                   "dedicatedMaxBytes": dedicated, "sharedMaxBytes": shared,
                   "sumOfIndependentMaximaBytes": dedicated + shared},
        "limitations": ["Three synthetic prompts; at most eight greedy tokens, EOS may end earlier.",
                        "Raw logits cover only the last prefill position, not every position.",
                        "A/B are +/- unit basis vectors, not learned directions.",
                        "KV explicitly cleared per request; no automatic invalidation claim.",
                        "No paired injection positions, IRL benchmark or full speed acceptance."],
        "files": [{"path": str(path), "bytes": path.stat().st_size, "sha256": sha256(path)}
                  for folder in [args.run, args.metadata] for path in sorted(folder.iterdir()) if path.is_file()],
    }
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(json.dumps({key: result[key] for key in ["requests", "apiUpdates", "languages", "memory"]}))


if __name__ == "__main__":
    main()
