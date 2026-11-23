"""Zero injection only. Preserve invalid outputs and enforce the preregistered gate."""
from __future__ import annotations

import argparse
from collections import defaultdict
import json
from pathlib import Path
import statistics
import time
import traceback
from typing import Any

from calibration_fixtures import LAYOUTS, cases, export, prompt
from probe import ENGINE, MODEL, MODEL_HASH, HERE, REPO, Probe, probabilities, save, sha


def summarize(rows: list[dict[str, Any]]) -> dict[str, Any]:
    def mean(key: str, subset: list[dict[str, Any]]) -> float:
        return statistics.mean(float(r[key]) for r in subset)

    overall = {k: mean(k, rows) for k in ("choice_valid", "final_valid", "correct", "choice_mass_ok", "final_mass_ok")}
    families = {f: mean("correct", [r for r in rows if r["family"] == f]) for f in sorted({r["family"] for r in rows})}
    layouts = {str(layout): mean("correct", [r for r in rows if r["layout"] == layout]) for layout in range(12)}
    answer_orders = {str(v): mean("correct", [r for r in rows if r["reverse"] == v]) for v in (False, True)}
    case_metrics = {}
    middle_families: set[str] = set()
    missing_with_middle = 0
    for case in cases():
        sub = [r for r in rows if r["case"] == case.id]
        ps = [float(r["p_relevant_conditional"]) for r in sub]
        middle = sum(0.05 < p < 0.95 for p in ps)
        if case.missing and middle:
            missing_with_middle += 1
            middle_families.add(case.family)
        counts: dict[str, int] = defaultdict(int)
        for r in sub:
            counts[r["actual_action"]] += 1
        case_metrics[case.id] = {"family": case.family, "missing": case.missing, "p_mean": statistics.mean(ps),
                                 "p_min": min(ps), "p_max": max(ps), "p_range": max(ps)-min(ps),
                                 "middle_layouts": middle, "actions": dict(counts), "oracle_correct": mean("correct", sub)}
    groups = {kind: {str(v): mean("p_relevant_conditional", [r for r in rows if r["missing"] and r[kind] == v])
                     for v in sorted({r[kind] for r in rows})} for kind in ("relevant_label", "relevant_position")}
    checks = {"format": overall["choice_valid"] >= .95 and overall["final_valid"] >= .95,
              "mass": overall["choice_mass_ok"] >= .95 and overall["final_mass_ok"] >= .95,
              "overall_capability": overall["correct"] >= .90,
              "family_capability": min(families.values()) >= .80,
              "layout_capability": min(layouts.values()) >= .80,
              "answer_mapping_capability": min(answer_orders.values()) >= .80,
              "dynamic_range": missing_with_middle >= 4 and len(middle_families) >= 2}
    return {"overall": overall, "family_accuracy": families, "layout_accuracy": layouts,
            "answer_mapping_accuracy": answer_orders, "per_case": case_metrics, "missing_groups": groups,
            "missing_with_middle": missing_with_middle, "middle_families": sorted(middle_families),
            "checks": checks, "usable": all(checks.values()), "n_scenarios": 24, "n_rows": len(rows)}


def run(output: Path, executable: Path, version: str) -> None:
    output.mkdir(parents=True, exist_ok=False)
    for folder in ("prompts", "vectors", "activations", "checks"):
        (output / folder).mkdir()
    started = time.monotonic()
    try:
        if sha(MODEL) != MODEL_HASH:
            raise RuntimeError("Pinned model mismatch")
        inputs = list(HERE.glob("*.py")) + [HERE / "choice_probe.cpp", HERE / "DEVLOG.md",
                    REPO / "spikes/carrier-engine/native_probe.hpp", executable]
        save(output / "binding.json", {"version": version, "model_sha256": MODEL_HASH,
             "files": {str(p): sha(p) for p in inputs}, "dlls": {p.name: sha(p) for p in ENGINE.glob("*.dll")}})
        save(output / "fixtures.json", export())
        rows: list[dict[str, Any]] = []
        with (output / "native-stderr.log").open("w", encoding="utf-8") as err:
            probe = Probe(executable, output, err)
            try:
                save(output / "label-token-ids.json", probe.labels)
                with (output / "episodes.jsonl").open("w", encoding="utf-8") as out:
                    for case in cases():
                        for layout, (mapping, reverse) in enumerate(LAYOUTS):
                            first = probe.request(prompt(case, layout, version, False))
                            last = probe.request(prompt(case, layout, version, True), nlabels=2)
                            choice_id, final_id = first["unconstrained_argmax"], last["unconstrained_argmax"]
                            valid_choice = choice_id in probe.labels
                            valid_final = final_id in probe.labels[:2]
                            selected = probe.labels.index(choice_id) if isinstance(choice_id, int) and valid_choice else -1
                            expected = int(not case.truth) ^ int(reverse)
                            relevant_index = mapping.index("relevant")
                            record = {"case": case.id, "family": case.family, "missing": case.missing,
                                      "layout": layout, "reverse": reverse, "version": version,
                                      "relevant_label": chr(65 + relevant_index),
                                      "relevant_position": 2-relevant_index if reverse else relevant_index,
                                      "choice_valid": valid_choice, "final_valid": valid_final,
                                      "choice_mass_ok": float(str(first["allowed_mass"])) >= .95,
                                      "final_mass_ok": float(str(last["allowed_mass"])) >= .95,
                                      "actual_action": mapping[selected] if valid_choice else "INVALID",
                                      "correct": valid_final and final_id == probe.labels[expected],
                                      "expected_label": chr(65 + expected), "final_token": final_id,
                                      "p_relevant_conditional": probabilities(first)[relevant_index],
                                      "first": first, "last": last}
                            rows.append(record)
                            out.write(json.dumps(record, allow_nan=False) + "\n")
                            out.flush()
                        print(f"{version}: {case.id} complete", flush=True)
            finally:
                probe.close()
        save(output / "summary.json", summarize(rows))
        save(output / "exit.json", {"status": "completed", "wall_seconds": time.monotonic()-started})
    except BaseException:
        (output / "failure.txt").write_text(traceback.format_exc(), encoding="utf-8")
        save(output / "exit.json", {"status": "failed", "wall_seconds": time.monotonic()-started})
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--executable", type=Path, required=True)
    parser.add_argument("--version", choices=("I0", "I1"), required=True)
    parser.add_argument("--quiet-window-confirmed", action="store_true")
    args = parser.parse_args()
    if not args.quiet_window_confirmed:
        raise SystemExit("User quiet window required")
    run(args.output.resolve(), args.executable.resolve(), args.version)
