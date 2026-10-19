"""Natural-time projection Phi, R envelope q, and event-driven counterexamples.

Spec references: F §2.2, §5, §11, §16.13, §16.17; O §5.1, §15, §18 (bounded);
S §6 (structural fixture); O §5.1 contraction sentence (premise check).
"""

from __future__ import annotations

import math
import random

from common import Check, Result, inf_norm

SEED = 20261019


def phi_diag(x: list[float], b: list[float], lam: list[float], dt: float) -> list[float]:
    """Reference form written in the spec: b + exp(-lambda*dt) * (x - b)."""
    return [bi + math.exp(-li * dt) * (xi - bi) for xi, bi, li in zip(x, b, lam, strict=True)]


def phi_diag_alt(x: list[float], b: list[float], lam: list[float], dt: float) -> list[float]:
    """Algebraically equal form x + (1 - exp(-lambda*dt)) * (b - x)."""
    return [xi + (-math.expm1(-li * dt)) * (bi - xi) for xi, bi, li in zip(x, b, lam, strict=True)]


def _rand_box(rng: random.Random, d: int) -> list[float]:
    return [rng.uniform(-1.0, 1.0) for _ in range(d)]


def check_identity_exactness() -> Result:
    # F §2.2: "实现须满足 Φˣ₀=Id". The literal formula is not exactly Id in binary64.
    x, b, lam = [1e-17], [0.5], [1.0]
    lit = phi_diag(x, b, lam, 0.0)
    alt = phi_diag_alt(x, b, lam, 0.0)
    rng = random.Random(SEED)
    lit_fail = alt_fail = 0
    for _ in range(20000):
        d = 4
        xs = [rng.uniform(-1, 1) * 10 ** rng.randint(-20, 0) for _ in range(d)]
        bs = _rand_box(rng, d)
        ls = [rng.uniform(0.01, 5.0) for _ in range(d)]
        lit_fail += phi_diag(xs, bs, ls, 0.0) != xs
        alt_fail += phi_diag_alt(xs, bs, ls, 0.0) != xs
    ok = lit[0] != x[0] and alt[0] == x[0] and lit_fail > 0 and alt_fail == 0
    return Result(
        "PHI-0",
        "F §2.2 Φ₀=Id; D I §10.6",
        "counterexample",
        ok,
        f"x=1e-17,b=0.5: literal formula gives {lit[0]!r} (not x), alt form gives {alt[0]!r}; "
        f"random 20000 cases: literal non-identity {lit_fail}, alt non-identity {alt_fail}",
    )


def check_semigroup() -> Result:
    rng = random.Random(SEED + 1)
    worst = 0.0
    for _ in range(20000):
        d = 4
        x, b = _rand_box(rng, d), _rand_box(rng, d)
        lam = [rng.uniform(0.01, 5.0) for _ in range(d)]
        s, t = rng.uniform(0, 10), rng.uniform(0, 10)
        lhs = phi_diag(x, b, lam, s + t)
        rhs = phi_diag(phi_diag(x, b, lam, s), b, lam, t)
        worst = max(worst, inf_norm([p - q for p, q in zip(lhs, rhs, strict=True)]))
    return Result(
        "PHI-1",
        "F §2.2, §11 Φ_{s+t}=Φ_t∘Φ_s; O §5.1; F01; D I §10.6",
        "property",
        worst < 1e-14,
        f"max |Φ_(s+t) - Φ_t∘Φ_s|∞ over 20000 random cases = {worst:.3e} (tolerance 1e-14)",
    )


def check_forward_invariance() -> Result:
    rng = random.Random(SEED + 2)
    outside = 0
    worst = 0.0
    n = 200000
    for _ in range(n):
        d = 4
        x, b = _rand_box(rng, d), _rand_box(rng, d)
        # Push some coordinates to the box edge, where rounding matters most.
        x = [math.copysign(1.0, v) if rng.random() < 0.3 else v for v in x]
        lam = [rng.uniform(0.001, 20.0) for _ in range(d)]
        y = phi_diag(x, b, lam, rng.uniform(0, 5))
        m = inf_norm(y)
        if m > 1.0:
            outside += 1
            worst = max(worst, m - 1.0)
    return Result(
        "PHI-2",
        "O §5.1 'X 前向不变', O §18 有界性",
        "property",
        outside == 0,
        f"{n} random projections with x on/inside [-1,1]^4: {outside} left the box in binary64 "
        f"(max excess {worst:.3e}); the convexity proof is exact-arithmetic, this is empirical only",
    )


