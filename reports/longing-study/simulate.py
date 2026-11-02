"""Small deterministic illustrations, not Nemu implementation or acceptance tests.

Run from repository root:
  python reports/longing-study/simulate.py --out reports/longing-study/evidence/run-01
The destination must not exist. Python standard library only; no model or storage.
All units are abstract. No values below are proposed personality parameters.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import platform
from pathlib import Path


def clip(x: float) -> float:
    return max(-1.0, min(1.0, x))


def dump(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def run(out: Path) -> None:
    out.mkdir(parents=True, exist_ok=False)
    decay = math.exp(-0.2)
    steps = 120
    # Each number is a signed impulse in ONE illustrative coordinate, not a
    # validated measure of longing and not a newly added Nemu state variable.
    schedules = {
        "no_recall": [0.0] * steps,
        "repeated_same_cue": [0.1] * steps,
        "recall_then_stop": [0.1] * 30 + [0.0] * 90,
        "mixed_valence": [0.1 if n % 2 == 0 else -0.08 for n in range(steps)],
        "warming_memory": [-0.1] * steps,
        # No events in the offline interval; only an observation at resume.
        "offline_gap": [0.1] * 10 + [0.0] * 80 + [0.15] + [0.1] * 29,
    }
    tapes: dict[str, list[dict[str, object]]] = {}
    rows: list[dict[str, object]] = []
    curves: dict[str, list[float]] = {}
    summary: dict[str, object] = {}
    for name, impulses in schedules.items():
        x = 0.0
        last_event = 0
        event_tape: list[dict[str, object]] = []
        curve = [x]
        for n, impulse in enumerate(impulses, 1):
            if impulse != 0.0:
                dt = n - last_event
                before = math.exp(-0.2 * dt) * x
                x = clip(before + impulse)
                last_event = n
                event_tape.append({"time": n, "delta_hex": impulse.hex(), "x_hex": x.hex()})
                projected = x
            else:
                before = math.exp(-0.2 * (n - last_event)) * x
                projected = before
            curve.append(projected)
            rows.append({"scenario": name, "step": n, "before": before,
                         "impulse": impulse, "after": projected,
                         "event_committed": impulse != 0.0,
                         "online": name != "offline_gap" or not (11 <= n <= 90)})
        tapes[name] = event_tape
        curves[name] = curve
        summary[name] = {"final": curve[-1], "max_abs": max(map(abs, curve)),
                         "event_count": len(event_tape), "step_10": curve[10],
                         "step_30": curve[30], "step_90": curve[90], "step_91": curve[91]}

    dump(out / "input_tapes.json", tapes)
    # Structural tape replay uses committed bytes, never resamples a thought.
    saved = json.loads((out / "input_tapes.json").read_text(encoding="utf-8"))
    replay_matches = 0
    for tape in saved.values():
        x, last_time = 0.0, 0
        for event in tape:
            now = int(event["time"])
            x = clip(math.exp(-0.2 * (now - last_time)) * x + float.fromhex(event["delta_hex"]))
            assert x.hex() == event["x_hex"]
            replay_matches += 1
            last_time = now

    pairs: list[dict[str, object]] = []
    for mode in ("fixed_tape", "forbidden_local_feedback", "adaptive_content"):
        left, right = -0.01, 0.01
        excess = 0.0
        for n in range(1, steps + 1):
            lp, rp = decay * left, decay * right
            if mode == "fixed_tape":
                dl = dr = 0.06 if n % 3 else -0.04
            elif mode == "forbidden_local_feedback":
                dl, dr = math.tanh(5 * lp), math.tanh(5 * rp)
            else:
                # Every fixed-mark jump is nonexpansive, but the policy chooses
                # different marks from different states: conditioning matters.
                dl = 0.1 if lp >= 0 else -0.1
                dr = 0.1 if rp >= 0 else -0.1
            left, right = clip(lp + dl), clip(rp + dr)
            distance = abs(right - left)
            bound = 0.02 * math.exp(-0.2 * n)
            excess = max(excess, distance - bound)
            pairs.append({"mode": mode, "step": n, "left": left, "right": right,
                          "left_impulse": dl, "right_impulse": dr,
                          "distance": distance, "same_tape_bound": bound})
        summary[mode] = {"final_distance": abs(right - left), "max_bound_excess": excess,
                         "bound_applies": mode == "fixed_tape"}

    # Closed-loop mean-field sensitivity: exact solution of dy/dt=-(lambda-k)y+u.
    # Boundary clipping below is a diagnostic showing that bounded != interior.
    feedback: list[dict[str, object]] = []
    for gain in (0.0, 0.05, 0.15, 0.2, 0.3):
        for n in range(steps + 1):
            rate = 0.2 - gain
            raw = 0.03 * n if rate == 0.0 else 0.03 * (-math.expm1(-rate * n)) / rate
            feedback.append({"gain": gain, "time": n, "unclipped": raw,
                             "projected": clip(raw), "strict_small_gain": gain < 0.2})

    for filename, data in (("trajectories.csv", rows), ("paired_paths.csv", pairs),
                           ("feedback_sensitivity.csv", feedback)):
        with (out / filename).open("w", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=list(data[0]))
            writer.writeheader()
            writer.writerows(data)

    plateau = 0.1 / (1 - decay)
    closed = plateau * (1 - decay ** steps)
    actual = curves["repeated_same_cue"][-1]
    assert all(math.isfinite(float(row["after"])) and abs(float(row["after"])) <= 1 for row in rows)
    assert abs(actual - closed) < 1e-12
    assert not any(11 <= int(e["time"]) <= 90 for e in tapes["offline_gap"])
    summary["diagnostics"] = {
        "replay_bitwise_matches": replay_matches,
        "constant_impulse_limit": plateau,
        "closed_form_abs_error": abs(actual - closed),
        "offline_events_11_to_90": 0,
        "assertions": "finite/box, recurrence oracle, exact stored-tape replay, no offline events",
        "tolerance_note": "1e-12 is only a floating-point illustration check; no Nemu threshold changed",
        "scope": "No G/C/R/theta/IDL implementation; not evidence of AC2 or product acceptance",
    }
    dump(out / "summary.json", summary)

    colors = ["#6b7280", "#b45309", "#2563eb", "#9333ea", "#059669", "#dc2626"]
    svg = ['<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="570" viewBox="0 0 1000 570">',
           '<rect width="1000" height="570" fill="white"/>',
           '<text x="60" y="30" font-family="sans-serif" font-size="20">Synthetic recall impulses: persistence is conditional</text>']
    for value in (-1, -0.5, 0, 0.5, 1):
        y = 245 - 175 * value
        svg.append(f'<path d="M60 {y} H960" stroke="#ddd"/><text x="12" y="{y+5}" font-size="14">{value}</text>')
    for n in (0, 30, 60, 90, 120):
        px = 60 + 900 * n / steps
        svg.append(f'<text x="{px-10}" y="447" font-size="14">{n}</text>')
    for (name, curve), color in zip(curves.items(), colors):
        points = " ".join(f'{60+900*n/steps:.2f},{245-175*y:.2f}' for n, y in enumerate(curve))
        svg.append(f'<polyline points="{points}" fill="none" stroke="{color}" stroke-width="2"/>')
    for i, (name, color) in enumerate(zip(curves, colors)):
        xx, yy = 60 + (i % 3) * 315, 485 + (i // 3) * 30
        svg.append(f'<text x="{xx}" y="{yy}" fill="{color}" font-size="15" font-family="sans-serif">{name}</text>')
    svg.append('<text x="60" y="552" font-family="sans-serif" font-size="13">Abstract time / signed coordinate; samples are projections or post-event values, not human data.</text></svg>')
    (out / "trajectories.svg").write_text("\n".join(svg) + "\n", encoding="utf-8")
    files = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir())}
    dump(out / "manifest.json", {"python": platform.python_version(), "units": "abstract",
         "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
         "parameters": {"lambda": 0.2, "steps": steps, "dt": 1, "constant_impulse": 0.1,
                        "feedback_gains": [0, 0.05, 0.15, 0.2, 0.3], "feedback_input": 0.03},
         "seed": None, "selection": "all listed deterministic scenarios, no parameter fit or seed selection",
         "files_sha256": files})
    print(json.dumps({"status": "illustrations generated", "rows": len(rows), "pairs": len(pairs),
                      "feedback_rows": len(feedback), "diagnostics": summary["diagnostics"]}, ensure_ascii=False))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, required=True)
    run(parser.parse_args().out)
