"""Read frozen P0-12 evidence only; no model loading or new experiments."""
from __future__ import annotations

import csv
import hashlib
import json
from pathlib import Path

import numpy as np

from summarize_injection import compare, floats, table

ROOT = Path(__file__).resolve().parents[2]
RAW = Path('D:/Dev/cache/P0-12/gpu-01')
OUT = ROOT / 'reports/P0-12/results'


def main() -> None:
    OUT.mkdir(exist_ok=False)
    results = []
    for lang in ['zh', 'en', 'ja']:
        one, two, three = (RAW / f'item-{i}' / lang for i in [1, 2, 3])
        zero = [compare(one / mode, one / 'plain', 248320) for mode in
                ['observe', 'attention-zero', 'full-zero', 'native-zero',
                 'old-zero-greedy', 'full-zero-greedy', 'native-zero-greedy']]
        updates = []
        for site in ['old', 'full', 'native']:
            zero.append(compare(two / (site + '-zero-after-B'), two / 'plain', 248320))
            for first, second in [('A-again', 'A'), ('fresh-A', 'A'), ('fresh-B', 'B'),
                                  ('diagnostic-retained-B', 'fresh-B')]:
                updates.append(compare(two / (site + '-' + first), two / (site + '-' + second), 248320))
        zero.append(compare(two / 'native-clear', two / 'plain', 248320))
        masks = []
        for mode, site, all_rows in [('attention-basis-last', 'old', False), ('attention-basis-all', 'old', True),
                                    ('full-basis-last', 'full', False), ('full-basis-all', 'full', True)]:
            before = floats(three / mode / (site + '-prefill-all-before.f32')).reshape(-1, 2560)
            after = floats(three / mode / (site + '-prefill-all-after.f32')).reshape(-1, 2560)
            delta = after.astype('float64') - before.astype('float64')
            expected = np.zeros_like(delta)
            expected[:, 0] = 1 if all_rows else 0
            if not all_rows:
                expected[-1, 0] = 1
            masks.append(dict(mode=mode, rows=len(before), changedRows=int(np.any(delta != 0, axis=1).sum()),
                              maximumAddError=float(np.abs(delta - expected).max()),
                              activations=table(three / mode / 'activations.tsv')))
        # Native vector is applied before l_out; paired no-vector callback observation is item 1.
        baseline = floats(one / 'observe/full-prefill-all-before.f32').reshape(-1, 2560)
        native = floats(three / 'native-basis-all/full-prefill-all-before.f32').reshape(-1, 2560)
        delta = native.astype('float64') - baseline.astype('float64')
        expected = np.zeros_like(delta); expected[:, 0] = 1
        masks.append(dict(mode='native-basis-all', rows=len(delta), changedRows=int(np.any(delta != 0, axis=1).sum()),
                          maximumAddError=float(np.abs(delta - expected).max())))
        greedy = {mode: [int(row['predicted']) for row in table(one / mode / 'tokens.tsv')]
                  for mode in ['plain', 'old-zero-greedy', 'full-zero-greedy', 'native-zero-greedy']}
        results.append(dict(language=lang, baselinePositions=len(greedy['plain']), zeroComparisons=zero,
                            updateComparisons=updates, masks=masks, greedyTokens=greedy))
    (OUT / 'injection-summary.json').write_text(json.dumps(results, ensure_ascii=False, indent=2), encoding='utf-8')
    entries = []
    for path in sorted(RAW.rglob('*')):
        if path.is_file():
            with path.open('rb') as stream:
                digest = hashlib.file_digest(stream, 'sha256').hexdigest()
            entries.append(dict(path=path.relative_to(RAW).as_posix(), bytes=path.stat().st_size, sha256=digest))
    (OUT / 'raw-inventory.json').write_text(json.dumps(dict(files=entries), indent=2), encoding='utf-8')
    for result in results:
        print(json.dumps(dict(language=result['language'], positions=result['baselinePositions'],
                              zero=[(p['first'], p['byteEqualPositions'], p['positionCount'], p['maxAbs']) for p in result['zeroComparisons']],
                              updates=[(p['first'], p['byteEqualPositions'], p['positionCount'], p['maxAbs']) for p in result['updateComparisons']],
                              masks=[{k:v for k,v in p.items() if k != 'activations'} for p in result['masks']]), ensure_ascii=False))
    print(json.dumps(dict(files=len(entries), rawBytes=sum(e['bytes'] for e in entries),
                          uniqueBytes=sum({e['sha256']:e['bytes'] for e in entries}.values()))))


if __name__ == '__main__':
    main()