def check_contraction_no_events() -> Result:
    rng = random.Random(SEED + 3)
    violations = 0
    for _ in range(20000):
        d = 4
        x, y, b = _rand_box(rng, d), _rand_box(rng, d), _rand_box(rng, d)
        lam = [rng.uniform(0.05, 5.0) for _ in range(d)]
        t = rng.uniform(0, 10)
        lhs = inf_norm([p - q for p, q in zip(phi_diag(x, b, lam, t), phi_diag(y, b, lam, t), strict=True)])
        rhs = math.exp(-min(lam) * t) * inf_norm([p - q for p, q in zip(x, y, strict=True)])
        violations += lhs > rhs * (1 + 1e-12) + 1e-15
    return Result(
        "PHI-3",
        "O §5.1 ||δx(t)||∞ ≤ exp(-λ_min t)||δx(0)||∞ (no events); I05; F03",
        "property",
        violations == 0,
        f"violations of the contraction bound over 20000 event-free cases: {violations}",
    )


def check_nondiagonal_generator() -> Result:
    # F §2.2 allows a general stable generator Λ(θ); take a Jordan block.
    # exp(-Λt) = e^{-λt} [[1, -μt], [0, 1]] for Λ = [[λ, μ], [0, λ]].
    lam, mu, t = 1.0, -4.0, 0.25
    e = math.exp(-lam * t)
    x = (1.0, 1.0)  # inside the box, b = 0 (a fixed point)
    y = (e * (x[0] - mu * t * x[1]), e * x[1])
    left_box = max(abs(y[0]), abs(y[1])) > 1.0
    expanded = max(abs(y[0]), abs(y[1])) > max(abs(x[0]), abs(x[1]))
    return Result(
        "PHI-4",
        "F §2.2 general Λ(θ) (closed by O §5.1 Λ=diag, 0<λ_min)",
        "counterexample",
        left_box and expanded,
        f"Λ=[[1,-4],[0,1]] (eigenvalues 1,1), b=0, x=(1,1), t=0.25 → Φ(x)=({y[0]:.4f},{y[1]:.4f}); "
        f"leaves [-1,1]^2: {left_box}; ∞-norm distance to b grows 1 → {max(map(abs, y)):.4f}",
    )


def check_zero_rate() -> Result:
    # F §2.2 "非负对角恢复率" admits λ=0: no contraction at all.
    x, y, b = [0.5], [-0.5], [0.0]
    d0 = abs(x[0] - y[0])
    dt = abs(phi_diag(x, b, [0.0], 1e6)[0] - phi_diag(y, b, [0.0], 1e6)[0])
    return Result(
        "PHI-5",
        "F §2.2 '非负对角' (closed by O §5.1 λ_i ≥ λ_min > 0; F03 'λ=0 版本失败')",
        "counterexample",
        dt == d0,
        f"λ=0: |δx| stays {dt} after 1e6 s (initial {d0})",
    )


def check_piecewise_equals_oneshot() -> Result:
    # F §5/§16.13: filtered commits keep the anchor; maintenance-only segments do not change the
    # natural evolution. Simulate random segmentations with filtered records interleaved.
    rng = random.Random(SEED + 4)
    worst = 0.0
    for _ in range(5000):
        d = 4
        x, b = _rand_box(rng, d), _rand_box(rng, d)
        lam = [rng.uniform(0.05, 3.0) for _ in range(d)]
        anchor, state = 0.0, list(x)
        clock = 0.0
        for _step in range(rng.randint(1, 30)):
            clock += rng.uniform(0, 0.5)
            if rng.random() < 0.4:
                continue  # filtered record: no Φ, anchor unchanged
            state = phi_diag(state, b, lam, clock - anchor)  # valid/maintenance: advance once
            anchor = clock
        final = phi_diag(state, b, lam, clock - anchor)  # read-only projection at the end
        oneshot = phi_diag(x, b, lam, clock)
        worst = max(worst, inf_norm([p - q for p, q in zip(final, oneshot, strict=True)]))
    return Result(
        "PHI-6",
        "F §5 '不会丢失或重复计算经过时间'; F §16.13; O §15",
        "property",
        worst < 1e-13,
        f"max |piecewise − one-shot| over 5000 random segmentations = {worst:.3e}",
    )


def _event_map(xb: float, wx: float) -> float:
    """1-D F_in with g=1, B=1, u=tanh(W_x x̄), no residual, Project_X = clamp to [-1,1]."""
    return max(-1.0, min(1.0, xb + math.tanh(wx * xb)))


