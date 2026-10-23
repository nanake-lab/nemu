"""Numerical checks for the reference law in the Echo Core v2.0 draft (read-only).

Spec: docs/echo-core/echo-core-v2.0-first-principles.md, §5.2 (reference form), §5.3
(corollaries 1–7), §6 (fast state) and §8 (θ reference form). Standard library only;
fixed seeds; no network, database or stored state.

Usage (repository root):  python reports/echo-core-v2-draft/check_law.py
Exit code 0 when every check passes.
"""

from __future__ import annotations

import itertools
import math
import platform
import random
import sys
from collections.abc import Callable
from dataclasses import dataclass

SEED = 20261019
TOL = 1e-9


@dataclass(frozen=True)
class Result:
    """Outcome of one check.

    kind:
      "property" -- a claimed property; ok means no violation was found in the random trials.
      "scenario" -- a claimed behaviour of the reference form; ok means it was reproduced.
    """

    check_id: str
    spec_ref: str
    kind: str
    ok: bool
    detail: str


Check = Callable[[], Result]


@dataclass
class Tendency:
    """Running sums of one tendency under the §5.2 reference form, in commit order."""

    n0: float
    kappa: float
    prior: float = 0.0  # innate prior evidence (attachment); never multiplied by ρ
    h0: float = 0.1
    nh: float = 10.0
    fixed_rho: float | None = None
    a: float = 0.0  # Σ ω·W·S
    b: float = 0.0  # Σ ω·W·|S|
    groups: int = 0
    flips: int = 0
    last_sign: int = 0

    def pi(self) -> float:
        return (self.prior + self.a) / (self.n0 + self.kappa + self.prior + self.b)

    def rho(self) -> float:
        if self.fixed_rho is not None:
            return self.fixed_rho
        return 1.0 - (self.h0 * self.nh + self.flips) / (self.nh + self.groups)

    def add(self, w: float, s: float) -> float:
        """Multiply old evidence by the current ρ, then add one group. Returns the ρ used."""
        rho = self.rho()
        self.a = rho * self.a + w * s
        self.b = rho * self.b + w * abs(s)
        sign = (s > 0) - (s < 0)
        if sign != 0:
            if self.last_sign != 0 and sign != self.last_sign:
                self.flips += 1
            self.last_sign = sign
        self.groups += 1
        return rho

    def consistent(self) -> float:  # E
        return (self.b + self.a) / 2

    def contrary(self) -> float:  # C
        return (self.b - self.a) / 2


def check_threshold() -> Result:
    rng = random.Random(SEED)
    bad = states = 0
    for _ in range(2000):
        rho = 1.0 if rng.random() < 0.5 else rng.uniform(0.8, 1.0)
        t = Tendency(n0=rng.uniform(0.5, 6), kappa=rng.uniform(0, 4), fixed_rho=rho)
        for _ in range(rng.randint(1, 40)):
            t.add(rng.uniform(0, 3), 1.0)
            gap = t.consistent() - (t.n0 + t.kappa)
            if abs(gap) < TOL:
                continue
            states += 1
            bad += (t.pi() > 0.5) != (gap > 0)
    return Result(
        "LAW-1",
        "§5.3(1) only consistent evidence: π > ½ ⟺ E > n₀ + κ",
        "property",
        bad == 0,
        f"{states} states with random n₀, κ, ρ and weights: {bad} violations",
    )


