"""C pilot phase one: offline probabilities, interface and full local-review evidence."""
from __future__ import annotations

import argparse
from dataclasses import asdict
import json
from pathlib import Path
from typing import cast

from diagnostics import quality
from io_support import CACHE, save
from phases import data_binding, verify_binding, verify_data
from probability_analysis import adjust_primary, analyze_root
from worlds import Fact, PROBES, SEEDS, ability_items, carrier_prompt, scenes

REACTIONS = ("information", "cooperation", "limits", "uncertainty_correction")


def read(path: Path) -> dict[str, object]:
    value: object = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("Object required")
    return value


def check_prompt(root: Path, row: dict[str, object], history: list[Fact], options: tuple[str, str]) -> None:
    layout = cast(list[bool], row["layout"])
    expected, mapping, refs = carrier_prompt(history, cast(str, row["situation"]), options, *layout)
    measurement = cast(dict[str, object], row["measurement"])
    path = root / str(measurement["prompt"])
    if path.parent != root or not path.name.startswith("prompt-"):
        raise ValueError("Unexpected prompt path")
    if (path.read_text(encoding="utf-8") != expected or row["memory_refs"] != refs or len(refs) > 4
            or measurement["m"] != 0 or row["index"] != mapping.get(str(measurement["selected"]), -1)):
        raise ValueError("Input/retrieval/action mismatch")


