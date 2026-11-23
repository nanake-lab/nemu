"""Offline summary of P0-12 continuation; raw failures remain in the archive."""
from __future__ import annotations

from datetime import datetime
import gzip
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import re
import sys
from typing import Any

from benchmark_plan import FIXED_SENTENCE
from benchmark_native import reject_constant
from http_probe import ROOT, write_json
from schema_validation import ValidationError, validator

RUN = Path('D:/Dev/cache/P0-12/gpu-02')
OUT = ROOT / 'reports/P0-12/continuation-02'


def rows(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text(encoding='utf-8').splitlines()] if path.exists() else []


def statistics(values: list[float]) -> dict[str, Any]:
    values = sorted(values)
    if not values:
        return dict(n=0)
    return dict(n=len(values), p50=values[math.ceil(len(values) * .5) - 1],
                p90=values[math.ceil(len(values) * .9) - 1], minimum=values[0], maximum=values[-1])


def memory(path: Path, begin: str | None = None, end: str | None = None) -> dict[str, Any]:
    snapshots = rows(path)
    samples = []
    errors = []
    for row in snapshots:
        if 'error' in row:
            errors.append(row)
            continue
        stamp = datetime.fromisoformat(row['startedUtc'])
        if begin and stamp < datetime.fromisoformat(begin) or end and stamp > datetime.fromisoformat(end):
            continue
        owned = {int(p['ProcessId']) for p in row['ownedProcesses']}
        matched = []
        for counter in row['counters']:
            match = re.search(r'pid_(\d+)_', counter['Name'], re.I)
            if match and int(match[1]) in owned:
                matched.append(counter)
        if matched:
            samples.append(dict(utc=row['startedUtc'], dedicated=sum(int(c['DedicatedUsage']) for c in matched),
                                shared=sum(int(c['SharedUsage']) for c in matched)))
    dedicated = max((r['dedicated'] for r in samples), default=None)
    shared = max((r['shared'] for r in samples), default=None)
    gaps = [(datetime.fromisoformat(b['utc']) - datetime.fromisoformat(a['utc'])).total_seconds()
            for a, b in zip(samples, samples[1:])]
    return dict(matchedSamples=len(samples), totalSnapshots=len(snapshots), errors=errors,
                dedicatedPeakBytes=dedicated, sharedPeakBytes=shared,
                independentPeakSumBytes=dedicated + shared if dedicated is not None and shared is not None else None,
                maximumMatchedStartGapSeconds=max(gaps, default=None), rangeStart=begin, rangeEnd=end,
                scope='WDDM owned process tree; independent sampled maxima, not continuous peak')


