"""Frozen C1 automatic analysis. Independent semantic results remain separate."""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
from dataclasses import dataclass
from pathlib import Path
from statistics import mean
from typing import cast

from c1_fixtures import Job


@dataclass(frozen=True)
class Score:
    total: float
    count: int
    ids: tuple[int, ...]


def load_score(folder: Path) -> Score:
    prefix = tuple(int(x) for x in (folder / 'prompt-ids.tsv').read_text().splitlines())
    complete = tuple(int(x) for x in (folder / 'complete-ids.tsv').read_text().splitlines())
    if not prefix or complete[:len(prefix)] != prefix:
        raise ValueError('Token boundary mismatch')
    ids = complete[len(prefix):]
    with (folder / 'logits.tsv').open(encoding='utf-8', newline='') as stream:
        rows = list(csv.DictReader(stream, delimiter='\t'))
    if not ids or len(rows) != len(ids):
        raise ValueError('Incomplete teacher-forcing records')
    values = []
    for step, (row, token) in enumerate(zip(rows,ids,strict=True)):
        logp = float(row['logp'])
        if (int(row['step']) != step or int(row['target']) != token or int(row['eog']) != 0
                or not math.isfinite(logp) or logp > 1e-10
                or abs((float(row['target_logit'])-float(row['logZ']))-logp) > 1e-10):
            raise ValueError('Invalid per-token evidence')
        values.append(logp)
    with (folder / 'completion.tsv').open(encoding='utf-8',newline='') as stream:
        summary = next(csv.DictReader(stream,delimiter='\t'))
    total = sum(values)
    if int(summary['steps']) != len(ids) or abs(total-float(summary['sum_logp'])) > 1e-8:
        raise ValueError('Completion sum differs')
    return Score(total,len(ids),ids)


def gain(context: Score, prior: Score) -> float:
    if context.ids != prior.ids or context.count != prior.count:
        raise ValueError('Different continuation tokenization across contexts')
    return (context.total-prior.total)/context.count


def inside(negative: float, neutral: float, positive: float) -> bool:
    return positive > negative and .1 <= (neutral-negative)/(positive-negative) <= .9


