"""Separate observable image log intervals without changing or rerunning measurements."""
from __future__ import annotations

import json
from pathlib import Path
import re

from http_probe import ROOT, write_json
from summarize_visual_remaining import statistics


def stamp_ms(line: str) -> float:
    # Pinned logger prefix: minutes.seconds.milliseconds.microseconds.
    minutes, seconds, milliseconds, microseconds = map(int, line.split()[0].split('.'))
    return minutes * 60000 + seconds * 1000 + milliseconds + microseconds / 1000


def main() -> None:
    output = ROOT / 'reports/P0-12/continuation-02'
    rows = json.loads((output / 'image-summary.json').read_text(encoding='utf-8'))
    for row in rows:
        lines = row['imageTimingLogLines']
        encode = [line for line in lines if 'encoding mtmd batch' in line]
        decode = [line for line in lines if 'decoding image batch' in line]
        finished = [line for line in lines if 'image decoded' in line]
        row['resolution'] = int(Path(row['id']).stem.split('-')[-1])
        row['encoderStartToImageDecodeStartMs'] = (
            stamp_ms(decode[0]) - stamp_ms(encode[0]) if len(encode) == 1 and decode else None)
        row['imageEmbeddingDecodeLogMs'] = (
            sum(int(re.search(r'in (\d+) ms', line)[1]) for line in finished) if finished else None)
        row['imageEmbeddingTokens'] = (
            sum(int(re.search(r'n_tokens_batch = (\d+)', line)[1]) for line in decode) if decode else None)
        row['imageEmbeddingBatches'] = len(decode)
    groups = []
    for resolution in [256, 512, 1024]:
        group = [row for row in rows if row['resolution'] == resolution]
        entry = dict(resolution=resolution, n=len(group))
        for key in ['encoderStartToImageDecodeStartMs', 'imageEmbeddingDecodeLogMs', 'ttftClientMs', 'completedClientMs']:
            entry[key] = statistics([row[key] for row in group if row[key] is not None])
        entry['nativePromptMs'] = statistics([row['timings']['prompt_ms'] for row in group])
        groups.append(entry)
    write_json(output / 'image-timing-boundaries-v2.json', dict(rows=rows, groups=groups,
        boundaries=dict(encoder='Log wall interval: encode_mtmd_chunk before mtmd_batch_encode to helper image-decode start; includes host/scheduling overhead, not a pure kernel timer.',
                        imagePrefill='Sum of pinned helper image-embedding llama_decode batch durations; integer-ms precision per batch.',
                        prompt='Native prompt_ms includes image encoding, embedding prefill, and text prompt work; not text-only prefill.',
                        preprocessing='PNG decode/resize/normalization not independently timed by these logs.',
                        client='Send /completion to first token/content event; file/base64/template preparation excluded.')))
    print(json.dumps(groups, ensure_ascii=False))


if __name__ == '__main__':
    main()
