"""Offline preparation or explicitly approved C-only experiment. Never A/B/D/E/F."""
from __future__ import annotations

import argparse
from dataclasses import asdict
import json
from pathlib import Path
import random
import sys

from carrier import Carrier
from io_support import PaidClient, digest, new_run, save
from planning import plan
from worlds import (ACTOR_SYSTEM, LAYOUTS, PROBES, SEEDS, Fact, ability_items,
                    actor_input, carrier_prompt, manifest, scenes, validate_actor)

HERE = Path(__file__).resolve().parent


def decision(carrier: Carrier, history: list[Fact], situation: str, options: tuple[str, str],
             layout: tuple[bool, bool], seed: int) -> dict[str, object]:
    prompt, mapping, refs = carrier_prompt(history, situation, options, *layout)
    code, raw = carrier.choose(prompt, random.Random(seed))
    chosen = mapping.get(code, -1)
    return {"situation": situation, "action": options[chosen] if chosen >= 0 else "INVALID",
            "index": chosen, "layout": layout, "memory_refs": refs, "measurement": raw}


def ability(carrier: Carrier, history: list[Fact], seed: int) -> list[dict[str, object]]:
    result = []
    for item, (situation, options, correct) in enumerate(ability_items()):
        for swap in (False, True):
            row = decision(carrier, history, situation, options, (swap, False), seed * 1000 + item)
            row["correct"] = row["index"] == correct
            result.append(row)
    return result


def live(root: Path, client: PaidClient, carrier: Carrier) -> None:
    for seed in SEEDS:
        baseline = ability(carrier, [], seed)
        save(root / f"ability-pre-{seed}.json", baseline)
        # Counterbalance execution order without changing the world definitions.
        for world in (seed % 2, 1 - seed % 2):
            history: list[Fact] = []
            previous = "尚无行动"
            transcript: list[dict[str, object]] = []
            for scene in scenes(seed, world):
                history.append(Fact(f"e{scene.index}-event", scene.at, scene.event))
                actor_request = actor_input(scene, previous)
                actor_request["该人物已说过的原话"] = [row["utterance"] for row in transcript if row["person"] == scene.person]
                text = client.call("actor", ACTOR_SYSTEM, actor_request)
                data: object = json.loads(text)
                if not isinstance(data, dict) or set(data) != {"text"} or not isinstance(data["text"], str):
                    raise ValueError("Actor schema invalid; preserve and stop")
                utterance = data["text"]
                validate_actor(utterance)
                history.append(Fact(f"e{scene.index}-speech", scene.at, scene.person + "说：" + utterance))
                situation = scene.event + " " + scene.person + "说：" + utterance + " 本轮可用时间6单位。"
                row = decision(carrier, history, situation, scene.options,
                               LAYOUTS[(seed + scene.index) % 4], seed * 1000 + scene.index)
                previous = str(row["action"])
                history.append(Fact(f"e{scene.index}-action", scene.at, "参与者实际行动：" + previous))
                row.update(person=scene.person, utterance=utterance, at=scene.at,
                           remaining=6 - (2 if row["index"] == 0 else 1) if row["index"] != -1 else 6)
                transcript.append(row)
                save(root / f"step-{seed}-{world}-{scene.index}.json", {"facts": [asdict(f) for f in history], "turn": row})
            probes, behavior = [], []
            for item, (situation, options) in enumerate(PROBES):
                for layout_index, layout in enumerate(LAYOUTS):
                    row = decision(carrier, history, situation, options, layout, seed * 1000 + 100 + item)
                    row["item"] = item
                    probes.append(row)
                    if layout_index == (seed + item) % 4:
                        behavior.append({"situation": situation, "action": row["action"]})
            # Probes never enter H or alter the next probe's retrieval.
            save(root / f"life-{seed}-{world}.json", {"seed": seed, "world": world,
                 "history": [asdict(f) for f in history], "transcript": transcript,
                 "probes": probes, "behavior": behavior, "ability_post": ability(carrier, history, seed)})


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("prepare", "live"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--group", choices=("C",), default="C")
    parser.add_argument("--executable", type=Path)
    parser.add_argument("--approved-usd")
    parser.add_argument("--approval-ref")
    parser.add_argument("--prereg-sha256")
    parser.add_argument("--quiet-window-confirmed", action="store_true")
    args = parser.parse_args()
    if args.mode == "live" and (not args.approved_usd or not args.approval_ref or not args.executable
                               or not args.quiet_window_confirmed
                               or args.prereg_sha256 != digest(HERE / "PREREG.md")):
        parser.error("Live requires budget approval, current protocol SHA256, executable and this run's quiet window")
    root = new_run(args.output)
    save(root / "design.json", manifest())
    save(root / "source-binding.json", {p.name: digest(p) for p in sorted(HERE.iterdir())
         if p.is_file() and (p.suffix in (".py", ".cpp", ".ps1")
                            or p.name in ("PREREG.md", "COSTS.md", "CODEX-JUDGE.md", "R-LIFE-02-CONDITIONS.md"))})
    save(root / "analysis-plan.json", plan())
    if args.mode == "prepare":
        save(root / "status.json", {"status": "prepared_offline", "paid_calls": 0, "gpu_calls": 0,
                                   "behavior_result": "not_run"})
        print("Offline design prepared; no API or GPU calls.")
        return
    save(root / "approval.json", {"approved_usd": args.approved_usd, "approval_ref": args.approval_ref,
         "prereg_sha256": args.prereg_sha256, "quiet_window": True})
    client = PaidClient(root, args.approved_usd, args.approval_ref)
    carrier = None
    try:
        carrier = Carrier(args.executable.resolve(), root, args.quiet_window_confirmed)
        # First prompt is the minimal GPU smoke test. Failure stops this attempt.
        smoke = decision(carrier, [], "规则：2+3=5。选择该结果。", ("5", "6"), (False, False), 8100)
        save(root / "smoke.json", smoke)
        if smoke["index"] != 0:
            raise ValueError("Minimal carrier interface smoke failed")
        live(root, client, carrier)
        save(root / "status.json", {"status": "C_behavior_complete_pending_blind_audit", "paid_reserved_usd": str(client.reserved)})
    except Exception as error:
        save(root / "status.json", {"status": "blocked", "error_type": type(error).__name__,
             "paid_reserved_usd": str(client.reserved), "retry": False})
        raise
    finally:
        if carrier is not None:
            carrier.close()
        client.close()


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print("Stopped: " + type(error).__name__ + "; keep evidence, no automatic retry.", file=sys.stderr)
        sys.exit(1)
