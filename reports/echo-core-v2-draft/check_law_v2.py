"""Original LAW tests unchanged in memory, plus independent whole-step checks.

Usage: python -B reports/echo-core-v2-draft/check_law_v2.py --suite legacy|extra
The legacy suite intentionally exits 1 if an old corollary fails. This is evidence,
not permission to weaken that test. Seeds/loops/assertions come from check_law.py.
"""
from __future__ import annotations

import argparse
import copy
import json
import math
import platform
import random
from dataclasses import asdict
from fractions import Fraction

import check_law as legacy
from law_v2_model import NumericFault, Tendency, bits, exact_shares
from law_v2_event_checks import event_checks
from law_v2_review_checks import review_checks
from law_v2_side_checks import side_checks
from law_v2_properties import property_checks


def original_checks() -> int:
    setattr(legacy, "Tendency", Tendency)  # in-memory adapter; original file is never written
    legacy.shares = exact_shares
    results = [check() for check in legacy.CHECKS[:10]]
    for result in results:
        print(json.dumps(asdict(result), ensure_ascii=False))
    failed = [r.check_id for r in results if not r.ok]
    print(json.dumps({"seed": legacy.SEED, "failed": failed}))
    return int(bool(failed))


def whole_steps() -> dict[str, object]:
    rng = random.Random(legacy.SEED + 100)
    accepted = groups = bad = faults = 0
    fault_rows: list[dict[str, object]] = []
    old_bad = 0
    for world in range(2000):
        options = dict(n0=rng.uniform(.5, 6), kappa=rng.uniform(0, 4),
                       prior=rng.choice((0., 60., 600.)), h0=rng.uniform(.02, .8))
        new = Tendency(n0=options["n0"], kappa=options["kappa"],
                       prior=options["prior"], h0=options["h0"])
        old = legacy.Tendency(n0=new.n0, kappa=new.kappa, prior=new.prior, h0=new.h0)
        for event in range(40):
            sign = rng.choice((-1., 1.))
            additions = [(10**rng.uniform(-6, 1), sign * rng.uniform(.01, 1))
                         for _ in range(rng.randint(1, 6))]
            before = new.pi()
            staged = copy.deepcopy(new)
            try:
                for w, s in additions:
                    staged.add(w, s)
            except NumericFault as exc:
                faults += 1
                fault_rows.append(dict(world=world, event=event, fault=str(exc),
                                       before=bits(new), additions=additions))
                # This synthetic lineage stops, preserving the failed event.
                break
            after = staged.pi()
            bad += sign * (after - before) <= 0
            groups += len(additions)
            accepted += 1
            new = staged
            old_before = old.pi()
            for w, s in additions:
                old.add(w, s)
            old_bad += sign * (old.pi() - old_before) <= 0
    return dict(check="LP1-whole-event", worlds=2000, planned_events=80000,
                accepted_events=accepted, groups=groups, violations=bad,
                old_violations_same_accepted_tapes=old_bad, faults=faults,
                fault_rows=fault_rows, scope="homogeneous append events, 1-6 new groups")


def counterexamples() -> dict[str, object]:
    rows: list[dict[str, object]] = []
    for cls, name in ((legacy.Tendency, "old"), (Tendency, "new")):
        t = cls(n0=1., kappa=0., h0=.3, nh=10.)
        for _ in range(3):
            t.add(3., 1.)
        before = t.pi()
        q = t.add(.01, 1.)
        rows.append(dict(form=name, before=before, after=t.pi(), q=q,
                         before_hex=before.hex(), after_hex=t.pi().hex()))
    # Isolate the replacement on exactly the OLD pre-state as well.
    old = legacy.Tendency(n0=1., kappa=0., h0=.3, nh=10.)
    for _ in range(3):
        old.add(3., 1.)
    same = Tendency(**asdict(old))
    before = same.pi()
    q = same.add(.01, 1.)
    rows.append(dict(form="new_from_old_prestate", before=before, after=same.pi(), q=q))
    # LP2 retroactive redistribution is not a same-sign append event.
    # First tendency previously owned weight 10; a new competitor takes a share.
    t = Tendency(n0=1., kappa=0., fixed_rho=1.)
    t.add(10., 1.)
    before = t.pi()
    share = exact_shares([before, 0.], [1., 1.])[0]
    replay = Tendency(n0=1., kappa=0., fixed_rho=1.)
    replay.add(10. * share, 1.)
    cf = replay.pi()
    replay.add(.01, 1.)
    return dict(check="ADR0018-and-redistribution", recurrence_rows=rows,
                redistribution=dict(before=before, counterfactual=cf, after=replay.pi(), old_share=1.,
                                    new_share=share, new_positive_weight=.01,
                                    classification="user three-clause LP1: both comparisons required",
                                    withdrawal_ok=cf <= before, marginal_ok=replay.pi() > cf),
                ok=cf <= before and replay.pi() > cf)


