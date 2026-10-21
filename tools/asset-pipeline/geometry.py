"""Pure deterministic tail geometry; no Blender or third-party imports."""
from __future__ import annotations
import math
from typing import cast
from errors import fail
Vec3 = tuple[float, float, float]

def chain_points(points: list[list[float]], length: float, segments: int) -> list[Vec3]:
    """Resample a polyline to equal arc-length bones, preserving its configured shape."""
    lengths = [math.dist(a, b) for a, b in zip(points, points[1:])]
    total = sum(lengths)
    if total <= 0 or any(size <= 0 for size in lengths):
        fail("tail", "tail.curve", "nonzero consecutive segments", points)
    if segments < 8 or length <= 0:
        fail("tail", "tail.boneCount/lengthM", ">=8 / >0", (segments, length))
    result: list[Vec3] = []
    for index in range(segments + 1):
        target = total * index / segments
        start = 0.0
        for a, b, size in zip(points, points[1:], lengths):
            if target <= start + size + 1e-12:
                t = min(1.0, (target - start) / size)
                result.append(cast(Vec3, tuple((a[k] + (b[k] - a[k]) * t - points[0][k]) * length / total for k in range(3))))
                break
            start += size
    return result


def radius_at(curve: list[list[float]], t: float) -> float:
    if curve[0][0] != 0 or curve[-1][0] != 1 or any(a[0] >= b[0] for a, b in zip(curve, curve[1:])):
        fail("tail", "tail.radiusCurve", "increasing t from 0 to 1", curve)
    for a, b in zip(curve, curve[1:]):
        if a[0] <= t <= b[0]:
            return float(a[1] + (b[1] - a[1]) * (t - a[0]) / (b[0] - a[0]))
    fail("tail", "radius t", "0..1", t)
    raise AssertionError("unreachable")


def weights_at(t: float, count: int) -> list[tuple[int, float]]:
    x = max(0.0, min(count - 1.0, t * count - 0.5))
    lower = math.floor(x)
    upper = min(lower + 1, count - 1)
    return [(lower, 1.0)] if lower == upper else [(lower, 1 - (x - lower)), (upper, x - lower)]