def analyze(root: Path) -> dict[str, object]:
    jobs = cast(list[Job],json.loads((root / 'jobs.json').read_text(encoding='utf-8')))
    status = json.loads((root / 'exit.json').read_text(encoding='utf-8'))
    if not status['all_requests_completed'] or status['completed'] != 1686 or len(jobs) != 1686:
        return {'decision':'C1_technical_stop','reason':'incomplete run','exit':status}
    scores = {job['id']:load_score(root / 'requests' / job['id']) for job in jobs if job['kind'] != 'free'}
    priors = {(j['root'],j['lang'],j['form'],j['sign']):scores[j['id']] for j in jobs if j['kind']=='prior'}
    targets = {(j['root'],j['lang'],j['form'],j['order'],j['level'],j['sign']):scores[j['id']] for j in jobs if j['kind']=='target'}
    contrasts: dict[tuple[str,str,str,int,int],float] = {}
    detailed = []
    for key in sorted(targets):
        scene,lang,form,order,level,sign = key
        if sign < 0:
            continue
        positive = targets[key]
        negative = targets[(scene,lang,form,order,level,-1)]
        bp,bn = (priors[(scene,lang,form,s)] for s in (1,-1))
        g = gain(positive,bp)-gain(negative,bn)
        contrasts[(scene,lang,form,order,level)] = g
        detailed.append({'root':scene,'lang':lang,'form':form,'order':order,'level':level,
            'gain_mean_contrast':g,'gain_sum_contrast':positive.total-bp.total-negative.total+bn.total,
            'raw_sum_contrast':positive.total-negative.total,
            'raw_mean_contrast':positive.total/positive.count-negative.total/negative.count,
            'positive_tokens':positive.count,'negative_tokens':negative.count})
    scene_concept = {j['root']:j['concept'] for j in jobs}
    roots = sorted(scene_concept)
    concepts = sorted(set(scene_concept.values()))
    forms = ('base','paraphrase','long','courtesy')
    languages = []
    gates = []
    for lang in ('zh','en','ja'):
        cells = []
        dynamic: dict[str,float] = {}
        for scene in roots:
            dynamic[scene] = mean(contrasts[(scene,lang,f,o,1)]-contrasts[(scene,lang,f,o,-1)] for f in forms for o in (0,1))
        for concept in concepts:
            values = [dynamic[scene] for scene in roots if scene_concept[scene] == concept]
            passed = mean(values) >= .10 and min(values) > 0
            cells.append({'concept':concept,'root_anchor_spans':values,'mean_span':mean(values),'pass':passed})
            gates.append(passed)
        headroom = sum(inside(contrasts[(scene,lang,f,o,-1)],contrasts[(scene,lang,f,o,0)],contrasts[(scene,lang,f,o,1)])
                       for scene in roots for f in forms for o in (0,1))
        position = [abs(contrasts[(scene,lang,f,0,level)]-contrasts[(scene,lang,f,1,level)]) <= .25*max(abs(dynamic[scene]),.1)
                    for scene in roots for f in forms for level in (-1,0,1)]
        representation = {}
        for form,tolerance in (('paraphrase',.5),('long',.25),('courtesy',.25)):
            checks = [abs(contrasts[(scene,lang,form,o,level)]-contrasts[(scene,lang,'base',o,level)]) <= tolerance*max(abs(dynamic[scene]),.1)
                      for scene in roots for o in (0,1) for level in (-1,0,1)]
            representation[form] = {'pass_count':sum(checks),'total':len(checks),'required_fraction':.75}
            gates.append(sum(checks)/len(checks) >= .75)
        facts = {(j['root'],j['level'],j['sign']):scores[j['id']] for j in jobs if j['kind']=='fact' and j['lang']==lang}
        factual = []
        for scene in roots:
            for level in (-1,1):
                truth,wrong = facts[(scene,level,level)],facts[(scene,level,-level)]
                factual.append(truth.total > wrong.total and truth.total/truth.count > wrong.total/wrong.count)
        free_checks = []
        for job in jobs:
            if job['kind'] != 'free' or job['lang'] != lang:
                continue
            folder = root / 'requests' / job['id']
            text = (folder / 'output.bin').read_bytes().decode('utf-8',errors='strict')
            with (folder / 'completion.tsv').open(encoding='utf-8',newline='') as stream:
                record = next(csv.DictReader(stream,delimiter='\t'))
            free_checks.append(bool(text.strip()) and int(record['eog']) == 1 and '<think>' not in text and '</think>' not in text)
        guards = [(root / 'requests' / f'c1-guard-{lang}-{i}' / 'first-logits.f32').read_bytes() for i in (0,1)]
        guard_equal = len(guards[0]) == 248320*4 and guards[0] == guards[1]
        gates.extend((headroom >= 48,sum(position)/len(position) >= .8,sum(factual) >= 14,
                      sum(free_checks)/len(free_checks) >= .95,guard_equal))
        languages.append({'lang':lang,'concept_cells':cells,'headroom_pass':headroom,'headroom_total':64,
            'position_pass':sum(position),'position_total':len(position),'representation':representation,
            'factual_both_sum_mean_correct':sum(factual),'factual_total':16,'free_technical_valid':sum(free_checks),
            'free_total':len(free_checks),'guards_byte_equal':guard_equal,
            'guard_sha256':[hashlib.sha256(data).hexdigest() for data in guards]})
    return {'decision':'C1_automatic_pass_semantic_pending' if all(gates) else 'C1_insufficient_stop',
        'automatic_gates_pass':all(gates),'semantic_gate':'not_run','languages':languages,'contrasts':detailed,
        'scored_items':[{'id':key,'sum_logp':score.total,'tokens':score.count,
                         'mean_logp':score.total/score.count} for key,score in scores.items()],
        'boundary':'Continuous gain is not a concept probability; anchor response is not steering evidence.'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--run',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args = parser.parse_args()
    result = analyze(args.run)
    with args.output.open('x',encoding='utf-8',newline='\n') as stream:
        json.dump(result,stream,ensure_ascii=False,indent=2)
        stream.write('\n')
