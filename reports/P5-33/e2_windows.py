"""Explicit change windows for E2; stable worlds have no detection endpoint."""
from __future__ import annotations

import argparse
from pathlib import Path

from detectors import METHODS,make_detector
from integration import Controlled
from run_study import Row,csv_write,json_write
from scenarios import e2_probability,uniforms


def run(out: Path) -> None:
    out.mkdir(parents=True,exist_ok=False)
    rows: list[Row]=[]
    for seed in range(41000,41016):
        tape=uniforms(seed,512)
        for world in ("abrupt","gradual","cycle"):
            windows=[(320,512,-1)] if world=="abrupt" else (
                [(192,512,-1)] if world=="gradual" else [(128,256,-1),(256,384,1),(384,512,-1)])
            for method in METHODS:
                model=Controlled(3.0,1.0,detector=make_detector(method))
                pre: list[tuple[float,float]]=[]
                post: list[tuple[float,float]]=[]
                for t,u in enumerate(tape):
                    pre.append((model.detector.prediction(),model.pi()))
                    model.add(1.0,1.0 if u<e2_probability(world,t) else -1.0)
                    post.append((model.detector.prediction(),model.pi()))
                for start,end,direction in windows:
                    # A threshold can already be lost before the change: left censor it.
                    risk_p=pre[start][0]>=.5 if direction<0 else pre[start][0]<=.5
                    risk_pi=pre[start][1]>=.5 if direction<0 else pre[start][1]<=.5
                    delay_p=delay_pi=five=None
                    streak=0
                    for t in range(start,end):
                        pp,pi=post[t]
                        p_cross=pp<.5 if direction<0 else pp>.5
                        pi_cross=pi<.5 if direction<0 else pi>.5
                        if risk_p and delay_p is None and p_cross:
                            delay_p=t-start+1
                        if risk_pi and delay_pi is None and pi_cross:
                            delay_pi=t-start+1
                        before_cross=pre[t][0]<.5 if direction<0 else pre[t][0]>.5
                        streak=streak+1 if before_cross else 0
                        if five is None and streak>=5:
                            five=t-start+1
                    rows.append(dict(seed=seed,world=world,method=method,start=start,end=end,
                        direction=direction,risk_prediction=int(risk_p),risk_pi=int(risk_pi),
                        delay_prediction=delay_p,delay_pi=delay_pi,delay_five_pre_prediction=five,
                        prediction_right_censored=int(risk_p and delay_p is None),
                        pi_right_censored=int(risk_pi and delay_pi is None)))
    csv_write(out/"per_seed.csv",rows)
    json_write(out/"scope.json",dict(rows=len(rows),gradual="delay from start of ramp at t192",
        stable="No changepoint; only false-crossing and error metrics in main study",
        cycle="all three changes with direction-specific crossing and right/left censoring"))
    print(f"E2 windows generated: {len(rows)}")


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--out",type=Path,required=True)
    run(parser.parse_args().out)