def check_one_shot_and_occam() -> Result:
    rng = random.Random(SEED + 1)
    bad = 0
    for _ in range(20000):
        n0, kappa, w = rng.uniform(0.5, 6), rng.uniform(0, 4), rng.uniform(0, 12)
        if abs(w - n0 - kappa) < TOL:
            continue
        t = Tendency(n0=n0, kappa=kappa)
        t.add(w, 1.0)
        bad += (t.pi() > 0.5) != (w > n0 + kappa)
    needed = []
    for kappa in range(6):
        t = Tendency(n0=3.0, kappa=float(kappa), fixed_rho=1.0)
        n = 0
        while t.pi() <= 0.5:
            t.add(1.0, 1.0)
            n += 1
        needed.append(n)
    monotone = all(x < y for x, y in itertools.pairwise(needed))
    return Result(
        "LAW-2",
        "§5.3(2), LP6: one group with W > n₀ + κ suffices; larger κ needs more ordinary groups",
        "property",
        bad == 0 and monotone,
        f"20000 single groups: {bad} violations; ordinary groups (W = 1, n₀ = 3) needed for "
        f"κ = 0…5: {needed}",
    )


def check_reversal() -> Result:
    rng = random.Random(SEED + 2)
    bad = states = 0
    for _ in range(2000):
        t = Tendency(n0=rng.uniform(0.5, 6), kappa=rng.uniform(0, 4), h0=rng.uniform(0.02, 0.3))
        for _ in range(rng.randint(1, 60)):
            t.add(rng.uniform(0, 3), rng.uniform(-1, 1))
            e, c = t.consistent(), t.contrary()
            margin = c - (e - t.n0 - t.kappa) / 3
            if abs(margin) < TOL or abs(t.pi() - 0.5) < TOL:
                continue
            states += 1
            bad += (t.pi() < 0.5) != (margin > 0)
    return Result(
        "LAW-3",
        "§5.3(3) π < ½ ⟺ C > (E − n₀ − κ)/3",
        "property",
        bad == 0,
        f"{states} states with mixed-sign groups and learned ρ: {bad} violations",
    )


def check_single_group_bound() -> Result:
    rng = random.Random(SEED + 3)
    bad = 0
    worst = 0.0
    for _ in range(30000):
        t = Tendency(
            n0=rng.uniform(0.5, 6),
            kappa=rng.uniform(0, 4),
            prior=rng.choice((0.0, rng.uniform(0, 50))),
            fixed_rho=1.0,
        )
        for _ in range(rng.randint(0, 30)):
            t.add(rng.uniform(0, 3), rng.uniform(-1, 1))
        before = t.pi()
        big_b = t.n0 + t.kappa + t.prior + t.b
        w, s = rng.uniform(0, 10), rng.uniform(-1, 1)
        t.add(w, s)
        bound = 2 * w * abs(s) / (big_b + w * abs(s))
        delta = abs(t.pi() - before)
        bad += delta > bound + 1e-12
        if bound > 0:
            worst = max(worst, delta / bound)
    return Result(
        "LAW-4",
        "§5.3(4), LP7: one group changes π by at most 2w/(B + w) (no recency weight)",
        "property",
        bad == 0,
        f"30000 random states: {bad} violations; largest |Δπ|/bound = {worst:.4f}",
    )


def _run_core(
    events: list[tuple[float, float, float]], x0: list[float], lam: list[float]
) -> tuple[float, list[float]]:
    """A minimal core loop: x follows Φ_x and drives; the law reads only (w, s). Returns (π, x)."""
    t = Tendency(n0=3.0, kappa=1.0)
    x = list(x0)
    for dt, w, s in events:
        x = [xi * math.exp(-li * dt) for xi, li in zip(x, lam, strict=True)]  # x_rest = 0 here
        x = [min(1.0, max(-1.0, xi + 0.3 * s)) for xi in x]
        t.add(w, s)
    return t.pi(), x


def check_time_and_state_independence() -> Result:
    rng = random.Random(SEED + 4)
    bad = 0
    for _ in range(500):
        base = [(rng.uniform(0, 10), rng.uniform(0, 3), rng.uniform(-1, 1)) for _ in range(30)]
        stretched = [(dt * rng.uniform(1, 1e6), w, s) for dt, w, s in base]
        lam = [rng.uniform(0.01, 2) for _ in range(4)]
        pi_a, _ = _run_core(base, [rng.uniform(-1, 1) for _ in range(4)], lam)
        pi_b, _ = _run_core(stretched, [rng.uniform(-1, 1) for _ in range(4)], lam)
        bad += pi_a != pi_b
    return Result(
        "LAW-5",
        "§5.3(5), LP5, LP8: π depends on neither elapsed time nor x",
        "property",
        bad == 0,
        f"500 event sequences with gaps stretched up to 10⁶× and different x₀: "
        f"{bad} bitwise differences in π",
    )


