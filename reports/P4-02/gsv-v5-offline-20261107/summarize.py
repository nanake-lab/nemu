"""Recompute saved P4-02 observations only; standard library, no GPU access."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import statistics
import zipfile
from collections import Counter
from pathlib import Path
from typing import Any

Row = dict[str, Any]
METRICS = ('dedicated', 'shared', 'rss', 'private')


def quantile(values: list[float], p: float) -> float:
    values = sorted(values)
    pos = (len(values) - 1) * p
    lo, hi = math.floor(pos), math.ceil(pos)
    return values[lo] + (values[hi] - values[lo]) * (pos - lo)


def coverage(rows: list[Row]) -> Row:
    times = [float(row['utc']) for row in rows]
    return {'n': len(rows), 'first_utc': min(times) if times else None,
            'last_utc': max(times) if times else None,
            'span_s': max(times) - min(times) if times else None}


def distribution(values: list[float]) -> Row:
    return {'n': len(values), 'p50': quantile(values, .5), 'p90': quantile(values, .9),
            'min': min(values), 'max': max(values), 'mean': statistics.mean(values)}


def request_stats(rows: list[Row]) -> Row:
    total = sum(float(row['total_seconds']) for row in rows)
    audio = sum(float(row['audio_seconds']) for row in rows)
    return {'completed_event_coverage': coverage(rows),
            'first_playable_s': distribution([float(r['first_playable_seconds']) for r in rows]),
            'rtf': distribution([float(r['rtf']) for r in rows]),
            'aggregate_rtf': total / audio, 'audio_seconds_per_synthesis_second': audio / total,
            'total_synthesis_s': total, 'audio_s': audio,
            'max_request': max(rows, key=lambda r: r['first_playable_seconds'])}


def derive_memory(raw: list[Row], complete_utc: float) -> list[Row]:
    result: list[Row] = []
    for line, row in enumerate(raw, 1):
        own = {int(pid) for pid in row['owned']}
        processes = row['process_memory']
        null = row['rss'] is None or row['private'] is None or not processes
        if null and not (row['utc'] > complete_utc and not processes
                         and row['rss'] is None and row['private'] is None):
            raise ValueError(f'运行期或不符合退出条件的空样本: memory.jsonl:{line}')
        by_pid: dict[int, Row] = {}
        for pid in own:
            counters = [c for c in row['counters'] if int(c['Name'].split('_')[1]) == pid]
            if len({c['Name'] for c in counters}) != len(counters):
                raise ValueError(f'duplicate WDDM instance at line {line}')
            by_pid[pid] = {'instances': len(counters),
                           'dedicated': sum(c['DedicatedUsage'] for c in counters) if counters else None,
                           'shared': sum(c['SharedUsage'] for c in counters) if counters else None}
        if not null:
            if {int(p['pid']) for p in processes} != own:
                raise ValueError(f'incomplete process memory tree at line {line}')
            for key in ('rss', 'private'):
                if sum(p[key] for p in processes) != row[key]:
                    raise ValueError(f'process sum mismatch {key}, line {line}')
        captured = [p for p in by_pid.values() if p['instances']]
        result.append({'raw_line': line, 'utc': row['utc'], 'owned': sorted(own),
                       'status': '未采到：complete 后进程已退出' if null else '已采到',
                       'wddm_by_pid': by_pid,
                       'dedicated': sum(p['dedicated'] for p in captured) if captured and not null else None,
                       'shared': sum(p['shared'] for p in captured) if captured and not null else None,
                       'rss': row['rss'], 'private': row['private']})
    return result


def memory_stats(rows: list[Row]) -> Row:
    result: Row = {'row_coverage': coverage(rows), 'metrics': {}}
    for key in METRICS:
        valid = [r for r in rows if r[key] is not None]
        if not valid:
            result['metrics'][key] = {'coverage': coverage(valid), 'status': '未采到'}
            continue
        xs = [float(r['utc']) - valid[0]['utc'] for r in valid]
        ys = [float(r[key]) for r in valid]
        xm, ym = statistics.mean(xs), statistics.mean(ys)
        denom = sum((x - xm) ** 2 for x in xs)
        slope = sum((x - xm) * (y - ym) for x, y in zip(xs, ys)) / denom if denom else 0
        result['metrics'][key] = {'coverage': coverage(valid), 'missing_n': len(rows) - len(valid),
                                  'first_bytes': ys[0], 'last_bytes': ys[-1],
                                  'delta_bytes': ys[-1] - ys[0], 'min_bytes': min(ys),
                                  'peak_bytes': max(ys), 'mean_bytes': ym,
                                  'peak_utc': max(valid, key=lambda r: r[key])['utc'],
                                  'ols_MiB_per_hour': slope * 3600 / 1048576}
    result['dedicated_plus_shared_independent_peaks_bytes'] = sum(
        result['metrics'][key]['peak_bytes'] for key in ('dedicated', 'shared')
        if 'peak_bytes' in result['metrics'][key]) or None
    return result


def build(data: dict[str, bytes]) -> tuple[Row, list[Row], list[Row]]:
    def lines(name: str) -> list[Row]:
        return [json.loads(line) for line in data[name].decode('utf-8-sig').splitlines()]
    events, raw, guards = lines('events.jsonl'), lines('memory.jsonl'), lines('load-guard.jsonl')
    cases: list[Row] = json.loads(data['cases.json'].decode('utf-8-sig'))
    complete = next(r for r in events if r['event'] == 'complete')
    start = next(r for r in events if r['event'] == 'stability_start')
    end = next(r for r in events if r['event'] == 'stability_end')
    loaded = next(r for r in events if r['event'] == 'model_loaded')
    load_end = next(r for r in events if r['event'] == 'load_observation_end')
    memory = derive_memory(raw, float(complete['utc']))
    requests: list[Row] = []
    for row in events:
        if row['event'] != 'request':
            continue
        case_id = int(row['index']) % len(cases) if row['phase'] == 'stability' else int(row['index'])
        case = cases[case_id]
        if case['case'] != row['case']:
            raise ValueError('case identity mismatch')
        requests.append({**row, 'request_ordinal': len(requests) + 1,
                         'case_id': case_id, 'text': case['text'],
                         'case_ordinal_in_language': case_id // 2 + 1})
    stable = [r for r in requests if r['phase'] == 'stability']
    if len(stable) != end['requests'] or end['duration_seconds'] < 1800:
        raise ValueError('incomplete stability evidence')
    for lang in ('zh', 'mixed'):
        if len([r for r in requests if r['phase'] == 'latency' and r['case'] == lang]) != 20:
            raise ValueError('latency group is not 20 requests')
    stable_mem = [r for r in memory if start['utc'] <= r['utc'] <= end['utc']]
    duration = float(end['utc']) - float(start['utc'])
    first = [r for r in stable if r['utc'] <= start['utc'] + duration * .1]
    last = [r for r in stable if r['utc'] >= end['utc'] - duration * .1]
    count = math.ceil(len(stable) * .1)
    hashes: list[Row] = []
    for idx, case in enumerate(cases):
        all_rows = [r for r in requests if r['case_id'] == idx]
        repeated = [r for r in all_rows if r['phase'] == 'stability']
        counts = Counter(str(r['pcm_sha256']) for r in all_rows)
        stable_counts = Counter(str(r['pcm_sha256']) for r in repeated)
        hashes.append({'case_id': idx, **case, 'coverage': coverage(all_rows),
                       'all_unique_hashes': len(counts), 'all_byte_identical': len(counts) == 1,
                       'stability_n': len(repeated), 'stability_unique_hashes': len(stable_counts),
                       'stability_byte_identical': len(stable_counts) == 1,
                       'hash_counts': dict(counts), 'stability_hash_counts': dict(stable_counts),
                       'audio_seconds_values': sorted({r['audio_seconds'] for r in all_rows})})
    stdout = data['stdout.log'].decode('utf-8-sig', errors='replace')
    stderr = data['stderr.log'].decode('utf-8-sig', errors='replace')
    warnings = Counter(line for line in stdout.splitlines() + stderr.splitlines()
                       if line.startswith(('WARNING:', 'INFO:')))
    stage_times: list[Row] = []
    previous: str | None = None
    for line_number, line in enumerate(stdout.splitlines(), 1):
        if re.fullmatch(r'\d+\.\d+\s+\d+\.\d+\s+\d+\.\d+\s+\d+\.\d+', line):
            previous = line
        elif line.startswith('{'):
            row = json.loads(line)
            if row.get('event') == 'request':
                stage_times.append({'phase': row['phase'], 'index': row['index'],
                                    'stdout_event_line': line_number, 'upstream_4_stage_s': previous})
                previous = None
    summary: Row = {
        'complete_event': complete, 'exit_code': int(data['exit.txt'].decode('utf-8-sig').strip()),
        'percentile_method': 'linear interpolation at (n-1)*p',
        'rtf_method': 'total_seconds/audio_seconds; aggregate=sum(total)/sum(audio)',
        'time_coverage_basis': 'UTC event completion timestamps; memory UTC is sampler start',
        'missing_memory': [r for r in memory if r['status'] != '已采到'],
        'wddm_missing_coverage': coverage([r for r in memory if r['dedicated'] is None]),
        'load_event': loaded, 'post_load': memory_stats([r for r in memory if loaded['utc'] <= r['utc'] <= load_end['utc']]),
        'whole_run': memory_stats(memory),
        'cold': request_stats([r for r in requests if r['phase'] == 'cold']),
        'latency': {lang: request_stats([r for r in requests if r['phase'] == 'latency' and r['case'] == lang])
                    for lang in ('zh', 'mixed')},
        'stability': {'duration_seconds': end['duration_seconds'], 'start_utc': start['utc'], 'end_utc': end['utc'],
                      'all': request_stats(stable),
                      'per_language': {lang: request_stats([r for r in stable if r['case'] == lang]) for lang in ('zh', 'mixed')},
                      'first_time_10pct': request_stats(first), 'last_time_10pct': request_stats(last),
                      'first_count_10pct': request_stats(stable[:count]), 'last_count_10pct': request_stats(stable[-count:]),
                      'memory': memory_stats(stable_mem),
                      'first_time_10pct_memory': memory_stats([r for r in stable_mem if r['utc'] <= start['utc'] + duration * .1]),
                      'last_time_10pct_memory': memory_stats([r for r in stable_mem if r['utc'] >= end['utc'] - duration * .1]),
                      'last_10min_memory': memory_stats([r for r in stable_mem if r['utc'] >= end['utc'] - 600])},
        'errors': {'failure_events': sum(r['event'] == 'failure' for r in events),
                   'guard_forbidden_observations': sum(bool(r['forbidden'] or r['external_heavy_engines']) for r in guards),
                   'traceback_headers': stdout.count('Traceback (most recent call last)') + stderr.count('Traceback (most recent call last)'),
                   'logged_error_lines': sum(line.startswith(('ERROR:', 'CRITICAL:')) for line in (stdout + '\n' + stderr).splitlines())},
        'warnings': dict(warnings), 'guard_coverage': coverage(guards),
        'hash_cases_all_identical': sum(r['all_byte_identical'] for r in hashes),
        'hash_cases_stability_identical': sum(r['stability_byte_identical'] for r in hashes),
        'request_count': len(requests), 'raw_memory_count': len(raw), 'derived_memory_count': len(memory),
        'sample_interval_s': distribution([b['utc'] - a['utc'] for a, b in zip(memory, memory[1:])]),
        'upstream_stage_times': stage_times[:6]}
    return summary, memory, hashes


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text(encoding='utf-8-sig'))
    with zipfile.ZipFile(args.archive) as archive:
        data = {item['file']: archive.read(item['file']) for item in manifest}
    for item in manifest:
        content = data[item['file']]
        if len(content) != item['bytes'] or hashlib.sha256(content).hexdigest() != item['sha256']:
            raise ValueError('raw evidence hash mismatch: ' + item['file'])
    summary, memory, hashes = build(data)
    args.output.mkdir(parents=True, exist_ok=True)
    for name, value in (('summary.json', summary), ('case-hashes.json', hashes)):
        (args.output / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    with (args.output / 'memory-derived.jsonl').open('w', encoding='utf-8') as stream:
        for row in memory:
            stream.write(json.dumps(row, ensure_ascii=False) + '\n')
    print(json.dumps({'status': 'complete', 'requests': summary['request_count'],
                      'raw_memory_rows': len(memory), 'missing_after_complete': len(summary['missing_memory'])}))


if __name__ == '__main__':
    main()
