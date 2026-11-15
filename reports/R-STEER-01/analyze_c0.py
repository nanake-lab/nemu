"""Offline C0 gate and complete per-cell tables; no carrier or evaluator calls."""
from __future__ import annotations

import argparse
import csv
import json
import math
from collections import defaultdict
from pathlib import Path
from typing import TypedDict

from calibrate import digest, write_json


class Row(TypedDict):
    id: str
    root: str
    lang: str
    kind: str
    concept: str
    style: int
    mapping: int
    order: int
    alphabet: str
    q: float
    mass: float
    valid: bool
    correct: bool


def mean(values: list[float]) -> float:
    return sum(values) / len(values)


def gates(rows: list[Row]) -> dict[str, object]:
    quality = []
    cells = []
    for lang in ('zh', 'en', 'ja'):
        expression = [r for r in rows if r['lang'] == lang and r['kind'] == 'expression']
        oracle = [r for r in rows if r['lang'] == lang and r['kind'] == 'oracle']
        valid = mean([float(r['valid']) for r in expression])
        mass = mean([float(r['mass'] >= 0.95) for r in expression])
        correct = mean([float(r['correct']) for r in oracle])
        oracle_mass = mean([float(r['mass'] >= 0.95) for r in oracle])
        quality.append(dict(lang=lang, expression_valid=valid, expression_mass=mass,
                            oracle_correct=correct, oracle_mass=oracle_mass,
                            passed=valid >= 0.95 and mass >= 0.95 and correct >= 0.90 and oracle_mass >= 0.95))
        for concept in ('connection', 'empathy', 'curiosity', 'accomplishment'):
            for style in (0, 1):
                grouped: dict[str, list[float]] = defaultdict(list)
                for row in expression:
                    if row['concept'] == concept and row['style'] == style:
                        grouped[row['root']].append(row['q'])
                roots = []
                for root, probabilities in sorted(grouped.items()):
                    typical = sorted(probabilities)[len(probabilities) // 2 - 1:len(probabilities) // 2 + 1]
                    median = mean(typical)
                    span = max(probabilities) - min(probabilities)
                    roots.append(dict(root=root, median=median, layout_range=span,
                                      dynamic=0.05 < median < 0.95, stable=span <= 0.20))
                cells.append(dict(lang=lang, concept=concept, style=style, roots=roots,
                                  passed=len(roots) == 2 and any(r['dynamic'] and r['stable'] for r in roots)))
    return dict(quality=quality, cells=cells,
                interface_passed=all(q['passed'] for q in quality) and all(c['passed'] for c in cells))


def analyze(source: Path) -> dict[str, object]:
    jobs = json.loads((source / 'jobs.json').read_text(encoding='utf-8'))
    exits = json.loads((source / 'exit.json').read_text(encoding='utf-8'))
    if not exits['all_requests_completed'] or exits['completed'] != len(jobs):
        raise ValueError('Incomplete run: no gate decision; preserve partial records')
    codes: dict[str, dict[str, int]] = {}
    for line in (source / 'protocol.log').read_text(encoding='utf-8').splitlines():
        parts = line.split('\t')
        codes[parts[1]] = dict(zip('ABXY', (int(x) for x in parts[2:]), strict=True))
    rows: list[Row] = []
    free = []
    for job in jobs:
        directory = source / 'requests' / job['id']
        records = list(csv.DictReader((directory / 'logits.tsv').read_text(encoding='utf-8').splitlines(), delimiter='\t'))
        if job['kind'] == 'free':
            end = list(csv.DictReader((directory / 'completion.tsv').read_text().splitlines(), delimiter='\t'))[0]
            raw_text = (directory / 'output.bin').read_bytes()
            try:
                text = raw_text.decode('utf-8')
                utf8_valid = True
            except UnicodeDecodeError:
                text = raw_text.decode('utf-8', errors='replace')
                utf8_valid = False
            free.append(dict(id=job['id'], text=text, truncated=end['eog'] == '0',
                             generated_tokens=int(end['steps']), utf8_valid=utf8_valid,
                             thinking_markers=[mark for mark in ('<think>', '</think>') if mark in text],
                             independent_semantic_rating='not_run'))
            continue
        if job['kind'] == 'guard':
            continue
        record = records[0]
        positive = float(record[job['target']])
        negative = float(record[next(c for c in job['alphabet'] if c != job['target'])])
        # Temperature 1; conditional probability and full-vocabulary mass remain separate.
        m = max(positive, negative)
        q = math.exp(positive - m) / (math.exp(positive - m) + math.exp(negative - m))
        mass = math.exp(positive - float(record['logZ'])) + math.exp(negative - float(record['logZ']))
        selected = int(record['argmax'])
        rows.append(Row(id=job['id'], root=job['root'], lang=job['lang'], kind=job['kind'],
                        concept=job['concept'], style=job['style'], mapping=job['mapping'], order=job['order'],
                        alphabet=job['alphabet'], q=q, mass=mass,
                        valid=selected in [codes[job['id']][c] for c in job['alphabet']],
                        correct=selected == codes[job['id']][job['target']]))
    controls = [dict(lang=lang, byte_equal=digest(source / f'requests/guard-{i}-0/first-logits.f32')
                    == digest(source / f'requests/guard-{i}-1/first-logits.f32')) for i, lang in enumerate(('zh', 'en', 'ja'))]
    result = gates(rows)
    result['repeat_controls'] = controls
    result['decision'] = 'C0_usable' if result['interface_passed'] and all(c['byte_equal'] for c in controls) else 'C0_insufficient_stop'
    result['rows'] = rows
    result['free_outputs'] = free
    # Differences expose alphabet, label, position and courtesy dependence; no rows filtered.
    factors: list[tuple[str, list[int | str]]] = [('style', [0, 1]), ('mapping', [0, 1]),
                                               ('order', [0, 1]), ('alphabet', ['AB', 'XY'])]
    result['factor_means'] = [{factor: value, 'lang': lang, 'q': mean([r['q'] for r in rows if r['kind'] == 'expression'
                              and r['lang'] == lang and r.get(factor) == value])}
                             for lang in ('zh', 'en', 'ja') for factor, values in factors for value in values]
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('source', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    write_json(args.output, analyze(args.source))


if __name__ == '__main__':
    main()