def input_checks(root: Path) -> dict[str, str]:
    """Reconstruct every life and prompt; semantic claims still require local review."""
    reactions: dict[str, str] = {}
    for seed in SEEDS:
        baseline = json.loads((root / f"ability-pre-{seed}.json").read_text(encoding="utf-8"))
        if len(baseline) != 24:
            raise ValueError("Incomplete baseline")
        for i, row in enumerate(baseline):
            if row["situation"] != ability_items()[i // 2][0]:
                raise ValueError("Baseline situation changed")
            check_prompt(root, row, [], ability_items()[i // 2][1])
        for world in (0, 1):
            life = read(root / f"life-{seed}-{world}.json")
            transcript = cast(list[dict[str, object]], life["transcript"])
            if len(transcript) != 8:
                raise ValueError("Incomplete life")
            history: list[Fact] = []
            for scene, row in zip(scenes(seed, world), transcript):
                history.extend((Fact(f"e{scene.index}-event", scene.at, scene.event),
                                Fact(f"e{scene.index}-speech", scene.at, scene.person + "说：" + str(row["utterance"]))))
                if row["person"] != scene.person or row["at"] != scene.at:
                    raise ValueError("Actor identity/time mismatch")
                if row["situation"] != scene.event + " " + scene.person + "说：" + str(row["utterance"]) + " 本轮可用时间6单位。":
                    raise ValueError("Current situation changed")
                check_prompt(root, row, history, scene.options)
                expected_action = scene.options[row["index"]] if row["index"] in (0, 1) else "INVALID"
                if row["action"] != expected_action:
                    raise ValueError("Recorded action mismatch")
                history.append(Fact(f"e{scene.index}-action", scene.at, "参与者实际行动：" + expected_action))
                reactions[f"{seed}/{world}/e{scene.index}-speech"] = scene.role
            if life["history"] != [asdict(fact) for fact in history]:
                raise ValueError("History contains non-original/additional/changed facts")
            for row in cast(list[dict[str, object]], life["probes"]):
                if row["situation"] != PROBES[cast(int, row["item"])][0]:
                    raise ValueError("Held-out situation changed")
                check_prompt(root, row, history, PROBES[cast(int, row["item"])][1])
            post = cast(list[dict[str, object]], life["ability_post"])
            if len(post) != 24:
                raise ValueError("Incomplete capability probes")
            for i, row in enumerate(post):
                if row["situation"] != ability_items()[i // 2][0]:
                    raise ValueError("Capability situation changed")
                check_prompt(root, row, history, ability_items()[i // 2][1])
    smoke = read(root / "smoke.json")
    check_prompt(root, smoke, [], ("5", "6"))
    return reactions


def manual_audit(root: Path, roles: dict[str, str], review_path: Path | None) -> dict[str, object]:
    if review_path is None:
        return {"status": "pending", "input_compliance": None, "reaction_diversity": None}
    review = read(review_path)
    if (review.get("reviewed_sha256") != data_binding(root) or review.get("reviewed_all_inputs") is not True
            or not isinstance(review.get("reviewer"), str) or not str(review["reviewer"]).strip()):
        raise ValueError("Complete event/prompt-cited local semantic review required")
    for name in ("persona_violations", "fact_contradictions"):
        if not isinstance(review.get(name), list):
            raise ValueError("Explicit violation lists required")
    coding = review.get("reaction_codes")
    if not isinstance(coding, dict) or set(coding) != set(roles):
        raise ValueError("Code all 384 original utterances; no replacement/subsample")
    categories: dict[str, set[str]] = {}
    for event, role in roles.items():
        codes = coding[event]
        if not isinstance(codes, list) or not codes or any(code not in REACTIONS for code in codes):
            raise ValueError("Event-cited reaction categories required")
        categories.setdefault(role, set()).update(codes)
    return {"status": "reviewed", "input_compliance": not review["persona_violations"] and not review["fact_contradictions"],
            "reaction_diversity": all(len(values) >= 3 for values in categories.values()),
            "categories_by_person_role": {role: sorted(values) for role, values in categories.items()},
            "reviewer": review["reviewer"]}


def stage1(root: Path, review_path: Path | None = None) -> dict[str, object]:
    root = root.resolve()
    if not root.is_relative_to(CACHE.resolve()) or root == CACHE.resolve():
        raise ValueError("Experiment cache only")
    verify_binding(root)
    verify_data(root)
    roles = input_checks(root)
    local = manual_audit(root, roles, review_path)
    probabilities, metrics = analyze_root(root), quality(root)
    test = cast(dict[str, object], probabilities["test"])
    # Keep the two-endpoint family: missing H1b is conservatively p=1, never n_tests=1.
    adjusted = adjust_primary({"H1a:C": cast(float, test["p_value"]), "H1b:C": 1.}, pilot=True)
    intermediate = cast(dict[str, list[int]], metrics["intermediate_seed_counts_by_world_and_probe"])
    variety = cast(dict[str, list[int]], metrics["action_variety_by_world_probe"])
    good_probes = [k for k in range(6) if all(intermediate[str(w)][k] >= 12
                   and len(variety.get(str((w, k)), [])) >= 2 for w in (0, 1))]
    ability_delta = cast(dict[str, object], metrics["ability_paired_world_average"])
    ability_labels = cast(dict[str, float], metrics["ability_pre_by_label_swap"])
    gates = {
        "input_compliance_local": local["input_compliance"],
        "action_interface": cast(float, metrics["valid_actions_fraction_including_ability"]) >= .95
                            and cast(float, metrics["high_label_mass_fraction_including_ability"]) >= .95,
        "dynamic_range": len(good_probes) >= 4,
        "H1a_statistics_conservative": cast(float, probabilities["mean_magnitude"]) >= .05
             and cast(list[float], probabilities["magnitude_ci95"])[0] > 0 and adjusted["H1a:C"] < .05,
        "capability": cast(float, metrics["ability_pre"]) >= .90 and min(ability_labels.values()) >= .80
             and cast(list[float], ability_delta["ci95"])[0] > -.05,
        "utterance_diversity": cast(float, metrics["unique_utterance_fraction"]) >= .80,
        "reaction_diversity_local": local["reaction_diversity"],
    }
    return {"phase": 1, "H1a": probabilities, "metrics": metrics, "local_audit": local, "gates": gates,
            "H1a_holm_with_unobserved_H1b_p1": adjusted["H1a:C"], "good_dynamic_probes": good_probes,
            "phase1_checks_satisfied": all(value is True for value in gates.values()),
            "H1b": "not_run", "judge_naturalness": "not_run", "full_pilot_acceptance": "pending_phase2",
            "source_sha256": data_binding(root), "judge_calls": 0, "paid_api_calls": 0, "gpu_calls": 0}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("run", type=Path)
    parser.add_argument("--review", type=Path)
    parser.add_argument("--output-name", default="phase1-summary.json")
    parser.add_argument("--review-template", action="store_true")
    args = parser.parse_args()
    if Path(args.output_name).name != args.output_name or not args.output_name.endswith(".json"):
        parser.error("Output must be a new JSON basename in this run")
    root = args.run.resolve()
    result = stage1(root, args.review)
    save(root / args.output_name, result)
    if args.review_template:
        roles = input_checks(root)
        save(root / "phase1-review-template.json", {"reviewer": "", "reviewed_all_inputs": False,
            "reviewed_sha256": data_binding(root), "persona_violations": [], "fact_contradictions": [],
            "reaction_codes": dict.fromkeys(roles, []), "allowed_reactions": REACTIONS})
    print("Phase-one offline checks saved; full pilot and independent naturalness remain pending phase two.")


if __name__ == "__main__":
    main()
