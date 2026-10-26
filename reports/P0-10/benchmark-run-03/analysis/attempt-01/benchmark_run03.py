"""Offline, lossless audit and descriptive statistics for frozen P0-10 run-03."""
from __future__ import annotations

from collections import Counter
from datetime import datetime, timezone
import gzip
import hashlib
import json
import math
from pathlib import Path
import re
import statistics
import sys

ROOT = Path('D:/Dev/nemu-wt/P0-10')
CACHE = Path('D:/Dev/cache/P0-10')
RUN = CACHE / 'benchmark-runs/run-03'
PREPARED = CACHE / 'prepared/full-v5'


def load(path: Path):
    return json.loads(path.read_text(encoding='utf-8-sig'))


def digest(path: Path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def lines(path: Path):
    return [json.loads(line) for line in path.read_text(encoding='utf-8-sig').splitlines() if line]


def describe(values):
    v = sorted(float(x) for x in values if x is not None)
    assert all(math.isfinite(x) for x in v)
    if not v:
        return {'count': 0, 'nearestRankP50': None, 'nearestRankP90': None,
                'linearP50': None, 'linearP90': None, 'mean': None, 'minimum': None, 'maximum': None}
    def linear(p):
        position = (len(v) - 1) * p
        i = math.floor(position)
        j = min(i + 1, len(v) - 1)
        return v[i] + (position - i) * (v[j] - v[i])
    return {'count': len(v), 'nearestRankP50': v[math.ceil(.5 * len(v)) - 1],
            'nearestRankP90': v[math.ceil(.9 * len(v)) - 1],
            'linearP50': linear(.5), 'linearP90': linear(.9), 'mean': statistics.fmean(v),
            'minimum': v[0], 'maximum': v[-1]}


def group_stats(rows):
    result = {'count': len(rows), 'schemaValid': sum(r['schemaValid'] is True for r in rows),
              'sourceBindingValid': sum(r['sourceBindingValid'] is True for r in rows),
              'nonEosStopIds': [r['id'] for r in rows if r['stopType'] != 'eos'],
              'tokenLimitIds': [r['id'] for r in rows if r['stopType'] == 'limit'],
              'schemaInvalid': [r['id'] for r in rows if r['schemaValid'] is False],
              'sourceInvalid': [r['id'] for r in rows if r['sourceBindingValid'] is False],
              'stopTypes': dict(Counter(r['stopType'] for r in rows))}
    for key in ['ttftClientMs', 'completedClientMs', 'firstSentenceClientMs', 'inputTokens',
                'returnedStreamTokenCount']:
        result[key] = describe(r[key] for r in rows)
    for key in ['prompt_n', 'prompt_ms', 'prompt_per_second', 'predicted_n', 'predicted_ms',
                'predicted_per_second']:
        result[key] = describe(r['timings'][key] for r in rows)
    # A residual between different timer boundaries; not an attribution to CPU or grammar.
    result['clientTotalMinusNativePromptAndDecodeMs'] = describe(
        r['completedClientMs'] - r['timings']['prompt_ms'] - r['timings']['predicted_ms'] for r in rows)
    result['clientTTFTMinusNativePromptMs'] = describe(
        r['ttftClientMs'] - r['timings']['prompt_ms'] for r in rows if r['ttftClientMs'] is not None)
    return result


def memory_stats(path):
    snapshots = lines(path)
    aggregates = []
    previous = None
    intervals = []
    query_lengths = []
    for snap in snapshots:
        start = datetime.fromisoformat(snap['startedUtc'])
        end = datetime.fromisoformat(snap['endedUtc'])
        query_lengths.append((end - start).total_seconds())
        if previous is not None:
            intervals.append((start - previous).total_seconds())
        previous = start
        owned = {p['ProcessId'] for p in snap['ownedProcesses']}
        counters = []
        for counter in snap['counters']:
            match = re.match(r'pid_(\d+)_', counter['Name'])
            if match and int(match[1]) in owned:
                counters.append(counter)
        if counters:
            aggregates.append({'utc': snap['startedUtc'],
                               'dedicated': sum(c['DedicatedUsage'] for c in counters),
                               'shared': sum(c['SharedUsage'] for c in counters)})
    if not aggregates:
        return {'samples': len(snapshots), 'matchedSamples': 0, 'dedicatedMaximum': None,
                'sharedMaximum': None, 'independentMaximaSum': None}
    dedicated = max(aggregates, key=lambda r: r['dedicated'])
    shared = max(aggregates, key=lambda r: r['shared'])
    return {'samples': len(snapshots), 'matchedSamples': len(aggregates),
            'dedicatedMaximum': dedicated['dedicated'], 'dedicatedMaximumUtc': dedicated['utc'],
            'sharedMaximum': shared['shared'], 'sharedMaximumUtc': shared['utc'],
            'independentMaximaSum': dedicated['dedicated'] + shared['shared'],
            'largestSameSnapshotSum': max(r['dedicated'] + r['shared'] for r in aggregates),
            'startIntervalsSeconds': describe(intervals), 'queryDurationsSeconds': describe(query_lengths),
            'scope': 'WDDM owned process tree; independent maxima; sampled not continuous; carrier only'}


def analyse():
    exit_record = load(CACHE / 'downloads/benchmark-rerun-run-03-exit.json')
    assert exit_record['exitCode'] == 0, 'Preserve failed run; this analysis requires a completed run'
    assert exit_record['runtimeCommit'] == 'a1f18b0f953a0291dd3f09701d35662cca436402'
    jobs = json.loads(gzip.decompress((PREPARED / 'jobs.json.gz').read_bytes()))
    by_id = {job['id']: job for job in jobs}
    assert len(jobs) == len(by_id) == 474
    plan = load(PREPARED / 'plan.json')
    assert load(RUN / 'plan.json') == plan
    result = {'analysedUtc': datetime.now(timezone.utc).isoformat(),
              'methodSha256': digest(Path(__file__)), 'runtime': exit_record,
              'planSha256': digest(RUN / 'plan.json'), 'groups': {}, 'audit': {}}
    expected = {'speech': 20, 'unconstrained': 50, 'wire': 200, 'compact': 200}
    baseline = load(ROOT / 'reports/P0-08/review-r1/local-breakdown/summary.json')
    result['oldP0_08FiftyBaseline'] = baseline['arms']
    baseline_rows = lines(ROOT / 'reports/P0-08/review-r1/local-breakdown/samples.jsonl')
    old_input = {r['caseId']: r['inputTokens'] for r in baseline_rows if not r['warmup']}
    fixed = '今天的努力值得肯定请带着信心继续向前走吧。'
    old_speech_ids = []
    for i in range(1, 4):
        old_speech_ids.append(load(ROOT / f'reports/P0-07/instruct-adr0012-v1/run-{i}/matrix/'
                                   'Qwen--Qwen3-4B-Instruct-2507-int4/concurrent.json')['input_token_ids'][0])
    result['sameP0_07InputIdsAllThreeRuns'] = all(by_id['speech-01']['inputTokenIds'] == ids for ids in old_speech_ids)
    assert result['sameP0_07InputIdsAllThreeRuns']
    for quant in ['Q4_K_M', 'Q8_0']:
        path = RUN / quant
        rows = lines(path / 'samples.jsonl')
        assert [r['id'] for r in rows] == [j['id'] for j in jobs]
        assert len(rows) == 474 and sum(r['warmup'] for r in rows) == 4
        raw_hashes = []
        outputs = {}
        fps_error = []
        for row in rows:
            job = by_id[row['id']]
            assert row['arm'] == job['arm'] and row['sample'] == job['sample']
            assert row['warmup'] == job['warmup'] and row['promptSha256'] == job['promptSha256']
            request = load(path / row['rawRequest'])
            response = load(path / row['rawResponse'])
            assert request['prompt'] == job['inputTokenIds']
            assert request['cache_prompt'] is False
            final = response['final']
            assert row['timings'] == final['timings']
            assert final['timings']['cache_n'] == 0
            assert final['timings']['prompt_n'] == len(job['inputTokenIds']) == row['inputTokens']
            assert final['truncated'] is False
            assert response['ttftClientMs'] == row['ttftClientMs']
            assert response['completedClientMs'] == row['completedClientMs']
            assert row['returnedStreamTokenCount'] == final['timings']['predicted_n']
            native = final['timings']
            predicted = (1000 * max(native['predicted_n'] - 1, 0) / native['predicted_ms']
                         if native['predicted_ms'] > 0 else 0.0)
            fps_error.append(abs(predicted - native['predicted_per_second']))
            outputs[row['id']] = response['content']
            raw_hashes.append({'id': row['id'], 'requestSha256': digest(path / row['rawRequest']),
                               'responseSha256': digest(path / row['rawResponse'])})
            if row['sample'] and int(row['sample']['id'].split('-')[1]) <= 50 and row['arm'] in ['wire', 'unconstrained']:
                assert row['inputTokens'] == old_input[row['sample']['id']]
        formal = [r for r in rows if not r['warmup']]
        groups = {}
        for arm, count in expected.items():
            group = [r for r in formal if r['arm'] == arm]
            assert len(group) == count
            groups[arm] = group_stats(group)
            groups[arm]['languages'] = {lang: group_stats([r for r in group if r['sample'] and r['sample']['lang'] == lang])
                                       for lang in ['zh', 'en', 'ja']} if arm != 'speech' else {}
        groups['wireFirst50'] = group_stats([r for r in formal if r['arm'] == 'wire' and int(r['sample']['id'].split('-')[1]) <= 50])
        speech = [r for r in formal if r['arm'] == 'speech']
        groups['speech']['exactFixedSentenceMatches'] = sum(outputs[r['id']] == fixed for r in speech)
        groups['speech']['distinctOutputs'] = dict(Counter(outputs[r['id']] for r in speech))
        wire = {r['sample']['id']: r for r in formal if r['arm'] == 'wire'}
        compact = {r['sample']['id']: r for r in formal if r['arm'] == 'compact'}
        groups['pairedCompactOverWireTimeRatio'] = describe(compact[k]['completedClientMs'] / wire[k]['completedClientMs'] for k in wire)
        groups['pairedCompactOverWireTokenRatio'] = describe(compact[k]['timings']['predicted_n'] / wire[k]['timings']['predicted_n'] for k in wire)
        groups['pairedCompactFasterCount'] = sum(compact[k]['completedClientMs'] < wire[k]['completedClientMs'] for k in wire)
        groups['pairedCompactFewerTokensCount'] = sum(compact[k]['timings']['predicted_n'] < wire[k]['timings']['predicted_n'] for k in wire)
        stderr = (path / 'server/server.stderr.log').read_text(encoding='utf-8', errors='replace')
        assert 'loaded CUDA backend' in stderr and 'offloaded 37/37 layers to GPU' in stderr
        shutdown = load(path / 'server/shutdown.json')
        assert shutdown['watchReturnCode'] == 0 and shutdown['serverTerminatedByHarness'] is True
        groups['memory'] = memory_stats(path / 'server/memory.jsonl')
        result['groups'][quant] = groups
        result['audit'][quant] = {'rows': len(rows), 'formal': len(formal), 'warmups': 4,
                                 'noCacheOrTruncation': True, 'completeJobsInOrder': True,
                                 'nativeTPSFormulaMaxAbsError': max(fps_error),
                                 'samplesSha256': digest(path / 'samples.jsonl'),
                                 'rawRequestResponseHashes': raw_hashes, 'shutdown': shutdown}
    observations = lines(CACHE / 'downloads/benchmark-rerun-run-03-resource-observations.jsonl')
    result['resourceObservations'] = {'count': len(observations),
                                     'runnerPresentObservations': [r for r in observations if r['runnerProcesses']]}
    assert not result['resourceObservations']['runnerPresentObservations']
    result['groupPolicy'] = 'All completed formal responses including invalid schema/source and token-limit results; warmups excluded only'
    result['percentiles'] = 'Nearest rank agrees with registered native summary; linear p50/p90 also given to compare prior reports'
    return result


if __name__ == '__main__':
    destination = Path(sys.argv[1])
    data = analyse()
    with destination.open('x', encoding='utf-8') as stream:
        json.dump(data, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
    print(json.dumps({'status': 'offline-analysis-completed', 'output': str(destination),
                      'quants': list(data['groups'])}))