def check_memory_bound() -> Result:
    rng = random.Random(SEED + 5)
    bad = 0
    for _ in range(3000):
        rho, w_max = rng.uniform(0.5, 0.999), rng.uniform(0.5, 5)
        t = Tendency(n0=3.0, kappa=1.0, fixed_rho=rho)
        cap = w_max / (1 - rho)
        for _ in range(rng.randint(1, 400)):
            t.add(rng.uniform(0, w_max), rng.choice((1.0, -1.0, rng.uniform(-1, 1))))
            bad += t.consistent() > cap * (1 + 1e-12) or t.contrary() > cap * (1 + 1e-12)
    return Result(
        "LAW-6",
        "§5.3(6) constant ρ: E, C ≤ w_max/(1 − ρ)",
        "property",
        bad == 0,
        f"3000 random sequences: {bad} violations",
    )


def check_recompute() -> Result:
    rng = random.Random(SEED + 6)
    max_rel = 0.0
    bitwise_bad = 0
    closed_differs = 0
    for _ in range(1000):
        groups = [(rng.uniform(0, 3), rng.uniform(-1, 1)) for _ in range(rng.randint(1, 200))]
        t = Tendency(n0=3.0, kappa=1.0)
        rhos = [t.add(w, s) for w, s in groups]
        replay = Tendency(n0=3.0, kappa=1.0)
        for w, s in groups:
            replay.add(w, s)
        bitwise_bad += (replay.a, replay.b) != (t.a, t.b)
        closed = 0.0
        for i, (w, s) in enumerate(groups):
            closed += math.prod(rhos[i + 1 :]) * w * s
        closed_differs += closed != t.a
        max_rel = max(max_rel, abs(closed - t.a) / max(1.0, abs(t.a)))
    return Result(
        "LAW-7",
        "§5.2 calculation order, §2.2: recomputation replays the recurrence bitwise",
        "property",
        bitwise_bad == 0 and max_rel < 1e-12,
        f"1000 sequences: replay differs bitwise in {bitwise_bad}; the mathematically equal "
        f"closed form Σ_c Πρ·W·S differs in the last bits in {closed_differs} "
        f"(max relative {max_rel:.1e}), so the recurrence is the canonical definition",
    )


def shares(pis: list[float], supports: list[float]) -> list[float]:
    """§5.2 share: split one group among tendencies on the same side, ∝ (1 + π)/2."""
    out = [0.0] * len(pis)
    for side in (1, -1):
        idx = [i for i, s in enumerate(supports) if (s > 0) - (s < 0) == side]
        total = sum((1 + pis[i]) / 2 for i in idx)
        for i in idx:
            out[i] = (1 + pis[i]) / 2 / total
    return out


def check_explaining_away() -> Result:
    rng = random.Random(SEED + 7)
    bad = 0
    for _ in range(20000):
        k = rng.randint(1, 8)
        pis = [rng.uniform(-0.999, 0.999) for _ in range(k)]
        sup = [rng.choice((0.0, rng.uniform(-1, 1))) for _ in range(k)]
        sh = shares(pis, sup)
        w_base = rng.uniform(0, 5)
        for side in (1, -1):
            idx = [i for i, s in enumerate(sup) if (s > 0) - (s < 0) == side]
            if idx:
                bad += abs(sum(sh[i] for i in idx) - 1) > 1e-12
                bad += sum(w_base * sh[i] * abs(sup[i]) for i in idx) > w_base * (1 + 1e-12)
        bad += any(not 0 <= v <= 1 for v in sh)
        bad += any(sh[i] != 0 for i in range(k) if sup[i] == 0)
    return Result(
        "LAW-8",
        "§5.2 shares, LP2, I25: each side's shares sum to 1; credit never exceeds the group",
        "property",
        bad == 0,
        f"20000 random groups with 1–8 matching tendencies: {bad} violations",
    )