def check_event_driven_no_contraction() -> Result:
    # O §5.1: "同一 R/θ 和未来条件下，两个初始 x 的差满足 ||δx(t)||∞ ≤ exp(−λ_min t)||δx(0)||∞".
    # If "未来条件" includes identical future events, an expansive drive map refutes it.
    lam, dt, steps = 1.0, 0.1, 1000
    x, y = 0.01, -0.01
    for _ in range(steps):
        x = _event_map(phi_diag([x], [0.0], [lam], dt)[0], 5.0)
        y = _event_map(phi_diag([y], [0.0], [lam], dt)[0], 5.0)
    gap = abs(x - y)
    bound = math.exp(-lam * dt * steps) * 0.02
    # Same schedule with a non-expansive event map (W_x = -0.5: Lipschitz |1 - 0.5 sech²| ≤ 1).
    xa, ya = 0.01, -0.01
    for _ in range(steps):
        xa = _event_map(phi_diag([xa], [0.0], [lam], dt)[0], -0.5)
        ya = _event_map(phi_diag([ya], [0.0], [lam], dt)[0], -0.5)
    gap_ok = abs(xa - ya)
    return Result(
        "PHI-7",
        "O §5.1 contraction sentence; I05 (premise: no event drive or non-expansive event map)",
        "counterexample",
        gap > 1.9 and gap_ok <= bound * (1 + 1e-9) + 1e-300,
        f"λ=1/s, identical event every 0.1 s, u=tanh(5x̄): after 100 s |δx|={gap:.4f} vs claimed "
        f"bound {bound:.3e}; with non-expansive u=tanh(-0.5x̄): |δx|={gap_ok:.3e} ≤ bound",
    )


def q_project(q: float, anchor: float, gamma: float, t: float) -> tuple[float, float]:
    """R envelope projection carrying its anchor: returns (q(t), t)."""
    return q * math.exp(-gamma * (t - anchor)), t


def check_q_envelope() -> Result:
    # O §15 q(t+Δ)=q(t)exp(−γΔ); S §6 fixture γ=0.05/s, λ=1/s.
    rng = random.Random(SEED + 5)
    worst = 0.0
    for _ in range(20000):
        q0, g = rng.uniform(0, 1), rng.uniform(0.001, 1)
        t1, t2 = rng.uniform(0, 50), rng.uniform(0, 50)
        lhs = q_project(q0, 0.0, g, t1 + t2)[0]
        mid = q_project(q0, 0.0, g, t1)
        rhs = q_project(mid[0], mid[1], g, t1 + t2)[0]
        worst = max(worst, abs(lhs - rhs))
    ratio = 1.0 / 0.05
    return Result(
        "PHI-8",
        "O §15 q envelope semigroup; O §15 λ_min/γ_max ≥ ρ > 1; S §6 fixture",
        "property",
        worst < 1e-14 and ratio > 1,
        f"max semigroup error {worst:.3e}; structural fixture λ_min/γ_max = 1/0.05 = {ratio:g} (>1)",
    )


def check_double_projection_hazard() -> Result:
    # O §15 passes R_bar = Phi_R(Δ;R) *and* t to CompileAuthorizedEffects; O §3.2 / S §12 say Compile
    # itself applies the q time envelope. If R_bar does not carry its new anchor, decay is applied twice.
    q0, gamma, anchor, t = 1.0, 0.05, 0.0, 10.0
    once = q_project(q0, anchor, gamma, t)[0]
    r_bar_with_anchor = q_project(q0, anchor, gamma, t)
    idempotent = q_project(*r_bar_with_anchor, gamma, t)[0]
    r_bar_without_anchor = (q_project(q0, anchor, gamma, t)[0], anchor)
    twice = q_project(*r_bar_without_anchor, gamma, t)[0]
    ok = abs(idempotent - once) < 1e-15 and abs(twice - math.exp(-2 * gamma * t)) < 1e-15
    return Result(
        "PHI-9",
        "O §3.2 / §12 / §15 / §10.1 and S §12 CompileAuthorizedEffects signatures; I04, I21",
        "counterexample",
        ok,
        f"q0=1, γ=0.05/s, Δ=10 s: correct q={once:.4f}; Compile on projected R̄ that keeps its "
        f"new anchor → {idempotent:.4f}; on R̄ that keeps the old anchor → {twice:.4f} (double decay)",
    )


CHECKS: list[Check] = [
    check_identity_exactness,
    check_semigroup,
    check_forward_invariance,
    check_contraction_no_events,
    check_nondiagonal_generator,
    check_zero_rate,
    check_piecewise_equals_oneshot,
    check_event_driven_no_contraction,
    check_q_envelope,
    check_double_projection_hazard,
]
