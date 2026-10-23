"""Frozen hybrid-reference selection and descriptive reanalysis; no model or DUT imports."""
from __future__ import annotations
import json
import math
from pathlib import Path
from typing import cast
from reference_data import file_sha, number, obj, objects, read_json, text

REPO = Path(__file__).resolve().parents[5]
PLAN_PATH = REPO / 'reports/P4-01/manual-reference-plan.json'
ATTEMPT_PATH = REPO / 'reports/P4-01/latency-attempt-2026-10-21T15-52-58-219Z.json'


def selection() -> tuple[dict[str, object], list[dict[str, object]]]:
    plan = obj(read_json(PLAN_PATH))
    evidence = REPO / text(plan['stableEvidencePath'])
    if file_sha(evidence) != plan['stableEvidenceSha256']:
        raise ValueError('Frozen stability evidence changed')
    rows = objects(obj(read_json(evidence))['samples'])
    stable = [r for r in rows if r['stable'] is True]
    if len(stable) != 39 or [r['id'] for r in stable] != plan['automaticIds']:
        raise ValueError('Stable selection differs from preregistration')
    if {r['id'] for r in rows if r['stable'] is False} != set(cast(list[str], plan['manualIds'])):
        raise ValueError('Manual selection differs from preregistration')
    return plan, stable


def primary(row: dict[str, object]) -> dict[str, object]:
    attempts = objects(row['attempts'])
    first = attempts[0]
    if first['anchorId'] != f"public-{row['lang']}-001" or first['status'] != 'valid':
        raise ValueError('Unexpected fixed primary anchor')
    return first


def quantile(values: list[float], proportion: float) -> float:
    if not values or not 0 <= proportion <= 1:
        raise ValueError('Nonempty values and proportion in [0,1] required')
    ordered = sorted(number(v) for v in values)
    index = (len(ordered)-1)*proportion
    low, high = math.floor(index), math.ceil(index)
    return ordered[low] + (ordered[high]-ordered[low])*(index-low)


def distribution(values: list[float]) -> dict[str, object]:
    edges = [-math.inf, 0, 200, 250, 300, 400, 600, 1000, math.inf]
    labels = ['<0', '[0,200)', '[200,250)', '[250,300)', '[300,400)',
              '[400,600)', '[600,1000)', '>=1000']
    return {'count': len(values),
            'quantilesMs': {name: quantile(values, q) for name, q in
                            [('min',0),('p05',.05),('p25',.25),('median',.5),
                             ('p75',.75),('p95',.95),('max',1)]},
            'histogram': {label: sum(a <= v < b for v in values)
                          for label, a, b in zip(labels,edges[:-1],edges[1:],strict=True)}}


def validate_annotations(raw: object, plan: dict[str, object],
                         inputs: list[dict[str, object]]) -> list[dict[str, object]]:
    document = obj(raw)
    if document.get('method') != plan['method'] or document.get('planSha256') != file_sha(PLAN_PATH):
        raise ValueError('Annotation method/plan mismatch')
    if document.get('status') != 'final' or document.get('sampleRate') != 16000:
        raise ValueError('Only final 16 kHz annotations are accepted')
    rows = objects(document['samples'])
    expected = cast(list[str], plan['presentationOrder'])
    ids = [text(r['id']) for r in rows]
    if len(ids) != len(set(ids)) or set(ids) != set(expected) or len(ids) != 26:
        raise ValueError('Exactly the 26 unique blind tasks are required')
    indexed = {text(r['id']): r for r in inputs}
    for row in rows:
        source = indexed[text(row['id'])]
        if row['sha256'] != source['sha256'] or row['frameCount'] != source['frameCount']:
            raise ValueError('Annotation audio identity mismatch')
        start, end, frames = row['startSample'], row['endSample'], row['frameCount']
        if any(isinstance(v,bool) or not isinstance(v,int) for v in (start,end,frames)):
            raise ValueError('Annotation boundaries must be integer sample indices')
        if not 0 <= cast(int,start) < cast(int,end) <= cast(int,frames):
            raise ValueError('Invalid annotation interval')
    return rows