def replay_and_shares() -> dict[str, object]:
    rng = random.Random(legacy.SEED + 101)
    bad = prefixes = 0
    for _ in range(200):
        t = Tendency(n0=3., kappa=1., prior=rng.choice((0., 60.)))
        history: list[tuple[float, float, float, float | None]] = []
        for _ in range(40):
            history.append((rng.uniform(.01, 3), rng.uniform(-1, 1), rng.random(),
                            None if rng.random() < .2 else rng.uniform(-2, 2)))
            t.add(*history[-1])
            restored = Tendency(**json.loads(json.dumps(asdict(t))))
            recomputed = Tendency(n0=3., kappa=1., prior=t.prior)
            for entry in history:
                recomputed.add(*entry)
            bad += bits(restored) != bits(recomputed)
            prefixes += 1
            t = restored
    share_bad = 0
    for _ in range(20000):
        count = rng.randint(1, 8)
        pis = [rng.uniform(-.999, .999) for _ in range(count)]
        support = [rng.choice((-1., 0., 1.)) for _ in range(count)]
        shares = exact_shares(pis, support)
        for sign in (-1., 1.):
            part = [v for v, s in zip(shares, support, strict=True) if s == sign]
            if part:
                share_bad += sum(part) != 1.
                share_bad += sum(map(Fraction.from_float, part), Fraction()) != 1
    return dict(check="I23-I25", scalar_prefixes=prefixes, bitwise_mismatches=bad,
                groups=20000, exact_share_failures=share_bad,
                scope="scalar recurrence serialization; not Rust G/IDL/store acceptance")


def numerical_edges() -> dict[str, object]:
    cases = [("nan", 1., math.nan), ("inf", math.inf, 1.),
             ("underflow", math.ulp(0.), .25), ("tiny_change", 1e-300, 1.),
             ("overflow", 1e308, 1.)]
    rows: list[dict[str, object]] = []
    for name, w, s in cases:
        t = Tendency(n0=3., kappa=1.)
        t.add(1., 1.)
        before = bits(t)
        fault = "not_raised"
        try:
            t.add(w, s)
        except NumericFault as exc:
            fault = str(exc)
        rows.append(dict(case=name, fault=fault, unchanged=bits(t) == before))
    t = Tendency(n0=3., kappa=1.)
    t.add(1., 1.)
    pi = t.pi()
    t.add(1., 0.)
    # Counter capacity must fail before any fields are committed.
    t.groups = 2**53 - 1
    before = bits(t)
    try:
        t.add(1., 1.)
    except NumericFault as exc:
        rows.append(dict(case="counter", fault=str(exc), unchanged=bits(t) == before))
    try:
        exact_shares([math.nextafter(-1., 0.), .9], [1., 1.])
    except NumericFault as exc:
        rows.append(dict(case="tiny_share", fault=str(exc), unchanged=True))
    moment = Tendency(n0=3., kappa=1., h0=.75, nh=100.)
    moment.add(1., 1., mu=math.ulp(0.), projection=2.)
    moment.add(1., -1.)
    subnormal_ok = moment.magnitude_weight == 0 and moment.magnitude_sum == math.ulp(0.)
    return dict(check="numeric-edges", cases=rows, zero_signal_pi_unchanged=t.pi() == pi,
                subnormal_m=moment.magnitude_sum.hex(), subnormal_u=moment.magnitude_weight.hex(),
                subnormal_ok=subnormal_ok,
                ok=len(rows) == 7 and all(row["fault"] != "not_raised" and row["unchanged"] for row in rows)
                and t.pi() == pi and subnormal_ok)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--suite", choices=("legacy", "extra"), required=True)
    args = parser.parse_args()
    print(json.dumps(dict(python=platform.python_version(), platform=platform.platform())))
    if args.suite == "legacy":
        return original_checks()
    results = [whole_steps(), counterexamples(), replay_and_shares(), numerical_edges(), event_checks(),
               property_checks(), review_checks(), side_checks()]
    for row in results:
        print(json.dumps(row, ensure_ascii=False, allow_nan=False))
    # This exit code is not a waiver of the separately recorded old LAW-6 failure,
    # nor a claim that arbitrary historical reallocation is sign-monotone.
    ok = (results[0]["violations"] == 0 and results[0]["faults"] == 0
          and results[1]["ok"] is True and results[3]["ok"] is True
          and results[2]["bitwise_mismatches"] == 0 and results[2]["exact_share_failures"] == 0
          and results[5]["ok"] is True and results[6]["ok"] is True and results[7]["ok"] is True
          and all(results[4][key] == 0 for key in (
              "append_violations", "marginal_violations", "withdrawal_violations",
              "replay_mismatches", "conservation_failures", "oracle_mismatches", "faults")))
    return int(not ok)


if __name__ == "__main__":
    raise SystemExit(main())
