"""Read-only probes for PR61 bb1c269; run in the detached review worktree."""
import copy
import json
import math
import random
import sys
from fractions import Fraction as F
from pathlib import Path

sys.path.insert(0, str(Path.cwd() / 'reports/echo-core-v2-draft'))
from law_v2_events import Engine, Event
from law_v2_model import NumericFault, Tendency, bits


def exact_step(p, c, e, opposition, h, w, sign):
    v, a = e + opposition, e - opposition
    before = (p + a) / (p + c + v)
    d = p - before * (p + c)
    u = w * (1 - sign * before)
    discount = h if sign * d >= 0 else h * u / (u + 2 * h * abs(d))
    q = 1 - discount
    e2 = q * e + (w if sign > 0 else 0)
    c2 = q * opposition + (w if sign < 0 else 0)
    after = (p + e2 - c2) / (p + c + e2 + c2)
    delta = (discount * d + sign * u) / (p + c + q * v + w)
    assert after - before == delta
    assert sign * delta >= u / (2 * (p + c + q * v + w)) > 0
    # Independently check ADR11.3 local open-cap construction when pressured.
    limit = max(v, w) + F(1, 100)
    if q * v + w >= limit:
        hi = (limit - w) / v
        lo = max(0, 1 - u / abs(d)) if sign * d < 0 else 0
        assert hi > lo >= 0
        capq = (hi + lo) / 2
        assert 0 < capq < 1 and capq * v + w < limit
        assert u + sign * (1 - capq) * d > 0


rng = random.Random(610023)
for _ in range(2000):
    p, c = F(rng.randrange(0, 601)), F(rng.randrange(1, 81), 10)
    e, o = F(rng.randrange(0, 1001), 10), F(rng.randrange(0, 1001), 10)
    h, w, s = F(rng.randrange(0, 100), 100), F(rng.randrange(1, 101), 100), rng.choice((-1, 1))
    exact_step(p, c, e, o, h, w, s)
    z = F(rng.randrange(1, 100), 100)
    before = (p + e - o) / (p + c + e + o)
    assert (p + z * e - o) / (p + c + z * e + o) <= before
    assert before <= (p + e - z * o) / (p + c + e + z * o)
    exact_step(p, c, z * e, z * o, h, w, s)

result = {'exact_rational': {'seed': 610023, 'states': 2000, 'append_cf_cases': 4000,
                            'withdrawal_cases': 4000, 'failures': 0}}
e = Engine()
e.append('old', 10., (('a', 1.),))
cf, _ = e.apply(Event(revision_group='old', competitor='b', competitor_support=1.))
result['previous_counterfactual_witness'] = {'cf': cf, 'after': {k: t.pi() for k, t in e.tendencies.items()}}
assert cf['b'] == 0. and e.tendencies['b'].pi() > 0
fault_rows = []
for entry in ('append', 'apply'):
    e = Engine()
    before = e.snapshot()
    try:
        if entry == 'append':
            e.append('tiny', math.ulp(0.), (('a', 1.), ('b', 1.)))
        else:
            e.apply(Event(new_group='tiny', new_weight=math.ulp(0.), new_supports=(('a', 1.), ('b', 1.))))
        fault = 'not_raised'
    except NumericFault as exc:
        fault = str(exc)
    fault_rows.append({'entry': entry, 'fault': fault, 'unchanged': e.snapshot() == before})
    assert fault == 'UnderflowEvidenceWeight' and e.snapshot() == before
result['previous_underflow_witness'] = fault_rows


def unaffected_side(old_support, competitor_support):
    e = Engine()
    e.append('old', 1., (('a', old_support), ('b', old_support)))
    e.append('later', 1., (('a', 1.),))
    before = copy.deepcopy(e)
    cf, changes = e.apply(Event(revision_group='old', competitor='z', competitor_support=competitor_support))
    old_bits = {k: bits(t) for k, t in before.tendencies.items()}
    new_bits = {k: bits(e.tendencies[k]) for k in old_bits}
    return {'old_support': old_support, 'competitor_support': competitor_support,
            'product': old_support * competitor_support,
            'before_pi': {k: t.pi() for k, t in before.tendencies.items()}, 'cf': cf,
            'after_pi': {k: t.pi() for k, t in e.tendencies.items()},
            'before_shares': {k: before.allocations[k][0].share for k in old_bits},
            'after_shares': {k: e.allocations[k][0].share for k in old_bits},
            'expected_existing_side_unchanged': True, 'actual_existing_side_unchanged': old_bits == new_bits,
            'conserved': e.exact_conservation(), 'changes': changes}


result['unaffected_side_probes'] = [unaffected_side(1., 0.), unaffected_side(-1e-200, 1e-200),
                                  unaffected_side(-1e-200, 1.)]
print(json.dumps(result, indent=2, allow_nan=False))
