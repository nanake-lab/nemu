"""Usage: python -B reports/P5-33/run_study.py --out reports/P5-33/evidence/run-01

No databases, stores, network, processes, models, or production code imports.
Output directories are created exclusively; an earlier run is never overwritten.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import platform
import statistics
from pathlib import Path
from typing import TypeAlias

from detectors import METHODS, ChangePoint, make_detector
from integration import Controlled
from law_v2_model import Tendency as Reference
from scenarios import e2_probability, life_scenarios, observations, uniforms
from check_law import Tendency as Legacy

Row: TypeAlias = dict[str, str | int | float | None]


def csv_write(path: Path, rows: list[Row]) -> None:
    keys = list(dict.fromkeys(key for row in rows for key in row))
    with path.open("x", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=keys)
        writer.writeheader()
        writer.writerows(rows)


def json_write(path: Path, value: object) -> None:
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2, allow_nan=False)
        stream.write("\n")


def run_case(method: str, tape: list[int], probabilities: list[float], warm: int,
             policy: str, return_day: int | None, keep_trace: bool) -> tuple[Row, list[Row]]:
    model = Controlled(3.0, 1.0, detector=make_detector(method))
    event_days = observations(policy, tape)
    traces: list[Row] = []
    delay_pred = delay_pi = recover_pred = recover_pi = None
    event_count = false_pred = false_pi = 0
    previous_pred, previous_pi = 0.5, 0.0
    mse = law_mse = brier = 0.0
    count = 0
    at_risk_prediction = at_risk_pi = False
    streak = 0
    delay_five = None
    for t, signals in enumerate(event_days):
        pre = model.detector.prediction()
        law_pre = (1.0+model.pi())/2.0
        if t == warm:
            at_risk_prediction = pre >= 0.5
            at_risk_pi = model.pi() >= 0.5
        if t >= 64:
            mse += (pre-probabilities[t])**2
            law_mse += (law_pre-probabilities[t])**2
            brier += (pre-tape[t])**2
            count += 1
        h = q = None
        for support in signals:
            q = model.add(1.0, support)
            assert model.fixed_rho is not None
            h = 1.0-model.fixed_rho
            event_count += 1
        post = model.detector.prediction()
        pi = model.pi()
        if t >= warm and (return_day is None or t < return_day):
            if at_risk_prediction and delay_pred is None and post < 0.5:
                delay_pred = t-warm+1
            if at_risk_pi and delay_pi is None and pi < 0.5:
                delay_pi = t-warm+1
            streak = streak+1 if pre < 0.5 else 0
            if delay_five is None and streak >= 5:
                delay_five = t-warm+1
        if return_day is not None and t >= return_day:
            if recover_pred is None and post > 0.5:
                recover_pred = t-return_day+1
            if recover_pi is None and pi > 0.5:
                recover_pi = t-return_day+1
        if t >= 64:
            false_pred += int(post < 0.5 <= previous_pred)
            false_pi += int(pi <= 0.5 < previous_pi)
        if keep_trace:
            traces.append(dict(day=t, y=tape[t], truth=probabilities[t], signals=len(signals),
                               pre_prediction=pre, post_prediction=post, law_prediction=law_pre,
                               after_pi=pi, h=h, q=q, event_count=event_count))
        previous_pred, previous_pi = post, pi
    tracker = model.detector
    result: Row = dict(delay_prediction=delay_pred, delay_pi=delay_pi,
        censored_prediction=int(delay_pred is None), censored_pi=int(delay_pi is None),
        at_risk_prediction=int(at_risk_prediction),at_risk_pi=int(at_risk_pi),
        delay_five_pre_prediction=delay_five,
        lost_prediction=int(delay_pred is not None),lost_pi=int(delay_pi is not None),
        recovery_prediction=recover_pred, recovery_pi=recover_pi,
        recovery_prediction_censored=int(return_day is not None and recover_pred is None),
        recovery_pi_censored=int(return_day is not None and recover_pi is None),
        prediction_mse=mse/max(1,count), law_mse=law_mse/max(1,count), brier=brier/max(1,count),
        downcross_prediction=false_pred, downcross_pi=false_pi, evaluated_days=count,
        event_count=event_count, final_pi=model.pi(), final_prediction=tracker.prediction(),
        dropped_max=tracker.dropped_max if isinstance(tracker, ChangePoint) else 0.0,
        dropped_total=tracker.dropped_total if isinstance(tracker, ChangePoint) else 0.0)
    return result, traces


def background() -> tuple[list[Row], list[Row]]:
    rows: list[Row] = []
    predictions: list[Row] = []
    for n in (30, 365, 1095):
        for name, cls in (("legacy", Legacy), ("candidate_b", Reference)):
            model = cls(3.0, 1.0)
            for _ in range(n):
                model.add(1.0, 1.0)
            before = model.pi()
            delay = 0
            while model.pi() >= 0.5 and delay < 500:
                model.add(1.0, -1.0)
                delay += 1
            estimate = {("legacy",365):53, ("legacy",1095):159,
                        ("candidate_b",365):72, ("candidate_b",1095):214}.get((name,n))
            rows.append(dict(history=n, form=name, before_pi=before, delay_pi=delay,
                             estimate=estimate, difference=None if estimate is None else delay-estimate))
        for cap in (128, 4096):
            d = ChangePoint(cap=cap)
            for t, s in enumerate([1.0]*n + [-1.0]*3 + [1.0]*3):
                h = d.observe(s)
                if t >= n:
                    predictions.append(dict(history=n, cap=cap, step=t-n+1, support=s,
                                             prediction=d.prediction(), boundary_posterior=h,
                                             dropped_max=d.dropped_max))
    return rows, predictions


def summarize(rows: list[Row], group_keys: tuple[str, ...]) -> list[dict[str, object]]:
    groups: dict[tuple[str, ...], list[Row]] = {}
    for row in rows:
        key = tuple(str(row[k]) for k in group_keys)
        groups.setdefault(key, []).append(row)
    out: list[dict[str, object]] = []
    for key, group in groups.items():
        item: dict[str, object] = dict(zip(group_keys, key, strict=True))
        item["seeds"] = len(group)
        for metric in ("delay_prediction", "delay_pi", "recovery_prediction", "recovery_pi",
                       "prediction_mse", "law_mse", "brier", "downcross_prediction", "downcross_pi",
                       "dropped_max", "delay_five_pre_prediction", "at_risk_prediction", "at_risk_pi"):
            values: list[float] = []
            for r in group:
                value = r.get(metric)
                if isinstance(value,(float,int)):
                    values.append(float(value))
            item[metric] = dict(observed=len(values), mean=statistics.mean(values) if values else None,
                                stdev=statistics.stdev(values) if len(values)>1 else None,
                                maximum=max(values) if values else None)
        out.append(item)
    return out


def run(out: Path) -> None:
    out.mkdir(parents=True, exist_ok=False)
    source = Path(__file__).resolve().parents[1]/"echo-core-v2-draft"
    hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(source.glob("*.py"))}
    rows, pred_rows = background()
    csv_write(out/"background.csv", rows)
    csv_write(out/"background_predictions.csv", pred_rows)
    life: list[Row] = []
    e2: list[Row] = []
    traces: list[Row] = []
    tapes: list[Row] = []
    deterministic: dict[tuple[str,str,str], Row] = {}
    trace: list[Row]
    for seed in range(41000, 41016):
        tape = uniforms(seed, 1595)
        tapes.extend(dict(seed=seed, t=t, uniform=u) for t,u in enumerate(tape))
        for case in life_scenarios():
            ys = [int(u<p) for u,p in zip(tape,case.probabilities)]
            for method in METHODS:
                for policy in ("a","b","c"):
                    key = (case.name,method,policy)
                    if case.name != "fade" and key in deterministic:
                        result, trace = deterministic[key], []
                    else:
                        result, trace = run_case(method, ys, list(case.probabilities), case.warm,
                                                policy, case.return_day, seed == 41000)
                        if case.name != "fade":
                            deterministic[key] = result
                    identity: Row = dict(seed=seed, scenario=case.name, method=method, policy=policy)
                    life.append(identity | result)
                    traces.extend(identity | row for row in trace)
        for world in ("stable","abrupt","gradual","cycle"):
            ps = [e2_probability(world,t) for t in range(512)]
            ys = [int(u<p) for u,p in zip(tape,ps)]
            for method in METHODS:
                result, trace = run_case(method,ys,ps,320,"a",None,seed==41000)
                identity = dict(seed=seed,world=world,method=method)
                e2.append(identity | result)
                traces.extend(identity | row for row in trace)
    csv_write(out/"life_per_seed.csv",life)
    csv_write(out/"e2_per_seed.csv",e2)
    csv_write(out/"traces_seed41000.csv",traces)
    csv_write(out/"uniform_tapes.csv",tapes)
    json_write(out/"summary.json",dict(life=summarize(life,("scenario","method","policy")),
                                      e2=summarize(e2,("world","method"))))
    json_write(out/"manifest.json",dict(python=platform.python_version(),platform=platform.platform(),
        source_commit="541be9e",baseline_hashes=hashes, seeds=list(range(41000,41016)),
        methods=METHODS,life_rows=len(life),e2_rows=len(e2),
        parameters=dict(c=4,prior=0,h0=.1,nh=10,hazard=.001,cap=128),
        deterministic_reuse="Identical noiseless tapes evaluated once per method/policy and copied to seed rows",
        metrics="post-update crossing; delays in calendar observation opportunities; blank=right censored",
        no_storage="synthetic in-memory states only; file I/O restricted to source and report outputs"))
    print(json.dumps(dict(out=str(out),life_rows=len(life),e2_rows=len(e2),status="generated")))


if __name__ == "__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--out",type=Path,required=True)
    run(parser.parse_args().out)
