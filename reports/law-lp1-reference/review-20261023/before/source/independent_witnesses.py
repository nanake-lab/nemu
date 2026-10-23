"""Read-only PR61 review witnesses. Run from the pinned review worktree."""
from fractions import Fraction as F
from pathlib import Path
import copy
import json
import math
import random
import sys

sys.path.insert(0, str(Path.cwd() / 'reports/echo-core-v2-draft'))
from law_v2_events import Engine, Event


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
    delta_identity = (discount * d + sign * u) / (p + c + q * v + w)
    assert after - before == delta_identity
    assert sign * (after - before) >= u / (2 * (p + c + q * v + w)) > 0
    return e2, c2


rng = random.Random(610023)
for _ in range(2000):
    p, c = F(rng.randrange(0, 601)), F(rng.randrange(1, 81), 10)
    e, opposition = F(rng.randrange(0, 1001), 10), F(rng.randrange(0, 1001), 10)
    h = F(rng.randrange(0, 100), 100)
    w = F(rng.randrange(1, 101), 100)
    sign = rng.choice((-1, 1))
    exact_step(p, c, e, opposition, h, w, sign)
    before = (p + e - opposition) / (p + c + e + opposition)
    removed = F(rng.randrange(1, 100), 100)
    after_support_withdrawal = (p + e * removed - opposition) / (p + c + e * removed + opposition)
    after_opposition_withdrawal = (p + e - opposition * removed) / (p + c + e + opposition * removed)
    assert after_support_withdrawal <= before <= after_opposition_withdrawal
    # Same fixed reallocation, remove every new contribution for the counterfactual.
    exact_step(p, c, e * removed, opposition * removed, h, w, sign)

result = {'exact_rational': {'seed': 610023, 'states': 2000,
          'append_and_counterfactual_cases': 4000, 'withdrawal_comparisons': 4000,
          'identity_and_direction_failures': 0}}

engine = Engine()
event = Event(new_group='tiny', new_weight=math.ulp(0.0),
              new_supports=(('a', 1.0), ('b', 1.0)))
engine.apply(event)
result['positive_weight_silently_underflows'] = {
    name: {'pi': t.pi(), 'groups': t.groups, 'a': t.a, 'v': t.b,
           'original_weight_hex': engine.allocations[name][0].weight.hex(),
           'share': engine.allocations[name][0].share,
           'actual_q': engine.allocations[name][0].q}
    for name, t in engine.tendencies.items()}

engine = Engine()
engine.append('old', 10.0, (('a', 1.0),))
before = copy.deepcopy(engine)
cf, _ = engine.apply(Event(revision_group='old', competitor='b', competitor_support=1.0))
# Reuse the actual share for a, retain the allocation plan, omit the new (old,b).
old_a = engine.allocations['a'][0]
oracle = F.from_float(old_a.weight) * F.from_float(old_a.share)
result['backfill_counterfactual_includes_new_evidence'] = {
    'reported_cf': cf, 'after': {k: t.pi() for k, t in engine.tendencies.items()},
    'expected_cf_b': 0.0, 'expected_cf_a': float(oracle / (4 + oracle)),
    'new_b_weight': engine.allocations['b'][0].weight * engine.allocations['b'][0].share}

print(json.dumps(result, indent=2))
