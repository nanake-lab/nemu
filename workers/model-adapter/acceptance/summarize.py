"""Read only completed or partial P5-13 evidence; missing trials never disappear."""
from __future__ import annotations

from argparse import ArgumentParser
from collections import Counter
import json
import math
from pathlib import Path
from statistics import median

from suite import SENTENCE, rows


def records(file: Path) -> list[dict]:
    return [json.loads(line) for line in file.read_text("utf-8-sig").splitlines()] if file.exists() else []


def latency(data: list[float]) -> dict:
    if not data:
        return {}
    values = sorted(data)
    return dict(p50_ms=median(values), p90_ms=values[math.ceil(0.9 * len(values)) - 1])


def summarize(root: Path) -> dict:
    summary: dict = {}
    irl = records(root / "mode-b-irl/irl.jsonl")
    expected = {r["id"] for r in rows("irl")}
    actual = [r["id"] for r in irl]
    full = len(actual) == len(expected) and set(actual) == expected
    passes = sum(r["schema_ok"] for r in irl)
    summary["irl"] = dict(observed=len(irl), denominator=len(expected), full=full,
        schema_passed=passes, source_passed=sum(r["source_ok"] for r in irl),
        schema_rate=passes/len(expected) if full else None, semantic_accuracy=None,
        **latency([r["elapsed_ms"] for r in irl]))
    identity = records(root / "identity/identity.jsonl")
    groups = Counter((r["mode"], r["arm"], r["lang"]) for r in identity)
    summary["identity"] = dict(observed=len(identity), planned=900,
        per_group={"-".join(k): v for k,v in sorted(groups.items())},
        adjudication="requires reading names, self-identification and negations in the actual outputs")
    for group in ("mode-b-irl", "mode-b-joint", "joint-1", "joint-2", "joint-3"):
        memory = records(root / (group + "-wddm.jsonl"))
        if not memory:
            continue
        dedicated = max(r["dedicated"] for r in memory)
        shared = max(r["shared"] for r in memory)
        meta = dict(dedicated_max=dedicated, shared_max=shared, sum_max_bytes=dedicated+shared,
                    nonempty_samples=sum(r["instances"] > 0 for r in memory))
        if group.startswith("joint-"):
            trial = records(root / group / "latency.jsonl")
            formal = [r for r in trial if r["id"] != "0"]
            times = [r["first_sentence_ms"] for r in formal if r["first_sentence_ms"] is not None]
            complete = [r["id"] for r in trial] == [str(i) for i in range(21)]
            exact = sum(r["output"] == SENTENCE for r in formal)
            meta.update(planned=20, measured=len(formal), exact_sentence=exact,
                        **latency(times), complete=complete)
            if complete and len(times) == exact == 20 and meta["nonempty_samples"]:
                meta["meets_latency_and_memory"] = median(times) <= 600 and dedicated + shared <= 8_000_000_000
        summary[group] = meta
    summary["window"] = records(root / "window.jsonl")
    return summary


if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    print(json.dumps(summarize(args.root), ensure_ascii=False, indent=2))
