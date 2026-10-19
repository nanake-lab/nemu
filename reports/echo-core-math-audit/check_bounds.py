"""Drive bound η_x and J amplitude bounds (‖W‖∞→2, β split, ‖U‖op ≤ 1).

Spec references: O §5.2 (drive assembly and η_x), O §12 / §12.1 (W_l, β_x,l, β_r,l, β_l),
O §13.1 (h' = h + U a, ‖U‖op ≤ 1, ‖a‖₂ ≤ β), J06.
"""

from __future__ import annotations

import itertools
import math
import random

from common import Check, Result, inf_norm, two_norm

SEED = 20261020

Matrix = list[list[float]]


def matvec(m: Matrix, v: list[float]) -> list[float]:
    return [sum(a * b for a, b in zip(row, v, strict=True)) for row in m]


def row_abs_sum_norm(m: Matrix) -> float:
    """Induced ∞→∞ norm (max absolute row sum)."""
    return max(sum(abs(a) for a in row) for row in m)


def col_abs_sum_norm(m: Matrix) -> float:
    """Induced 1→1 norm (max absolute column sum)."""
    return max(sum(abs(m[i][j]) for i in range(len(m))) for j in range(len(m[0])))


def rand_matrix(rng: random.Random, r: int, c: int, scale: float = 1.0) -> Matrix:
    return [[rng.uniform(-scale, scale) for _ in range(c)] for _ in range(r)]


def norm_inf_to_2_exact(w: Matrix) -> float:
    """max over the cube vertices (the maximum of a convex function on [-1,1]^d)."""
    d = len(w[0])
    return max(two_norm(matvec(w, list(s))) for s in itertools.product((-1.0, 1.0), repeat=d))


def norm_inf_to_2_colsum(w: Matrix) -> float:
    """O §12.1 conservative bound: sum of column 2-norms."""
    d = len(w[0])
    return sum(two_norm([row[j] for row in w]) for j in range(d))


def check_drive_bound() -> Result:
    rng = random.Random(SEED)
    violations = 0
    for _ in range(5000):
        d = rng.randint(1, 6)
        b0 = rand_matrix(rng, d, d)
        dbs = [rand_matrix(rng, d, d, 0.3) for _ in range(rng.randint(0, 3))]
        res = [(rand_matrix(rng, d, d, 0.2), [rng.uniform(-0.2, 0.2) for _ in range(d)])
               for _ in range(rng.randint(0, 3))]
        eta = row_abs_sum_norm(b0) + sum(row_abs_sum_norm(m) for m in dbs) + sum(
            row_abs_sum_norm(m) + inf_norm(v) for m, v in res)
        b = [[b0[i][j] + sum(m[i][j] for m in dbs) for j in range(d)] for i in range(d)]
        for _ in range(20):
            xb = [rng.uniform(-1, 1) for _ in range(d)]
            u = [rng.choice((-1.0, 1.0)) if rng.random() < 0.5 else rng.uniform(-1, 1)
                 for _ in range(d)]
            g = [rng.uniform(0, 1) for _ in range(d)]
            bu = matvec(b, u)
            tot = [bu[i] + sum(matvec(m, xb)[i] + v[i] for m, v in res) for i in range(d)]
            delta = [g[i] * tot[i] for i in range(d)]
            violations += inf_norm(delta) > eta * (1 + 1e-12)
    # Cancellation: B0 = [[1]], ΔB = [[-1]] → net B = 0 but the admission bound counts 2.
    cancel_bound = row_abs_sum_norm([[1.0]]) + row_abs_sum_norm([[-1.0]])
    return Result(
        "DRV-1",
        "O §5.2 ‖δ‖∞ ≤ η_x with absolute-value sums; F02; J06 (no cancellation)",
        "property",
        violations == 0 and cancel_bound == 2.0,
        f"100000 random (B0, ΔB_j, residual, u, g, x̄): {violations} exceed the absolute-sum bound "
        f"‖B0‖∞+Σ‖ΔB_j‖∞+Σ(‖M_j‖∞+‖v_j‖∞); cancelling pair B0=1, ΔB=-1 is charged {cancel_bound:g}",
    )


def check_inf_to_2_bound() -> Result:
    rng = random.Random(SEED + 1)
    violations = 0
    worst_ratio = 1.0
    for _ in range(2000):
        r, d = rng.randint(1, 6), rng.randint(1, 8)
        w = rand_matrix(rng, r, d)
        exact = norm_inf_to_2_exact(w)
        bound = norm_inf_to_2_colsum(w)
        violations += exact > bound * (1 + 1e-12)
        worst_ratio = max(worst_ratio, bound / exact)
    d = 8
    ident = [[1.0 if i == j else 0.0 for j in range(d)] for i in range(d)]
    ratio_id = norm_inf_to_2_colsum(ident) / norm_inf_to_2_exact(ident)
    return Result(
        "JW-1",
        "O §12.1 ‖W_l‖∞→2 ≤ Σ_j ‖w_j‖₂ (conservative bound)",
        "property",
        violations == 0 and abs(ratio_id - math.sqrt(d)) < 1e-12,
        f"2000 random W (vertex enumeration): {violations} cases where exact > bound; worst "
        f"looseness {worst_ratio:.3f}; for W=I_{d} the bound is √d={math.sqrt(d):.3f} times exact",
    )


