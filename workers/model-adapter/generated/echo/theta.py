"""P5-36 semantic boundary. Decode, validate here, then check trusted bindings."""

from __future__ import annotations
import math
from typing import Any
from .runtime import Limits, digest, fail, normalize

NUMERICS_ID = "P5-25-v2-93-98-libm-0.2.16"
EXACT = 1 << 53
NAMES = {
    "ThetaEstimatorParameters",
    "ThetaRunningTime",
    "ThetaLastEvent",
    "ThetaEstimatorState",
    "ThetaSnapshotV2",
}


def scale(v: dict[str, Any]) -> list[float]:
    return [float(v[k]) for k in ("connection", "empathy", "curiosity", "achievement")]


def parameters(p: dict[str, Any]) -> None:
    fail(p["numerics_id"] == NUMERICS_ID, "theta numerics version")
    for k in (
        "baseline_weight",
        "variance_weight",
        "duration_weight",
        "change_prior_weight",
        "gain_min",
        "gain_max",
        "recovery_min",
        "recovery_max",
        "detector_shift_sigma",
        "detector_threshold",
    ):
        fail(0 < p[k] < EXACT, "theta positive exact parameter")
    for k in ("change_prior", "change_min"):
        fail(0 < p[k] < 1 and 1.0 - p[k] != 1.0, "theta representable retention")
    for k in ("gain_prior", "sigma_prior", "tau_prior_s"):
        fail(all(x > 0 for x in scale(p[k])), "theta positive prior")
    fail(
        p["gain_min"] <= p["gain_max"] and p["recovery_min"] <= p["recovery_max"],
        "theta ordered bounds",
    )
    fail(
        all(p["gain_min"] <= x <= p["gain_max"] for x in scale(p["gain_prior"])),
        "theta gain prior bounds",
    )
    fail(
        all(math.isfinite(x * x) and x * x > 0 for x in scale(p["sigma_prior"])),
        "theta prior variance",
    )
    fail(p["jump_notice"] > 0 and 0 < p["drift_run"] < EXACT, "theta calibration")


def running(t: dict[str, Any]) -> None:
    fail(
        t["event_ns"] >= t["origin_ns"]
        and t["stopped_ns"] <= t["event_ns"] - t["origin_ns"]
        and t["running_ns"] == t["event_ns"] - t["origin_ns"] - t["stopped_ns"],
        "theta running time identity",
    )


def state(s: dict[str, Any]) -> None:
    for k in (
        "reaction_sum",
        "square_sum",
        "variance",
        "duration_log_sum",
        "duration_mass",
        "duration_log_mean",
        "zero_duration_count",
        "run_sign",
        "run_start_ns",
        "detector_mean",
        "detector_positive",
        "detector_negative",
    ):
        fail(len(s[k]) == 4, "theta four channels")
    for k in ("drift_sign", "drift_count"):
        fail(len(s[k]) == 12, "theta twelve components")
    n, h = s["event_count"], s["h"]
    fail(n < EXACT, "theta exact event count")
    fail(0 < h < 1 and 1.0 - h != 1.0, "theta representable h")
    fail(
        all(x > 0 for x in scale(s["sigma"]) + scale(s["tau_s"]) + s["variance"]),
        "theta positive estimates",
    )
    for k in ("detector_segment_count", "detector_changes"):
        fail(s[k] <= n, "theta detector count")
    for k in ("zero_duration_count", "drift_count"):
        fail(all(x <= n for x in s[k]), "theta count exceeds events")
    has = "last_event" in s
    fail(
        has == (n > 0) and ("last_running_time" in s) == has,
        "theta last event presence",
    )
    if has:
        running(s["last_running_time"])
        fail(
            s["last_event"]["time_ns"] == s["last_running_time"]["event_ns"],
            "theta last time mismatch",
        )
        fail(
            s["detector_segment_count"] > 0 and s["weight"] > 0,
            "theta nonempty statistics",
        )
    else:
        fail(s["weight"] == 0, "theta genesis weight")
        for k in (
            "reaction_sum",
            "square_sum",
            "duration_log_sum",
            "duration_mass",
            "detector_mean",
            "detector_positive",
            "detector_negative",
        ):
            fail(all(x == 0 for x in s[k]), "theta genesis statistics")
    now = s["last_running_time"]["running_ns"] if has else 0
    for i in range(4):
        sign, start = s["run_sign"][i], s["run_start_ns"][i]
        fail(
            sign in (-1, 0, 1)
            and start <= now
            and (sign != 0 or start == 0)
            and (has or sign == 0),
            "theta open run",
        )
        fail(
            s["duration_mass"][i] != 0 or s["duration_log_sum"][i] == 0,
            "theta empty duration sum",
        )
    for sign, count in zip(s["drift_sign"], s["drift_count"], strict=True):
        fail(
            sign in (-1, 0, 1) and (sign == 0) == (count == 0), "theta drift sign/count"
        )


def check_theta_message(name: str, value: dict[str, Any], limits: Limits) -> None:
    if name not in NAMES:
        return
    v = normalize(name, value, limits)
    if name == "ThetaEstimatorParameters":
        parameters(v)
    elif name == "ThetaRunningTime":
        running(v)
    elif name == "ThetaEstimatorState":
        state(v)
    elif name == "ThetaSnapshotV2":
        p, s = v["parameters"], v["estimator"]
        parameters(p)
        state(s)
        fail(
            v["law_version"]["object_type"] == "echo.law-version"
            and v["law_version"]["resource_version"] == p["law_id"],
            "theta law binding",
        )
        fail(
            v["parameters_digest"].hex()
            == digest("ThetaEstimatorParameters", p, limits),
            "theta parameter digest",
        )
        fail(s["h"] >= p["change_min"], "theta h floor")
        if s["event_count"] == 0:
            fail(s["h"] == max(p["change_prior"], p["change_min"]), "theta initial h")
        for field, lo, hi in (
            ("gain", "gain_min", "gain_max"),
            ("recovery_per_s", "recovery_min", "recovery_max"),
        ):
            fail(
                all(p[lo] <= x <= p[hi] for x in scale(v["values"][field])),
                "theta value bounds",
            )