def analyze() -> dict[str, object]:
    plan, stable = selection()
    attempt = obj(read_json(ATTEMPT_PATH))
    previous = {text(r['id']): r for r in objects(attempt['results'])}
    output: list[dict[str, object]] = []
    for reference in stable:
        old = previous[text(reference['id'])]
        point = primary(reference)
        trace = objects(old['trace'])
        starts = [number(t['at']) for t in trace if t['type']=='voice.vad' and t.get('state')=='speechStart']
        ends = [number(t['at']) for t in trace if t['type']=='voice.vad' and t.get('state')=='speechEnd']
        barges = [number(t['at']) for t in trace if t['type']=='voice.bargeIn']
        if not starts or not ends or not barges:
            raise ValueError('Incomplete old trace; do not silently exclude a stable case')
        start, end = number(point['speechStartMs']), number(point['speechEndMs'])
        output.append({'id': reference['id'], 'lang': reference['lang'],
            'canonicalSha256':old['sha256'],'speechStartMs':start,'speechEndMs':end,
            'vadStartEventMs':starts[0],'vadEndEventMs':ends[-1],'bargeEventMs':barges[0],
            'vadEndLatencyMs':ends[-1]-end,'bargeLatencyMs':barges[0]-start,
            'onsetToNormalVadStartMs':starts[0]-start,
            'normalVadStartToBargeMs':barges[0]-starts[0],
            'automaticStartInsideDeclaredLeadingPadding':start<1000,
            'configuredBargePersistenceMs':200,'configuredEndSilenceMs':200,
            'bargeLatencyMinusConfiguredPersistenceMs':barges[0]-start-200,
            'vadInferenceMs':None,'processingMs':None,'eventSendMs':None,
            'maximumPacketLatenessMs':old['maximumLatenessMs']})
    summary = {}
    for language in ['all','zh','en','ja']:
        rows = [r for r in output if language=='all' or r['lang']==language]
        summary[language] = {
            'vadEnd':distribution([number(r['vadEndLatencyMs']) for r in rows]),
            'barge':distribution([number(r['bargeLatencyMs']) for r in rows]),
            'onsetToNormalVadStart':distribution([number(r['onsetToNormalVadStartMs']) for r in rows]),
            'normalVadStartToBarge':distribution([number(r['normalVadStartToBargeMs']) for r in rows]),
            'vadEndOver300':sum(number(r['vadEndLatencyMs'])>300 for r in rows),
            'bargeOver250':sum(number(r['bargeLatencyMs'])>250 for r in rows),
            'earlyVadEndIds':[r['id'] for r in rows if number(r['vadEndLatencyMs'])<0],
            'earlyBargeIds':[r['id'] for r in rows if number(r['bargeLatencyMs'])<0],
            'bargeBelow200Ids':[r['id'] for r in rows if number(r['bargeLatencyMs'])<200],
            'earlyNormalVadStartIds':[r['id'] for r in rows if number(r['onsetToNormalVadStartMs'])<0]}
    return {'method':plan['method'],'kind':'retrospective-stable39-not-formal60',
            'planSha256':file_sha(PLAN_PATH),'attemptSha256':file_sha(ATTEMPT_PATH),
            'attemptPath':str(ATTEMPT_PATH.relative_to(REPO)),
            'referenceSelection':'fixed same-language anchor001, no averaging or DUT-based selection',
            'count':len(output),'summary':summary,'samples':output,
            'limitations':['No isolated native VAD inference / JS handling / event send spans were recorded',
                          'Local PCM replay to event.ts sampled inside publish before schema validation/emit; excludes physical microphone and main/stage IPC',
                          'Stable contexts are not calibrated ground truth; five human audits pending']}


if __name__ == '__main__':
    result = analyze()
    path = REPO/'reports/P4-01/stable39-latency.json'
    path.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    for language, stats in obj(result['summary']).items():
        print(language, json.dumps(stats,ensure_ascii=False))