def group_support(
    reactions: list[list[float]],
    base: list[float],
    d: list[float],
    sigma: list[float],
    r_tilde: float,
    mu: float,
    w_max: float,
) -> tuple[float, float]:
    """§5.2 support S and strength of one evidence group for one tendency."""
    dev = [[ai - bi for ai, bi in zip(a, base, strict=True)] for a in reactions]
    mean = [sum(col) / len(dev) for col in zip(*dev, strict=True)]
    p = sum(m * di for m, di in zip(mean, d, strict=True))
    s = mu * min(1.0, max(-1.0, 2 * p / r_tilde - 1))
    sigma_k = math.sqrt(sum((di * si) ** 2 for di, si in zip(d, sigma, strict=True)))
    peak = max(abs(sum(v * di for v, di in zip(row, d, strict=True))) for row in dev)
    return s, min(w_max, max(r_tilde, peak) / sigma_k)


def check_grouping() -> Result:
    rng = random.Random(SEED + 8)
    bad = 0
    for _ in range(5000):
        base = [rng.uniform(-0.5, 0.5) for _ in range(4)]
        raw = [rng.uniform(-1, 1) for _ in range(4)]
        norm = math.sqrt(sum(v * v for v in raw)) or 1.0
        d = [v / norm for v in raw]
        sigma = [rng.uniform(0.05, 0.5) for _ in range(4)]
        group = [[rng.uniform(-1, 1) for _ in range(4)] for _ in range(rng.randint(1, 6))]
        args = (base, d, sigma, rng.uniform(0.05, 1), rng.uniform(0, 1), 8.0)
        once = group_support(group, *args)
        repeated = group_support(group * rng.randint(2, 9), *args)
        bad += abs(once[0] - repeated[0]) > 1e-12 or abs(once[1] - repeated[1]) > 1e-12
    base, d, sigma = [0.0] * 4, [1.0, 0.0, 0.0, 0.0], [0.2] * 4
    silent = group_support([[0.0] * 4], base, d, sigma, 0.2, 1.0, 8.0)
    striking = group_support([[1.0, 0, 0, 0]], base, d, sigma, 0.2, 1.0, 8.0)
    ok = bad == 0 and silent == (-1.0, 1.0) and striking == (1.0, 5.0)
    return Result(
        "LAW-9",
        "§5.2 support and strength, LP3, LP6: repeats add nothing; a missing reaction counts",
        "property",
        ok,
        f"5000 groups repeated 2–9 times: {bad} changes in (S, strength); no reaction where "
        f"0.2 was predicted gives (S, strength) = {silent}; a 5σ reaction gives {striking}",
    )


def _flip_after(t: Tendency, pattern: Callable[[int], float], limit: int) -> int | None:
    for n in range(1, limit + 1):
        t.add(1.0, pattern(n))
        if t.pi() < 0.5:
            return n
    return None