def main() -> None:
    with gzip.open(ROOT / 'reports/P0-12/inputs/jobs.json.gz', 'rt', encoding='utf-8') as stream:
        jobs = {job['id']: job for job in json.load(stream)}
    helper = ROOT / 'spikes/codex-app-server/src'
    sys.path.insert(0, str(helper))
    spec = importlib.util.spec_from_file_location('original_p0_08_validation', helper / 'local_irl.py')
    if spec is None or spec.loader is None:
        raise RuntimeError('Original source validation unavailable')
    original = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(original)
    full = validator(json.loads((ROOT / 'spikes/codex-app-server/semantic-candidate.schema.json').read_text()))
    measured = rows(RUN / 'item-4/samples.jsonl')
    for row in measured:
        row['schemaValid'] = None
        row['sourceValid'] = None
        if row['arm'] == 'speech':
            row['exactFixedSentence'] = row['content'] == FIXED_SENTENCE
        else:
            row['schemaValid'] = False
            row['sourceValid'] = False
            try:
                candidate = json.loads(row['content'], parse_constant=reject_constant)
                full.validate(candidate)
                row['schemaValid'] = True
                row['sourceValid'] = original.source_valid(candidate, jobs[row['id']]['sample'])
            except (ValueError, TypeError, KeyError, IndexError, ValidationError) as error:
                row['validationError'] = f'{type(error).__name__}: {error}'
    summaries = []
    for arm in ['speech', 'unconstrained', 'wire']:
        for lang in ['all', 'zh', 'en', 'ja']:
            group = [r for r in measured if r['arm'] == arm and not r['warmup'] and (lang == 'all' or r['lang'] == lang)]
            summary = dict(arm=arm, language=lang, n=len(group),
                schemaValid=sum(r['schemaValid'] is True for r in group),
                sourceValid=sum(r['sourceValid'] is True for r in group),
                exactSentence=sum(r.get('exactFixedSentence') is True for r in group))
            for key in ['ttftClientMs', 'completedClientMs']:
                summary[key] = statistics([r[key] for r in group if r.get(key) is not None])
            for key in ['prompt_n', 'prompt_ms', 'predicted_n', 'predicted_ms', 'predicted_per_second']:
                summary[key] = statistics([r['timings'][key] for r in group if key in r['timings']])
            summaries.append(summary)
    write_json(OUT / 'text-summary.json', dict(samples=measured, groups=summaries,
        warmups=sum(r['warmup'] for r in measured), percentile='nearest rank; no slow/invalid sample exclusion'))
    visual = RUN / 'items-5-7'
    image_rows = rows(visual / 'item-5/samples.jsonl')
    if image_rows:
        log = (visual / 'server/server.stderr.log').read_bytes()
        for row in image_rows:
            excerpt = log[row['serverLogStartByte']:row['serverLogEndByte']].decode('utf-8', errors='replace')
            row['imageTimingLogLines'] = [line for line in excerpt.splitlines()
                if re.search(r'encode|image|mtmd|preprocess', line, re.I)]
        write_json(OUT / 'image-summary.json', image_rows)
    if (visual / 'server/memory.jsonl').exists():
        start_path, end_path = visual / 'item-6/started.json', visual / 'item-6/completed.json'
        start = json.loads(start_path.read_text())['startedUtc'] if start_path.exists() else None
        end = json.loads(end_path.read_text())['endedUtc'] if end_path.exists() else None
        write_json(OUT / 'memory-summary.json', dict(allLoaded=memory(visual / 'server/memory.jsonl'),
            item6=memory(visual / 'server/memory.jsonl', start, end) if start and end else None))
    stable = rows(visual / 'item-7/samples.jsonl')
    if stable:
        groups = []
        for lang in ['zh', 'en', 'ja']:
            group = [r for r in stable if r['language'] == lang]
            groups.append(dict(language=lang, n=len(group), uniqueTextOutputs=len({r['content'] for r in group}),
                generatedThinkMarkers=[r['id'] for r in group if re.search(r'</?think>|思考过程|思考過程', r['content'])],
                outputs=[dict(id=r['id'], content=r['content']) for r in group]))
        write_json(OUT / 'stability-summary.json', groups)
    entries = []
    for path in sorted(RUN.rglob('*')):
        if path.is_file():
            with path.open('rb') as stream:
                digest = hashlib.file_digest(stream, 'sha256').hexdigest()
            entries.append(dict(path=path.relative_to(RUN).as_posix(), bytes=path.stat().st_size, sha256=digest))
    write_json(OUT / 'raw-inventory.json', dict(files=entries))
    before = json.loads(Path('D:/Dev/cache/P0-12/gpu-01/item-4/server/binding.json').read_text())['arguments']
    after = json.loads((RUN / 'item-4/server/binding.json').read_text())['arguments']
    at = after.index('--slot-save-path')
    write_json(OUT / 'command-comparison.json', dict(before=before, after=after,
        onlyAddedSlotSavePath=after[:at] + after[at + 2:] == before,
        slotDirectory=after[at + 1], slotDirectoryContents=[p.name for p in Path(after[at + 1]).iterdir()]))
    print(json.dumps(dict(textRequests=len(measured), imageRequests=len(image_rows), stabilityRequests=len(stable),
                          rawFiles=len(entries), rawBytes=sum(e['bytes'] for e in entries))))


if __name__ == '__main__':
    main()