def check_alt_bound() -> Result:
    # Design suggestion: √d · sqrt(‖W‖₁‖W‖∞) also upper-bounds ‖W‖∞→2 (since ‖W‖₂² ≤ ‖W‖₁‖W‖∞).
    rng = random.Random(SEED + 2)
    violations = 0
    tighter = 0
    for _ in range(2000):
        r, d = rng.randint(1, 6), rng.randint(1, 8)
        w = rand_matrix(rng, r, d)
        exact = norm_inf_to_2_exact(w)
        alt = math.sqrt(d) * math.sqrt(col_abs_sum_norm(w) * row_abs_sum_norm(w))
        violations += exact > alt * (1 + 1e-12)
        tighter += min(alt, norm_inf_to_2_colsum(w)) < norm_inf_to_2_colsum(w)
    return Result(
        "JW-2",
        "design suggestion for O §12.1: min(Σ‖w_j‖₂, √d·√(‖W‖₁‖W‖∞))",
        "property",
        violations == 0,
        f"2000 random W: {violations} cases where exact > √d·√(‖W‖₁‖W‖∞); the min of both bounds "
        f"is strictly tighter than the column sum in {tighter} cases (I_d: exact √d)",
    )


def orthonormal_columns(rng: random.Random, n: int, r: int) -> Matrix:
    cols: list[list[float]] = []
    while len(cols) < r:
        v = [rng.gauss(0, 1) for _ in range(n)]
        for c in cols:
            p = sum(a * b for a, b in zip(v, c, strict=True))
            v = [a - p * b for a, b in zip(v, c, strict=True)]
        nv = two_norm(v)
        if nv > 1e-9:
            cols.append([a / nv for a in v])
    return [[cols[j][i] for j in range(r)] for i in range(n)]


def check_carrier_chain() -> Result:
    rng = random.Random(SEED + 3)
    violations = 0
    for _ in range(3000):
        d, r, dm = rng.randint(1, 6), rng.randint(1, 4), 8
        beta = rng.uniform(0.5, 3.0)
        share_x = rng.uniform(0.1, 0.9)
        beta_x = beta * share_x
        w = rand_matrix(rng, r, d)
        s = norm_inf_to_2_colsum(w)
        w = [[a * beta_x / s for a in row] for row in w]  # ‖W‖∞→2 ≤ colsum = β_x
        k = rng.randint(0, 3)
        betas_r = [beta * (1 - share_x) / k for _ in range(k)] if k else []
        u = orthonormal_columns(rng, dm, r)  # ‖U‖op = 1
        for _ in range(10):
            x = [rng.choice((-1.0, 1.0)) for _ in range(d)]
            a = matvec(w, x)
            for br in betas_r:
                e = [rng.gauss(0, 1) for _ in range(r)]
                ne = two_norm(e)
                a = [ai + br * ei / ne for ai, ei in zip(a, e, strict=True)]
            h_delta = matvec(u, a)
            violations += two_norm(a) > beta * (1 + 1e-12) or two_norm(h_delta) > beta * (1 + 1e-12)
    return Result(
        "JW-3",
        "O §12.1 Σβ_r,l+β_x,l ≤ β_l ⇒ ‖a_l‖₂ ≤ β_l; O §13.1 ‖U_l a_l‖₂ ≤ β_l",
        "property",
        violations == 0,
        f"30000 random (W, J_direct, U, x∈vertices of X): {violations} violations of ‖a‖₂ ≤ β "
        f"or ‖Ua‖₂ ≤ β",
    )


def check_reference_point() -> Result:
    # O §12: "可减去固定初始化参考点 ... 第一版取固定原点". With x_ref ≠ 0 the β_x bound breaks.
    w, beta_x, x_ref, x = 1.0, 1.0, -1.0, 1.0
    out = abs(w * (x - x_ref))
    return Result(
        "JW-4",
        "O §12 reference point (closed in v1 by '第一版取固定原点')",
        "counterexample",
        out > beta_x,
        f"d=1, W=[1], β_x=1, x_ref=-1, x=1: |W(x-x_ref)| = {out:g} > β_x",
    )


CHECKS: list[Check] = [
    check_drive_bound,
    check_inf_to_2_bound,
    check_alt_bound,
    check_carrier_chain,
    check_reference_point,
]
