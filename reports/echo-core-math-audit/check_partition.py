"""ε-compatibility, minimal refinement and merge checks for χ partitions.

Spec references: F §6.1 (~_R, "等价类", "最小必要细分"), O §6.1 (~_E), O §6.3 (compatibility is
not transitive, split/merge rules, canonical ordering), D II §2.4 (lexicographic selection), G01.
"""

from __future__ import annotations

import itertools
import random
from collections.abc import Iterator

from common import Check, Result

SEED = 20261024


def compatible(a: float, b: float, eps: float) -> bool:
    return abs(a - b) <= eps


def set_partitions(items: list[float]) -> Iterator[list[list[float]]]:
    if not items:
        yield []
        return
    first, rest = items[0], items[1:]
    for part in set_partitions(rest):
        for i in range(len(part)):
            yield part[:i] + [[first] + part[i]] + part[i + 1:]
        yield [[first]] + part


def valid_block(block: list[float], eps: float) -> bool:
    return all(compatible(a, b, eps) for a, b in itertools.combinations(block, 2))


def check_g01_non_transitive() -> Result:
    h, eps = [0.0, 0.75, 1.5], 1.0
    ok = compatible(h[0], h[1], eps) and compatible(h[1], h[2], eps) and not compatible(h[0], h[2], eps)
    return Result(
        "G01-1",
        "F §6.1 ~_R / O §6.1 ~_E are not transitive (closed by O §6.3); G01",
        "counterexample",
        ok,
        "0~0.75 and 0.75~1.5 with ε=1, but |0−1.5| = 1.5 > ε: the 'equivalence classes' of F §6.1 "
        "are not defined by ~ itself",
    )


def check_minimal_refinement_not_unique() -> Result:
    h, eps = [0.0, 0.75, 1.5], 1.0
    feasible = [p for p in set_partitions(h) if all(valid_block(b, eps) for b in p)]
    k = min(len(p) for p in feasible)
    minimal = sorted(sorted(sorted(b) for b in p) for p in feasible if len(p) == k)
    # Canonical tie-break (O §6.3: ... 规范 ID 排序): pick the lexicographically smallest encoding.
    chosen = minimal[0]
    return Result(
        "G01-2",
        "F §6.1 '最小必要细分' is not unique (closed by O §6.3 / D II §2.4 fixed ordering)",
        "counterexample",
        k == 2 and len(minimal) == 2,
        f"merged class {{0,0.75,1.5}}, ε=1: {len(minimal)} distinct minimal refinements {minimal}; "
        f"a fixed canonical order selects {chosen}",
    )


def check_transitive_closure_drift() -> Result:
    # Taking the transitive closure of compatibility on a chain merges points arbitrarily far apart.
    rng = random.Random(SEED)
    eps = 1.0
    pts = sorted(rng.uniform(0, 50) for _ in range(400))
    chain_ok = all(compatible(a, b, eps) for a, b in zip(pts, pts[1:], strict=False))
    spread = pts[-1] - pts[0]
    return Result(
        "G01-3",
        "O §6.3 '不能对 A≈B、B≈C 做传递闭包'",
        "counterexample",
        chain_ok and spread > 40 * eps,
        f"400 random points on [0,50] with consecutive gaps ≤ ε: the closure of ~ is one class of "
        f"diameter {spread:.1f} = {spread / eps:.0f}ε",
    )


def check_merge_needs_whole_domain() -> Result:
    eps = 1.0
    left, right = [0.0, 0.9], [1.0, 1.9]
    each_ok = valid_block(left, eps) and valid_block(right, eps)
    merged_ok = valid_block(left + right, eps)
    return Result(
        "G01-4",
        "O §6.3 'merge 必须验证整个合并域的误差界'",
        "counterexample",
        each_ok and not merged_ok,
        "two ε-compatible leaves {0,0.9} and {1.0,1.9} (ε=1) whose union is not ε-compatible",
    )


CHECKS: list[Check] = [
    check_g01_non_transitive,
    check_minimal_refinement_not_unique,
    check_transitive_closure_drift,
    check_merge_needs_whole_domain,
]
