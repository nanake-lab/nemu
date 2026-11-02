"""Sensitivity and LP4 world diagnostics. No thresholds are chosen here."""
from __future__ import annotations

import argparse
from pathlib import Path

from detectors import METHODS, ChangePoint, make_detector
from integration import Controlled
from run_study import Row,csv_write,json_write
from scenarios import uniforms


def run(out: Path) -> None:
    out.mkdir(parents=True,exist_ok=False)
    rows: list[Row]=[]
    # Exact unpruned detector until 1095+80 observations; fixed parameters otherwise.
    sequences={"long": [1.0]*1095+[-1.0]*40+[1.0]*40,
               "noise": [1.0 if u<.7 else -1.0 for u in uniforms(41000,512)]}
    for name,signals in sequences.items():
        exact=Controlled(3.0,1.0,detector=ChangePoint(cap=4096))
        baseline: list[tuple[float,float,float]]=[]
        for s in signals:
            exact.add(1.0,s)
            assert exact.fixed_rho is not None
            baseline.append((exact.detector.prediction(),exact.pi(),1.0-exact.fixed_rho))
        for cap in (16,64,128,4096):
            model=Controlled(3.0,1.0,detector=ChangePoint(cap=cap))
            dp=dpi=dh=0.0
            for s,(p,pi,h) in zip(signals,baseline,strict=True):
                model.add(1.0,s)
                assert model.fixed_rho is not None
                dp=max(dp,abs(model.detector.prediction()-p))
                dpi=max(dpi,abs(model.pi()-pi))
                dh=max(dh,abs((1.0-model.fixed_rho)-h))
            tracker=model.detector
            assert isinstance(tracker,ChangePoint)
            rows.append(dict(sequence=name,cap=cap,max_prediction_difference=dp,
                             max_pi_difference=dpi,max_h_difference=dh,
                             dropped_total=tracker.dropped_total,dropped_max=tracker.dropped_max))
    csv_write(out/"pruning.csv",rows)
    hazards: list[Row]=[]
    for hazard in (.0001,.001,.01):
        d=ChangePoint(hazard=hazard)
        for _ in range(365):
            d.observe(1.0)
        for i,s in enumerate([-1.0]*3+[1.0]*3):
            h=d.observe(s)
            hazards.append(dict(hazard=hazard,step=i+1,h=h,prediction=d.prediction()))
    csv_write(out/"hazard.csv",hazards)
    lp4: list[Row]=[]
    for method in METHODS:
        for world,signals in (("stable",[1.0]*512),("alternating",[1.0,-1.0]*256),
                              ("regimes",([1.0]*64+[-1.0]*64)*4)):
            detector=make_detector(method)
            hs=[detector.observe(s) for s in signals]
            lp4.append(dict(method=method,world=world,mean_h_after64=sum(hs[64:])/448,
                            final_h=hs[-1],prediction=detector.prediction()))
    csv_write(out/"lp4_worlds.csv",lp4)
    json_write(out/"scope.json",dict(cap_validation="No pruning at cap4096 within these finite prefixes",
        hazards="sensitivity only, not selected from outcomes",numeric_order="same loops as main study"))
    print("sensitivity generated")


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--out",type=Path,required=True)
    run(parser.parse_args().out)