def check_attachment() -> Result:
    def history(prior: float) -> Tendency:
        t = Tendency(n0=3.0, kappa=1.0, prior=prior)
        for _ in range(200):
            t.add(1.0, 1.0)
        return t

    limit = 100000
    n_ordinary = _flip_after(history(0.0), lambda n: -1.0, limit)
    n_att = _flip_after(history(60.0), lambda n: -1.0, limit)
    n_att_big = _flip_after(history(600.0), lambda n: -1.0, limit)
    t = history(60.0)
    flipped_at = _flip_after(t, lambda n: -1.0, limit)
    exact = (
        flipped_at is not None and t.contrary() > (t.prior + t.consistent() - t.n0 - t.kappa) / 3
    )
    alternating = history(60.0)
    lowest = 1.0
    for n in range(5000):
        alternating.add(1.0, -1.0 if n % 2 == 0 else 1.0)
        lowest = min(lowest, alternating.pi())
    ok = (
        n_ordinary is not None
        and n_att is not None
        and n_att_big is not None
        and n_ordinary < n_att < n_att_big
        and exact
        and lowest > 0.5
    )
    return Result(
        "LAW-10",
        "§5.3(7) attachment: hard to move, but finite; alternating contrary evidence cannot",
        "scenario",
        ok,
        f"after 200 supporting groups, consistent contrary groups needed to bring π below ½: "
        f"ordinary {n_ordinary}, A_att = 60: {n_att}, A_att = 600: {n_att_big}; the flip "
        f"matches C > (A + E − n₀ − κ)/3: {exact}; 5000 alternating groups keep "
        f"π_att ≥ {lowest:.3f}",
    )


def check_fast_state() -> Result:
    rng = random.Random(SEED + 9)
    bad = 0
    for _ in range(3000):
        dim = rng.randint(1, 8)
        lam_min = rng.uniform(0.01, 1)
        lam = [lam_min] + [rng.uniform(lam_min, 3) for _ in range(dim - 1)]
        rest = [rng.uniform(-1, 1) for _ in range(dim)]
        x = [rng.uniform(-1, 1) for _ in range(dim)]
        y = [rng.uniform(-1, 1) for _ in range(dim)]
        gap0 = max(abs(a - b) for a, b in zip(x, y, strict=True))
        elapsed = 0.0
        for _ in range(rng.randint(1, 30)):
            dt = rng.expovariate(1.0)
            elapsed += dt
            decay = [math.exp(-li * dt) for li in lam]
            x = [r + e * (v - r) for v, r, e in zip(x, rest, decay, strict=True)]
            y = [r + e * (v - r) for v, r, e in zip(y, rest, decay, strict=True)]
            drive = [rng.uniform(-3, 3) for _ in range(dim)]  # D(e) does not read x̄
            x = [min(1.0, max(-1.0, v + dv)) for v, dv in zip(x, drive, strict=True)]
            y = [min(1.0, max(-1.0, v + dv)) for v, dv in zip(y, drive, strict=True)]
            gap = max(abs(a - b) for a, b in zip(x, y, strict=True))
            bad += gap > math.exp(-lam_min * elapsed) * gap0 * (1 + 1e-12) + 1e-15
    return Result(
        "FX-1",
        "§6, ADR 0008 §1: with D(e) independent of x̄, ‖δx(t)‖∞ ≤ e^{−λ_min t}‖δx(0)‖∞",
        "property",
        bad == 0,
        f"3000 trajectory pairs with shared events: {bad} violations",
    )


def check_theta() -> Result:
    rng = random.Random(SEED + 10)
    bad = 0
    g_min, g_max, g0, sigma0 = 0.2, 3.0, 1.0, 0.3
    for _ in range(500):
        n_b, n_s, rho = rng.uniform(1, 20), rng.uniform(1, 20), rng.uniform(0.9, 1.0)
        b0 = rng.uniform(-1, 1)
        sw = sa = sq = 0.0
        b, var = b0, sigma0**2
        for _ in range(rng.randint(1, 300)):
            a = rng.uniform(-1, 1)
            exogenous = rng.random() < 0.6
            before = (b, var)
            if exogenous:
                sq = rho * sq + (a - b) ** 2
                sw, sa = rho * sw + 1, rho * sa + a
                b = (n_b * b0 + sa) / (n_b + sw)
                var = (n_s * sigma0**2 + sq) / (n_s + sw)
            else:
                bad += (b, var) != before
            g = min(g_max, max(g_min, g0 * sigma0 / math.sqrt(var)))
            bad += not -1 <= b <= 1 or not g_min <= g <= g_max
    return Result(
        "THETA-1",
        "§8 reference form: b stays in [−1, 1], g in range, events she caused leave θ unchanged",
        "property",
        bad == 0,
        f"500 random exogenous/endogenous streams (constant ρ_θ): {bad} violations",
    )


