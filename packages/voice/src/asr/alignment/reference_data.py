"""Independent alignment input checks and reference extraction; no ASR/VAD imports."""
from __future__ import annotations
from dataclasses import dataclass
import hashlib
import json
import math
from pathlib import Path
import subprocess
import unicodedata
from typing import cast

METHOD = 'P4-01-forced-alignment-v1'
ANNOTATION = 'qwen3-forced-alignment-v1'


def read_json(path: Path) -> object:
    value: object = json.loads(path.read_text(encoding='utf-8-sig'))
    return value


def obj(value: object) -> dict[str, object]:
    if not isinstance(value, dict) or not all(isinstance(k, str) for k in value):
        raise ValueError('Expected JSON object')
    return cast(dict[str, object], value)


def objects(value: object) -> list[dict[str, object]]:
    if not isinstance(value, list):
        raise ValueError('Expected JSON array')
    return [obj(item) for item in value]


def text(value: object) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError('Expected non-empty string')
    return value


def number(value: object) -> float:
    if isinstance(value, bool) or not isinstance(value, (float, int)) or not math.isfinite(value):
        raise ValueError('Non-finite alignment time')
    return float(value)


def file_sha(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def text_sha(value: str) -> str:
    return hashlib.sha256(value.encode('utf-8')).hexdigest()


def inside(root: Path, relative: str) -> Path:
    path = (root / relative).resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError('Audio path escapes local evaluation directory')
    return path


@dataclass(frozen=True)
class Recording:
    id: str
    lang: str
    source: Path
    source_path: str
    source_sha256: str
    transcript: str
    transcript_sha256: str


def recordings(root: Path, plan: dict[str, object]) -> list[Recording]:
    expected = objects(plan['samples'])
    rows = objects(read_json(root / 'user.json'))
    indexed = {text(row['id']): row for row in rows}
    if len(rows) != 60 or len(indexed) != 60 or set(indexed) != {text(r['id']) for r in expected}:
        raise ValueError(f'All 60 preregistered user recordings required; registered={len(rows)}')
    output: list[Recording] = []
    for sample in expected:
        row = indexed[text(sample['id'])]
        if row['lang'] != sample['lang'] or row['path'] != sample['path']:
            raise ValueError(f'Registered sample differs from preregistration: {sample["id"]}')
        source_path = text(row['path'])
        source = inside(root, source_path)
        transcript = source.with_suffix('.txt').read_text(encoding='utf-8-sig').strip()
        if transcript != row['text']:
            raise ValueError(f'Human transcript changed; register again: {sample["id"]}')
        digest = file_sha(source)
        if digest != row['sha256']:
            raise ValueError(f'User audio changed; register again: {sample["id"]}')
        output.append(Recording(text(row['id']), text(row['lang']), source, source_path,
                                digest, transcript, text_sha(transcript)))
    return output


def canonicalize(recording: Recording, root: Path) -> Path:
    target = inside(root, f'alignment-input/{recording.id}.wav')
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        return target
    # Preserve leading/trailing silence. No silence detector or threshold is used.
    subprocess.run(['ffmpeg', '-v', 'error', '-y', '-i', str(recording.source), '-ar', '16000',
                    '-ac', '1', '-c:a', 'pcm_s16le', str(target)], check=True, capture_output=True)
    return target


def boundaries(items: list[dict[str, object]], duration_seconds: float) -> tuple[float, float]:
    lexical: list[tuple[float, float]] = []
    for item in items:
        if not any(unicodedata.category(ch)[0] in ('L', 'N') for ch in text(item['text'])):
            continue
        start, end = number(item['start_time']), number(item['end_time'])
        if start < 0 or end <= start or end > duration_seconds:
            raise ValueError('Invalid/out-of-range alignment span; no silent clamping')
        if lexical and (start < lexical[-1][0] or end < lexical[-1][1]):
            raise ValueError('Non-monotonic alignment spans')
        lexical.append((start, end))
    if not lexical:
        raise ValueError('No lexical units aligned')
    return lexical[0][0] * 1000, lexical[-1][1] * 1000


def freeze(path: Path, value: object) -> None:
    if path.exists():
        if read_json(path) != value:
            raise ValueError('Frozen alignment inputs changed; preserve prior attempt and preregister amendment')
        return
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
