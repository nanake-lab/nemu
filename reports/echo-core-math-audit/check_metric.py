"""Reference θ distance d_Θ = w_p‖p−p′‖₁ + w_c·Hamming(c,c′) on canonical slot representations.

Spec references: F §2 ((Θ,d_Θ) metric space), F §8 (d_Θ ≤ κΣw), O §10.3 (reference metric,
raw-value comparison of discrete fields), S §18.
"""

from __future__ import annotations

import hashlib
import random

from common import Check, Result

SEED = 20261023

Rep = tuple[tuple[float, ...], tuple[str, ...]]


def d_theta(a: Rep, b: Rep, wp: float, wc: float) -> float:
    (pa, ca), (pb, cb) = a, b
    if len(pa) != len(pb) or len(ca) != len(cb):
        raise ValueError("canonical representations must have the same slot layout")
    numeric = sum(abs(x - y) for x, y in zip(pa, pb, strict=True))
    discrete = sum(x != y for x, y in zip(ca, cb, strict=True))
    return wp * numeric + wc * discrete


def rand_rep(rng: random.Random, m: int, k: int) -> Rep:
    p = tuple(rng.choice((0.0, rng.uniform(-1, 1))) for _ in range(m))
    c = tuple(rng.choice(("on", "off", "R1", "R2", "s:a", "s:b")) for _ in range(k))
    return p, c


def check_metric_axioms() -> Result:
    rng = random.Random(SEED)
    bad = 0
    for _ in range(20000):
        m, k = 6, 5
        wp, wc = rng.uniform(0.01, 3), rng.uniform(0.01, 3)
        a, b, c = rand_rep(rng, m, k), rand_rep(rng, m, k), rand_rep(rng, m, k)
        if rng.random() < 0.2:
            b = a  # exercise identity
        dab, dba, dbc, dac = (d_theta(a, b, wp, wc), d_theta(b, a, wp, wc),
                              d_theta(b, c, wp, wc), d_theta(a, c, wp, wc))
        bad += d_theta(a, a, wp, wc) != 0.0
        bad += abs(dab - dba) > 1e-12
        bad += dac > dab + dbc + 1e-12
        bad += (dab == 0.0) != (a == b)
        bad += dab < 0
    return Result(
        "DT-1",
        "O §10.3 d_Θ is a metric on canonical slot representations (w_p, w_c > 0)",
        "property",
        bad == 0,
        f"20000 random triples: {bad} violations of identity/symmetry/triangle/non-negativity",
    )


def check_zero_weight() -> Result:
    a: Rep = ((0.5, 0.0), ("on", "R1"))
    b: Rep = ((0.5, 0.0), ("off", "R1"))
    return Result(
        "DT-2",
        "O §10.3 premise w_c > 0 (w_c = 0 gives only a pseudometric)",
        "counterexample",
        a != b and d_theta(a, b, 1.0, 0.0) == 0.0,
        "same parameters, different enable flag: d_Θ = 0 when w_c = 0",
    )


def _short_hash(s: str) -> str:
    return hashlib.sha256(s.encode()).hexdigest()[:2]  # 8-bit digest to force a collision


def check_hash_comparison() -> Result:
    seen: dict[str, str] = {}
    pair: tuple[str, str] | None = None
    for i in range(10000):
        s = f"scope:{i}"
        h = _short_hash(s)
        if h in seen:
            pair = (seen[h], s)
            break
        seen[h] = s
    assert pair is not None
    a: Rep = ((0.1,), (pair[0],))
    b: Rep = ((0.1,), (pair[1],))
    ha: Rep = ((0.1,), (_short_hash(pair[0]),))
    hb: Rep = ((0.1,), (_short_hash(pair[1]),))
    return Result(
        "DT-3",
        "O §10.3 '离散字段使用规范原值比较而非把哈希碰撞当相等'",
        "counterexample",
        d_theta(a, b, 1.0, 1.0) > 0 and d_theta(ha, hb, 1.0, 1.0) == 0.0,
        f"scopes {pair[0]!r} and {pair[1]!r} collide under an 8-bit digest: hash comparison gives "
        "distance 0 for distinct θ, raw comparison gives 1",
    )


def check_rank_encoding() -> Result:
    # Design note: if composition order is stored as a per-slot rank, inserting one module ahead of
    # n existing modules changes n+1 discrete fields; a stable sort key changes only the new slot.
    n = 8
    slots = n + 1
    before_rank = tuple(str(i) for i in range(n)) + ("-",)
    after_rank = tuple(str(i + 1) for i in range(n)) + ("0",)
    before_key = tuple(f"k{i + 10}" for i in range(n)) + ("-",)
    after_key = tuple(f"k{i + 10}" for i in range(n)) + ("k05",)
    p = (0.0,) * slots
    h_rank = d_theta((p, before_rank), (p, after_rank), 1.0, 1.0)
    h_key = d_theta((p, before_key), (p, after_key), 1.0, 1.0)
    return Result(
        "DT-4",
        "design note for O §10.3 / S §18 '组合次序' encoding",
        "counterexample",
        h_rank == n + 1 and h_key == 1,
        f"insert one module ahead of {n}: Hamming part = {h_rank:g} with rank encoding, "
        f"{h_key:g} with a stable order key",
    )


CHECKS: list[Check] = [
    check_metric_axioms,
    check_zero_weight,
    check_hash_comparison,
    check_rank_encoding,
]
