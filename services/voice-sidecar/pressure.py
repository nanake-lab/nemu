"""User-approved 2026-10-21 retest criterion; does not reinterpret old evidence."""
from __future__ import annotations
from collections.abc import Sequence
import statistics


def slope_mib_hour(samples: Sequence[tuple[float, int, int]], start: float, end: float) -> float:
    selected = [(t, rss/2**20) for t, rss, _ in samples if start <= t <= end]
    if len(selected) < 2:
        raise ValueError("PRESSURE_WINDOW_HAS_TOO_FEW_SAMPLES")
    times, rss = zip(*selected, strict=True)
    return statistics.linear_regression(times, rss).slope * 3600


def assess(samples: Sequence[tuple[float, int, int]]) -> dict[str, object]:
    slope = slope_mib_hour(samples, 1200, 1800)
    unchanged = samples[0][2] == samples[-1][2]
    return {"policy": "user-approved-2026-10-21-retest-only", "allocated_first_bytes": samples[0][2],
            "allocated_last_bytes": samples[-1][2], "allocated_unchanged": unchanged,
            "rss_20_30_min_slope_mib_hour": slope, "rss_slope_limit_mib_hour": 12,
            "passed_30_min": unchanged and slope <= 12,
            "extension_required": not (unchanged and slope <= 12)}