def _feedback_world(
    k: float,
    mu: float,
    n_b: float,
    rho: float,
    steps: int,
    burst: tuple[int, int] | None,
    seed: int,
) -> list[float]:
    """§8 reference baseline b when others' replies follow her shown mood with slope k.

    Her shown mood rests at b between events (§6), so a reply's reaction is
    a = clip(mu + k·b + noise). During a burst the world is coldly hostile regardless of her.
    """
    rng = random.Random(seed)
    b0, sw, sa, b = 0.0, 0.0, 0.0, 0.0
    traj = []
    for t in range(steps):
        if burst is not None and burst[0] <= t < burst[1]:
            a = -0.8 + 0.2 * rng.gauss(0, 1)
        else:
            a = mu + k * b + 0.2 * rng.gauss(0, 1)
        a = min(1.0, max(-1.0, a))
        sw, sa = rho * sw + 1, rho * sa + a
        b = (n_b * b0 + sa) / (n_b + sw)
        traj.append(b)
    return traj


def check_feedback_loop() -> Result:
    rho, mu, steps, burst = 0.99, -0.1, 6000, (1000, 1300)
    cap = 1 / (1 - rho)  # S, the ceiling of Σω_e
    rows, ok = [], True
    for n_b, ks in ((10.0, (0.0, 0.3, 0.6, 0.9, 1.2)), (cap, (1.0, 1.5))):
        share = cap / (n_b + cap)
        for k in ks:
            k_eff = k * share
            base = (cap * mu) / (n_b + cap)  # fixed point without the loop (b0 = 0)
            predicted = base / (1 - k_eff) if k_eff < 1 else -math.inf
            plain = _feedback_world(k, mu, n_b, rho, steps, None, SEED + 11)
            hit = _feedback_world(k, mu, n_b, rho, steps, burst, SEED + 11)
            final, after = sum(plain[-500:]) / 500, sum(hit[-500:]) / 500
            interior = predicted > -0.95
            if interior:  # converges to the predicted interior point and recovers after the burst
                ok &= abs(final - predicted) < 0.03 and abs(after - final) < 0.03
            else:  # amplified past the boundary: b is pinned near it
                ok &= final < -0.7
            rows.append(
                f"n_b={n_b:g} k={k:g}: k_eff={k_eff:.2f}, b={final:+.2f}, after burst {after:+.2f}"
            )
    return Result(
        "THETA-2",
        "§8 reply loop: b amplifies the world's bias by 1/(1 − k_eff); "
        "anchoring n_b ≈ 1/ĥ_θ halves k_eff",
        "scenario",
        ok,
        "world bias −0.1, ρ_θ = 0.99; " + "; ".join(rows),
    )


CHECKS: list[Check] = [
    check_threshold,
    check_one_shot_and_occam,
    check_reversal,
    check_single_group_bound,
    check_time_and_state_independence,
    check_memory_bound,
    check_recompute,
    check_explaining_away,
    check_grouping,
    check_attachment,
    check_fast_state,
    check_theta,
    check_feedback_loop,
]


def main() -> int:
    print(f"python {platform.python_version()} on {platform.system()} {platform.machine()}")
    results = [check() for check in CHECKS]
    for r in results:
        print(f"[{'OK  ' if r.ok else 'FAIL'}] {r.check_id:<8} {r.kind:<9} {r.spec_ref}")
        print(f"          {r.detail}")
    failed = [r.check_id for r in results if not r.ok]
    print(f"\n{len(results)} checks; failed: {failed or 'none'}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
