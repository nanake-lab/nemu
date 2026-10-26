"""Offline analysis of frozen P0-10 activation evidence; never loads a model."""
from __future__ import annotations

import argparse
import csv
import importlib
import json
from pathlib import Path
import re
from typing import Any

from http_probe import sha256, write_json

np = importlib.import_module("numpy")


def floats(path: Path) -> Any:
    value = np.frombuffer(path.read_bytes(), dtype="<f4")
    if not len(value) or not bool(np.isfinite(value).all()):
        raise ValueError("Missing/nonfinite saved values: " + str(path))
    return value


def table(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8") as stream:
        return list(csv.DictReader(stream, delimiter="\t"))


def compare(first: Path, second: Path) -> dict[str, Any]:
    one = {int(p.name.split("-")[0]): p for p in first.glob("*-logits.f32")}
    two = {int(p.name.split("-")[0]): p for p in second.glob("*-logits.f32")}
    positions = []
    for index in sorted(one.keys() & two.keys()):
        a, b = floats(one[index]), floats(two[index])
        if a.shape != (151936,) or b.shape != a.shape:
            raise ValueError("Unexpected saved logit dimensions")
        delta = np.abs(a.astype("float64") - b.astype("float64"))
        positions.append({"step": index, "byteEqual": one[index].read_bytes() == two[index].read_bytes(),
            "maxAbs": float(delta.max()), "changedLogits": int(np.count_nonzero(delta)),
            "argmaxEqual": int(a.argmax()) == int(b.argmax())})
    return {"first": first.name, "second": second.name, "samePositionSet": one.keys() == two.keys(),
        "positionCount": len(positions), "byteEqualPositions": sum(p["byteEqual"] for p in positions),
        "argmaxEqualPositions": sum(p["argmaxEqual"] for p in positions),
        "maxAbs": max((p["maxAbs"] for p in positions), default=None), "positions": positions}


def memory(path: Path, pid: int) -> dict[str, Any]:
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        if "error" in row:
            raise ValueError("Sampler error; preserve and report")
        counters = [c for c in row["counters"] if re.search(r"pid_" + str(pid) + "_", c["Name"], re.I)]
        rows.append({"dedicated": sum(int(c["DedicatedUsage"]) for c in counters),
                     "shared": sum(int(c["SharedUsage"]) for c in counters), "matched": len(counters)})
    dedicated = max((r["dedicated"] for r in rows), default=0)
    shared = max((r["shared"] for r in rows), default=0)
    return {"scope": "sampled carrier PID, no TTS/renderer; independent maxima, not continuous peak",
        "pid": pid, "samples": len(rows), "samplesWithPid": sum(r["matched"] > 0 for r in rows),
        "dedicatedMaxBytes": dedicated, "sharedMaxBytes": shared, "sumOfMaximaBytes": dedicated + shared}


def analyze(run: Path) -> dict[str, Any]:
    raw = run / "injection/raw"
    if not (raw / "execution-completed.txt").is_file():
        raise ValueError("Injection execution not completed")
    pairs = [("observe", "plain"), ("old-zero", "plain"), ("full-zero", "plain"),
        ("native-zero", "plain"), ("native-zero-after-B", "plain"), ("native-clear", "plain"),
        ("old-zero-greedy", "plain"), ("full-zero-greedy", "plain"),
        ("native-zero-greedy", "plain"), ("native-clear-greedy", "plain"),
        ("old-basis-last", "full-basis-last"), ("full-basis-last", "full-basis-all"),
        ("full-basis-all", "native-basis-all"), ("native-A-again", "native-A"),
        ("fresh-A", "native-A"), ("fresh-B", "native-B"), ("native-A", "native-B"),
        ("old-reextracted-last", "full-reextracted-last"),
        ("diagnostic-KV-retained-B", "fresh-B")]
    languages = []
    for language in ["zh", "en", "ja"]:
        folder = raw / language
        modes = sorted(p for p in folder.iterdir() if p.is_dir())
        if len(modes) != 24 or any(not (p / "execution-completed.txt").is_file() for p in modes):
            raise ValueError("Incomplete registered request modes")
        comparisons = [compare(folder / first, folder / second) for first, second in pairs]
        greedy = {mode: [int(r["predicted"]) for r in table(folder / mode / "tokens.tsv")]
                  for mode in ["plain", "old-zero-greedy", "full-zero-greedy", "native-zero-greedy", "native-clear-greedy"]}
        masks: dict[str, Any] = {}
        for mode, site, all_rows in [("old-basis-last", "old", False), ("full-basis-last", "full", False),
                                    ("full-basis-all", "full", True)]:
            before = floats(folder / mode / (site + "-prefill-all-before.f32")).reshape(-1, 2560)
            after = floats(folder / mode / (site + "-prefill-all-after.f32")).reshape(-1, 2560)
            delta = after.astype("float64") - before.astype("float64")
            expected = np.zeros_like(delta)
            if all_rows:
                expected[:, 0] = 1
            else:
                expected[-1, 0] = 1
            masks[mode] = {"rows": len(before), "changedRows": int(np.count_nonzero(np.any(delta != 0, axis=1))),
                "maxAbsDeltaError": float(np.abs(delta - expected).max()),
                "nonLastMaxDelta": float(np.abs(delta[:-1]).max()) if len(delta) > 1 else None}
        baseline = floats(folder / "observe/full-prefill-all-before.f32").reshape(-1, 2560)
        native = floats(folder / "native-basis-all/full-prefill-all-before.f32").reshape(-1, 2560)
        delta = native.astype("float64") - baseline.astype("float64")
        expected = np.zeros_like(delta)
        expected[:, 0] = 1
        masks["native-basis-all-vs-observe"] = {"rows": len(delta),
            "changedRows": int(np.count_nonzero(np.any(delta != 0, axis=1))),
            "maxAbsDeltaError": float(np.abs(delta - expected).max()),
            "deltaL2PerRow": [float(np.linalg.norm(row)) for row in delta]}
        records = [table(p / "settings.tsv")[0] for p in modes]
        languages.append({"language": language, "modes": len(modes), "comparisons": comparisons,
            "greedyTokens": greedy, "greedyZeroEqualsPlain": {k: v == greedy["plain"] for k, v in greedy.items() if k != "plain"},
            "nativeInstallCalls": sum(r["native_action"] == "1" for r in records),
            "nonzeroApiReturns": sum(r["api_return"] != "0" for r in records), "prefillMasks": masks})
    directions = {}
    for site in ["old", "full"]:
        total = np.zeros(2560, dtype="float64")
        for index in range(3):
            calm = floats(raw / "extraction" / f"{index}-0" / ("0-" + site + "-before.f32"))
            excited = floats(raw / "extraction" / f"{index}-1" / ("0-" + site + "-before.f32"))
            total += (excited.astype("float64") - calm.astype("float64")) / 3
        extracted = floats(raw / ("direction-" + site + ".f32"))
        recomputed = (total / np.linalg.norm(total)).astype("float32")
        directions[site] = {"elements": len(extracted), "sha256": sha256(raw / ("direction-" + site + ".f32")),
            "l2": float(np.linalg.norm(extracted.astype("float64"))), "meanContrastL2": float(np.linalg.norm(total)),
            "recomputedMaxAbs": float(np.abs(recomputed.astype("float64") - extracted.astype("float64")).max())}
    one = floats(raw / "direction-old.f32").astype("float64")
    two = floats(raw / "direction-full.f32").astype("float64")
    binding = json.loads((run / "injection/binding.json").read_text(encoding="utf-8"))
    server = json.loads((run / "benchmark/Q4_K_M/server/binding.json").read_text(encoding="utf-8"))
    return {"status": "injection-executed-benchmark-blocked-not-complete-acceptance", "languages": languages,
        "extraction": directions, "directionCosine": float(np.dot(one, two) / np.linalg.norm(one) / np.linalg.norm(two)),
        "injectionMemory": memory(run / "injection/memory.jsonl", binding["pid"]),
        "failedServerMemory": memory(run / "benchmark/Q4_K_M/server/memory.jsonl", server["pid"]),
        "failure": json.loads((run / "failure.json").read_text(encoding="utf-8")),
        "methodSha256": sha256(Path(__file__)),
        "limitations": ["Three fixed synthetic prompts, at most 24 tokens; teacher-forced nonzero arms.",
            "Only three calm/excited contrast pairs, not four nature channels or complete concept validation.",
            "Explicit KV clear; retained-KV diagnostic is invalid for admission.",
            "No speed/IRL samples; missing server logs do not establish CPU fallback."]}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = analyze(args.run)
    write_json(args.output, result)
    print(json.dumps({"output": str(args.output), "languages": len(result["languages"]),
                      "directionCosine": result["directionCosine"], "memory": result["injectionMemory"]}))


if __name__ == "__main__":
    main()
