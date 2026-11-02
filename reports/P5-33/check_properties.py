"""Bounded reference checks, not product acceptance or a substitute for P5-30."""
from __future__ import annotations

import argparse
import copy
import itertools
import json
import random
from pathlib import Path

from detectors import METHODS, ChangePoint, make_detector
from integration import Controlled, Event, ResearchEngine
from law_v2_event_checks import oracle_counterfactual
from law_v2_model import NumericFault, exact_shares, Tendency as Reference
from run_study import csv_write, json_write
from scenarios import observations


def detector_oracle() -> dict[str, object]:
    """Exhaust every segmentation of every binary prefix up to length seven."""
    maximum = 0.0
    comparisons = 0
    for n in range(1,8):
        for ys in itertools.product((0,1),repeat=n):
            detector = ChangePoint(cap=128)
            h = 0.0
            for y in ys:
                h = detector.observe(2.0*y-1.0)
            total = reset = forecast = 0.0
            for boundaries in itertools.product((0,1),repeat=n-1):
                mass = 1.0
                yes = no = 0
                for i,y in enumerate(ys):
                    if i:
                        boundary = boundaries[i-1]
                        mass *= .001 if boundary else .999
                        if boundary:
                            yes = no = 0
                    mass *= (1.0+(yes if y else no))/(2.0+yes+no)
                    yes += y
                    no += 1-y
                total += mass
                if n>1 and boundaries[-1]:
                    reset += mass
                forecast += mass*(.001*.5+.999*(1.0+yes)/(2.0+yes+no))
            error = abs(detector.prediction()-forecast/total)
            if n>1:
                error = max(error,abs(h-reset/total))
            maximum = max(maximum,error)
            comparisons += 1
    return dict(prefixes=comparisons, maximum_absolute_error=maximum,
                ok=maximum<1e-12, scope="independent finite-enumeration oracle, float diagnostic tolerance")


def properties(method: str) -> dict[str, object]:
    rng = random.Random(20261033)
    violations: dict[str,int] = {key:0 for key in
        ("LP1","LP2_weight","LP3_duplicate","LP4_matched_state","LP5","LP6","LP7", "LP8_I23", "I25")}
    faults: list[dict[str,object]] = []
    scalar = Controlled(3.0,1.0,detector=make_detector(method))
    tape: list[tuple[float,float]] = []
    for index in range(72):
        w, s = rng.uniform(.1,3.0), rng.choice((-1.0,0.0,1.0))
        prior_state = scalar.full_state()
        pi_before = scalar.pi()
        scalar.add(w,s,.5,.2)
        violations["LP1"] += int(s != 0 and s*(scalar.pi()-pi_before)<=0)
        # Exercise the same policy-to-event path as the scenarios for 100 absent nights.
        idle_before=scalar.full_state()
        for signals in observations("c",[0]*100):
            for absent_support in signals:
                scalar.add(1.0,absent_support)
            scalar.detector.prediction()  # read-only forecast must not decay state
        violations["LP5"] += int(scalar.full_state()!=idle_before)
        tape.append((w,s))
        replay = Controlled(3.0,1.0,detector=make_detector(method))
        for rw,rs in tape:
            replay.add(rw,rs,.5,.2)
        violations["LP8_I23"] += int(scalar.full_state()!=replay.full_state())
        assert prior_state != scalar.full_state()
    for _ in range(200):
        p = rng.choice((0.0,60.0,600.0))
        pi = rng.uniform(-.7,.7)
        mass = max(4*pi/(1-pi),(2*p-(p+4)*(1+pi))/(1+pi),0.0)+1000.0
        a = pi*(p+4+mass)-p
        detector = make_detector(method)
        for _ in range(8):
            detector.observe(rng.choice((-1.0,1.0)))
        h = copy.copy(detector).observe(1.0)
        small=Reference(3.0,1.0,prior=p,a=a,b=mass,fixed_rho=1-h)
        large=copy.deepcopy(small)
        more_h=copy.deepcopy(small)
        more_mass=Reference(3.0,1.0,prior=p,a=pi*(p+4+2*mass)-p,b=2*mass,fixed_rho=1-h)
        w=rng.uniform(.1,3.0)
        old_pi=small.pi()
        old_mass_pi=more_mass.pi()
        q=small.add(w,1.0)
        large.add(2*w,1.0)
        more_h.fixed_rho=(1-h)/2
        q_more=more_h.add(w,1.0)
        more_mass.add(w,1.0)
        violations["LP2_weight"] += int(large.pi()<=small.pi())
        violations["LP4_matched_state"] += int(q_more>q)
        violations["LP7"] += int(abs(more_mass.pi()-old_mass_pi)>abs(small.pi()-old_pi))
        shares=exact_shares([pi,-pi,0.0],[1.0,1.0,-1.0])
        violations["I25"] += int(shares[0]+shares[1]!=1.0 or shares[2]!=1.0)
    plain=Controlled(3.0,0.0,detector=make_detector(method))
    complex_=Controlled(3.0,5.0,detector=make_detector(method))
    for _ in range(24):
        plain.add(1.0,1.0)
        complex_.add(1.0,1.0)
        violations["LP6"] += int(complex_.pi()>plain.pi())
    duplicate=ResearchEngine(method=method)
    duplicate.append("same",1.0,(("a",1.0),))
    duplicate_before=duplicate.full_state()
    rejected=False
    try:
        duplicate.append("same",1.0,(("a",1.0),))
    except NumericFault:
        rejected=True
    violations["LP3_duplicate"] += int(not rejected or duplicate.full_state()!=duplicate_before)
    # An independently reconstructed cf contains only OLD allocation identities.
    event_counts=dict(cf=0,withdrawal=0,new=0,replay=0,conservation=0)
    for world in range(4):
        engine=ResearchEngine(method=method,prior=60.0 if world%2 else 0.0)
        history: list[Event]=[]
        for i in range(10):
            event=Event(revision_group=f"g{rng.randrange(i)}" if i and i%2 else None,
                competitor=f"r{i}",competitor_support=rng.choice((-1.0,0.0,1.0)),
                new_group=f"g{i}",new_weight=rng.uniform(.1,3.0),
                new_supports=tuple((k,rng.choice((-1.0,1.0))) for k in ("a","b","c")))
            before=copy.deepcopy(engine)
            try:
                cf,changes=engine.apply(event)
            except NumericFault as exc:
                faults.append(dict(world=world,event=i,fault=str(exc),snapshot=before.snapshot()))
                break
            event_counts["cf"] += int(cf!=oracle_counterfactual(before,engine))
            targets=list(event.new_supports)
            if event.revision_group and event.competitor_support:
                targets.append((event.competitor,event.competitor_support))
            for name,s in targets:
                event_counts["new"] += int(s*(engine.tendencies[name].pi()-cf[name])<=0)
            for name,old,new,s in changes:
                event_counts["withdrawal"] += int(new<old and s*(cf[name]-before.tendencies[name].pi())>0)
            history.append(event)
            replay_engine=ResearchEngine(method=method,prior=engine.prior)
            for saved in history:
                replay_engine.apply(saved)
            event_counts["replay"] += int(engine.full_state()!=replay_engine.full_state())
            event_counts["conservation"] += int(not engine.exact_conservation())
    # Numeric fault must not commit either detector or scalar.
    underflow=Controlled(3.0,1.0,detector=make_detector(method))
    before_fault=underflow.full_state()
    fault="missing"
    try:
        underflow.add(2.0**-1074,.5)
    except NumericFault as exc:
        fault=str(exc)
    atomic=underflow.full_state()==before_fault and fault!="missing"
    return dict(method=method,violations=violations,event_violations=event_counts,
                numeric_faults=faults,explicit_fault=fault,atomic=atomic,
                scope="reference fixtures only; fixed control does not learn h; no product acceptance",
                ok=not any(violations.values()) and not any(event_counts.values()) and not faults and atomic)


