"""Precommitted norm-relative informational study; exact archived direction, no fitting."""
from __future__ import annotations

from datetime import datetime, timezone
import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess

from direction_probe import NEUTRAL_PROMPTS
from instruct_runtime import InstructRuntime, MODEL, PLAN, REVISION, ROOT, coefficient_for_ratio


def read_committed_plan() -> tuple[dict[str, object], str]:
    relative = PLAN.relative_to(ROOT).as_posix()
    committed = subprocess.check_output(["git", "show", f"HEAD:{relative}"], cwd=ROOT)
    if committed.decode("utf-8").replace("\r\n", "\n") != PLAN.read_text(encoding="utf-8"):
        raise ValueError("Dose plan must be committed before GPU execution")
    plan = json.loads(committed)
    if plan["model"] != MODEL or plan["revision"] != REVISION:
        raise ValueError("Plan model binding mismatch")
    sha = subprocess.check_output(["git", "log", "-1", "--format=%H", "--", relative],
                                  cwd=ROOT, text=True).strip()
    return plan, sha


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    plan, commit = read_committed_plan()
    args.output.mkdir(parents=True, exist_ok=False)
    result: dict[str, object] = dict(
        status="started", model=dict(id=MODEL, revision=REVISION),
        timestamp=datetime.now(timezone.utc).isoformat(), dose_plan=plan, plan_commit=commit,
        plan_sha256=hashlib.sha256(PLAN.read_bytes()).hexdigest(),
        generation_prompts=NEUTRAL_PROMPTS, samples=[],
        limitation="Informational only; fixed archived direction and precommitted ratios; not P5-31",
    )
    norm_samples: list[dict[str, object]] = []
    samples: list[dict[str, object]] = []
    result["samples"] = samples
    try:
        runtime = InstructRuntime()
        result.update(u_sha256=runtime.u_sha256, native_rms_count=runtime.native_rms_count,
                      layer_index_zero_based=runtime.index, rank=8,
                      eos_token_ids=sorted(runtime.stopping))
        wrapper = runtime.install(beta=1)
        medians: dict[str, dict[str, float]] = {}
        for language, prompt in NEUTRAL_PROMPTS.items():
            norms: dict[str, list[float]] = dict(prefill=[], decode=[])
            for seed in (731, 732, 733):
                generation = runtime.generate([dict(role="user", content=prompt)], 0, seed,
                                              observe=True)
                positions: list[dict[str, object]] = []
                for index, observation in enumerate(wrapper.observations):
                    vector = observation["before"]
                    if not isinstance(vector, list) or len(vector) != 1:
                        raise ValueError("Expected one before-residual vector")
                    norm = math.sqrt(math.fsum(float(v) ** 2 for v in vector[0]))
                    phase = str(observation["phase"])
                    norms[phase].append(norm)
                    positions.append(dict(position=index, phase=phase, h_l2=norm))
                norm_samples.append(dict(language=language, seed=seed, positions=positions,
                                         generation=generation.to_dict()))
            medians[language] = {phase: statistics.median(values) for phase, values in norms.items()}
        reference = statistics.median([value for phases in medians.values() for value in phases.values()])
        ratios = [-0.4, -0.2, -0.1, -0.05, 0.0, 0.05, 0.1, 0.2, 0.4]
        if plan["ratios"] != ratios or plan["seeds"] != [731, 732, 733]:
            raise ValueError("Plan ratios or seeds changed; use a new registered protocol")
        coefficients = [coefficient_for_ratio(ratio, reference, runtime.direction_norm) for ratio in ratios]
        norms_result = dict(
            status="measured-informational-only", model=dict(id=MODEL, revision=REVISION),
            plan_commit=commit, plan_sha256=result["plan_sha256"], u_sha256=runtime.u_sha256,
            h_l2_medians=medians, reference_norm=reference, reference_rule=plan["reference_rule"],
            direction_norm=runtime.direction_norm, ratios=ratios, coefficients=coefficients,
            samples=norm_samples,
        )
        # Persist the measured fixed scale before any nonzero-dose generation.
        (args.output / "norms.json").write_text(json.dumps(norms_result, ensure_ascii=False, indent=2),
                                                encoding="utf-8")
        beta = max(abs(c) for c in coefficients)
        runtime.install(beta)
        result.update(reference_norm=reference, h_l2_medians=medians, ratios=ratios,
                      coefficients=coefficients, beta=beta, seeds=[731, 732, 733],
                      budget="48 tokens / 25 seconds; temperature .7; matched seeds; fresh KV")
        for language, prompt in NEUTRAL_PROMPTS.items():
            for seed in (731, 732, 733):
                for ratio, coefficient in zip(ratios, coefficients, strict=True):
                    generation = runtime.generate([dict(role="user", content=prompt)], coefficient, seed)
                    samples.append(dict(language=language, seed=seed, ratio=ratio,
                                        coefficient=coefficient, **generation.to_dict()))
                    print(f"{language} seed={seed} ratio={ratio} stop={generation.stop}", flush=True)
        result["status"] = "measured-informational-only"
    except Exception as error:
        result.update(status="failed", error_type=type(error).__name__, error=str(error),
                      norm_samples=norm_samples)
        raise
    finally:
        with (args.output / "direction.json").open("x", encoding="utf-8") as stream:
            json.dump(result, stream, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
