"""Independent parameter comparisons and a retained withdrawal counterexample."""
from __future__ import annotations

import json
import random
from pathlib import Path

from law_v2_model import Tendency


def property_checks() -> dict[str, object]:
    rng = random.Random(20261122)
    weight_bad = recency_bad = inertia_bad = complexity_bad = 0
    for _ in range(10000):
        p = rng.choice((0., 60., 600.))
        pi = rng.uniform(-.8, .8)
        # Choose two valid states with identical pi, c, P, h and different mass.
        mass = max(4 * pi / (1-pi), (2*p - (p+4)*(1+pi))/(1+pi), 0.) + 1000.
        a = pi * (p + 4 + mass) - p
        h = rng.uniform(.02, .8)
        w, sign = rng.uniform(.1, 5.), rng.choice((-1., 1.))
        small = Tendency(3., 1., prior=p, fixed_rho=1-h, a=a, b=mass)
        large_weight = Tendency(3., 1., prior=p, fixed_rho=1-h, a=a, b=mass)
        more_h = Tendency(3., 1., prior=p, fixed_rho=(1-h)/2, a=a, b=mass)
        more_mass = Tendency(3., 1., prior=p, fixed_rho=1-h,
                              a=pi*(p+4+2*mass)-p, b=2*mass)
        before = small.pi()
        q = small.add(w, sign)
        large_weight.add(2*w, sign)
        q_more_h = more_h.add(w, sign)
        mass_before = more_mass.pi()
        more_mass.add(w, sign)
        weight_bad += sign*(large_weight.pi()-small.pi()) <= 0
        recency_bad += q_more_h > q or not 1-h <= q <= 1
        inertia_bad += abs(more_mass.pi()-mass_before) > abs(small.pi()-before)
        plain, complex_ = Tendency(3., 0.), Tendency(3., 5.)
        for _ in range(10):
            signal = rng.uniform(.1, 3.)
            plain.add(signal, 1.)
            complex_.add(signal, 1.)
            complexity_bad += complex_.pi() > plain.pi()

    # Recompute adaptive q after reducing old opposition: the wrong design.
    path = Path(__file__).resolve().parents[1] / "law-lp1-reference" / "withdrawal-recomputed-q-counterexample.json"
    tape: list[list[float]] = json.loads(path.read_text(encoding="utf-8"))["tape"]
    before_t, wrong, frozen = Tendency(3., 1.), Tendency(3., 1.), Tendency(3., 1.)
    for index, (weight, sign) in enumerate(tape):
        q = before_t.add(weight, sign)
        revised_weight = weight * .1 if index == 0 else weight
        wrong.add(revised_weight, sign)
        frozen.a = q * frozen.a + revised_weight * sign
        frozen.b = q * frozen.b + revised_weight * abs(sign)
    frozen.validate()
    withdrawal_ok = frozen.pi() >= before_t.pi() and wrong.pi() < before_t.pi()
    return dict(check="LP2-LP4-LP6-LP7-and-frozen-q", seed=20261122, trials=10000,
                weight_monotonicity_violations=weight_bad, recency_violations=recency_bad,
                inertia_violations=inertia_bad, complexity_violations=complexity_bad,
                withdrawal_counterexample=dict(n0=3., kappa=1., prior=0., h0=.1, nh=10.,
                    changed_index=0, retained_weight_fraction=.1,
                    before=before_t.pi(), wrongly_recomputed_q=wrong.pi(), frozen_q=frozen.pi()),
                ok=not any((weight_bad, recency_bad, inertia_bad, complexity_bad)) and withdrawal_ok,
                scope="matched-state inequalities; not arbitrary different histories or full G")
