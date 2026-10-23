"""Independent R2 witnesses: side membership must be checked before quantization.

This file runs unchanged against both the reviewed code and the repaired code;
the pre-fix failures are retained separately. No input-state injection is used.
"""
from __future__ import annotations

import copy
import json
import random
import struct
from dataclasses import asdict

from law_v2_event_checks import oracle_counterfactual
from law_v2_events import Engine, Event
from law_v2_model import NumericFault, bits


def side(value: float) -> int:
    return (value > 0) - (value < 0)


def participant_bytes(engine: Engine, name: str) -> tuple[str, tuple[tuple[str, str], ...]]:
    rows = tuple((r.group, struct.pack(">4d", r.weight, r.support, r.share, r.q).hex())
                 for r in engine.allocations[name])
    return bits(engine.tendencies[name]), rows


def side_checks() -> dict[str, object]:
    rng = random.Random(20261023 + 2)
    bad_state = bad_share = bad_cf = bad_plan = bad_first = bad_i25 = bad_replay = faults = 0
    cases = prefixes = unchanged_targets = affected_targets = 0
    failures: list[dict[str, object]] = []
    witnesses: list[dict[str, object]] = []
    families: dict[str, int] = {}

    def run_case(label: str, old: tuple[tuple[str, float], ...], support: float,
                 weight: float = 1., later_weight: float = 1., later_support: float = 1.,
                 prior: float = 0., witness: bool = False) -> None:
        nonlocal cases, prefixes, unchanged_targets, affected_targets, faults
        nonlocal bad_state, bad_share, bad_cf, bad_plan, bad_first, bad_i25, bad_replay
        initial = [Event(new_group="old", new_weight=weight, new_supports=old),
                   Event(new_group="later", new_weight=later_weight, new_supports=(("a", later_support),))]
        engine = Engine(prior=prior)
        for e in initial:
            engine.apply(e)
        before = copy.deepcopy(engine)
        event = Event(revision_group="old", competitor="z", competitor_support=support)
        unchanged = [name for name, s in old if side(support) == 0 or side(s) != side(support)]
        affected = [name for name, s in old if side(support) != 0 and side(s) == side(support)]
        cases += 1
        families[label] = families.get(label, 0) + 1
        try:
            cf, _ = engine.apply(event)
            prepared, plan = before.prepare(event)
        except NumericFault as exc:
            faults += 1
            record = dict(family=label, fault=str(exc), event=asdict(event), before=before.snapshot(),
                          unchanged_after_failure=engine.snapshot() == before.snapshot())
            failures.append(record)
            if witness:
                witnesses.append(record)
            return
        state_errors = share_errors = cf_errors = 0
        for name in unchanged:
            unchanged_targets += 1
            old_state, old_rows = participant_bytes(before, name)
            new_state, new_rows = participant_bytes(engine, name)
            state_errors += old_state != new_state
            share_errors += old_rows != new_rows
            state_errors += participant_bytes(prepared, name) != (old_state, old_rows)
            cf_errors += cf[name].hex() != before.tendencies[name].pi().hex()
        affected_targets += len(affected)
        # Unlike the earlier cf oracle, this checks the plan against OLD signs.
        plan_error = {c[0] for c in plan.changes} != set(affected)
        first_error = (len(plan.first) != 1 or plan.first[0].name != "z"
                       or plan.first[0].share != (0. if support == 0 else
                           next(r.share for r in engine.allocations["z"] if r.group == "old")))
        first_error |= any(r.group == "old" for r in prepared.allocations["z"])
        first_error |= cf["z"] != before.prior / (before.prior + 4.)
        if support == 0:
            first_error |= engine.tendencies["z"].pi() != cf["z"]
        else:
            first_error |= side(support) * (engine.tendencies["z"].pi() - cf["z"]) <= 0
        first_error |= engine.tendencies["z"].groups != 1
        cf_errors += cf != oracle_counterfactual(before, engine)
        conserve_error = not engine.exact_conservation()
        replay = Engine(prior=prior)
        for e in [*initial, event]:
            replay.apply(e)
        replay_error = engine.snapshot() != replay.snapshot()
        prefixes += 1
        bad_state += state_errors
        bad_share += share_errors
        bad_cf += cf_errors
        bad_plan += plan_error
        bad_first += first_error
        bad_i25 += conserve_error
        bad_replay += replay_error
        if witness or any((state_errors, share_errors, cf_errors, plan_error, first_error,
                           conserve_error, replay_error)):
            record = dict(family=label, old_supports=old, candidate_support=support,
                          before_pi={k: t.pi() for k, t in before.tendencies.items()},
                          cf=cf, after_pi={k: t.pi() for k, t in engine.tendencies.items()},
                          before_shares={k: before.allocations[k][0].share for k in unchanged},
                          after_shares={k: engine.allocations[k][0].share for k in unchanged},
                          unaffected_state_errors=state_errors, unaffected_share_errors=share_errors,
                          cf_errors=cf_errors, plan_error=plan_error, first_error=first_error,
                          conserved=not conserve_error, replay_mismatch=replay_error)
            if witness:
                witnesses.append(record)
            else:
                failures.append(record)

    # The two public-entry examples and the unrelated-side quantization fault.
    run_case("review-zero", (("a", 1.), ("b", 1.)), 0., witness=True)
    run_case("review-tiny-opposite", (("a", -1e-200), ("b", -1e-200)), 1e-200, witness=True)
    run_case("review-quantization", (("a", 1.), ("b", 1.)), -1.,
             later_weight=float(2**55-8), later_support=-1., witness=True)
    # Positive control: fixing the bug must not disable same-side redistribution.
    run_case("same-side-control", (("a", 1.), ("b", 1.)), 1., witness=True)

    for _ in range(200):
        sign = rng.choice((-1., 1.))
        strength, later = rng.uniform(.5, 2.), rng.uniform(.5, 2.)
        prior = rng.choice((0., 60., 600.))
        run_case("random-zero", (("a", sign), ("b", sign)), rng.choice((0., -0.)),
                 weight=strength, later_weight=later, prior=prior)
        tiny_old, tiny_new = 10**rng.uniform(-240, -180), 10**rng.uniform(-240, -180)
        run_case("random-tiny-opposite", (("a", sign*tiny_old), ("b", sign*tiny_old)),
                 -sign*tiny_new, weight=strength, later_weight=later, later_support=-sign)
        run_case("random-quantization", (("a", 1.), ("b", 1.)), -1.,
                 later_weight=float(2**55-8), later_support=-1.)
        # Both sides and a zero member exist; exactly one nonzero side may change.
        order = [("a", .7), ("b", .8), ("c", -.6), ("d", -.9), ("u", 0.)]
        rng.shuffle(order)
        run_case("random-two-sides", tuple(order), rng.choice((-1., 0., 1.)),
                 weight=strength, later_weight=later, prior=prior)
    return dict(check="PR61-R2-side-membership", seed=20261025, cases=cases, families=families,
                unchanged_targets=unchanged_targets, affected_targets=affected_targets,
                unaffected_state_violations=bad_state, unaffected_share_violations=bad_share,
                counterfactual_violations=bad_cf, plan_violations=bad_plan,
                first_allocation_violations=bad_first, conservation_failures=bad_i25,
                history_prefixes=prefixes, replay_mismatches=bad_replay, faults=faults,
                witnesses=witnesses, failures=failures,
                ok=not any((bad_state, bad_share, bad_cf, bad_plan, bad_first, bad_i25, bad_replay, faults)))


if __name__ == "__main__":
    outcome = side_checks()
    print(json.dumps(outcome, ensure_ascii=False, allow_nan=False))
    raise SystemExit(int(not outcome["ok"]))
