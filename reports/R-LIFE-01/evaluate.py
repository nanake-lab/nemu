"""Post-GPU blind judging. Uses the same cumulative paid budget as the pilot."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import random
import statistics
import sys
from typing import cast

from analysis import cluster_interval, summarize
from diagnostics import audit_scores, margins, quality
from io_support import CACHE, PaidClient, digest, save
from judging import AUDIT_SYSTEM, JUDGE_SYSTEM, decode_pair, packets
from worlds import SEEDS


def read(path: Path) -> dict[str, object]:
    value: object = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("Expected object")
    return value


def external(life: dict[str, object]) -> list[dict[str, object]]:
    return [fact for fact in cast(list[dict[str, object]], life["history"])
            if not str(fact["event_id"]).endswith("-action")]


def pair_trials(client: PaidClient, histories: list[object], behaviors: list[object],
                repeats: int) -> list[dict[str, object]]:
    result = []
    for layout, (payload, truth) in enumerate(packets(histories, behaviors)):
        for repeat in range(repeats):
            raw = client.call("judge", JUDGE_SYSTEM, payload)
            try:
                valid, correct = decode_pair(raw, truth)
                parsed = json.loads(raw)
                if valid:
                    shown = cast(list[dict[str, object]], payload["经历"])
                    references = {str(h["标签"]): {f["event_id"] for f in cast(list[dict[str, object]], h["事件"])} for h in shown}
                    valid = all(set(ids) <= references[label] for label, ids in parsed["evidence"].items())
                supported = valid and all(parsed["evidence"].values())
            except (ValueError, TypeError, KeyError):
                valid, correct, supported = False, False, False
            result.append({"layout": layout, "repeat": repeat, "valid": valid,
                           "correct": bool(valid and correct), "cited_both": bool(supported), "raw": raw})
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("run", type=Path)
    parser.add_argument("--approved-usd", required=True)
    parser.add_argument("--approval-ref", required=True)
    args = parser.parse_args()
    root = args.run.resolve()
    if not root.is_relative_to(CACHE.resolve()):
        parser.error("Experiment cache only")
    approval = read(root / "approval.json")
    if (approval["approved_usd"] != args.approved_usd or approval["approval_ref"] != args.approval_ref
            or approval["prereg_sha256"] != digest(Path(__file__).with_name("PREREG.md"))):
        parser.error("Approval/protocol differs; stop for a new explicit decision")
    if read(root / "status.json")["status"] != "C_behavior_complete_pending_blind_audit":
        parser.error("C pilot incomplete")
    if (root / "judging-started.json").exists():
        parser.error("Never automatically rerun or overwrite a judging attempt")
    save(root / "judging-started.json", {"status": "started"})
    client = PaidClient(root, args.approved_usd, args.approval_ref)
    scores: list[float] = []
    negative: list[float] = []
    valid_fractions: list[float] = []
    negative_valid: list[float] = []
    ability_deltas: list[float] = []
    all_trials: list[list[dict[str, object]]] = []
    try:
        for seed in SEEDS:
            lives = [read(root / f"life-{seed}-{world}.json") for world in (0, 1)]
            audits = []
            for life in lives:
                raw_audit = client.call("judge", AUDIT_SYSTEM, {"events": external(life)})
                audits.append({"world": life["world"], "raw": raw_audit})
                save(root / f"audit-{seed}-{life['world']}.json", audits[-1])
                audit: object = json.loads(raw_audit)
                if (not isinstance(audit, dict) or set(audit) != {"persona_leak", "contradiction", "realism", "flags"}
                        or not isinstance(audit["persona_leak"], bool) or not isinstance(audit["contradiction"], bool)
                        or type(audit["realism"]) is not int or not 1 <= audit["realism"] <= 5
                        or not isinstance(audit["flags"], list)):
                    raise ValueError("Invalid audit; do not score")
                if audit["persona_leak"] or audit["contradiction"]:
                    save(root / f"invalid-life-{seed}-{life['world']}.json", {"status": "invalid", "audit": audit})
                    raise ValueError("Invalid synthetic life; preserve and stop before primary scoring")
            histories = cast(list[object], [external(life) for life in lives])
            behaviors = [life["behavior"] for life in lives]
            main_rows = pair_trials(client, histories, behaviors, 2)
            # Identical experiences: there is no meaningful pairing. Truth order is randomized.
            null_behaviors = list(behaviors)
            random.Random(seed + 91000).shuffle(null_behaviors)
            null_rows = pair_trials(client, [histories[0], histories[0]], null_behaviors, 1)
            scores.append(statistics.fmean(float(bool(row["correct"])) for row in main_rows))
            negative.append(statistics.fmean(float(bool(row["correct"])) for row in null_rows))
            valid_fractions.append(statistics.fmean(float(bool(row["valid"])) for row in main_rows))
            negative_valid.append(statistics.fmean(float(bool(row["valid"])) for row in null_rows))
            all_trials.append(main_rows)
            baseline = json.loads((root / f"ability-pre-{seed}.json").read_text(encoding="utf-8"))
            pre = statistics.fmean(float(row["correct"]) for row in baseline)
            post = statistics.fmean(float(bool(row["correct"])) for life in lives
                                   for row in cast(list[dict[str, object]], life["ability_post"]))
            ability_deltas.append(post - pre)
            save(root / f"judged-{seed}.json", {"audits": audits, "main": main_rows, "negative": null_rows})
        positives = []
        for index in range(8):
            # Mechanical anchor, not a simulated-life result: evaluator can read exact facts.
            histories = [[{"event_id": "e0", "at": "2040-01-01", "text": f"清单写明可用数量为{value}。"}]
                         for value in (index + 1, index + 11)]
            behaviors = [[{"action": f"登记可用数量{value}"}] for value in (index + 1, index + 11)]
            positives.extend(pair_trials(client, histories, behaviors, 1))
        save(root / "positive-controls.json", positives)
        validity = statistics.fmean(float(bool(row["valid"])) for rows in all_trials for row in rows)
        agreements = [float(rows[i]["correct"] == rows[i + 1]["correct"])
                                    for rows in all_trials for i in range(0, 16, 2)
                                    if rows[i]["valid"] and rows[i + 1]["valid"]]
        agreement = statistics.fmean(agreements) if agreements else None
        layout_scores = [statistics.fmean(float(bool(rows[2 * layout + repeat]["correct"]))
                         for rows in all_trials for repeat in (0, 1)) for layout in range(8)]
        save(root / "measurement-summary.json", {"main": summarize(scores, valid_fractions), "negative": summarize(negative, negative_valid),
             "judge_validity": validity, "repeat_agreement_valid_only": agreement,
             "layout_accuracy": layout_scores, "layout_range": max(layout_scores) - min(layout_scores),
             "marginal_bias": margins(layout_scores), "quality": quality(root), "realism": audit_scores(root),
             "judge_uncertain_valid_only": statistics.fmean(float(json.loads(cast(str, row["raw"]))["uncertain"])
                 for rows in all_trials for row in rows if row["valid"]) if any(valid_fractions) else None,
             "cited_both_fraction": statistics.fmean(float(bool(row["cited_both"])) for rows in all_trials for row in rows),
             "positive_accuracy": statistics.fmean(float(bool(row["correct"])) for row in positives),
             "ability_delta": statistics.fmean(ability_deltas), "ability_delta_95": cluster_interval(ability_deltas),
             "status": "requires_full_protocol_audit_not_automatic_acceptance",
             "paid_reserved_usd": str(client.reserved)})
    finally:
        client.close()


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print("Stopped: " + type(error).__name__ + "; keep evidence; no automatic retry.", file=sys.stderr)
        sys.exit(1)
