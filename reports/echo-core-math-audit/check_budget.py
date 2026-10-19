"""θ path budgets: S §18 P/A prefix index, F §8 windows vs O §15 any-interval bound, root budgets.

Spec references: F §8 (Σ_j w_j(e) ≤ b(e), d_Θ ≤ κΣw, ε_θ, B_θ(W)); O §11, §15
(path_Θ([s,t]) ≤ v_Θ(t−s)+ε_Θ, τ_θ ≥ (D−ε_Θ)/v_Θ), O §18 (finite-root path); S §18 (P/A index); X07.
Exact rational arithmetic is used so that agreement is not a floating-point accident.
"""

from __future__ import annotations

import random
from fractions import Fraction

from common import Check, Result

SEED = 20261022


def brute_force_ok(times: list[int], dists: list[Fraction], v: Fraction, eps: Fraction) -> bool:
    """path over every closed interval [t_a, t_b] spanned by updates a..b is ≤ v(t_b-t_a)+ε."""
    n = len(times)
    for a in range(n):
        acc = Fraction(0)
        for b in range(a, n):
            acc += dists[b]
            if acc > v * (times[b] - times[a]) + eps:
                return False
    return True


class PrefixIndex:
    """S §18: P = committed path, A = max_j (v·t_j − P_(j−1)); empty sequence initialises A."""

    def __init__(self, v: Fraction, eps: Fraction) -> None:
        self.v, self.eps = v, eps
        self.p = Fraction(0)
        self.a: Fraction | None = None

    def try_commit(self, t: int, d: Fraction) -> bool:
        cand = self.v * t - self.p
        a_new = cand if self.a is None else max(self.a, cand)
        if self.p + d - self.v * t + a_new <= self.eps:
            self.p += d
            self.a = a_new
            return True
        return False


def check_prefix_index() -> Result:
    rng = random.Random(SEED)
    mismatches = 0
    decisions = 0
    accepted_total = 0
    for _ in range(3000):
        v = Fraction(rng.randint(1, 5), rng.randint(1, 5))
        eps = Fraction(rng.randint(1, 10), rng.randint(1, 4))
        idx = PrefixIndex(v, eps)
        t = rng.randint(0, 5)
        acc_t: list[int] = []
        acc_d: list[Fraction] = []
        for _step in range(rng.randint(1, 25)):
            t += rng.choice((0, 0, 1, 2, 3, 7))  # equal process times allowed
            d = Fraction(rng.randint(0, 40), 10) * eps / 2
            expect = brute_force_ok(acc_t + [t], acc_d + [d], v, eps)
            got = idx.try_commit(t, d)
            decisions += 1
            mismatches += expect != got
            if got:
                acc_t.append(t)
                acc_d.append(d)
                accepted_total += 1
                if d > eps:
                    mismatches += 1  # single step must be implied
        if not brute_force_ok(acc_t, acc_d, v, eps):
            mismatches += 1
    return Result(
        "PA-1",
        "S §18 P/A prefix index ≡ O §15 any-interval bound (incl. single step and equal times)",
        "property",
        mismatches == 0,
        f"{decisions} candidate decisions ({accepted_total} accepted) over 3000 random sequences: "
        f"{mismatches} disagreements with brute-force O(n²) interval check",
    )


def check_fixed_window_burst() -> Result:
    # F §8 "Σ_{j∈W} dΘ ≤ Bθ(W)" read as calendar-aligned windows [kW, (k+1)W).
    width, budget, eps_step = 10_000, Fraction(1), Fraction(1)  # times in ms
    updates = [(9_990, Fraction(1)), (10_000, Fraction(1))]
    per_window: dict[int, Fraction] = {}
    fixed_ok = True
    for t, d in updates:
        per_window[t // width] = per_window.get(t // width, Fraction(0)) + d
        fixed_ok &= d <= eps_step and per_window[t // width] <= budget
    rolling_ok = sum(d for _, d in updates) <= budget  # both lie in one length-W window
    v = budget / width
    idx = PrefixIndex(v, eps_step)
    any_ok = all(idx.try_commit(t, d) for t, d in updates)
    return Result(
        "WIN-1",
        "F §8 window budget (aligned reading) vs O §15 any-interval bound; X07",
        "counterexample",
        fixed_ok and not rolling_ok and not any_ok,
        "W=10 s, B=1, ε=1: d=1 at 9.99 s and d=1 at 10.00 s pass aligned windows "
        f"({fixed_ok}); path 2 in 10 ms. Rolling-window reading passes: {rolling_ok}; "
        f"O §15 bound with v=B/W passes: {any_ok}",
    )


def check_finite_root_path() -> Result:
    rng = random.Random(SEED + 1)
    violations = 0
    for _ in range(3000):
        roots = rng.randint(1, 6)
        b = [Fraction(rng.randint(1, 20), 4) for _ in range(roots)]
        remaining = list(b)
        kappa = Fraction(rng.randint(1, 9), 3)
        path = Fraction(0)
        for _u in range(rng.randint(1, 30)):
            w = [Fraction(0)] * roots
            for e in range(roots):
                if remaining[e] > 0 and rng.random() < 0.5:
                    w[e] = remaining[e] * Fraction(rng.randint(1, 10), 10)
            if sum(w) == 0:
                continue  # rejected candidates consume nothing
            d = kappa * sum(w) * Fraction(rng.randint(0, 10), 10)  # d_Θ ≤ κ Σ_e w(e)
            remaining = [r - x for r, x in zip(remaining, w, strict=True)]
            path += d
        violations += path > kappa * sum(b) or any(r < 0 for r in remaining)
    return Result(
        "BUD-1",
        "F §8 / O §18 finite-root path ≤ κ Σ_e b(e) (O §11: κ_Θ, b_Θ(e))",
        "property",
        violations == 0,
        f"3000 random ledgers: {violations} with cumulative ordinary path > κΣb(e) or negative balance",
    )


def check_tau_theta_bound() -> Result:
    # O §15: to accumulate path D > ε_Θ the elapsed time must be ≥ (D − ε_Θ)/v_Θ.
    rng = random.Random(SEED + 2)
    violations = 0
    for _ in range(2000):
        v, eps = Fraction(rng.randint(1, 4), 3), Fraction(rng.randint(1, 6), 2)
        idx = PrefixIndex(v, eps)
        ts: list[int] = []
        ds: list[Fraction] = []
        t = 0
        for _s in range(30):
            t += rng.randint(0, 4)
            d = Fraction(rng.randint(0, 20), 10) * eps
            if idx.try_commit(t, d):
                ts.append(t)
                ds.append(d)
        for a in range(len(ts)):
            acc = Fraction(0)
            for b_ in range(a, len(ts)):
                acc += ds[b_]
                if acc > eps and ts[b_] - ts[a] < (acc - eps) / v:
                    violations += 1
    return Result(
        "TAU-1",
        "O §15 τ_θ lower bound (D−ε_Θ)/v_Θ for accepted sequences",
        "property",
        violations == 0,
        f"2000 accepted sequences: {violations} intervals reached path D>ε_Θ faster than (D−ε_Θ)/v_Θ",
    )


CHECKS: list[Check] = [
    check_prefix_index,
    check_fixed_window_burst,
    check_finite_root_path,
    check_tau_theta_bound,
]
