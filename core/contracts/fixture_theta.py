"""P5-36 fixtures captured from actual C states, not a hand-built snapshot oracle."""

from __future__ import annotations

import copy
import json
from pathlib import Path
from typing import Any

NAMES = {
    "ThetaEstimatorParameters",
    "ThetaEstimatorState",
    "ThetaSnapshotV2",
    "ThetaLastEvent",
    "ThetaRunningTime",
}
STATES: list[dict[str, Any]] = json.loads(
    (Path(__file__).parent / "fixtures/theta-states.json").read_text(encoding="utf-8")
)


def sample_theta(name: str, seed: int) -> dict[str, Any]:
    snapshot = copy.deepcopy(STATES[seed % 3]["value"])
    state = snapshot["estimator"]
    if name == "ThetaSnapshotV2":
        return dict(snapshot)
    if name == "ThetaEstimatorParameters":
        result = dict(snapshot["parameters"])
        result["baseline_weight"] += seed
        return result
    if name == "ThetaEstimatorState":
        return dict(state)
    if name == "ThetaLastEvent":
        return dict(
            state.get(
                "last_event",
                dict(
                    schema_version=1,
                    ordinal="0",
                    time_ns="0",
                    event_id="synthetic-first",
                ),
            )
        )
    if name == "ThetaRunningTime":
        return dict(
            state.get(
                "last_running_time",
                dict(
                    schema_version=1,
                    origin_ns="0",
                    event_ns="0",
                    stopped_ns="0",
                    running_ns="0",
                ),
            )
        )
    raise ValueError(name)


def illegal_theta() -> list[dict[str, Any]]:
    from fixture_inputs import request

    rows: list[dict[str, Any]] = []

    def bad(name: str, path: str, value: Any, seed: int = 1) -> None:
        obj = sample_theta(name, seed)
        current = obj
        keys = path.split(".")
        for key in keys[:-1]:
            current = current[key]
        if value is None:
            del current[keys[-1]]
        else:
            current[keys[-1]] = value
        rows.append(dict(label="theta-" + path, request=request(name, obj)))

    for key, value in [
        ("numerics_id", "unknown"),
        ("baseline_weight", 0),
        ("variance_weight", -1),
        ("duration_weight", 2**53),
        ("change_prior", 1),
        ("change_min", 1e-300),
        ("gain_min", 4),
        ("recovery_min", 6),
        ("gain_prior.connection", 9),
        ("sigma_prior.connection", 1e-300),
        ("tau_prior_s.connection", 0),
        ("detector_threshold", 0),
        ("jump_notice", 0),
        ("drift_run", str(2**53)),
        ("drift_margin", -1),
    ]:
        bad("ThetaEstimatorParameters", key, value)
    for key, value in [
        ("origin_ns", str(2**64 - 1)),
        ("stopped_ns", str(2**64 - 1)),
        ("running_ns", "1"),
    ]:
        bad("ThetaRunningTime", key, value)
    for key, value in [("event_id", ""), ("ordinal", "-1"), ("time_ns", str(2**64))]:
        bad("ThetaLastEvent", key, value)
    for key, value in [
        ("weight", -1),
        ("reaction_sum", [0, 0, 0]),
        ("square_sum", [-1] * 4),
        ("sigma.connection", 0),
        ("variance", [0] * 4),
        ("duration_mass", [-1] * 4),
        ("zero_duration_count", ["999"] * 4),
        ("tau_s.connection", 0),
        ("run_sign", ["2"] * 4),
        ("run_start_ns", [str(2**64 - 1)] * 4),
        ("detector_segment_count", "999"),
        ("detector_changes", "999"),
        ("detector_mean", [2] * 4),
        ("detector_positive", [-1] * 4),
        ("detector_negative", [-1] * 4),
        ("event_count", str(2**53)),
        ("h", 0),
        ("last_event", None),
        ("last_running_time", None),
        ("last_event.time_ns", "1"),
        ("drift_sign", ["0"] * 11),
        ("drift_count", ["999"] * 12),
    ]:
        bad("ThetaEstimatorState", key, value)
    bad("ThetaEstimatorState", "weight", 1, seed=0)
    bad("ThetaEstimatorState", "reaction_sum", [1] * 4, seed=0)
    for key, value in [
        ("parameters_digest", "00" * 32),
        ("parameters.detector_threshold", 99),
        ("parameters.drift_run", "11"),
        ("law_version.resource_version", "wrong"),
        ("law_version.object_type", "echo.other"),
        ("estimator.h", 0.001),
        ("values.gain.connection", 4),
        ("values.recovery_per_s.connection", 6),
    ]:
        bad("ThetaSnapshotV2", key, value)
    bad("ThetaSnapshotV2", "estimator.h", 0.1, seed=0)
    return rows
