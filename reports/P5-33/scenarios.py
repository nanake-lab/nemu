"""Paired random tapes; absence policies are assumptions to compare, not decisions."""
from __future__ import annotations

import random
from dataclasses import dataclass


@dataclass(frozen=True)
class Scenario:
    name: str
    warm: int
    probabilities: tuple[float, ...]
    return_day: int | None = None


def life_scenarios() -> list[Scenario]:
    cases = [Scenario(f"stop_{n}", n, tuple([1.0]*n + [0.0]*500)) for n in (30, 365, 1095)]
    cases += [Scenario(f"gap_{n}", 365, tuple([1.0]*365 + [0.0]*n + [1.0]*60), 365+n)
              for n in (3, 7)]
    fade = tuple([1.0]*90 + [1.0-(t+1)/180.0 for t in range(180)] + [0.0]*180)
    cases.append(Scenario("fade", 90, fade))
    return cases


def e2_probability(world: str, t: int) -> float:
    if world == "stable":
        return 0.7
    if world == "abrupt":
        return 0.7 if t < 320 else 0.3
    if world == "gradual":
        return 0.7 - 0.4*min(1.0, max(0.0, (t-192)/192))
    return 0.7 if (t//128) % 2 == 0 else 0.3


def uniforms(seed: int, length: int) -> list[float]:
    rng = random.Random(seed)
    return [rng.random() for _ in range(length)]


def observations(policy: str, tape: list[int]) -> list[list[float]]:
    output: list[list[float]] = []
    missing = False
    for y in tape:
        if policy == "a":
            output.append([2.0*y-1.0])
        elif policy == "b":
            # Illustration only: one interval summary at return, then the present event.
            output.append(([-1.0] if missing else []) + [1.0] if y else [])
            missing = not bool(y)
        else:
            output.append([1.0] if y else [])
    return output