def competition() -> list[dict[str,str|int|float|None]]:
    rows: list[dict[str,str|int|float|None]]=[]
    for method in METHODS:
        for policy in ("a","b","c"):
            engine=ResearchEngine(method=method)
            days=observations(policy,[1]*30+[0]*7+[1]*14)
            group=0
            for day,signals in enumerate(days):
                for support in signals:
                    # Opposite predictions belong to separate I25 sides, not half a vote each.
                    engine.append(f"g{group}",1.0,(("owner_evening",support),("owner_busy",-support)))
                    group+=1
                if day>=29:
                    rows.append(dict(method=method,policy=policy,day=day,events=group,
                        evening_pi=engine.tendencies["owner_evening"].pi(),
                        busy_pi=engine.tendencies["owner_busy"].pi(),
                        conservation=int(engine.exact_conservation())))
    return rows


def shared_side_control() -> list[dict[str,str|int|float|None]]:
    """Both supplied interpretations explain the SAME side; no natural-language claim."""
    rows: list[dict[str,str|int|float|None]]=[]
    for method in METHODS:
        engine=ResearchEngine(method=method)
        for day,support in enumerate([1.0]*30+[-1.0]*7+[1.0]*14):
            engine.append(f"g{day}",1.0,(("explanation_a",support),("explanation_b",support)))
            shares=[engine.allocations[k][-1].share for k in sorted(engine.allocations)]
            rows.append(dict(method=method,day=day,share_a=shares[0],share_b=shares[1],
                pi_a=engine.tendencies["explanation_a"].pi(),
                pi_b=engine.tendencies["explanation_b"].pi(),
                conservation=int(engine.exact_conservation())))
    return rows


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--out",type=Path,required=True)
    args=parser.parse_args()
    args.out.mkdir(parents=True,exist_ok=False)
    oracle_result=detector_oracle()
    method_results=[properties(m) for m in METHODS]
    result=dict(detector_oracle=oracle_result,methods=method_results)
    json_write(args.out/"properties.json",result)
    csv_write(args.out/"competition.csv",competition())
    csv_write(args.out/"shared_side_control.csv",shared_side_control())
    print(json.dumps(result,indent=2))
    sys_ok=oracle_result["ok"] is True and all(row["ok"] is True for row in method_results)
    raise SystemExit(0 if sys_ok else 1)
